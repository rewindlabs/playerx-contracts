use cosmwasm_std::StdError;
use cw_ownable::OwnershipError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error(transparent)]
    Ownership(#[from] OwnershipError),

    #[error(transparent)]
    Version(#[from] cw2::VersionError),

    #[error("token_id already claimed")]
    Claimed {},

    #[error("Cannot set approval that is already expired")]
    Expired {},

    #[error("Approval not found for: {spender}")]
    ApprovalNotFound { spender: String },

    #[error("No withdraw address set")]
    NoWithdrawAddress {},

    #[error("Invalid quantity")]
    InvalidQuantity {},

    #[error("Standard minting is disabled")]
    MintDisabled,

    #[error("Allowlist sale is closed")]
    AllowlistSaleClosed {},

    #[error("Public sale is closed")]
    PublicSaleClosed {},

    #[error("Not on allowlist")]
    NotOnAllowlist {},

    #[error("Max supply reached")]
    MaxSupplyReached {},

    #[error("Cannot mint this many")]
    MaxMintReached {},

    #[error("Insufficient funds sent")]
    InsufficientFunds {},

    #[error("Burning is disabled")]
    BurnDisabled {},
}
