use crate::{error::ContractError, state::ROYALTY_CONFIG};
use cosmwasm_std::{Addr, DepsMut, MessageInfo, Response};

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
