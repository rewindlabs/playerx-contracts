pub mod contract_tests;
pub mod error;
pub mod execute;
pub mod msg;
pub mod query;
pub mod state;

use cosmwasm_std::{to_json_binary, Empty};
use cw721_base::{Cw721Contract, InstantiateMsg as BaseInstantiateMsg};

use crate::error::ContractError;
pub use crate::msg::InstantiateMsg;
use crate::msg::{Cw2981LevelingExecuteMsg, Cw2981LevelingQueryMsg};

// Version info for migration
const CONTRACT_NAME: &str = "crates.io:cw2981-leveling";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Cw2981LevelingContract<'a> =
    Cw721Contract<'a, Empty, Empty, Cw2981LevelingExecuteMsg, Cw2981LevelingQueryMsg>;
pub type ExecuteMsg = cw721_base::ExecuteMsg<Empty, Cw2981LevelingExecuteMsg>;
pub type QueryMsg = cw721_base::QueryMsg<Cw2981LevelingQueryMsg>;

#[cfg(not(feature = "library"))]
pub mod entry {
    use self::execute::{toggle_leveling, update_leveling_config, update_royalty_config};
    use self::msg::{LevelingConfigResponse, RoyaltyConfigResponse};
    use self::query::{
        check_royalties, query_leveling_config, query_royalties_info, query_royalty_config,
        query_token_level,
    };
    use self::state::{LEVELING_CONFIG, ROYALTY_CONFIG};

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

        let leveling_config = LevelingConfigResponse {
            leveling_open: false,
            max_experience: 0,
        };
        LEVELING_CONFIG.save(deps.storage, &leveling_config)?;

        // Prepare the base InstantiateMsg
        let base_msg = BaseInstantiateMsg {
            name: msg.name,
            symbol: msg.symbol,
            base_token_uri: msg.base_token_uri,
            collection_size: msg.collection_size,
            max_per_allowlist: msg.max_per_allowlist,
            max_per_public: msg.max_per_public,
            allowlist_price: msg.allowlist_price,
            public_price: msg.public_price,
        };
        Ok(Cw2981LevelingContract::default().instantiate(deps.branch(), env, info, base_msg)?)
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
                Cw2981LevelingExecuteMsg::UpdateRoyaltyConfig {
                    royalty_percentage,
                    royalty_payment_address,
                } => update_royalty_config(deps, info, royalty_percentage, royalty_payment_address),
                Cw2981LevelingExecuteMsg::UpdateLevelingConfig {
                    leveling_open,
                    max_experience,
                } => update_leveling_config(deps, env, info, leveling_open, max_experience),
                Cw2981LevelingExecuteMsg::ToggleLeveling { token_id } => {
                    toggle_leveling(deps, env, info, token_id)
                }
            },
            _ => Cw2981LevelingContract::default()
                .execute(deps, env, info, msg)
                .map_err(Into::into),
        }
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::Extension { msg } => match msg {
                Cw2981LevelingQueryMsg::RoyaltyConfig {} => {
                    to_json_binary(&query_royalty_config(deps)?)
                }
                Cw2981LevelingQueryMsg::RoyaltyInfo {
                    token_id,
                    sale_price,
                } => to_json_binary(&query_royalties_info(deps, token_id, sale_price)?),
                Cw2981LevelingQueryMsg::CheckRoyalties {} => {
                    to_json_binary(&check_royalties(deps)?)
                }
                Cw2981LevelingQueryMsg::LevelingConfig {} => {
                    to_json_binary(&query_leveling_config(deps)?)
                }
                Cw2981LevelingQueryMsg::TokenLevel { token_id } => {
                    to_json_binary(&query_token_level(deps, token_id)?)
                }
            },
            _ => Cw2981LevelingContract::default().query(deps, env, msg),
        }
    }
}
