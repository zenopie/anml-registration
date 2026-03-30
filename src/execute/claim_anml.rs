use cosmwasm_std::{
    DepsMut, Env, MessageInfo, Response, StdError, StdResult, Timestamp, CosmosMsg, WasmMsg,
    to_binary,
};
use crate::state::{REGISTRATIONS, CONFIG, STATE, query_registry};
use crate::msg::SendMsg;
use secret_toolkit::snip20::HandleMsg;
use crate::execute::allocation::update_reward_index;

pub fn claim_anml(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {

    // Load config to get registration validity period
    let config = CONFIG.load(deps.storage)?;

    // Attempt to retrieve registration data using the new DualKeymap
    if let Some(mut registration) = REGISTRATIONS.get_by_address(deps.storage, &info.sender)? {

        // Check registration validity
        let registration_age = env.block.time.seconds() - registration.registration_timestamp.seconds();
        if registration_age > config.registration_validity_seconds {
            return Err(StdError::generic_err("Registration has expired"));
        }

        // Check last ANML claim time
        let elapsed_time = env.block.time.seconds() - registration.last_anml_claim.seconds();
        let seconds_in_a_day = 86400;
        if elapsed_time < seconds_in_a_day {
            return Err(StdError::generic_err(
                "One day hasn't passed since the last claim",
            ));
        }

        // Update global reward index (O(1) - no allocation iteration)
        let mut state = STATE.load(deps.storage)?;
        update_reward_index(&mut state, env.block.time);

        // Set last_anml_claim to midnight of the current day
        let midnight_timestamp = Timestamp::from_seconds(
            (env.block.time.seconds() / seconds_in_a_day) * seconds_in_a_day
        );
        registration.last_anml_claim = midnight_timestamp;

        // Update the registration in storage
        REGISTRATIONS.insert(
            deps.storage,
            info.sender.clone(),
            registration.id_hash.clone(),
            registration.clone()
        )?;

        // Query registry for contract references
        let deps_ref = deps.as_ref();
        let contracts = query_registry(
            &deps_ref,
            &config.registry_contract,
            &config.registry_hash,
            vec!["erth_token", "anml_token", "exchange"],
        )?;
        let erth_token = &contracts[0];
        let anml_token = &contracts[1];
        let exchange = &contracts[2];

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
            contract_addr: erth_token.address.to_string(),
            code_hash: erth_token.code_hash.clone(),
            msg: to_binary(&mint_erth)?,
            funds: vec![],
        }));

        // Swap Erth for ANML
        let swap_msg = HandleMsg::Send {
            recipient: exchange.address.to_string(),
            recipient_code_hash: Some(exchange.code_hash.clone()),
            amount: buyback_amount.into(),
            msg: Some(to_binary(&SendMsg::AnmlBuybackSwap {})?),
            memo: None,
            padding: None,
        };
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: erth_token.address.to_string(),
            code_hash: erth_token.code_hash.clone(),
            msg: to_binary(&swap_msg)?,
            funds: vec![],
        }));

        // Save state
        STATE.save(deps.storage, &state)?;

        // Create message for minting ANML to the user
        let mint_anml = HandleMsg::Mint {
            recipient: info.sender.to_string(),
            amount: 1_000_000u32.into(),
            padding: None,
            memo: None,
        };
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: anml_token.address.to_string(),
            code_hash: anml_token.code_hash.clone(),
            msg: to_binary(&mint_anml)?,
            funds: vec![],
        }));

        Ok(Response::new()
            .add_messages(messages)
            .add_attribute("action", "claim")
            .add_attribute("buyback_amount", buyback_amount.to_string()))
    } else {
        Err(StdError::generic_err("User not registered"))
    }
}
