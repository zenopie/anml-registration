// src/state/config.rs
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Uint128, Timestamp, Deps, StdResult, to_binary, QueryRequest, WasmQuery};
use secret_toolkit_storage::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Config {
    pub registration_address: Addr,
    pub registration_wallet: Addr,
    pub contract_manager: Addr,
    pub registration_validity_seconds: u64,
    pub registry_contract: Addr,
    pub registry_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub registrations: u32,
    pub last_anml_buyback: Timestamp,
    pub total_allocations: Uint128,
    pub allocation_counter: u32,
    pub registration_reward: Uint128,
    pub last_upkeep: Timestamp,
    pub reward_index: Uint128,
    pub epoch: u32,
}

// Minimal registry types for cross-contract queries
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistryQueryMsg {
    GetContracts { names: Vec<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContractInfo {
    pub address: Addr,
    pub code_hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct ContractResponse {
    pub name: String,
    pub info: ContractInfo,
}

#[derive(Serialize, Deserialize)]
pub struct AllContractsResponse {
    pub contracts: Vec<ContractResponse>,
}

/// Query the registry for multiple contracts, returned as a Vec in the same order as names
pub fn query_registry(
    deps: &Deps,
    registry_addr: &Addr,
    registry_hash: &str,
    names: Vec<&str>,
) -> StdResult<Vec<ContractInfo>> {
    let query_msg = RegistryQueryMsg::GetContracts {
        names: names.iter().map(|n| n.to_string()).collect(),
    };
    let expected_count = names.len();
    let response: AllContractsResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: registry_addr.to_string(),
        code_hash: registry_hash.to_string(),
        msg: to_binary(&query_msg)?,
    }))?;
    if response.contracts.len() != expected_count {
        return Err(cosmwasm_std::StdError::generic_err(
            format!("Registry returned {} contracts, expected {}", response.contracts.len(), expected_count)
        ));
    }
    Ok(response.contracts.into_iter().map(|c| c.info).collect())
}

pub static CONFIG: Item<Config> = Item::new(b"config");
pub static STATE: Item<State> = Item::new(b"state");
