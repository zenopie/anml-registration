// src/state/config.rs
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Uint128, Timestamp};
use secret_toolkit_storage::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub registration_address: Addr,
    pub registration_wallet: Addr,
    pub contract_manager: Addr,
    //pub max_registrations: u32,
    pub registration_validity_seconds: u64,
    pub anml_token_contract: Addr,
    pub anml_token_hash: String,
    pub erth_token_contract: Addr,
    pub erth_token_hash: String,
    pub anml_pool_contract: Addr,
    pub anml_pool_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub registrations: u32,
    pub last_anml_buyback: Timestamp,
    pub total_allocations: Uint128,
    pub allocation_counter: u32,
    pub registration_reward: Uint128,
    pub last_upkeep: Timestamp,
}


pub static CONFIG: Item<Config> = Item::new(b"config");
pub static STATE: Item<State> = Item::new(b"state");
