use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SignerInfo {
    pub public_key: PubKey,
    pub sequence: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PubKey {
    pub type_url: String,
    pub key: Vec<u8>,
}

pub const SIGNER_INFO: Item<SignerInfo> = Item::new("signer_info");
