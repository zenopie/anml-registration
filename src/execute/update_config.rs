// src/execute/config.rs
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, StdResult};
use crate::state::{CONFIG, Config};

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    config: Config,
) -> StdResult<Response> {
    let old_config = CONFIG.load(deps.storage)?;
    
    if info.sender != old_config.contract_manager {
        return Err(StdError::generic_err("Unauthorized"));
    }
    
    CONFIG.save(deps.storage, &config)?;
    
    Ok(Response::new()
        .add_attribute("action", "update_config"))
}
