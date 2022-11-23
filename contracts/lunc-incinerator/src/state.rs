use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub stable_denom: String,

    pub burn_address: String,

    pub community_owner: String,
    pub community_dev: String,
}

pub const CONFIG: Item<Config> = Item::new("config");
