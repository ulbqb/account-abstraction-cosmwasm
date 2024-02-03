use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SignerInfo {
    pub sequence: u64,
}

pub const SIGNER_INFO: Item<SignerInfo> = Item::new("signer_info");
