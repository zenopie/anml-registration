// src/state/registration.rs
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Timestamp};
use secret_toolkit_storage::Keymap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Id {
    pub registration_status: String,
    pub country: String,
    pub wallet_address: Addr,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: Timestamp,
    pub document_number: String,
    pub id_type: String,
    pub document_expiration: Timestamp,
    pub registration_timestamp: Timestamp,
    pub last_anml_claim: Timestamp,
}

pub static IDS_BY_ADDRESS: Keymap<Addr, Id> = Keymap::new(b"ids_by_address");
pub static IDS_BY_DOCUMENT_NUMBER: Keymap<String, Id> = Keymap::new(b"ids_by_document_number");


