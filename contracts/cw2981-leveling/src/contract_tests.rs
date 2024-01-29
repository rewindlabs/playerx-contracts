#[cfg(test)]
mod tests {
    use crate::error::ContractError;
    use crate::msg::{
        CheckRoyaltiesResponse, Cw2981LevelingExecuteMsg, Cw2981LevelingQueryMsg, InstantiateMsg,
        LevelingConfigResponse, RoyaltiesInfoResponse, TokenLevelResponse,
    };
    use crate::query::{check_royalties, query_royalties_info};
    use crate::{entry, Cw2981LevelingContract};

    use cosmwasm_std::{coins, from_json, Addr, Empty, StdError, Uint128};

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cw721_base::msg::{QueryMsg, SaleConfigResponse};
    use cw721_base::ExecuteMsg;
    use cw_ownable::OwnershipError;

    const CREATOR: &str = "creator";

    #[test]
    fn check_instantiate() {
        let mut deps = mock_dependencies();

        let info = mock_info(CREATOR, &[]);
        let token_uri = "token_uri";
        let royalty_payment_address = Addr::unchecked("address");
        let royalty_percentage = 10;
        let init_msg = InstantiateMsg {
            name: "PlayerX".to_string(),
            symbol: "PX".to_string(),
            base_token_uri: token_uri.to_string(),
            royalty_payment_address: royalty_payment_address.clone(),
            royalty_percentage: royalty_percentage.clone(),
            collection_size: 10,
            max_per_public: 5,
            max_per_allowlist: 1,
            public_price: Uint128::from(1000000u64),
            allowlist_price: Uint128::from(1000000u64),
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        // check config
        let query_msg = QueryMsg::SaleConfig {};
        let query_res: SaleConfigResponse =
            from_json(entry::query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
        assert_eq!(
            query_res,
            SaleConfigResponse {
                max_per_public: 5,
                max_per_allowlist: 1,
                public_price: Uint128::from(1000000u64),
                allowlist_price: Uint128::from(1000000u64),
                public_sale_open: false,
                allowlist_sale_open: false,
            }
        );
    }

    #[test]
    fn validate_royalty_information() {
        let mut deps = mock_dependencies();
        let _contract = Cw2981LevelingContract::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "PlayerX".to_string(),
            symbol: "PX".to_string(),
            base_token_uri: "uri".to_string(),
            royalty_payment_address: Addr::unchecked("address"),
            royalty_percentage: 101,
            collection_size: 10,
            max_per_public: 5,
            max_per_allowlist: 1,
            public_price: Uint128::from(1000000u64),
            allowlist_price: Uint128::from(1000000u64),
        };
        let err =
            entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap_err();
        assert_eq!(err, ContractError::InvalidRoyaltyPercentage);
    }

    #[test]
    fn check_royalties_response() {
        let mut deps = mock_dependencies();
        let _contract = Cw2981LevelingContract::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "PlayerX".to_string(),
            symbol: "PX".to_string(),
            base_token_uri: "uri".to_string(),
            royalty_payment_address: Addr::unchecked("address"),
            royalty_percentage: 10,
            collection_size: 10,
            max_per_public: 5,
            max_per_allowlist: 1,
            public_price: Uint128::from(1000000u64),
            allowlist_price: Uint128::from(1000000u64),
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let expected = CheckRoyaltiesResponse {
            royalty_payments: true,
        };
        let res = check_royalties(deps.as_ref()).unwrap();
        assert_eq!(res, expected);

        // also check the longhand way
        let query_msg = QueryMsg::Extension {
            msg: Cw2981LevelingQueryMsg::CheckRoyalties {},
        };
        let query_res: CheckRoyaltiesResponse =
            from_json(entry::query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
        assert_eq!(query_res, expected);
    }

    #[test]
    fn check_token_royalties() {
        let mut deps = mock_dependencies();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "PlayerX".to_string(),
            symbol: "PX".to_string(),
            base_token_uri: "uri".to_string(),
            royalty_payment_address: Addr::unchecked("address"),
            royalty_percentage: 4,
            collection_size: 10,
            max_per_public: 5,
            max_per_allowlist: 1,
            public_price: Uint128::from(1000000u64),
            allowlist_price: Uint128::from(1000000u64),
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let err =
            query_royalties_info(deps.as_ref(), "0".to_string(), Uint128::new(100)).unwrap_err();
        assert!(matches!(err, StdError::NotFound { .. }));

        // also check the longhand way
        let query_msg = QueryMsg::Extension {
            msg: Cw2981LevelingQueryMsg::RoyaltyInfo {
                token_id: "1".to_string(),
                sale_price: Uint128::new(100),
            },
        };
        let err = entry::query(deps.as_ref(), mock_env(), query_msg).unwrap_err();
        assert!(matches!(err, StdError::NotFound { .. }));

        // Mint a token
        let mint_msg = ExecuteMsg::MintTeam {
            quantity: 1,
            extension: Empty {},
        };
        entry::execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap();

        // Try to get royality info
        let query_msg = QueryMsg::Extension {
            msg: Cw2981LevelingQueryMsg::RoyaltyInfo {
                token_id: "0".to_string(),
                sale_price: Uint128::new(100),
            },
        };
        let query_res: RoyaltiesInfoResponse =
            from_json(entry::query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
        let expected = RoyaltiesInfoResponse {
            address: "address".into(),
            royalty_amount: Uint128::new(4),
        };
        assert_eq!(query_res, expected);

        // Try with a number that needs to be rounded
        let query_msg = QueryMsg::Extension {
            msg: Cw2981LevelingQueryMsg::RoyaltyInfo {
                token_id: "0".to_string(),
                sale_price: Uint128::new(43),
            },
        };
        let query_res: RoyaltiesInfoResponse =
            from_json(entry::query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
        // 43 * .4 = 1.72
        // Rounds down to 1
        let expected = RoyaltiesInfoResponse {
            address: "address".into(),
            royalty_amount: Uint128::new(1),
        };
        assert_eq!(query_res, expected);
    }

    #[test]
    fn check_update_royalty_info() {
        let mut deps = mock_dependencies();

        let funds = coins(5000000, "usei");
        let info = mock_info(CREATOR, &funds);
        let init_msg = InstantiateMsg {
            name: "PlayerX".to_string(),
            symbol: "PX".to_string(),
            base_token_uri: "uri".to_string(),
            royalty_payment_address: Addr::unchecked("address"),
            royalty_percentage: 4,
            collection_size: 10,
            max_per_public: 5,
            max_per_allowlist: 1,
            public_price: Uint128::from(1000000u64),
            allowlist_price: Uint128::from(1000000u64),
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        // Mint a token
        let exec_msg = ExecuteMsg::MintPublic {
            quantity: 5,
            extension: Empty {},
        };
        entry::execute(deps.as_mut(), mock_env(), info.clone(), exec_msg).unwrap();

        // Check royalty info
        let expected = RoyaltiesInfoResponse {
            address: "address".into(),
            royalty_amount: Uint128::new(4),
        };
        let query_msg = QueryMsg::Extension {
            msg: Cw2981LevelingQueryMsg::RoyaltyInfo {
                token_id: "0".to_string(),
                sale_price: Uint128::new(100),
            },
        };
        let query_res: RoyaltiesInfoResponse =
            from_json(entry::query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap()).unwrap();
        assert_eq!(query_res, expected);

        // Random can't update royalty info
        let random_info = mock_info("random", &[]);
        let extension = Cw2981LevelingExecuteMsg::UpdateRoyaltyConfig {
            royalty_percentage: 8,
            royalty_payment_address: Addr::unchecked("random"),
        };
        let exec_msg = ExecuteMsg::Extension { msg: extension };
        let err =
            entry::execute(deps.as_mut(), mock_env(), random_info.clone(), exec_msg).unwrap_err();
        assert_eq!(err, ContractError::Ownership(OwnershipError::NotOwner));

        // Update royalty info with error
        let extension = Cw2981LevelingExecuteMsg::UpdateRoyaltyConfig {
            royalty_percentage: 101,
            royalty_payment_address: Addr::unchecked("address"),
        };
        let exec_msg = ExecuteMsg::Extension { msg: extension };
        let err = entry::execute(deps.as_mut(), mock_env(), info.clone(), exec_msg).unwrap_err();
        assert_eq!(err, ContractError::InvalidRoyaltyPercentage);

        // Update royalty info with new address and percentage
        let expected = RoyaltiesInfoResponse {
            address: "address_new".into(),
            royalty_amount: Uint128::new(5),
        };
        let extension = Cw2981LevelingExecuteMsg::UpdateRoyaltyConfig {
            royalty_percentage: 5,
            royalty_payment_address: Addr::unchecked("address_new"),
        };
        let exec_msg = ExecuteMsg::Extension { msg: extension };
        entry::execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();
        let query_res: RoyaltiesInfoResponse =
            from_json(entry::query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
        assert_eq!(query_res, expected);
    }

    #[test]
    fn update_leveling_config() {
        let mut deps = mock_dependencies();
        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "PlayerX".to_string(),
            symbol: "PX".to_string(),
            base_token_uri: "uri".to_string(),
            royalty_payment_address: Addr::unchecked("address"),
            royalty_percentage: 4,
            collection_size: 10,
            max_per_public: 5,
            max_per_allowlist: 1,
            public_price: Uint128::from(1000000u64),
            allowlist_price: Uint128::from(1000000u64),
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let query_msg = QueryMsg::Extension {
            msg: Cw2981LevelingQueryMsg::LevelingConfig {},
        };
        let query_res: LevelingConfigResponse =
            from_json(entry::query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap()).unwrap();
        assert_eq!(
            query_res,
            LevelingConfigResponse {
                leveling_open: false,
                max_experience: 0
            }
        );

        // Attempt to update leveling config by an unauthorized user
        let random_info = mock_info("random_user", &[]);
        let extension = Cw2981LevelingExecuteMsg::UpdateLevelingConfig {
            leveling_open: true,
            max_experience: 1000,
        };
        let err = entry::execute(
            deps.as_mut(),
            mock_env(),
            random_info,
            ExecuteMsg::Extension {
                msg: extension.clone(),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::Ownership(OwnershipError::NotOwner));

        // Update leveling config by the creator
        entry::execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::Extension { msg: extension },
        )
        .unwrap();

        // Query and verify the updated leveling config
        let query_res: LevelingConfigResponse =
            from_json(entry::query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap()).unwrap();
        assert_eq!(
            query_res,
            LevelingConfigResponse {
                leveling_open: true,
                max_experience: 1000
            }
        );

        entry::execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Extension {
                msg: Cw2981LevelingExecuteMsg::UpdateLevelingConfig {
                    leveling_open: false,
                    max_experience: 1000,
                },
            },
        )
        .unwrap();

        let query_res: LevelingConfigResponse =
            from_json(entry::query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
        assert_eq!(
            query_res,
            LevelingConfigResponse {
                leveling_open: false,
                max_experience: 1000
            }
        );
    }

    #[test]
    fn toggle_leveling_status() {
        let mut deps = mock_dependencies();
        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "PlayerX".to_string(),
            symbol: "PX".to_string(),
            base_token_uri: "uri".to_string(),
            royalty_payment_address: Addr::unchecked("address"),
            royalty_percentage: 4,
            collection_size: 10,
            max_per_public: 5,
            max_per_allowlist: 1,
            public_price: Uint128::from(1000000u64),
            allowlist_price: Uint128::from(1000000u64),
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        // Mint a token
        let mint_msg = ExecuteMsg::MintTeam {
            quantity: 1,
            extension: Empty {},
        };
        entry::execute(deps.as_mut(), mock_env(), info.clone(), mint_msg).unwrap();

        // Toggle leveling fails bc leveling not open yet
        let extension = Cw2981LevelingExecuteMsg::ToggleLeveling {
            token_id: "0".to_string(),
        };
        let err = entry::execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::Extension {
                msg: extension.clone(),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::LevelingNotOpen {});

        // Turn on leveling
        entry::execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::Extension {
                msg: Cw2981LevelingExecuteMsg::UpdateLevelingConfig {
                    leveling_open: true,
                    max_experience: 100,
                },
            },
        )
        .unwrap();

        // Toggle leveling for the token
        let env = mock_env();
        entry::execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Extension {
                msg: extension.clone(),
            },
        )
        .unwrap();

        // Query and verify the token's leveling status
        let query_msg = QueryMsg::Extension {
            msg: Cw2981LevelingQueryMsg::TokenLevel {
                token_id: "0".to_string(),
            },
        };
        let query_res: TokenLevelResponse =
            from_json(entry::query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap()).unwrap();
        assert_eq!(
            query_res,
            TokenLevelResponse {
                leveling: true,
                leveling_start_timestamp: env.block.time.seconds(),
                total_exp: 0
            }
        );

        // Attempt to toggle leveling for a token by a non-owner
        let random_info = mock_info("random_user", &[]);
        let err = entry::execute(
            deps.as_mut(),
            mock_env(),
            random_info,
            ExecuteMsg::Extension {
                msg: extension.clone(),
            },
        )
        .unwrap_err();
        assert_eq!(err, ContractError::UnauthorizedLeveling);

        // Toggle leveling again to turn it off
        let mut env = mock_env();
        env.block.time = env.block.time.plus_seconds(10);
        entry::execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Extension {
                msg: extension.clone(),
            },
        )
        .unwrap();

        // Query and verify the token's leveling status again
        let query_res: TokenLevelResponse =
            from_json(entry::query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap()).unwrap();
        assert_eq!(
            query_res,
            TokenLevelResponse {
                leveling: false,
                leveling_start_timestamp: 0,
                total_exp: 10
            }
        );

        // Toggle back on
        entry::execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::Extension {
                msg: extension.clone(),
            },
        )
        .unwrap();

        // Turn off leveling in config
        env.block.time = env.block.time.plus_seconds(100);
        entry::execute(
            deps.as_mut(),
            env,
            info,
            ExecuteMsg::Extension {
                msg: Cw2981LevelingExecuteMsg::UpdateLevelingConfig {
                    leveling_open: false,
                    max_experience: 100,
                },
            },
        )
        .unwrap();

        // Query and verify the token's leveling status again
        let query_res: TokenLevelResponse =
            from_json(entry::query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
        assert_eq!(
            query_res,
            TokenLevelResponse {
                leveling: false,
                leveling_start_timestamp: 0,
                // max_experience is set to 100
                total_exp: 100
            }
        );
    }

    #[test]
    fn grant_bonus_experience() {
        let mut deps = mock_dependencies();
        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "PlayerX".to_string(),
            symbol: "PX".to_string(),
            base_token_uri: "uri".to_string(),
            royalty_payment_address: Addr::unchecked("address"),
            royalty_percentage: 4,
            collection_size: 10,
            max_per_public: 5,
            max_per_allowlist: 1,
            public_price: Uint128::from(1000000u64),
            allowlist_price: Uint128::from(1000000u64),
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        // Mint tokens
        let mint_msg = ExecuteMsg::MintTeam {
            quantity: 2,
            extension: Empty {},
        };
        entry::execute(deps.as_mut(), mock_env(), info.clone(), mint_msg).unwrap();

        // Turn on leveling
        entry::execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::Extension {
                msg: Cw2981LevelingExecuteMsg::UpdateLevelingConfig {
                    leveling_open: true,
                    max_experience: 100,
                },
            },
        )
        .unwrap();

        // Random doesn't have access
        let msg = ExecuteMsg::Extension {
            msg: Cw2981LevelingExecuteMsg::GrantBonusExperience {
                token_ids: vec!["0".to_string(), "1".to_string()],
                experience: 10,
            },
        };
        let random = mock_info("random", &[]);
        let err = entry::execute(deps.as_mut(), mock_env(), random, msg.clone()).unwrap_err();
        assert_eq!(err, ContractError::Ownership(OwnershipError::NotOwner));

        // Grant bonus experience to the tokens
        entry::execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let query_msg = QueryMsg::Extension {
            msg: Cw2981LevelingQueryMsg::TokenLevel {
                token_id: "0".to_string(),
            },
        };
        let query_res: TokenLevelResponse =
            from_json(entry::query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
        assert_eq!(
            query_res,
            TokenLevelResponse {
                leveling: false,
                leveling_start_timestamp: 0,
                total_exp: 10
            }
        );

        let query_msg = QueryMsg::Extension {
            msg: Cw2981LevelingQueryMsg::TokenLevel {
                token_id: "1".to_string(),
            },
        };
        let query_res: TokenLevelResponse =
            from_json(entry::query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap()).unwrap();
        assert_eq!(
            query_res,
            TokenLevelResponse {
                leveling: false,
                leveling_start_timestamp: 0,
                total_exp: 10
            }
        );

        // grant more than max
        let msg = ExecuteMsg::Extension {
            msg: Cw2981LevelingExecuteMsg::GrantBonusExperience {
                token_ids: vec!["1".to_string()],
                experience: 150,
            },
        };
        entry::execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let query_res: TokenLevelResponse =
            from_json(entry::query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
        assert_eq!(
            query_res,
            TokenLevelResponse {
                leveling: false,
                leveling_start_timestamp: 0,
                total_exp: 100
            }
        );
    }
}
