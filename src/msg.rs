use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp};

use crate::state::State;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub registration_address: String,
    pub contract_manager: String,
    pub anml_token_contract: String,
    pub anml_token_hash: String,
    pub erth_token_contract: String,
    pub erth_token_hash: String,
    pub anml_pool_contract: String,
    pub anml_pool_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct UserObject {
    pub country: String,
    pub address: Addr,
    pub first_name: String,
    pub last_name: String,
    pub date_of_birth: String,
    pub document_number: String,
    pub id_type: String,
    pub document_expiration: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateState {
        key: String,
        value: String,
    },
    Register {user_object: UserObject},
    Claim {},
}

/// This struct represents the message to send to the other contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SendMsg {
    AnmlBuyback {},
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {
    Migrate {
        erth_token_contract: String,
        erth_token_hash: String,
        anml_pool_contract: String,
        anml_pool_hash: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    QueryState {},
    RegistrationStatus {address: Addr},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct RegistrationStatusResponse {
    pub registration_status: bool,
    pub last_claim: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct StateResponse {
    pub state: State,
}


