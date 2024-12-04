use cosmwasm_std::{Uint128};
use cw_storage_plus::{Item};

pub const TOTAL_DISTRIBUTED_PRYSM: Item<u128> = Item::new("total_distributed_prysm");
pub const TOTAL_DISTRIBUTED_BTC: Item<u128> = Item::new("total_distributed_btc");
pub const LAST_DISTRIBUTION_BLOCK: Item<u64> = Item::new("last_distribution_block");
