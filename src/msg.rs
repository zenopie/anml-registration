use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint256, Timestamp};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub registration_address: Addr,
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

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
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
        amount: Uint256,
    },
}

impl Snip20Msg {
    pub fn mint_msg(recipient: Addr, amount: Uint256) -> Self {
        Snip20Msg::Mint {
            recipient,
            amount,
        }
    }
}


