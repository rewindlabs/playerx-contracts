use crate::msg::RoyaltyConfigResponse;
use cw_storage_plus::Item;

pub const ROYALTY_CONFIG: Item<RoyaltyConfigResponse> = Item::new("royalty_config");
