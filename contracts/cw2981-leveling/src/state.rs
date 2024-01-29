use crate::msg::{LevelingConfigResponse, RoyaltyConfigResponse, TokenLevelResponse};
use cw_storage_plus::{Item, Map};

pub const ROYALTY_CONFIG: Item<RoyaltyConfigResponse> = Item::new("royalty_config");
pub const LEVELING_CONFIG: Item<LevelingConfigResponse> = Item::new("leveling_config");
pub const TOKEN_LEVELS: Map<&str, TokenLevelResponse> = Map::new("token_levels");
