use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Coin, Uint128};
use cw721::Expiration;
use cw_ownable::{cw_ownable_execute, cw_ownable_query};
use schemars::JsonSchema;

#[cw_serde]
pub struct InstantiateMsg {
    /// Name of the NFT contract
    pub name: String,
    /// Symbol of the NFT contract
    pub symbol: String,
    // Base token URI of the contract
    pub base_token_uri: String,
    // Collection size
    pub collection_size: u64,
    // Sale config
    pub og_price: Uint128,
    pub allowlist_price: Uint128,
    pub public_price: Uint128,
    pub max_per_og: u64,
    pub max_per_allowlist: u64,
    pub max_per_public: u64,
}

/// This is like Cw721ExecuteMsg but we add a few mint configs and functions
#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg<T, E> {
    /// Transfer is a base message to move a token to another account without triggering actions
    TransferNft { recipient: String, token_id: String },
    /// Send is a base message to transfer a token to a contract and trigger an action
    /// on the receiving contract.
    SendNft {
        contract: String,
        token_id: String,
        msg: Binary,
    },
    /// Allows operator to transfer / send the token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted Approval
    Revoke { spender: String, token_id: String },
    /// Allows operator to transfer / send any token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted ApproveAll permission
    RevokeAll { operator: String },

    /// Mint for the team
    MintTeam { quantity: u64, extension: T },

    /// Mint for og
    MintOg { quantity: u64, extension: T },

    /// Mint for allowlisted addresses
    MintAllowlist { quantity: u64, extension: T },

    /// Mint for public
    MintPublic { quantity: u64, extension: T },

    /// DO NOT USE: This mint is disabled
    Mint {
        /// Unique ID of the NFT
        token_id: String,
        /// The owner of the newly minter NFT
        owner: String,
        /// Universal resource identifier for this NFT
        /// Should point to a JSON file that conforms to the ERC721
        /// Metadata JSON Schema
        token_uri: Option<String>,
        /// Any custom extension used by this contract
        extension: T,
    },

    /// Burn an NFT the sender has access to
    Burn { token_id: String },

    /// Extension msg
    Extension { msg: E },

    /// Sets address to send withdrawn fees to. Only owner can call this.
    SetWithdrawAddress { address: String },
    /// Removes the withdraw address, so fees are sent to the contract. Only owner can call this.
    RemoveWithdrawAddress {},
    /// Withdraw from the contract to the given address. Anyone can call this,
    /// which is okay since withdraw address has been set by owner.
    WithdrawFunds { amount: Coin },
    /// Sets the base_token_uri for the contract
    SetBaseTokenUri { base_token_uri: String },
    // /// Sets the sale config
    SetSaleConfig {
        og_price: Uint128,
        allowlist_price: Uint128,
        public_price: Uint128,
        max_per_og: u64,
        max_per_allowlist: u64,
        max_per_public: u64,
    },
    /// Add addresses to allowlist
    AddToAllowlist { addresses: Vec<String> },
    /// Remove addresses from allowlist
    RemoveFromAllowlist { addresses: Vec<String> },
    /// Add addresses to og list
    AddToOgList { addresses: Vec<String> },
    /// Remove addresses from allowlist
    RemoveFromOgList { addresses: Vec<String> },
    /// Sets state of allowlist sale
    SetAllowlistSale { open: bool },
    /// Sets state of allowlist sale
    SetOgSale { open: bool },
    /// Sets state of public sale
    SetPublicSale { open: bool },
    /// Sets collection size
    SetCollectionSize { collection_size: u64 },
}

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg<Q: JsonSchema> {
    /// Return the owner of the given token, error if token does not exist
    #[returns(cw721::OwnerOfResponse)]
    OwnerOf {
        token_id: String,
        /// unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },
    /// Return operator that can access all of the owner's tokens.
    #[returns(cw721::ApprovalResponse)]
    Approval {
        token_id: String,
        spender: String,
        include_expired: Option<bool>,
    },
    /// Return approvals that a token has
    #[returns(cw721::ApprovalsResponse)]
    Approvals {
        token_id: String,
        include_expired: Option<bool>,
    },
    /// Return approval of a given operator for all tokens of an owner, error if not set
    #[returns(cw721::OperatorResponse)]
    Operator {
        owner: String,
        operator: String,
        include_expired: Option<bool>,
    },
    /// List all operators that can access all of the owner's tokens
    #[returns(cw721::OperatorsResponse)]
    AllOperators {
        owner: String,
        /// unset or false will filter out expired items, you must set to true to see them
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// Total number of tokens issued
    #[returns(cw721::NumTokensResponse)]
    NumTokens {},

    /// With MetaData Extension.
    /// Returns top-level metadata about the contract
    #[returns(cw721::ContractInfoResponse)]
    ContractInfo {},
    /// With MetaData Extension.
    /// Returns metadata about one particular token, based on *ERC721 Metadata JSON Schema*
    /// but directly from the contract
    #[returns(cw721::NftInfoResponse<Q>)]
    NftInfo { token_id: String },
    /// With MetaData Extension.
    /// Returns the result of both `NftInfo` and `OwnerOf` as one query as an optimization
    /// for clients
    #[returns(cw721::AllNftInfoResponse<Q>)]
    AllNftInfo {
        token_id: String,
        /// unset or false will filter out expired approvals, you must set to true to see them
        include_expired: Option<bool>,
    },

    /// With Enumerable extension.
    /// Returns all tokens owned by the given address, [] if unset.
    #[returns(cw721::TokensResponse)]
    Tokens {
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    /// With Enumerable extension.
    /// Requires pagination. Lists all token_ids controlled by the contract.
    #[returns(cw721::TokensResponse)]
    AllTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },

    /// Return the admin
    #[returns(AdminResponse)]
    Admin {},

    /// Extension query
    #[returns(())]
    Extension { msg: Q },

    #[returns(Option<String>)]
    WithdrawAddress {},

    /// Returns collection size
    #[returns(CollectionSizeResponse)]
    CollectionSize {},

    /// Returns base token uri
    #[returns(BaseTokenUriResponse)]
    BaseTokenUri {},

    /// Returns sale config
    #[returns(SaleConfigResponse)]
    SaleConfig {},
}

/// Shows who can mint these tokens
#[cw_serde]
pub struct AdminResponse {
    pub admin: Option<String>,
}

#[cw_serde]
pub struct CollectionSizeResponse {
    pub collection_size: u64,
}

#[cw_serde]
pub struct BaseTokenUriResponse {
    pub base_token_uri: String,
}

#[cw_serde]
pub struct SaleConfigResponse {
    pub og_price: Uint128,
    pub allowlist_price: Uint128,
    pub public_price: Uint128,
    pub max_per_og: u64,
    pub max_per_allowlist: u64,
    pub max_per_public: u64,
    pub og_sale_open: bool,
    pub allowlist_sale_open: bool,
    pub public_sale_open: bool,
}
