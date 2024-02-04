#![cfg(test)]
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

use cosmwasm_std::{
    coin, coins, from_json, to_json_binary, Addr, BankMsg, Coin, CosmosMsg, DepsMut, Empty,
    Response, StdError, Uint128, WasmMsg,
};

use cw721::{
    AllNftInfoResponse, Approval, ApprovalResponse, ContractInfoResponse, Cw721Query,
    Cw721ReceiveMsg, Expiration, NftInfoResponse, OperatorResponse, OperatorsResponse,
    OwnerOfResponse,
};
use cw_ownable::OwnershipError;

use crate::msg::{AdminResponse, BaseTokenUriResponse, CollectionSizeResponse, SaleConfigResponse};
use crate::{ContractError, Cw721Contract, ExecuteMsg, Extension, InstantiateMsg, QueryMsg};

const ADMIN: &str = "creator";
const CONTRACT_NAME: &str = "PlayerX";
const SYMBOL: &str = "PX";
const BASE_TOKEN_URI: &str = "base_token_uri";
const COLLECTION_SIZE: u64 = 100;
const MAX_PER_PUBLIC: u64 = 5;
const MAX_PER_ALLOWLIST: u64 = 1;
const MAX_PER_OG: u64 = 1;
const PUBLIC_PRICE: u64 = 100000;
const ALLOWLIST_PRICE: u64 = 100000;
const OG_PRICE: u64 = 100000;

fn setup_contract(deps: DepsMut<'_>) -> Cw721Contract<'static, Extension, Empty, Empty, Empty> {
    let contract = Cw721Contract::default();
    let msg = InstantiateMsg {
        name: CONTRACT_NAME.to_string(),
        symbol: SYMBOL.to_string(),
        base_token_uri: BASE_TOKEN_URI.to_string(),
        collection_size: COLLECTION_SIZE,
        max_per_public: MAX_PER_PUBLIC,
        max_per_allowlist: MAX_PER_ALLOWLIST,
        max_per_og: MAX_PER_OG,
        public_price: Uint128::from(PUBLIC_PRICE),
        allowlist_price: Uint128::from(ALLOWLIST_PRICE),
        og_price: Uint128::from(OG_PRICE),
    };
    let info = mock_info(ADMIN, &[]);
    let res = contract.instantiate(deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    contract
}

#[test]
fn proper_instantiation() {
    let mut deps = mock_dependencies();
    let contract = Cw721Contract::<Extension, Empty, Empty, Empty>::default();

    let msg = InstantiateMsg {
        name: CONTRACT_NAME.to_string(),
        symbol: SYMBOL.to_string(),
        base_token_uri: BASE_TOKEN_URI.to_string(),
        collection_size: COLLECTION_SIZE,
        max_per_public: MAX_PER_PUBLIC,
        max_per_allowlist: MAX_PER_ALLOWLIST,
        max_per_og: MAX_PER_OG,
        public_price: Uint128::from(PUBLIC_PRICE),
        allowlist_price: Uint128::from(ALLOWLIST_PRICE),
        og_price: Uint128::from(OG_PRICE),
    };
    let info = mock_info(ADMIN, &[]);

    // we can just call .unwrap() to assert this was a success
    let res = contract
        .instantiate(deps.as_mut(), mock_env(), info, msg)
        .unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = contract.admin(deps.as_ref()).unwrap();
    assert_eq!(Some(ADMIN.to_string()), res.admin);
    let info = contract.contract_info(deps.as_ref()).unwrap();
    assert_eq!(
        info,
        ContractInfoResponse {
            name: CONTRACT_NAME.to_string(),
            symbol: SYMBOL.to_string(),
        }
    );

    let withdraw_address = contract
        .withdraw_address
        .may_load(deps.as_ref().storage)
        .unwrap();
    assert_eq!(Some(ADMIN.to_string()), withdraw_address);

    let count = contract.num_tokens(deps.as_ref()).unwrap();
    assert_eq!(0, count.count);

    // list the token_ids
    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    assert_eq!(0, tokens.tokens.len());
}

#[test]
fn update_sale_config() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    // Verify current sales config
    let expected = SaleConfigResponse {
        max_per_public: MAX_PER_PUBLIC,
        max_per_allowlist: MAX_PER_ALLOWLIST,
        max_per_og: MAX_PER_OG,
        public_price: Uint128::from(PUBLIC_PRICE),
        allowlist_price: Uint128::from(ALLOWLIST_PRICE),
        og_price: Uint128::from(OG_PRICE),
        public_sale_open: false,
        allowlist_sale_open: false,
        og_sale_open: false,
    };
    let sale_config: SaleConfigResponse = from_json(
        contract
            .query(deps.as_ref(), mock_env(), QueryMsg::SaleConfig {})
            .unwrap(),
    )
    .unwrap();
    assert_eq!(expected, sale_config);

    // Random can't update
    let msg = ExecuteMsg::SetSaleConfig {
        og_price: Uint128::from(200000u64),
        allowlist_price: Uint128::from(200000u64),
        public_price: Uint128::from(200000u64),
        max_per_og: 2,
        max_per_allowlist: 2,
        max_per_public: 10,
    };
    let random = mock_info("random", &[]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), random.clone(), msg.clone())
        .unwrap_err();
    assert_eq!(err, ContractError::Ownership(OwnershipError::NotOwner));

    // Update and verify new sales config
    let info = mock_info(ADMIN, &[]);
    contract
        .execute(deps.as_mut(), mock_env(), info, msg)
        .unwrap();

    let expected = SaleConfigResponse {
        og_price: Uint128::from(200000u64),
        allowlist_price: Uint128::from(200000u64),
        public_price: Uint128::from(200000u64),
        max_per_og: 2,
        max_per_allowlist: 2,
        max_per_public: 10,
        public_sale_open: false,
        allowlist_sale_open: false,
        og_sale_open: false,
    };
    let sale_config: SaleConfigResponse = from_json(
        contract
            .query(deps.as_ref(), mock_env(), QueryMsg::SaleConfig {})
            .unwrap(),
    )
    .unwrap();
    assert_eq!(expected, sale_config);
}

#[test]
fn update_sale_state() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    // Update allowlist sale and verify
    let info = mock_info(ADMIN, &[]);
    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::SetAllowlistSale { open: true },
        )
        .unwrap();
    let expected = SaleConfigResponse {
        max_per_og: MAX_PER_OG,
        max_per_public: MAX_PER_PUBLIC,
        max_per_allowlist: MAX_PER_ALLOWLIST,
        og_price: Uint128::from(OG_PRICE),
        public_price: Uint128::from(PUBLIC_PRICE),
        allowlist_price: Uint128::from(ALLOWLIST_PRICE),
        public_sale_open: false,
        allowlist_sale_open: true,
        og_sale_open: false,
    };
    let sale_config: SaleConfigResponse = from_json(
        contract
            .query(deps.as_ref(), mock_env(), QueryMsg::SaleConfig {})
            .unwrap(),
    )
    .unwrap();
    assert_eq!(expected, sale_config);

    // Update public sale
    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::SetPublicSale { open: true },
        )
        .unwrap();
    let expected = SaleConfigResponse {
        max_per_og: MAX_PER_OG,
        max_per_public: MAX_PER_PUBLIC,
        max_per_allowlist: MAX_PER_ALLOWLIST,
        og_price: Uint128::from(OG_PRICE),
        public_price: Uint128::from(PUBLIC_PRICE),
        allowlist_price: Uint128::from(ALLOWLIST_PRICE),
        og_sale_open: false,
        public_sale_open: true,
        allowlist_sale_open: true,
    };
    let sale_config: SaleConfigResponse = from_json(
        contract
            .query(deps.as_ref(), mock_env(), QueryMsg::SaleConfig {})
            .unwrap(),
    )
    .unwrap();
    assert_eq!(expected, sale_config);

    // Update og sale
    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::SetOgSale { open: true },
        )
        .unwrap();
    let expected = SaleConfigResponse {
        max_per_og: MAX_PER_OG,
        max_per_public: MAX_PER_PUBLIC,
        max_per_allowlist: MAX_PER_ALLOWLIST,
        og_price: Uint128::from(OG_PRICE),
        public_price: Uint128::from(PUBLIC_PRICE),
        allowlist_price: Uint128::from(ALLOWLIST_PRICE),
        og_sale_open: true,
        public_sale_open: true,
        allowlist_sale_open: true,
    };
    let sale_config: SaleConfigResponse = from_json(
        contract
            .query(deps.as_ref(), mock_env(), QueryMsg::SaleConfig {})
            .unwrap(),
    )
    .unwrap();
    assert_eq!(expected, sale_config);

    // Random can't update
    let random = mock_info("random", &[]);
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random.clone(),
            ExecuteMsg::SetAllowlistSale { open: false },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::Ownership(OwnershipError::NotOwner));
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random.clone(),
            ExecuteMsg::SetPublicSale { open: false },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::Ownership(OwnershipError::NotOwner));
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random,
            ExecuteMsg::SetOgSale { open: false },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::Ownership(OwnershipError::NotOwner));
}

#[test]
fn update_base_token_uri() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    // Check base token uri
    let expected = BaseTokenUriResponse {
        base_token_uri: BASE_TOKEN_URI.to_string(),
    };
    let base_token_uri: BaseTokenUriResponse = from_json(
        contract
            .query(deps.as_ref(), mock_env(), QueryMsg::BaseTokenUri {})
            .unwrap(),
    )
    .unwrap();
    assert_eq!(expected, base_token_uri);

    // Random can't update
    let new_base_token_uri = "new_base_token_uri";
    let msg = ExecuteMsg::SetBaseTokenUri {
        base_token_uri: new_base_token_uri.to_string(),
    };
    let random = mock_info("random", &[]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), random, msg.clone())
        .unwrap_err();
    assert_eq!(err, ContractError::Ownership(OwnershipError::NotOwner));

    // Update and verify
    let info = mock_info(ADMIN, &[]);
    contract
        .execute(deps.as_mut(), mock_env(), info.clone(), msg)
        .unwrap();
    let expected = BaseTokenUriResponse {
        base_token_uri: new_base_token_uri.to_string(),
    };
    let base_token_uri: BaseTokenUriResponse = from_json(
        contract
            .query(deps.as_ref(), mock_env(), QueryMsg::BaseTokenUri {})
            .unwrap(),
    )
    .unwrap();
    assert_eq!(expected, base_token_uri);
}

#[test]
fn update_collection_size() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    // Check collection size
    let collection_size: CollectionSizeResponse = from_json(
        contract
            .query(deps.as_ref(), mock_env(), QueryMsg::CollectionSize {})
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        collection_size,
        CollectionSizeResponse {
            collection_size: COLLECTION_SIZE
        }
    );

    // Mint a few tokens
    let mint_msg = ExecuteMsg::MintTeam {
        quantity: 10,
        extension: None,
    };
    let admin = mock_info(ADMIN, &[]);
    let _ = contract
        .execute(deps.as_mut(), mock_env(), admin.clone(), mint_msg)
        .unwrap();

    // Random can't update
    let random = mock_info("random", &[]);
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random,
            ExecuteMsg::SetCollectionSize {
                collection_size: 50,
            },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::Ownership(OwnershipError::NotOwner));

    // Admin can't update to number lower than amount minted
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin.clone(),
            ExecuteMsg::SetCollectionSize { collection_size: 9 },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::InvalidCollectionSize {});

    // Admin can update
    let _ = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin.clone(),
            ExecuteMsg::SetCollectionSize {
                collection_size: 20,
            },
        )
        .unwrap();
    let collection_size: CollectionSizeResponse = from_json(
        contract
            .query(deps.as_ref(), mock_env(), QueryMsg::CollectionSize {})
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        collection_size,
        CollectionSizeResponse {
            collection_size: 20
        }
    );
}

#[test]
fn mint_team() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    let mint_msg = ExecuteMsg::MintTeam {
        quantity: 10,
        extension: None,
    };

    // random cannot mint
    let random = mock_info("random", &[]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), random, mint_msg.clone())
        .unwrap_err();
    assert_eq!(err, ContractError::Ownership(OwnershipError::NotOwner));

    // can't mint 0
    let allowed = mock_info(ADMIN, &[]);
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            allowed.clone(),
            ExecuteMsg::MintTeam {
                quantity: 0,
                extension: None,
            },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::InvalidQuantity {});

    // admin can mint
    let allowed = mock_info(ADMIN, &[]);
    let _ = contract
        .execute(deps.as_mut(), mock_env(), allowed.clone(), mint_msg)
        .unwrap();

    // ensure num tokens increases
    let count = contract.num_tokens(deps.as_ref()).unwrap();
    assert_eq!(10, count.count);

    // ensure nft info is correct
    let info = contract.nft_info(deps.as_ref(), "0".to_string()).unwrap();
    let expected_token_uri = format!("{}/{}", BASE_TOKEN_URI, "0");
    assert_eq!(
        info,
        NftInfoResponse::<Extension> {
            token_uri: Some(expected_token_uri),
            extension: None,
        }
    );

    let all_info: AllNftInfoResponse<Option<Empty>> = contract
        .all_nft_info(deps.as_ref(), mock_env(), "8".to_string(), true)
        .unwrap();
    let expected_token_uri = format!("{}/{}", BASE_TOKEN_URI, "8");
    assert_eq!(
        all_info,
        AllNftInfoResponse::<Extension> {
            access: OwnerOfResponse {
                owner: ADMIN.to_string(),
                approvals: vec![],
            },
            info: NftInfoResponse {
                token_uri: Some(expected_token_uri),
                extension: None,
            }
        }
    );

    // ensure owner info is correct
    let owner = contract
        .owner_of(deps.as_ref(), mock_env(), "0".to_string(), true)
        .unwrap();
    assert_eq!(
        owner,
        OwnerOfResponse {
            owner: ADMIN.to_string(),
            approvals: vec![],
        }
    );

    // can't mint past collection size
    let mint_msg = ExecuteMsg::MintTeam {
        quantity: 91,
        extension: None,
    };
    let err = contract
        .execute(deps.as_mut(), mock_env(), allowed, mint_msg)
        .unwrap_err();
    assert_eq!(err, ContractError::MaxSupplyReached {});
}

#[test]
fn mint_og() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    // Open og minting
    let admin = mock_info(ADMIN, &[]);
    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin.clone(),
            ExecuteMsg::SetOgSale { open: true },
        )
        .unwrap();

    // random can't mint if not og
    let random = mock_info("random", &[]);
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random.clone(),
            ExecuteMsg::MintOg {
                quantity: 1,
                extension: None,
            },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::NotOnOgList {});

    // random can't add to og list
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random.clone(),
            ExecuteMsg::AddToOgList {
                addresses: vec!["random".to_string()],
            },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::Ownership(OwnershipError::NotOwner));

    // Admin can add to og list
    let _ = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin.clone(),
            ExecuteMsg::AddToOgList {
                addresses: vec!["random".to_string(), "user".to_string()],
            },
        )
        .unwrap();

    // Random can't mint more than max
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random.clone(),
            ExecuteMsg::MintOg {
                quantity: 2,
                extension: None,
            },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::MaxMintReached {});

    // Cannot mint without funds
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random,
            ExecuteMsg::MintOg {
                quantity: 1,
                extension: None,
            },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::InsufficientFunds {});

    // Successful mint
    let funds = coins(100000, "usei");
    let random_with_funds = mock_info("random", &funds);
    let _ = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random_with_funds.clone(),
            ExecuteMsg::MintOg {
                quantity: 1,
                extension: None,
            },
        )
        .unwrap();

    // ensure num tokens increases
    let count = contract.num_tokens(deps.as_ref()).unwrap();
    assert_eq!(1, count.count);

    // Can't mint again
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random_with_funds,
            ExecuteMsg::MintOg {
                quantity: 1,
                extension: None,
            },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::NotOnOgList {});

    // Remove user from allowlist
    let _ = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin,
            ExecuteMsg::RemoveFromOgList {
                addresses: vec!["user".to_string()],
            },
        )
        .unwrap();

    // User can't mint
    let user = mock_info("user", &[]);
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            user,
            ExecuteMsg::MintOg {
                quantity: 1,
                extension: None,
            },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::NotOnOgList {});
}

#[test]
fn mint_allowlist() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    // Open allowlist minting
    let admin = mock_info(ADMIN, &[]);
    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin.clone(),
            ExecuteMsg::SetAllowlistSale { open: true },
        )
        .unwrap();

    // random can't mint if not allowlisted
    let random = mock_info("random", &[]);
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random.clone(),
            ExecuteMsg::MintAllowlist {
                quantity: 1,
                extension: None,
            },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::NotOnAllowlist {});

    // random can't add to allowlist
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random.clone(),
            ExecuteMsg::AddToAllowlist {
                addresses: vec!["random".to_string()],
            },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::Ownership(OwnershipError::NotOwner));

    // Admin can add to allowlist
    let _ = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin.clone(),
            ExecuteMsg::AddToAllowlist {
                addresses: vec!["random".to_string(), "user".to_string()],
            },
        )
        .unwrap();

    // Random can't mint more than max
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random.clone(),
            ExecuteMsg::MintAllowlist {
                quantity: 2,
                extension: None,
            },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::MaxMintReached {});

    // Cannot mint without funds
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random,
            ExecuteMsg::MintAllowlist {
                quantity: 1,
                extension: None,
            },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::InsufficientFunds {});

    // Successful mint
    let funds = coins(100000, "usei");
    let random_with_funds = mock_info("random", &funds);
    let _ = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random_with_funds.clone(),
            ExecuteMsg::MintAllowlist {
                quantity: 1,
                extension: None,
            },
        )
        .unwrap();

    // ensure num tokens increases
    let count = contract.num_tokens(deps.as_ref()).unwrap();
    assert_eq!(1, count.count);

    // Can't mint again
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random_with_funds,
            ExecuteMsg::MintAllowlist {
                quantity: 1,
                extension: None,
            },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::NotOnAllowlist {});

    // Remove user from allowlist
    let _ = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin,
            ExecuteMsg::RemoveFromAllowlist {
                addresses: vec!["user".to_string()],
            },
        )
        .unwrap();

    // User can't mint
    let user = mock_info("user", &[]);
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            user,
            ExecuteMsg::MintAllowlist {
                quantity: 1,
                extension: None,
            },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::NotOnAllowlist {});
}

#[test]
fn mint_public() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    // Open public minting
    let admin = mock_info(ADMIN, &[]);
    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin.clone(),
            ExecuteMsg::SetPublicSale { open: true },
        )
        .unwrap();

    // random can't mint over the max limit
    let random = mock_info("random", &[]);
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random,
            ExecuteMsg::MintPublic {
                quantity: 6,
                extension: None,
            },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::MaxMintReached {});

    // can't mint without funds
    let funds = coins(100000, "ibc/test");
    let random = mock_info("random", &funds);
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random,
            ExecuteMsg::MintPublic {
                quantity: 5,
                extension: None,
            },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::InsufficientFunds {});

    let funds = coins(500000, "usei");
    let random = mock_info("random", &funds);
    let _ = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random.clone(),
            ExecuteMsg::MintPublic {
                quantity: 5,
                extension: None,
            },
        )
        .unwrap();

    // ensure num tokens increases
    let count = contract.num_tokens(deps.as_ref()).unwrap();
    assert_eq!(5, count.count);

    // can't mint past collection size
    let mint_msg = ExecuteMsg::MintTeam {
        quantity: 95,
        extension: None,
    };
    let _ = contract
        .execute(deps.as_mut(), mock_env(), admin, mint_msg)
        .unwrap();
    let count = contract.num_tokens(deps.as_ref()).unwrap();
    assert_eq!(100, count.count);

    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random,
            ExecuteMsg::MintPublic {
                quantity: 1,
                extension: None,
            },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::MaxSupplyReached {});
}

#[test]
fn minting() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    // Standard mint fails
    let token_id = "1".to_string();
    let token_uri = "token_uri".to_string();

    let mint_msg = ExecuteMsg::Mint {
        token_id: token_id.clone(),
        owner: String::from("test"),
        token_uri: Some(token_uri.clone()),
        extension: None,
    };

    // random cannot mint
    let random = mock_info("random", &[]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), random, mint_msg.clone())
        .unwrap_err();
    assert_eq!(err, ContractError::MintDisabled);

    // admin can't mint
    let admin = mock_info(ADMIN, &[]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), admin, mint_msg)
        .unwrap_err();
    assert_eq!(err, ContractError::MintDisabled);
}

#[test]
fn update_admin() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    // Update the owner to "random". The new owner should be able to
    // team mint and not the admin.
    let admin_info = mock_info(ADMIN, &[]);
    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin_info.clone(),
            ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                new_owner: "random".to_string(),
                expiry: None,
            }),
        )
        .unwrap();

    // Admin does not change until ownership transfer completes.
    let admin: AdminResponse = from_json(
        contract
            .query(deps.as_ref(), mock_env(), QueryMsg::Admin {})
            .unwrap(),
    )
    .unwrap();
    assert_eq!(admin.admin, Some(ADMIN.to_string()));

    // Pending ownership transfer should be discoverable via query.
    let ownership: cw_ownable::Ownership<Addr> = from_json(
        contract
            .query(deps.as_ref(), mock_env(), QueryMsg::Ownership {})
            .unwrap(),
    )
    .unwrap();

    assert_eq!(
        ownership,
        cw_ownable::Ownership::<Addr> {
            owner: Some(Addr::unchecked(ADMIN)),
            pending_owner: Some(Addr::unchecked("random")),
            pending_expiry: None,
        }
    );

    // Accept the ownership transfer.
    let random_info = mock_info("random", &[]);
    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random_info.clone(),
            ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership),
        )
        .unwrap();

    // Minter changes after ownership transfer is accepted.
    let admin: AdminResponse = from_json(
        contract
            .query(deps.as_ref(), mock_env(), QueryMsg::Admin {})
            .unwrap(),
    )
    .unwrap();
    assert_eq!(admin.admin, Some("random".to_string()));

    let mint_msg = ExecuteMsg::MintTeam {
        quantity: 10,
        extension: None,
    };
    // Old owner cannot team mint
    let err: ContractError = contract
        .execute(deps.as_mut(), mock_env(), admin_info, mint_msg.clone())
        .unwrap_err();
    assert_eq!(err, ContractError::Ownership(OwnershipError::NotOwner));

    // New owner can mint
    let _ = contract
        .execute(deps.as_mut(), mock_env(), random_info, mint_msg)
        .unwrap();
}

#[test]
fn burning() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    // mint some NFT
    let admin = mock_info(ADMIN, &[]);
    let _ = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin.clone(),
            ExecuteMsg::MintTeam {
                quantity: 1,
                extension: None,
            },
        )
        .unwrap();

    let burn_msg = ExecuteMsg::Burn {
        token_id: "0".to_string(),
    };
    let err = contract
        .execute(deps.as_mut(), mock_env(), admin.clone(), burn_msg)
        .unwrap_err();
    assert_eq!(err, ContractError::BurnDisabled {});
}

#[test]
fn transferring_nft() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    let admin = mock_info(ADMIN, &[]);
    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin,
            ExecuteMsg::SetPublicSale { open: true },
        )
        .unwrap();

    // Mint a token
    let funds = coins(100000, "usei");
    let minter = mock_info("minter", &funds);
    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            ExecuteMsg::MintPublic {
                quantity: 1,
                extension: None,
            },
        )
        .unwrap();

    // random cannot transfer
    let random = mock_info("random", &[]);
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: String::from("random"),
        token_id: "0".to_string(),
    };
    let err = contract
        .execute(deps.as_mut(), mock_env(), random, transfer_msg)
        .unwrap_err();
    assert_eq!(err, ContractError::Ownership(OwnershipError::NotOwner));

    // owner can
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: String::from("random"),
        token_id: "0".to_string(),
    };
    let res = contract
        .execute(deps.as_mut(), mock_env(), minter, transfer_msg)
        .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_attribute("action", "transfer_nft")
            .add_attribute("sender", "minter")
            .add_attribute("recipient", "random")
            .add_attribute("token_id", "0")
    );
}

#[test]
fn sending_nft() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    let admin = mock_info(ADMIN, &[]);
    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin,
            ExecuteMsg::SetPublicSale { open: true },
        )
        .unwrap();

    // Mint a token
    let funds = coins(100000, "usei");
    let minter = mock_info("minter", &funds);
    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            ExecuteMsg::MintPublic {
                quantity: 1,
                extension: None,
            },
        )
        .unwrap();

    let msg = to_json_binary("msg").unwrap();
    let target = String::from("another_contract");
    let send_msg = ExecuteMsg::SendNft {
        contract: target.clone(),
        token_id: "0".to_string(),
        msg: msg.clone(),
    };

    // random can't transfer
    let random = mock_info("random", &[]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), random, send_msg.clone())
        .unwrap_err();
    assert_eq!(err, ContractError::Ownership(OwnershipError::NotOwner));

    // but minter can
    let res = contract
        .execute(deps.as_mut(), mock_env(), minter, send_msg)
        .unwrap();

    let payload = Cw721ReceiveMsg {
        sender: String::from("minter"),
        token_id: "0".to_string(),
        msg,
    };
    let expected = payload.into_cosmos_msg(target.clone()).unwrap();
    // ensure expected serializes as we think it should
    match &expected {
        CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, .. }) => {
            assert_eq!(contract_addr, &target)
        }
        m => panic!("Unexpected message type: {m:?}"),
    }
    // and make sure this is the request sent by the contract
    assert_eq!(
        res,
        Response::new()
            .add_message(expected)
            .add_attribute("action", "send_nft")
            .add_attribute("sender", "minter")
            .add_attribute("recipient", "another_contract")
            .add_attribute("token_id", "0")
    );
}

#[test]
fn approving_revoking() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    let admin = mock_info(ADMIN, &[]);
    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin,
            ExecuteMsg::SetPublicSale { open: true },
        )
        .unwrap();

    // Mint a token
    let funds = coins(100000, "usei");
    let minter = mock_info("minter", &funds);
    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            ExecuteMsg::MintPublic {
                quantity: 1,
                extension: None,
            },
        )
        .unwrap();

    // token owner shows in approval query
    let res = contract
        .approval(
            deps.as_ref(),
            mock_env(),
            "0".to_string(),
            String::from("minter"),
            false,
        )
        .unwrap();
    assert_eq!(
        res,
        ApprovalResponse {
            approval: Approval {
                spender: String::from("minter"),
                expires: Expiration::Never {}
            }
        }
    );

    // Give random transferring power
    let approve_msg = ExecuteMsg::Approve {
        spender: String::from("random"),
        token_id: "0".to_string(),
        expires: None,
    };
    let owner = mock_info("minter", &[]);
    let res = contract
        .execute(deps.as_mut(), mock_env(), owner, approve_msg)
        .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_attribute("action", "approve")
            .add_attribute("sender", "minter")
            .add_attribute("spender", "random")
            .add_attribute("token_id", "0")
    );

    // test approval query
    let res = contract
        .approval(
            deps.as_ref(),
            mock_env(),
            "0".to_string(),
            String::from("random"),
            true,
        )
        .unwrap();
    assert_eq!(
        res,
        ApprovalResponse {
            approval: Approval {
                spender: String::from("random"),
                expires: Expiration::Never {}
            }
        }
    );

    // random can now transfer
    let random = mock_info("random", &[]);
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: String::from("person"),
        token_id: "0".to_string(),
    };
    contract
        .execute(deps.as_mut(), mock_env(), random, transfer_msg)
        .unwrap();

    // Approvals are removed / cleared
    let query_msg = QueryMsg::OwnerOf {
        token_id: "0".to_string(),
        include_expired: None,
    };
    let res: OwnerOfResponse = from_json(
        contract
            .query(deps.as_ref(), mock_env(), query_msg.clone())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res,
        OwnerOfResponse {
            owner: String::from("person"),
            approvals: vec![],
        }
    );

    // Approve, revoke, and check for empty, to test revoke
    let approve_msg = ExecuteMsg::Approve {
        spender: String::from("random"),
        token_id: "0".to_string(),
        expires: None,
    };
    let owner = mock_info("person", &[]);
    contract
        .execute(deps.as_mut(), mock_env(), owner.clone(), approve_msg)
        .unwrap();

    let revoke_msg = ExecuteMsg::Revoke {
        spender: String::from("random"),
        token_id: "0".to_string(),
    };
    contract
        .execute(deps.as_mut(), mock_env(), owner, revoke_msg)
        .unwrap();

    // Approvals are now removed / cleared
    let res: OwnerOfResponse = from_json(
        contract
            .query(deps.as_ref(), mock_env(), query_msg)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res,
        OwnerOfResponse {
            owner: String::from("person"),
            approvals: vec![],
        }
    );
}

#[test]
fn approving_all_revoking_all() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    let admin = mock_info(ADMIN, &[]);
    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin,
            ExecuteMsg::SetPublicSale { open: true },
        )
        .unwrap();

    // Mint a couple tokens (from the same owner)
    let funds = coins(200000, "usei");
    let minter = mock_info("minter", &funds);
    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            minter.clone(),
            ExecuteMsg::MintPublic {
                quantity: 2,
                extension: None,
            },
        )
        .unwrap();
    let token_id1 = "0".to_string();
    let token_id2 = "1".to_string();

    // paginate the token_ids
    let tokens = contract.all_tokens(deps.as_ref(), None, Some(1)).unwrap();
    assert_eq!(1, tokens.tokens.len());
    assert_eq!(vec![token_id1.clone()], tokens.tokens);
    let tokens = contract
        .all_tokens(deps.as_ref(), Some(token_id1.clone()), Some(3))
        .unwrap();
    assert_eq!(1, tokens.tokens.len());
    assert_eq!(vec![token_id2.clone()], tokens.tokens);

    // minter gives random full (operator) power over their tokens
    let approve_all_msg = ExecuteMsg::ApproveAll {
        operator: String::from("random"),
        expires: None,
    };
    let owner = mock_info("minter", &[]);
    let res = contract
        .execute(deps.as_mut(), mock_env(), owner, approve_all_msg)
        .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_attribute("action", "approve_all")
            .add_attribute("sender", "minter")
            .add_attribute("operator", "random")
    );

    // random can now transfer
    let random = mock_info("random", &[]);
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: String::from("person"),
        token_id: token_id1,
    };
    contract
        .execute(deps.as_mut(), mock_env(), random.clone(), transfer_msg)
        .unwrap();

    // random can now send
    let inner_msg = WasmMsg::Execute {
        contract_addr: "another_contract".into(),
        msg: to_json_binary("You now also have the growing power").unwrap(),
        funds: vec![],
    };
    let msg: CosmosMsg = CosmosMsg::Wasm(inner_msg);

    let send_msg = ExecuteMsg::SendNft {
        contract: String::from("another_contract"),
        token_id: token_id2,
        msg: to_json_binary(&msg).unwrap(),
    };
    contract
        .execute(deps.as_mut(), mock_env(), random, send_msg)
        .unwrap();

    // Approve_all, revoke_all, and check for empty, to test revoke_all
    let approve_all_msg = ExecuteMsg::ApproveAll {
        operator: String::from("operator"),
        expires: None,
    };
    // person is now the owner of the tokens
    let owner = mock_info("person", &[]);
    contract
        .execute(deps.as_mut(), mock_env(), owner, approve_all_msg)
        .unwrap();

    // query for operator should return approval
    let res = contract
        .operator(
            deps.as_ref(),
            mock_env(),
            String::from("person"),
            String::from("operator"),
            true,
        )
        .unwrap();
    assert_eq!(
        res,
        OperatorResponse {
            approval: Approval {
                spender: String::from("operator"),
                expires: Expiration::Never {}
            }
        }
    );

    // query for other should throw error
    let res = contract.operator(
        deps.as_ref(),
        mock_env(),
        String::from("person"),
        String::from("other"),
        true,
    );
    match res {
        Err(StdError::NotFound { kind }) => assert_eq!(kind, "Approval not found"),
        _ => panic!("Unexpected error"),
    }

    let res = contract
        .operators(
            deps.as_ref(),
            mock_env(),
            String::from("person"),
            true,
            None,
            None,
        )
        .unwrap();
    assert_eq!(
        res,
        OperatorsResponse {
            operators: vec![cw721::Approval {
                spender: String::from("operator"),
                expires: Expiration::Never {}
            }]
        }
    );

    // second approval
    let buddy_expires = Expiration::AtHeight(1234567);
    let approve_all_msg = ExecuteMsg::ApproveAll {
        operator: String::from("buddy"),
        expires: Some(buddy_expires),
    };
    let owner = mock_info("person", &[]);
    contract
        .execute(deps.as_mut(), mock_env(), owner.clone(), approve_all_msg)
        .unwrap();

    // and paginate queries
    let res = contract
        .operators(
            deps.as_ref(),
            mock_env(),
            String::from("person"),
            true,
            None,
            Some(1),
        )
        .unwrap();
    assert_eq!(
        res,
        OperatorsResponse {
            operators: vec![cw721::Approval {
                spender: String::from("buddy"),
                expires: buddy_expires,
            }]
        }
    );
    let res = contract
        .operators(
            deps.as_ref(),
            mock_env(),
            String::from("person"),
            true,
            Some(String::from("buddy")),
            Some(2),
        )
        .unwrap();
    assert_eq!(
        res,
        OperatorsResponse {
            operators: vec![cw721::Approval {
                spender: String::from("operator"),
                expires: Expiration::Never {}
            }]
        }
    );

    let revoke_all_msg = ExecuteMsg::RevokeAll {
        operator: String::from("operator"),
    };
    contract
        .execute(deps.as_mut(), mock_env(), owner, revoke_all_msg)
        .unwrap();

    // query for operator should return error
    let res = contract.operator(
        deps.as_ref(),
        mock_env(),
        String::from("person"),
        String::from("operator"),
        true,
    );
    match res {
        Err(StdError::NotFound { kind }) => assert_eq!(kind, "Approval not found"),
        _ => panic!("Unexpected error"),
    }

    // Approvals are removed / cleared without affecting others
    let res = contract
        .operators(
            deps.as_ref(),
            mock_env(),
            String::from("person"),
            false,
            None,
            None,
        )
        .unwrap();
    assert_eq!(
        res,
        OperatorsResponse {
            operators: vec![cw721::Approval {
                spender: String::from("buddy"),
                expires: buddy_expires,
            }]
        }
    );

    // ensure the filter works (nothing should be here
    let mut late_env = mock_env();
    late_env.block.height = 1234568; //expired
    let res = contract
        .operators(
            deps.as_ref(),
            late_env.clone(),
            String::from("person"),
            false,
            None,
            None,
        )
        .unwrap();
    assert_eq!(0, res.operators.len());

    // query operator should also return error
    let res = contract.operator(
        deps.as_ref(),
        late_env,
        String::from("person"),
        String::from("buddy"),
        false,
    );

    match res {
        Err(StdError::NotFound { kind }) => assert_eq!(kind, "Approval not found"),
        _ => panic!("Unexpected error"),
    }
}

#[test]
fn update_withdraw_address() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    // validate default is admin
    let withdraw_address: Option<String> = from_json(
        contract
            .query(deps.as_ref(), mock_env(), QueryMsg::WithdrawAddress {})
            .unwrap(),
    )
    .unwrap();
    assert_eq!(withdraw_address, Some(ADMIN.to_string()));

    // Random can't update
    let random = mock_info("random", &[]);
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random.clone(),
            ExecuteMsg::SetWithdrawAddress {
                address: "random".to_string(),
            },
        )
        .unwrap_err();
    assert_eq!(err, ContractError::Ownership(OwnershipError::NotOwner));

    // admin can set
    let admin = mock_info(ADMIN, &[]);
    let _ = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin,
            ExecuteMsg::SetWithdrawAddress {
                address: "new".to_string(),
            },
        )
        .unwrap();
    let withdraw_address = contract
        .withdraw_address
        .load(deps.as_ref().storage)
        .unwrap();
    assert_eq!(withdraw_address, "new".to_string())
}

#[test]
fn test_remove_withdraw_address() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    // random cant remove
    let random = mock_info("random", &[]);
    let err = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            random,
            ExecuteMsg::RemoveWithdrawAddress {},
        )
        .unwrap_err();
    assert_eq!(err, ContractError::Ownership(OwnershipError::NotOwner));

    // admin can remove
    let admin = mock_info(ADMIN, &[]);
    let _ = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin.clone(),
            ExecuteMsg::RemoveWithdrawAddress {},
        )
        .unwrap();

    // validate withdraw is removed
    let withdraw_address: Option<String> = from_json(
        contract
            .query(deps.as_ref(), mock_env(), QueryMsg::WithdrawAddress {})
            .unwrap(),
    )
    .unwrap();
    assert_eq!(withdraw_address, None);

    // test that we can set again
    let _ = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin.clone(),
            ExecuteMsg::SetWithdrawAddress {
                address: "new_address".to_string(),
            },
        )
        .unwrap();
    let withdraw_address: Option<String> = from_json(
        contract
            .query(deps.as_ref(), mock_env(), QueryMsg::WithdrawAddress {})
            .unwrap(),
    )
    .unwrap();
    assert_eq!(withdraw_address, Some("new_address".to_string()));
}

#[test]
fn withdraw_funds() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    let amount = coin(100000, "usei");
    let res = contract
        .withdraw_funds(deps.as_mut().storage, &amount)
        .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_message(BankMsg::Send {
                to_address: ADMIN.to_string(),
                amount: vec![amount.clone()],
            })
            .add_attribute("action", "withdraw_funds")
            .add_attribute("amount", amount.amount.to_string())
            .add_attribute("denom", amount.denom.to_string())
    );

    // remove withdraw address
    let admin = mock_info(ADMIN, &[]);
    let _ = contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin.clone(),
            ExecuteMsg::RemoveWithdrawAddress {},
        )
        .unwrap();
    let err = contract
        .withdraw_funds(deps.as_mut().storage, &Coin::new(100, "usei"))
        .unwrap_err();
    assert_eq!(err, ContractError::NoWithdrawAddress {});
}

#[test]
fn query_tokens_by_owner() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    let admin = mock_info(ADMIN, &[]);
    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            admin.clone(),
            ExecuteMsg::SetPublicSale { open: true },
        )
        .unwrap();

    // Mint a couple tokens (from the same owner)
    let minter = "minter";
    let funds = coins(200000, "usei");
    let minter_info = mock_info(minter, &funds);
    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            minter_info.clone(),
            ExecuteMsg::MintPublic {
                quantity: 2,
                extension: None,
            },
        )
        .unwrap();
    let token_id1 = "0".to_string();
    let token_id2 = "1".to_string();

    // Mint token from another other
    let another_minter = "another_minter";
    let funds = coins(100000, "usei");
    let minter_info = mock_info(another_minter, &funds);
    contract
        .execute(
            deps.as_mut(),
            mock_env(),
            minter_info.clone(),
            ExecuteMsg::MintPublic {
                quantity: 1,
                extension: None,
            },
        )
        .unwrap();
    let token_id3 = "2".to_string();

    // get all tokens in order:
    let expected = vec![token_id1.clone(), token_id2.clone(), token_id3.clone()];
    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    assert_eq!(&expected, &tokens.tokens);
    // paginate
    let tokens = contract.all_tokens(deps.as_ref(), None, Some(2)).unwrap();
    assert_eq!(&expected[..2], &tokens.tokens[..]);
    let tokens = contract
        .all_tokens(deps.as_ref(), Some(expected[1].clone()), None)
        .unwrap();
    assert_eq!(&expected[2..], &tokens.tokens[..]);

    // get by owner
    let by_minter = vec![token_id1, token_id2];
    let by_another_minter = vec![token_id3];
    // all tokens by owner
    let tokens = contract
        .tokens(deps.as_ref(), minter.to_string(), None, None)
        .unwrap();
    assert_eq!(&by_minter, &tokens.tokens);
    let tokens = contract
        .tokens(deps.as_ref(), another_minter.to_string(), None, None)
        .unwrap();
    assert_eq!(&by_another_minter, &tokens.tokens);

    // paginate for demeter
    let tokens = contract
        .tokens(deps.as_ref(), minter.to_string(), None, Some(1))
        .unwrap();
    assert_eq!(&by_minter[..1], &tokens.tokens[..]);
    let tokens = contract
        .tokens(
            deps.as_ref(),
            minter.to_string(),
            Some(by_minter[0].clone()),
            Some(3),
        )
        .unwrap();
    assert_eq!(&by_minter[1..], &tokens.tokens[..]);
}
