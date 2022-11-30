use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub stable_denom: String,

    pub burn_address: String,

    pub admin: String,
    pub community_owner: String,
    pub community_dev: String,

    pub owner_recovery_param: u8,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const NONCE: Map<&Addr, u64> = Map::new("nonce");
