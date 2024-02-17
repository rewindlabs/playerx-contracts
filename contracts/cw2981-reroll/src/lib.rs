pub mod contract_tests;
pub mod error;
pub mod execute;
pub mod msg;
pub mod query;
pub mod state;

use cosmwasm_std::{to_json_binary, Empty};
use cw721_reroll::{Cw721Contract, InstantiateMsg as BaseInstantiateMsg};

use crate::error::ContractError;
pub use crate::msg::InstantiateMsg;
use crate::msg::{Cw2981RerollExecuteMsg, Cw2981RerollQueryMsg};

// Version info for migration
const CONTRACT_NAME: &str = "crates.io:cw2981-reroll";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Cw2981RerollContract<'a> =
    Cw721Contract<'a, Empty, Empty, Cw2981RerollExecuteMsg, Cw2981RerollQueryMsg>;
pub type ExecuteMsg = cw721_reroll::ExecuteMsg<Empty, Cw2981RerollExecuteMsg>;
pub type QueryMsg = cw721_reroll::QueryMsg<Cw2981RerollQueryMsg>;

#[cfg(not(feature = "library"))]
pub mod entry {
    use self::execute::update_royalty_config;
    use self::msg::RoyaltyConfigResponse;
    use self::query::{check_royalties, query_royalties_info, query_royalty_config};
    use self::state::ROYALTY_CONFIG;

    use super::*;

    use cosmwasm_std::entry_point;
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

    #[entry_point]
    pub fn instantiate(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        // Validate royalty percentage
        if msg.royalty_percentage > 100 {
            return Err(ContractError::InvalidRoyaltyPercentage);
        }

        let royalty_config = RoyaltyConfigResponse {
            royalty_percentage: msg.royalty_percentage,
            royalty_payment_address: msg.royalty_payment_address,
        };
        ROYALTY_CONFIG.save(deps.storage, &royalty_config)?;

        // Prepare the base InstantiateMsg
        let base_msg = BaseInstantiateMsg {
            name: msg.name,
            symbol: msg.symbol,
            base_token_uri: msg.base_token_uri,
            minter: msg.minter,
            withdraw_address: msg.withdraw_address,
        };
        Ok(Cw2981RerollContract::default().instantiate(deps.branch(), env, info, base_msg)?)
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::Extension { msg } => match msg {
                Cw2981RerollExecuteMsg::UpdateRoyaltyConfig {
                    royalty_percentage,
                    royalty_payment_address,
                } => update_royalty_config(deps, info, royalty_percentage, royalty_payment_address),
            },
            _ => Cw2981RerollContract::default()
                .execute(deps, env, info, msg)
                .map_err(Into::into),
        }
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::Extension { msg } => match msg {
                Cw2981RerollQueryMsg::RoyaltyConfig {} => {
                    to_json_binary(&query_royalty_config(deps)?)
                }
                Cw2981RerollQueryMsg::RoyaltyInfo {
                    token_id,
                    sale_price,
                } => to_json_binary(&query_royalties_info(deps, token_id, sale_price)?),
                Cw2981RerollQueryMsg::CheckRoyalties {} => to_json_binary(&check_royalties(deps)?),
            },
            _ => Cw2981RerollContract::default().query(deps, env, msg),
        }
    }
}
