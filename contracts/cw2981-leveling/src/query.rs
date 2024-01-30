use crate::msg::{
    AllTokenLevelsResponse, CheckRoyaltiesResponse, LevelingConfigResponse, RoyaltiesInfoResponse,
    RoyaltyConfigResponse, TokenLevelResponse,
};
use crate::state::{LEVELING_CONFIG, ROYALTY_CONFIG, TOKEN_LEVELS};
use crate::Cw2981LevelingContract;
use cosmwasm_std::{Decimal, Deps, Order, StdResult, Uint128};
use cw_storage_plus::Bound;

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 1000;

pub fn query_royalty_config(deps: Deps) -> StdResult<RoyaltyConfigResponse> {
    let royalty_config = ROYALTY_CONFIG.load(deps.storage)?;
    Ok(RoyaltyConfigResponse {
        royalty_payment_address: royalty_config.royalty_payment_address,
        royalty_percentage: royalty_config.royalty_percentage,
    })
}

/// NOTE: default behaviour here is to round down
/// EIP2981 specifies that the rounding behaviour is at the discretion of the implementer
pub fn query_royalties_info(
    deps: Deps,
    token_id: String,
    sale_price: Uint128,
) -> StdResult<RoyaltiesInfoResponse> {
    let contract = Cw2981LevelingContract::default();
    // Will cause query to fail if token_id doesn't exist
    let _token_info = contract.tokens.load(deps.storage, &token_id)?;

    let royalty_config = ROYALTY_CONFIG.load(deps.storage)?;
    let royalty_percentage = Decimal::percent(royalty_config.royalty_percentage);
    let royalty_from_sale_price = sale_price * royalty_percentage;
    let royalty_address = royalty_config.royalty_payment_address;

    Ok(RoyaltiesInfoResponse {
        address: royalty_address.into_string(),
        royalty_amount: royalty_from_sale_price,
    })
}

/// As our default implementation here specifies royalties at contract level
/// this will always be true
pub fn check_royalties(_deps: Deps) -> StdResult<CheckRoyaltiesResponse> {
    Ok(CheckRoyaltiesResponse {
        royalty_payments: true,
    })
}

pub fn query_leveling_config(deps: Deps) -> StdResult<LevelingConfigResponse> {
    let leveling_config = LEVELING_CONFIG.load(deps.storage)?;
    Ok(LevelingConfigResponse {
        leveling_open: leveling_config.leveling_open,
        max_experience: leveling_config.max_experience,
    })
}

pub fn query_token_level(deps: Deps, token_id: String) -> StdResult<TokenLevelResponse> {
    let token_level =
        TOKEN_LEVELS
            .may_load(deps.storage, &token_id)?
            .unwrap_or(TokenLevelResponse {
                leveling: false,
                leveling_start_timestamp: 0,
                total_exp: 0,
            });
    Ok(token_level)
}

pub fn query_all_token_levels(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<AllTokenLevelsResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_bound = start_after.as_deref().map(Bound::exclusive);

    let token_levels: StdResult<Vec<(String, TokenLevelResponse)>> = TOKEN_LEVELS
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (key, token_level) = item?;
            Ok((key, token_level))
        })
        .collect();

    Ok(AllTokenLevelsResponse {
        token_levels: token_levels?,
    })
}
