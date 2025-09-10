use cosmwasm_std::{
    DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128, Timestamp, CosmosMsg, WasmMsg,
    to_binary,
};
use secret_toolkit::snip20::{self, HandleMsg};
use crate::state::{CONFIG, STATE, REGISTRATIONS, Registration, NEW_REGISTRATIONS_COUNT};
use crate::msg::SendMsg;

pub fn register(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: String,
    id_hash: String,
    affiliate: Option<String>,
) -> StdResult<Response> {
    let mut state = STATE.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    // Check if the sender is authorized
    if info.sender != config.registration_address {
        return Err(StdError::generic_err("Not authorized"));
    }

    // Validate the registree address
    let wallet_address_addr = deps.api.addr_validate(&address)?;

    // Check existing registration
    let existing_registration = REGISTRATIONS.get_by_address(deps.storage, &wallet_address_addr)?;
    if let Some(reg) = existing_registration {
        // Calculate expiration time
        let expiration = reg.registration_timestamp.plus_seconds(config.registration_validity_seconds);
        if env.block.time <= expiration {
            return Err(StdError::generic_err("Registration still valid, cannot re-register yet"));
        }
        // If expired, remove the old registration to allow re-registration
        REGISTRATIONS.remove(deps.storage, &wallet_address_addr, &reg.id_hash)?;
    }

    // Check if the hash is already registered and track if it's a new ID hash
    let mut is_new_id_hash = true;
    if let Some(existing_reg) = REGISTRATIONS.get_by_hash(deps.storage, &id_hash)? {
        is_new_id_hash = false; // This ID hash has been registered before
        let expiration = existing_reg.registration_timestamp.plus_seconds(config.registration_validity_seconds);
        if env.block.time > expiration {
            REGISTRATIONS.remove(deps.storage, &existing_reg.address, &id_hash)?;
        } else {
            return Err(StdError::generic_err("ID hash already registered and not expired"));
        }
    }

    // Set last_anml_claim to midnight of the current day
    let seconds_in_a_day = 86400;
    let midnight_timestamp = Timestamp::from_seconds(
        (env.block.time.seconds() / seconds_in_a_day) * seconds_in_a_day
    );

    // Create the Registration object
    let registration = Registration {
        id_hash: id_hash.clone(),
        registration_timestamp: env.block.time,
        last_anml_claim: midnight_timestamp,
        address: wallet_address_addr.clone(),
    };

    // Insert into DualKeymap using registree_address
    REGISTRATIONS.insert(deps.storage, wallet_address_addr.clone(), id_hash, registration)?;

    // Increment registration count
    state.registrations += 1;

    // Track new registrations and calculate rewards only for brand new ID hashes
    let mut messages = vec![];
    if is_new_id_hash {
        // Increment new registrations counter
        let current_new_count = NEW_REGISTRATIONS_COUNT.may_load(deps.storage)?.unwrap_or(0);
        NEW_REGISTRATIONS_COUNT.save(deps.storage, &(current_new_count + 1))?;

        // Calculate the .1% registration reward
        let reward = state.registration_reward.u128() / 1000;

        // Subtract the total reward from state.registration_reward
        let total_reward = if affiliate.is_some() {
            reward * 2 // 1% to registree, 1% to affiliate
        } else {
            reward // 1% to registree
        };
        state.registration_reward = Uint128::from(state.registration_reward.u128().saturating_sub(total_reward));

        // Create SNIP-20 transfer messages for rewards
        messages.push(
            // Transfer reward to registrant (registree_address)
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.erth_token_contract.to_string(),
                code_hash: config.erth_token_hash.clone(),
                msg: to_binary(&snip20::HandleMsg::Transfer {
                    recipient: wallet_address_addr.to_string(),
                    amount: Uint128::from(reward),
                    memo: None,
                    padding: None,
                })?,
                funds: vec![],
            }),
        );

        // If there's an affiliate, send them 1% as well
        if let Some(affiliate_address) = affiliate {
            let affiliate_addr = deps.api.addr_validate(&affiliate_address)?;
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: config.erth_token_contract.to_string(),
                code_hash: config.erth_token_hash.clone(),
                msg: to_binary(&snip20::HandleMsg::Transfer {
                    recipient: affiliate_addr.to_string(),
                    amount: Uint128::from(reward),
                    memo: None,
                    padding: None,
                })?,
                funds: vec![],
            }));
        }
    }


     // Create message for minting ANML to the user
    let mint_anml = HandleMsg::Mint {
        recipient: address.clone(),
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


    // Execute claim_allocation message
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: env.contract.address.to_string(),
        code_hash: env.contract.code_hash.clone(),
        msg: to_binary(&SendMsg::ClaimAllocation {
            allocation_id: 1,
        })?,
        funds: vec![],
    }));

    // Update state after successful registration
    STATE.save(deps.storage, &state)?;

    // Respond with the transaction messages
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "register")
        .add_attribute("address", wallet_address_addr.to_string()))
}
