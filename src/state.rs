use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp};
use secret_toolkit_storage::{Keymap, Item};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub registrations: u128,
    pub declines: u128,
    pub last_upkeep: Timestamp,
}

pub static STATE: Item<State> = Item::new(b"state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Params {
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

