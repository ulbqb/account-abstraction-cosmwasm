use cosmos_sdk_proto::traits::MessageExt;
use cosmrs::{
    bank::MsgSend,
    crypto::secp256k1,
    tx::{self, AccountNumber, Fee, Msg, SignDoc, SignerInfo},
    AccountId, Coin,
};
use std::{
    env,
    str::{self, FromStr},
};

/// Chain ID to use for tests
const CHAIN_ID: &str = "simd-testing";

/// RPC port
const _RPC_PORT: u16 = 26657;

/// Expected account number
const ACCOUNT_NUMBER: AccountNumber = 0;

/// Bech32 prefix for an account
const ACCOUNT_PREFIX: &str = "link";

/// Denom name
const DENOM: &str = "cony";

/// Example memo
const MEMO: &str = "test memo";

fn main() {
    let args: Vec<String> = env::args().collect();
    let account_contract: &str = args[1].as_str();
    let account_sequence: u64 = args[2].parse().unwrap();
    let private_key: &str = args[3].as_str();

    let decoded_key = hex::decode(private_key).expect("Decoding failed");
    let sender_private_key = secp256k1::SigningKey::from_slice(decoded_key.as_slice()).unwrap();
    let sender_public_key = sender_private_key.public_key();

    let recipient_private_key = secp256k1::SigningKey::random();
    let recipient_account_id = recipient_private_key
        .public_key()
        .account_id(ACCOUNT_PREFIX)
        .unwrap();

    let account_contract_id = AccountId::from_str(account_contract).unwrap();

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
    let gas = 100_000u64;
    let fee = Fee::from_amount_and_gas(amount, gas);

    let tx_body = tx::BodyBuilder::new().msg(msg_send).memo(MEMO).finish();
    let auth_info =
        SignerInfo::single_direct(Some(sender_public_key), account_sequence).auth_info(fee);
    let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, ACCOUNT_NUMBER).unwrap();
    let tx_raw: cosmos_sdk_proto::cosmos::tx::v1beta1::TxRaw =
        sign_doc.clone().sign(&sender_private_key).unwrap().into();

    println!("{:?}", tx_raw.to_bytes().unwrap());
}
