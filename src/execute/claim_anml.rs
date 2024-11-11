// src/execute/claim.rs
use cosmwasm_std::{
    DepsMut, Env, MessageInfo, Response, StdError, StdResult, Timestamp, CosmosMsg, WasmMsg,
    to_binary,
};
use crate::state::{IDS_BY_ADDRESS, CONFIG, STATE};
use crate::msg::SendMsg;
use secret_toolkit::snip20::{HandleMsg};

pub fn claim_anml(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    if let Some(mut user_data) = IDS_BY_ADDRESS.get(deps.storage, &info.sender) {
        let elapsed_time = env.block.time.seconds() - user_data.last_anml_claim.seconds();
        let seconds_in_a_day = 86400;
        if elapsed_time < seconds_in_a_day {
            return Err(StdError::generic_err(
                "One day hasn't passed since the last claim",
            ));
        }

        let midnight_timestamp = Timestamp::from_seconds((env.block.time.seconds() / seconds_in_a_day) * seconds_in_a_day);
        user_data.last_anml_claim = midnight_timestamp;
        IDS_BY_ADDRESS.insert(deps.storage, &info.sender, &user_data)?;

        let mut state = STATE.load(deps.storage)?;
        let config = CONFIG.load(deps.storage)?;

        let buyback_amount = (env.block.time.seconds() - state.last_anml_buyback.seconds()) * 1_000_000;

        state.last_anml_buyback = env.block.time;

        let mut messages = vec![];

        // Create messages for minting ERTH for the ANML buyback
        let mint_erth = HandleMsg::Mint {
            recipient: env.contract.address.to_string(),
            amount: buyback_amount.into(),
            padding: None,
            memo: None,
        };
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.erth_token_contract.to_string(),
            code_hash: config.erth_token_hash.clone(),
            msg: to_binary(&mint_erth)?,
            funds: vec![],
        }));

        // Swap Erth for ANML
        let swap_msg = HandleMsg::Send {
            recipient: config.anml_pool_contract.to_string(),
            recipient_code_hash: Some(config.anml_pool_hash.clone()),
            amount: buyback_amount.into(),
            msg: Some(to_binary(&SendMsg::AnmlBuybackSwap {})?),
            memo: None,
            padding: None,
        };

        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.erth_token_contract.to_string(),
            code_hash: config.erth_token_hash.clone(),
            msg: to_binary(&swap_msg)?,
            funds: vec![],
        }));

        // Save state
        STATE.save(deps.storage, &state)?;

        // Create messages for transferring tokens from the user to the contract using allowances
        let mint_anml = HandleMsg::Mint {
            recipient: info.sender.clone().to_string(),
            amount: 1_000_000u32.into(),
            padding: None,
            memo: None,
        };
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: config.anml_token_contract.to_string(),
            code_hash: config.anml_token_hash.clone(),
            msg: to_binary(&mint_anml)?,
            funds: vec![],
        }));

        let response = Response::new()
            .add_messages(messages)
            .add_attribute("action", "claim")
            .add_attribute("buyback_amount", buyback_amount.to_string());
        Ok(response)
    } else {
        return Err(StdError::generic_err("User data not found"))
    }
}
