// src/execute/instantiate.rs
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdResult, Uint128};
use crate::msg::InstantiateMsg;
use crate::state::{Config, STATE, CONFIG, State};

pub fn execute_instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let registration_address_addr = deps.api.addr_validate(&msg.registration_address)?;
    let contract_manager_addr = deps.api.addr_validate(&msg.contract_manager)?;
    let registration_wallet_addr = deps.api.addr_validate(&msg.registration_wallet)?;
    let registry_contract_addr = deps.api.addr_validate(&msg.registry_contract)?;

    let state = State {
        registrations: 0,
        last_anml_buyback: env.block.time,
        total_allocations: Uint128::zero(),
        allocation_counter: 0,
        registration_reward: Uint128::zero(),
        last_upkeep: env.block.time,
        reward_index: Uint128::zero(),
        epoch: 0,
    };

    let config = Config {
        contract_manager: contract_manager_addr,
        registration_address: registration_address_addr,
        registration_wallet: registration_wallet_addr,
        registration_validity_seconds: 60 * 60 * 24 * 30, // 30 days
        registry_contract: registry_contract_addr,
        registry_hash: msg.registry_hash,
    };

    STATE.save(deps.storage, &state)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}
