use anchor_lang::prelude::*;
use spl_math::uint::U256;

#[cfg(feature = "test")]
mod keys {
    use super::*;
    pub const ADMIN: Pubkey = pubkey!("AXF3tTrMUD5BLzv5Fmyj63KXwvkuGdxMQemSJHtTag4j");
    pub const SPL_GOVERNANCE_PROGRAM_ID: Pubkey =
        pubkey!("HwXcHGabc19PxzYFVSfKvuaDSNpbLGL8fhVtkcTyEymj");
}
#[cfg(all(feature = "dev", not(feature = "test")))]
mod keys {
    use super::*;
    pub const ADMIN: Pubkey = pubkey!("AXF3tTrMUD5BLzv5Fmyj63KXwvkuGdxMQemSJHtTag4j");
    pub const SPL_GOVERNANCE_PROGRAM_ID: Pubkey =
        pubkey!("HwXcHGabc19PxzYFVSfKvuaDSNpbLGL8fhVtkcTyEymj");
}
#[cfg(all(not(feature = "dev"), not(feature = "test")))]
mod keys {
    use super::*;
    pub const ADMIN: Pubkey = pubkey!("BesRAQsmAL45fGMJHNqP8xNT51Nr9ewvpzUzTRiU8A1M");
    // TODO: Change key and update program if we want to have daos.
    // We don't plan to deploy the SPL governance program on mainnet.
    // Setting key to default key.
    pub const SPL_GOVERNANCE_PROGRAM_ID: Pubkey = pubkey!("11111111111111111111111111111111");
}

pub use keys::*;

pub const ONE_U256: U256 = U256([1, 0, 0, 0]);
pub const D9_U256: U256 = U256([1_000_000_000, 0, 0, 0]);
pub const D18_U256: U256 = U256([1_000_000_000_000_000_000, 0, 1, 0]);
pub const D9_U128: u128 = 1_000_000_000;
pub const D18_U128: u128 = 1_000_000_000_000_000_000;

pub const MAX_DAO_FEE: u128 = 500_000_000_000_000_000;
pub const MAX_FEE_FLOOR: u128 = 1_500_000_000_000_000;
pub const FEE_DENOMINATOR: u128 = 1_000_000_000_000_000_000;

pub const MAX_TVL_FEE: u128 = 100_000_000_000_000_000;
pub const DAY_IN_SECONDS: u64 = 86400;
pub const YEAR_IN_SECONDS: u64 = 365 * DAY_IN_SECONDS;

pub const MAX_MINT_FEE: u128 = 50_000_000_000_000_000;

pub const MIN_AUCTION_LENGTH: u64 = 60;
pub const MAX_AUCTION_LENGTH: u64 = 604800;

pub const MAX_TTL: u64 = 604800 * 4;

pub const MAX_RATE: u128 = 1_000_000_000_000_000_000_000_000_000;

pub const MAX_PRICE_RANGE: u128 = D9_U128;
pub const MAX_TOKEN_PRICE_RANGE: u128 = 100;

pub const MAX_TOKEN_PRICE: u128 = D18_U128 * D18_U128;

pub const RESTRICTED_AUCTION_BUFFER: u64 = 0;

pub const MAX_FEE_RECIPIENTS: usize = 64;

pub const MAX_FEE_RECIPIENTS_PORTION: u128 = 1_000_000_000_000_000_000;

pub const MAX_INFINIX_TOKEN_AMOUNT: usize = 100;

pub const MAX_USER_PENDING_BASKET_TOKEN_AMOUNTS: usize = 110;

pub const MAX_REBALANCE_DETAILS_TOKENS: usize = 30;

pub const MAX_CONCURRENT_AUCTIONS: usize = 16;

pub const MAX_REWARD_TOKENS: usize = 4;

pub const MAX_REWARD_HALF_LIFE: u64 = 604800 * 2;
pub const MIN_REWARD_HALF_LIFE: u64 = 604800;

pub const LN_2: u128 = 693_147_180_559_945_309;

pub const INFINIX_PROGRAM_ID: Pubkey = pubkey!("DTF4yDGBkXJ25Ech1JVQpfwVb1vqYW4RJs5SuGNWdDev");

pub const REWARDS_PROGRAM_ID: Pubkey = pubkey!("7GiMvNDHVY8PXWQLHjSf1REGKpiDsVzRr4p7Y3xGbSuf");

pub enum PendingBasketType {
    MintProcess,
    RedeemProcess,
}