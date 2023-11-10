use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp, Uint256};
use secret_toolkit_storage::{Keymap, Item};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub registrations: u128,
    pub declines: u128,
    pub total_erth_staked: Uint256,
    pub last_upkeep: Timestamp,
    pub fee_balance: Uint256,
}

pub static STATE: Item<State> = Item::new(b"state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Params {
    pub scaled_swap_fee: Uint256,
    pub registration_address: Addr,
    pub max_registrations: u128,
    pub erth_contract: Addr,
    pub erth_hash: String,
    pub anml_contract: Addr,
    pub anml_hash: String,
}

pub static PARAMS: Item<Params> = Item::new(b"params");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Id {
    pub registration_status: String,
    pub country: String,
    pub address: Addr,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: String,
    pub document_number: String,
    pub id_type: String,
    pub document_expiration: String,
    pub registration_timestamp: Timestamp,
    pub last_anml_claim: Timestamp
}

pub static IDS_BY_ADDRESS: Keymap<Addr, Id> = Keymap::new(b"ids_by_address");
pub static IDS_BY_DOCUMENT_NUMBER: Keymap<String, Id> = Keymap::new(b"ids_by_document_number");
pub static DECLINE: Keymap<Addr, Id> = Keymap::new(b"decline");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct UnstakeRequest {
    pub amount: Uint256,
    pub request_time: Timestamp, 
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct StakerInfo {
    pub staked_amount: Uint256,
    pub last_reward_claim: Timestamp,
    pub unstake_requests: Option<Vec<UnstakeRequest>>, 
}

pub static STAKER_INFO: Keymap<Addr, StakerInfo> = Keymap::new(b"staker_info");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct RewardRateChange {
    pub rate: Uint256,
    pub since: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Pool {
    pub anml_balance: Uint256,
    pub other_balance: Uint256,
    pub other_contract: Addr,
    pub other_hash: String,
    pub reward_rate: Vec<RewardRateChange>,
    pub volume: Uint256,
    pub shares: Uint256,
}

pub static POOL: Keymap<Addr, Pool> = Keymap::new(b"pool");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ProviderInfo {
    pub provide_amount: Uint256,
    pub last_claim: Timestamp,
    pub withdraw_requests: Option<Vec<UnstakeRequest>>, 
}

pub static PROVIDER_INFO: Keymap<Addr, ProviderInfo> = Keymap::new(b"provider_info");