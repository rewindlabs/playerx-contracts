use crate::{
    error::ContractError,
    msg::TokenLevelResponse,
    state::{LEVELING_CONFIG, ROYALTY_CONFIG, TOKEN_LEVELS},
    Cw2981LevelingContract,
};
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Order, Response, StdResult};

pub fn update_royalty_config(
    deps: DepsMut,
    info: MessageInfo,
    royalty_percentage: u64,
    royalty_payment_address: Addr,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    if royalty_percentage > 100 {
        return Err(ContractError::InvalidRoyaltyPercentage);
    }

    let mut royalty_config = ROYALTY_CONFIG.load(deps.storage)?;
    royalty_config.royalty_percentage = royalty_percentage;
    royalty_config.royalty_payment_address = royalty_payment_address.clone();
    ROYALTY_CONFIG.save(deps.storage, &royalty_config)?;

    let res = Response::new()
        .add_attribute("action", "update_royalty_config")
        .add_attribute("royalty_percentage", royalty_percentage.to_string())
        .add_attribute(
            "royalty_payment_address",
            royalty_payment_address.to_string(),
        );
    Ok(res)
}

// Used to calculate the total experience based on the time elapsed
fn calculate_and_cap_experience(
    token_level: &mut TokenLevelResponse,
    current_block_seconds: u64,
    max_experience: u64,
) {
    let elapsed_time = current_block_seconds - token_level.leveling_start_timestamp;
    token_level.total_exp += elapsed_time;
    if token_level.total_exp > max_experience {
        token_level.total_exp = max_experience;
    }
}

pub fn update_leveling_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    leveling_open: bool,
    max_experience: u64,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut leveling_config = LEVELING_CONFIG.load(deps.storage)?;
    leveling_config.leveling_open = leveling_open;
    leveling_config.max_experience = max_experience;
    LEVELING_CONFIG.save(deps.storage, &leveling_config)?;

    // If turning off leveling, turn off leveling for all tokens
    if !leveling_open {
        // TODO: There might be a better way to handle this
        let tokens_to_update: StdResult<Vec<(String, TokenLevelResponse)>> = TOKEN_LEVELS
            .range(deps.storage, None, None, Order::Ascending)
            .filter(|item| {
                // Only consider tokens that are currently leveling
                match item {
                    Ok((_, token_level)) => token_level.leveling,
                    _ => false,
                }
            })
            .collect();
        for item in tokens_to_update? {
            let (token_id, mut token_level) = item;
            if token_level.leveling {
                token_level.leveling = false;
                calculate_and_cap_experience(
                    &mut token_level,
                    env.block.time.seconds(),
                    leveling_config.max_experience,
                );
                token_level.leveling_start_timestamp = 0;
                TOKEN_LEVELS.save(deps.storage, &token_id, &token_level)?;
            }
        }
    }

    let res = Response::new()
        .add_attribute("action", "update_leveling_config")
        .add_attribute("leveling_open", leveling_open.to_string())
        .add_attribute("max_experience", max_experience.to_string());
    Ok(res)
}

pub fn toggle_leveling(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    // Verify that this token is owned by the sender
    let contract = Cw2981LevelingContract::default();
    let token_info = contract.tokens.load(deps.storage, &token_id)?;
    if token_info.owner != info.sender {
        return Err(ContractError::UnauthorizedLeveling {});
    }

    // Get leveling status
    let leveling_config = LEVELING_CONFIG.load(deps.storage)?;
    let mut token_level =
        TOKEN_LEVELS
            .may_load(deps.storage, &token_id)?
            .unwrap_or(TokenLevelResponse {
                leveling: false,
                leveling_start_timestamp: 0,
                total_exp: 0,
            });

    if token_level.leveling {
        // Turn off leveling
        token_level.leveling = false;
        calculate_and_cap_experience(
            &mut token_level,
            env.block.time.seconds(),
            leveling_config.max_experience,
        );
        token_level.leveling_start_timestamp = 0;
    } else {
        if !leveling_config.leveling_open {
            return Err(ContractError::LevelingNotOpen {});
        }
        // Turn on leveling
        token_level.leveling = true;
        token_level.leveling_start_timestamp = env.block.time.seconds();
    }

    TOKEN_LEVELS.save(deps.storage, &token_id, &token_level)?;

    Ok(Response::new()
        .add_attribute("action", "toggle_leveling")
        .add_attribute("token_id", token_id)
        .add_attribute("leveling_status", token_level.leveling.to_string()))
}
