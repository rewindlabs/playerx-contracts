use crate::msg::{CheckRoyaltiesResponse, RoyaltiesInfoResponse, RoyaltyConfigResponse};
use crate::state::ROYALTY_CONFIG;
use crate::Cw2981RerollContract;
use cosmwasm_std::{Decimal, Deps, StdResult, Uint128};

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
    let contract = Cw2981RerollContract::default();
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
