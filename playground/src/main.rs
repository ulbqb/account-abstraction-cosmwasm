use base64::{engine::general_purpose, Engine as _};
use cosmos_sdk_proto::traits::MessageExt;
use cosmrs::{
    bank::MsgSend,
    crypto::secp256k1,
    tx::{self, AccountNumber, Fee, Msg, SignDoc, SignerInfo},
    AccountId, Coin,
};
use k256::ecdsa::signature::Verifier;
pub use k256::ecdsa::{Signature, SigningKey, VerifyingKey};
use std::str::{self, FromStr};

/// Chain ID to use for tests
const CHAIN_ID: &str = "simd-testing";

/// RPC port
const _RPC_PORT: u16 = 26657;

/// Expected account number
const ACCOUNT_NUMBER: AccountNumber = 1;

/// Bech32 prefix for an account
const ACCOUNT_PREFIX: &str = "link";

/// Denom name
const DENOM: &str = "cony";

/// Example memo
const MEMO: &str = "test memo";

/// account contract address
const ACCOUNT_CONTRACT: &str = "link1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftquem4c6";

fn main() {
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
    let tx_raw: cosmos_sdk_proto::cosmos::tx::v1beta1::TxRaw =
        sign_doc.clone().sign(&sender_private_key).unwrap().into();

    let vk = VerifyingKey::from_sec1_bytes(sender_public_key.to_bytes().as_slice()).unwrap();
    let sig = &Signature::from_slice(tx_raw.signatures[0].as_slice()).unwrap();
    let ok = vk.verify(sign_doc.clone().into_bytes().unwrap().as_slice(), sig);
    match ok {
        Ok(_) => println!("success"),
        Err(e) => println!("failed"),
    }
}
