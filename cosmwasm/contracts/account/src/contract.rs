#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;
use finschia_std::types::cosmos::tx::v1beta1::{AuthInfo, TxBody, TxRaw};
use prost::Message;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, SignerInfoResponse};
use crate::state::{SignerInfo, SIGNER_INFO};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:account";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Handling contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    SIGNER_INFO.save(deps.storage, &SignerInfo { sequence: 0 })?;

    // With `Response` type, it is possible to dispatch message to invoke external logic.
    // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

/// Handling contract migration
/// To make a contract migratable, you need
/// - this entry_point implemented
/// - only contract admin can migrate, so admin has to be set at contract initiation time
/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    match msg {
        // Find matched incoming message variant and execute them with your custom logic.
        //
        // With `Response` type, it is possible to dispatch message to invoke external logic.
        // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages
    }
}

/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SendTx { tx } => execute_send_tx(deps, env, info, tx),
    }
}

pub fn execute_send_tx(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    tx_bytes: Vec<u8>,
) -> Result<Response, ContractError> {
    // decode tx
    let tx_raw = TxRaw::decode(tx_bytes.as_slice()).expect("cannot decode tx raw");

    // TODO: validate signature
    // verify(signature, nonce, messages)

    // validate nonce
    let auth_info =
        AuthInfo::decode(tx_raw.auth_info_bytes.as_slice()).expect("cannot decode auth info");
    let account_sequence = auth_info.signer_infos[0].sequence;
    let mut signer_info = SIGNER_INFO.load(deps.storage)?;
    if signer_info.sequence != account_sequence {
        return Err(ContractError::CustomError {
            val: "invaid nonce".into(),
        });
    }
    signer_info.sequence += 1;
    SIGNER_INFO.save(deps.storage, &signer_info)?;

    // set messages
    let mut res = Response::new();
    res = res.add_attribute("action", "send_tx");
    let tx_body = TxBody::decode(tx_raw.body_bytes.as_slice()).expect("cannot decode tx body");
    for msg in tx_body.messages.iter() {
        res = res.add_message(CosmosMsg::Stargate {
            type_url: msg.clone().type_url,
            value: msg.clone().value.into(),
        });
    }
    Ok(res)
}

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::SignerInfo {} => to_binary(&query_signer_info(deps)?),
    }
}

fn query_signer_info(deps: Deps) -> StdResult<SignerInfoResponse> {
    let info = SIGNER_INFO.load(deps.storage)?;
    Ok(SignerInfoResponse {
        sequence: info.sequence,
    })
}

/// Handling submessage reply.
/// For more info on submessage and reply, see https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#submessages
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    // With `Response` type, it is still possible to dispatch message to invoke external logic.
    // See: https://github.com/CosmWasm/cosmwasm/blob/main/SEMANTICS.md#dispatching-messages

    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmos_sdk_proto::cosmos::tx::v1beta1::TxRaw;
    use cosmos_sdk_proto::traits::MessageExt;
    use cosmrs::{
        bank::MsgSend,
        crypto::secp256k1,
        tx::{self, AccountNumber, Fee, Msg, SignDoc, SignerInfo},
        AccountId, Coin,
    };
    use cosmwasm_std::from_binary;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use std::str::{self, FromStr};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &vec![]);

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::SignerInfo {}).unwrap();
        let value: SignerInfoResponse = from_binary(&res).unwrap();
        assert_eq!(0, value.sequence);
    }

    #[test]
    fn send_tx() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &vec![]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("bundler", &vec![]);
        let msg = ExecuteMsg::SendTx { tx: make_tx() };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::SignerInfo {}).unwrap();
        let value: SignerInfoResponse = from_binary(&res).unwrap();
        assert_eq!(1, value.sequence);
    }

    fn make_tx() -> Vec<u8> {
        const CHAIN_ID: &str = "chain-id";
        const ACCOUNT_NUMBER: AccountNumber = 1;
        const ACCOUNT_PREFIX: &str = "prefix";
        const DENOM: &str = "token";
        const MEMO: &str = "memo";
        const ACCOUNT_CONTRACT: &str = "preifx1qqke80wg";

        let sender_private_key = secp256k1::SigningKey::random();
        let sender_public_key = sender_private_key.public_key();

        let recipient_private_key = secp256k1::SigningKey::random();
        let recipient_account_id = recipient_private_key
            .public_key()
            .account_id(ACCOUNT_PREFIX)
            .unwrap();

        let account_contract_id = AccountId::from_str(ACCOUNT_CONTRACT).unwrap();

        let amount = Coin {
            amount: 1u8.into(),
            denom: DENOM.parse().unwrap(),
        };

        let msg_send = MsgSend {
            from_address: account_contract_id.clone(),
            to_address: recipient_account_id,
            amount: vec![amount.clone()],
        }
        .to_any()
        .unwrap();

        let chain_id = CHAIN_ID.parse().unwrap();
        let sequence_number = 0;
        let gas = 100_000u64;
        let fee = Fee::from_amount_and_gas(amount, gas);

        let tx_body = tx::BodyBuilder::new().msg(msg_send).memo(MEMO).finish();
        let auth_info =
            SignerInfo::single_direct(Some(sender_public_key), sequence_number).auth_info(fee);
        let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, ACCOUNT_NUMBER).unwrap();
        let tx_raw: TxRaw = sign_doc.clone().sign(&sender_private_key).unwrap().into();
        tx_raw.to_bytes().unwrap()
    }
}
