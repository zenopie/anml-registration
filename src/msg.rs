use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128, Timestamp};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub registration_address: Addr,
    pub manager_address: Addr,
    pub anml_contract: Addr,
    pub anml_hash: String,
    pub erth_contract: Addr,
    pub erth_hash: String,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct UpdateStateMsg {
    pub registrations: Option<u32>,
    pub registration_address: Option<Addr>,
    pub manager_address: Option<Addr>,
    pub max_registrations: Option<u32>,
    pub anml_contract: Option<Addr>,
    pub anml_hash: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateState {msg: UpdateStateMsg},
    Register {user_object: UserObject},
    Claim {},
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    RegistrationStatus {address: Addr},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct RegistrationStatusResponse {
    pub registration_status: String,
    pub last_claim: Timestamp,
}


// Messages sent to SNIP-20 contracts
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Snip20Msg {
    Mint {
        recipient: Addr,
        amount: Uint128,
    },
}

impl Snip20Msg {
    pub fn mint_msg(recipient: Addr, amount: Uint128) -> Self {
        Snip20Msg::Mint {
            recipient,
            amount,
        }
    }
}


