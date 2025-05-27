// src/msg.rs
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Binary, Uint128, Timestamp};
use crate::state::{AllocationConfig, AllocationPercentage, Config,};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    pub registration_address: String,
    pub registration_wallet: String,
    pub contract_manager: String,
    pub anml_token_contract: String,
    pub anml_token_hash: String,
    pub erth_token_contract: String,
    pub erth_token_hash: String,
    pub anml_pool_contract: String,
    pub anml_pool_hash: String,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        config: Config,
    },
    Register {
        address: String,
        id_hash: String,
        affiliate: Option<String>,
    },
    ClaimAnml {},
    SetAllocation {
        percentages: Vec<AllocationPercentage>,
    },
    ClaimAllocation {
        allocation_id: u32,
    },
    AddAllocation {
        receive_addr: Addr,
        receive_hash: Option<String>,
        manager_addr: Option<Addr>,
        claimer_addr: Option<Addr>,
        use_send: bool,
    },
    EditAllocation {
        allocation_id: u32,
        config: AllocationConfig,
    },
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint128,
        memo: Option<String>,
        msg: Binary,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    AllocationSend {
        allocation_id: u32,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SendMsg {
    AnmlBuybackSwap {},
    AllocationSend {
        allocation_id: u32,
    },
    ClaimAllocation {
        allocation_id: u32,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {
    Migrate {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    QueryState {},
    QueryConfig {},
    QueryRegistrationStatus { address: String },
    QueryUserAllocations { address: String },
    QueryAllocationOptions {},
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct RegistrationStatusResponse {
    pub registration_status: bool,
    pub last_claim: Timestamp,
}
