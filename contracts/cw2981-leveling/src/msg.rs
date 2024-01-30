use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, CustomMsg, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    /// Name of the NFT contract
    pub name: String,
    /// Symbol of the NFT contract
    pub symbol: String,
    // Token URI
    pub base_token_uri: String,
    // Collection size
    pub collection_size: u64,
    // Royalty info
    pub royalty_percentage: u64,
    pub royalty_payment_address: Addr,
    // Sale config
    pub allowlist_price: Uint128,
    pub public_price: Uint128,
    pub max_per_allowlist: u64,
    pub max_per_public: u64,
}

#[cw_serde]
pub enum Cw2981LevelingExecuteMsg {
    /// Updates royalty info for the contract
    UpdateRoyaltyConfig {
        royalty_percentage: u64,
        royalty_payment_address: Addr,
    },
    /// Update leveling config for the contract
    UpdateLevelingConfig {
        leveling_open: bool,
        max_experience: u64,
    },
    /// Toggles leveling for the token id
    ToggleLeveling { token_id: String },
    /// Grants bonus experience to tokens
    GrantBonusExperience {
        token_ids: Vec<String>,
        experience: u64,
    },
}

impl CustomMsg for Cw2981LevelingExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum Cw2981LevelingQueryMsg {
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
    /// Returns leveling config
    #[returns(LevelingConfigResponse)]
    LevelingConfig {},
    /// Returns token level info
    #[returns(TokenLevelResponse)]
    TokenLevel { token_id: String },
    /// Returns token levels for all tokens paginated
    #[returns(AllTokenLevelsResponse)]
    AllTokenLevels {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

impl Default for Cw2981LevelingQueryMsg {
    fn default() -> Self {
        Cw2981LevelingQueryMsg::CheckRoyalties {}
    }
}

impl CustomMsg for Cw2981LevelingQueryMsg {}

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

#[cw_serde]
pub struct LevelingConfigResponse {
    pub leveling_open: bool,
    pub max_experience: u64,
}

#[cw_serde]
pub struct TokenLevelResponse {
    pub leveling: bool,
    pub leveling_start_timestamp: u64,
    pub total_exp: u64,
}

#[cw_serde]
pub struct AllTokenLevelsResponse {
    pub token_levels: Vec<(String, TokenLevelResponse)>,
}
