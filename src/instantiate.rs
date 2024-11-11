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
    let anml_token_contract_addr = deps.api.addr_validate(&msg.anml_token_contract)?;
    let erth_token_contract_addr = deps.api.addr_validate(&msg.erth_token_contract)?;
    let contract_manager_addr = deps.api.addr_validate(&msg.contract_manager)?;
    let anml_pool_contract_addr = deps.api.addr_validate(&msg.anml_pool_contract)?;
    let registration_wallet_addr = deps.api.addr_validate(&msg.registration_wallet)?;

    let state = State {
        registrations: 0,
        last_anml_buyback: env.block.time,
        total_allocations: Uint128::zero(),
        allocation_counter: 0,
        registration_reward: Uint128::zero(),
    };

    let config = Config {
        contract_manager: contract_manager_addr,
        registration_address: registration_address_addr,
        registration_wallet: registration_wallet_addr,
        max_registrations: 50,
        anml_token_contract: anml_token_contract_addr,
        anml_token_hash: msg.anml_token_hash,
        erth_token_contract: erth_token_contract_addr,
        erth_token_hash: msg.erth_token_hash,
        anml_pool_contract: anml_pool_contract_addr,
        anml_pool_hash: msg.anml_pool_hash,
    };

    STATE.save(deps.storage, &state)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}
