use cosmwasm_std::StdError;
use cw_ownable::OwnershipError;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Base(#[from] cw721_reroll::ContractError),

    #[error(transparent)]
    Ownership(#[from] OwnershipError),

    #[error("Royalty percentage must be between 0 and 100")]
    InvalidRoyaltyPercentage,

    #[error("Account is not authorized to toggle leveling for this token")]
    UnauthorizedLeveling,

    #[error("Leveling is not open")]
    LevelingNotOpen,
}
