#[cfg(test)]
mod tests {
    use crate::error::ContractError;
    use crate::msg::{
        CheckRoyaltiesResponse, Cw2981RerollExecuteMsg, Cw2981RerollQueryMsg, InstantiateMsg,
        RoyaltiesInfoResponse,
    };
    use crate::query::{check_royalties, query_royalties_info};
    use crate::{entry, Cw2981RerollContract};

    use cosmwasm_std::{coins, from_json, Addr, Empty, StdError, Uint128};

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cw721_reroll::msg::QueryMsg;
    use cw721_reroll::ExecuteMsg;
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
            base_token_uri: Some(token_uri.to_string()),
            royalty_payment_address: royalty_payment_address.clone(),
            royalty_percentage: royalty_percentage.clone(),
            minter: Some(String::from(CREATOR)),
            withdraw_address: Some(String::from(CREATOR)),
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();
    }

    #[test]
    fn validate_royalty_information() {
        let mut deps = mock_dependencies();
        let _contract = Cw2981RerollContract::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "PlayerX".to_string(),
            symbol: "PX".to_string(),
            base_token_uri: Some("uri".to_string()),
            royalty_payment_address: Addr::unchecked("address"),
            royalty_percentage: 101,
            minter: Some(String::from(CREATOR)),
            withdraw_address: Some(String::from(CREATOR)),
        };
        let err =
            entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap_err();
        assert_eq!(err, ContractError::InvalidRoyaltyPercentage);
    }

    #[test]
    fn check_royalties_response() {
        let mut deps = mock_dependencies();
        let _contract = Cw2981RerollContract::default();

        let info = mock_info(CREATOR, &[]);
        let init_msg = InstantiateMsg {
            name: "PlayerX".to_string(),
            symbol: "PX".to_string(),
            base_token_uri: Some("uri".to_string()),
            royalty_payment_address: Addr::unchecked("address"),
            royalty_percentage: 10,
            minter: Some(String::from(CREATOR)),
            withdraw_address: Some(String::from(CREATOR)),
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let expected = CheckRoyaltiesResponse {
            royalty_payments: true,
        };
        let res = check_royalties(deps.as_ref()).unwrap();
        assert_eq!(res, expected);

        // also check the longhand way
        let query_msg = QueryMsg::Extension {
            msg: Cw2981RerollQueryMsg::CheckRoyalties {},
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
            base_token_uri: Some("uri".to_string()),
            royalty_payment_address: Addr::unchecked("address"),
            royalty_percentage: 4,
            minter: Some(String::from(CREATOR)),
            withdraw_address: Some(String::from(CREATOR)),
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let err =
            query_royalties_info(deps.as_ref(), "0".to_string(), Uint128::new(100)).unwrap_err();
        assert!(matches!(err, StdError::NotFound { .. }));

        // also check the longhand way
        let query_msg = QueryMsg::Extension {
            msg: Cw2981RerollQueryMsg::RoyaltyInfo {
                token_id: "1".to_string(),
                sale_price: Uint128::new(100),
            },
        };
        let err = entry::query(deps.as_ref(), mock_env(), query_msg).unwrap_err();
        assert!(matches!(err, StdError::NotFound { .. }));

        // Mint a token
        let mint_msg = ExecuteMsg::Mint {
            quantity: 1,
            extension: Empty {},
        };
        entry::execute(deps.as_mut(), mock_env(), info, mint_msg).unwrap();

        // Try to get royality info
        let query_msg = QueryMsg::Extension {
            msg: Cw2981RerollQueryMsg::RoyaltyInfo {
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
            msg: Cw2981RerollQueryMsg::RoyaltyInfo {
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
            base_token_uri: Some("uri".to_string()),
            royalty_payment_address: Addr::unchecked("address"),
            royalty_percentage: 4,
            minter: Some(String::from(CREATOR)),
            withdraw_address: Some(String::from(CREATOR)),
        };
        entry::instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        // Mint a token
        let exec_msg = ExecuteMsg::Mint {
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
            msg: Cw2981RerollQueryMsg::RoyaltyInfo {
                token_id: "0".to_string(),
                sale_price: Uint128::new(100),
            },
        };
        let query_res: RoyaltiesInfoResponse =
            from_json(entry::query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap()).unwrap();
        assert_eq!(query_res, expected);

        // Random can't update royalty info
        let random_info = mock_info("random", &[]);
        let extension = Cw2981RerollExecuteMsg::UpdateRoyaltyConfig {
            royalty_percentage: 8,
            royalty_payment_address: Addr::unchecked("random"),
        };
        let exec_msg = ExecuteMsg::Extension { msg: extension };
        let err =
            entry::execute(deps.as_mut(), mock_env(), random_info.clone(), exec_msg).unwrap_err();
        assert_eq!(err, ContractError::Ownership(OwnershipError::NotOwner));

        // Update royalty info with error
        let extension = Cw2981RerollExecuteMsg::UpdateRoyaltyConfig {
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
        let extension = Cw2981RerollExecuteMsg::UpdateRoyaltyConfig {
            royalty_percentage: 5,
            royalty_payment_address: Addr::unchecked("address_new"),
        };
        let exec_msg = ExecuteMsg::Extension { msg: extension };
        entry::execute(deps.as_mut(), mock_env(), info, exec_msg).unwrap();
        let query_res: RoyaltiesInfoResponse =
            from_json(entry::query(deps.as_ref(), mock_env(), query_msg).unwrap()).unwrap();
        assert_eq!(query_res, expected);
    }
}
