use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /* Denomination of the stable asset
    https://docs.terra.money/docs/develop/module-specifications/spec-market.html#market */
    pub stable_denom: String,

    /* Id of the contract uploaded for the first time to the chain
    https://docs.terra.money/docs/develop/module-specifications/spec-wasm.html#code-id */
    pub community_owner: String,
    pub community_dev: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /* Handle the deposits of native tokens into the smart contract to mint
    the new pegged token 1:1 with LUNA or to increase circulation supply. */
    Deposit {},

    Withdraw { recipient: String, amount: Uint128 },

    ChangeCommunityInfo { role: CommunityRole, value: String },

    /* Handle burn of pegged tokens 1:1 with LUNA which are added to
    MINTED_TOKENS list and return the LUNA stored into the contract. */
    Burn { amount: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CommunityRole {
    Owner {},
    Developer {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    CommunityOwner {},
    CommunityDeveloper {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
