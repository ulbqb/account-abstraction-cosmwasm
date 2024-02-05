use account_contract::msg::ExecuteMsg;
use base64::{engine::general_purpose, Engine as _};
use cosmrs::{
    cosmwasm::MsgExecuteContract,
    crypto::secp256k1,
    dev, rpc,
    tx::{self, AccountNumber, Fee, Msg, SignDoc, SignerInfo},
    AccountId, Coin,
};
use std::{
    env, panic,
    str::{self, FromStr},
};

/// Chain ID to use for tests
const CHAIN_ID: &str = "simd-testing";

/// RPC port
const RPC_PORT: u16 = 26657;

/// Expected account number
const ACCOUNT_NUMBER: AccountNumber = 47;

/// Bech32 prefix for an account
const ACCOUNT_PREFIX: &str = "link";

/// Denom name
const DENOM: &str = "cony";

/// Example memo
const MEMO: &str = "test memo";

fn main() {
    let args: Vec<String> = env::args().collect();
    let private_key: &str = args[1].as_str();
    let account_contract: &str = args[2].as_str();
    let tx_bytes: &str = args[3].as_str();
    let account_sequence: u64 = args[4].parse().unwrap();

    let decoded_key = hex::decode(private_key).expect("Decoding failed");
    let sender_private_key = secp256k1::SigningKey::from_slice(decoded_key.as_slice()).unwrap();
    let sender_public_key = sender_private_key.public_key();
    let sender_account_id = sender_public_key.account_id(ACCOUNT_PREFIX).unwrap();

    let account_contract_id = AccountId::from_str(account_contract).unwrap();

    let amount = Coin {
        amount: 50000u128,
        denom: DENOM.parse().unwrap(),
    };
    let mut msg_bytes = Vec::<u8>::new();
    general_purpose::STANDARD
        .decode_vec(tx_bytes, &mut msg_bytes)
        .unwrap();
    let msg_execute = MsgExecuteContract {
        sender: sender_account_id.clone(),
        contract: account_contract_id.clone(),
        msg: serde_json::to_vec(&ExecuteMsg::SendTx { tx: msg_bytes }).unwrap(),
        funds: vec![],
    }
    .to_any()
    .unwrap();

    let chain_id = CHAIN_ID.parse().unwrap();
    let gas = 150_000u64;
    let fee = Fee::from_amount_and_gas(amount, gas);

    let tx_body = tx::BodyBuilder::new().msg(msg_execute).memo(MEMO).finish();
    let auth_info =
        SignerInfo::single_direct(Some(sender_public_key), account_sequence).auth_info(fee);
    let sign_doc = SignDoc::new(&tx_body, &auth_info, &chain_id, ACCOUNT_NUMBER).unwrap();
    let tx_raw = sign_doc.sign(&sender_private_key).unwrap();

    // println!("{}", serde_json::to_string(&tx_body));

    // init_tokio_runtime().block_on(async {
    //     let rpc_address = format!("http://host.docker.internal:{}", RPC_PORT);
    //     let rpc_client = rpc::HttpClient::new(rpc_address.as_str()).unwrap();

    //     let tx_commit_response = tx_raw.broadcast_commit(&rpc_client).await.unwrap();

    //     println!("hash {}", tx_commit_response.hash);

    //     if tx_commit_response.check_tx.code.is_err() {
    //         panic!("check_tx failed: {:?}", tx_commit_response.check_tx);
    //     }

    //     if tx_commit_response.deliver_tx.code.is_err() {
    //         panic!("tx_result error: {:?}", tx_commit_response.deliver_tx);
    //     }

    //     let _tx = dev::poll_for_tx(&rpc_client, tx_commit_response.hash).await;
    // })
}

// Initialize Tokio runtime
// fn init_tokio_runtime() -> tokio::runtime::Runtime {
//     tokio::runtime::Builder::new_current_thread()
//         .enable_all()
//         .build()
//         .unwrap()
// }
