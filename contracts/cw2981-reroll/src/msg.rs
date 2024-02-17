use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, CustomMsg, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    /// Name of the NFT contract
    pub name: String,
    /// Symbol of the NFT contract
    pub symbol: String,
    /// The minter is the only one who can create new NFTs.
    pub minter: Option<String>,
    pub withdraw_address: Option<String>,
    pub base_token_uri: Option<String>,
    // Royalty info
    pub royalty_percentage: u64,
    pub royalty_payment_address: Addr,
}

#[cw_serde]
pub enum Cw2981RerollExecuteMsg {
    /// Updates royalty info for the contract
    UpdateRoyaltyConfig {
        royalty_percentage: u64,
        royalty_payment_address: Addr,
    },
}

impl CustomMsg for Cw2981RerollExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum Cw2981RerollQueryMsg {
    /// Returns contract config
    #[returns(RoyaltyConfigResponse)]
    RoyaltyConfig {},
    /// Should be called on sale to see if royalties are owed
    /// by the marketplace selling the NFT, if CheckRoyalties
    /// returns true
    /// See https://eips.ethereum.org/EIPS/eip-2981
    #[returns(RoyaltiesInfoResponse)]
    RoyaltyInfo {
        token_id: String,
        // the denom of this sale must also be the denom returned by RoyaltiesInfoResponse
        // this was originally implemented as a Coin
        // however that would mean you couldn't buy using CW20s
        // as CW20 is just mapping of addr -> balance
        sale_price: Uint128,
    },
    /// Called against contract to determine if this NFT
    /// implements royalties. Should return a boolean as part of
    /// CheckRoyaltiesResponse - default can simply be true
    /// if royalties are implemented at token level
    /// (i.e. always check on sale)
    #[returns(CheckRoyaltiesResponse)]
    CheckRoyalties {},
}

impl Default for Cw2981RerollQueryMsg {
    fn default() -> Self {
        Cw2981RerollQueryMsg::CheckRoyalties {}
    }
}

impl CustomMsg for Cw2981RerollQueryMsg {}

#[cw_serde]
pub struct RoyaltyConfigResponse {
    pub royalty_percentage: u64,
    pub royalty_payment_address: Addr,
}

#[cw_serde]
pub struct RoyaltiesInfoResponse {
    pub address: String,
    // Note that this must be the same denom as that passed in to RoyaltyInfo
    // rounding up or down is at the discretion of the implementer
    pub royalty_amount: Uint128,
}

/// Shows if the contract implements royalties
/// if royalty_payments is true, marketplaces should pay them
#[cw_serde]
pub struct CheckRoyaltiesResponse {
    pub royalty_payments: bool,
}
