use crate::utils::{
    structs::FeeRecipient, FixedSizeString, InfinixTokenBasket, PricesInAuction, RebalanceDetails,
    UserTokenBasket,
};
use anchor_lang::prelude::*;
use shared::constants::MAX_FEE_RECIPIENTS;

#[account]
#[derive(Default, InitSpace)]
pub struct Actor {
    pub bump: u8,
    pub authority: Pubkey,
    pub infinix: Pubkey,
    pub roles: u8,
}

impl Actor {
    pub const SIZE: usize = 8 + Actor::INIT_SPACE;
}

#[account(zero_copy)]
#[derive(InitSpace, Default)]
#[repr(C)]
pub struct Infinix {
    pub bump: u8,
    pub status: u8,
    pub _padding: [u8; 14],
    pub infinix_token_mint: Pubkey,
    pub tvl_fee: u128,
    pub mint_fee: u128,
    pub dao_pending_fee_shares: u128,
    pub fee_recipients_pending_fee_shares: u128,
    pub auction_length: u64,
    pub last_poke: u64,
    pub mandate: FixedSizeString,
    pub fee_recipients_pending_fee_shares_to_be_minted: u128,
}

impl Infinix {
    pub const SIZE: usize = 8 + Infinix::INIT_SPACE;
}

#[account(zero_copy)]
#[derive(InitSpace)]
pub struct FeeRecipients {
    pub bump: u8,
    pub _padding: [u8; 7],
    pub distribution_index: u64,
    pub infinix: Pubkey,
    pub fee_recipients: [FeeRecipient; MAX_FEE_RECIPIENTS],
}

impl FeeRecipients {
    pub const SIZE: usize = 8 + FeeRecipients::INIT_SPACE;
}

impl Default for FeeRecipients {
    fn default() -> Self {
        Self {
            bump: 0,
            _padding: [0; 7],
            distribution_index: 0,
            infinix: Pubkey::default(),
            fee_recipients: [FeeRecipient::default(); MAX_FEE_RECIPIENTS],
        }
    }
}

#[account(zero_copy)]
#[derive(InitSpace)]
pub struct FeeDistribution {
    pub bump: u8,
    pub _padding: [u8; 7],
    pub index: u64,
    pub infinix: Pubkey,
    pub cranker: Pubkey,
    pub amount_to_distribute: u128,
    pub fee_recipients_state: [FeeRecipient; MAX_FEE_RECIPIENTS],
}

impl FeeDistribution {
    pub const SIZE: usize = 8 + FeeDistribution::INIT_SPACE;
}

impl Default for FeeDistribution {
    fn default() -> Self {
        Self {
            bump: 0,
            _padding: [0; 7],
            index: 0,
            infinix: Pubkey::default(),
            cranker: Pubkey::default(),
            amount_to_distribute: 0,
            fee_recipients_state: [FeeRecipient::default(); MAX_FEE_RECIPIENTS],
        }
    }
}


#[account(zero_copy)]
#[derive(InitSpace, Default)]
pub struct InfinixBasket {
    pub bump: u8,
    pub _padding: [u8; 7],
    pub infinix: Pubkey,
    pub basket: InfinixTokenBasket,
}

impl InfinixBasket {
    pub const SIZE: usize = 8 + InfinixBasket::INIT_SPACE;
}

#[account(zero_copy)]
#[derive(InitSpace, Default)]
pub struct UserPendingBasket {
    pub bump: u8,
    pub _padding: [u8; 7],
    pub owner: Pubkey,
    pub infinix: Pubkey,
    pub basket: UserTokenBasket,
}

impl UserPendingBasket {
    pub const SIZE: usize = 8 + UserPendingBasket::INIT_SPACE;
}

#[account(zero_copy)]
#[derive(Default, InitSpace)]
#[repr(C)]
pub struct Rebalance {
    pub bump: u8,
    pub all_rebalance_details_added: u8,
    pub _padding: [u8; 6],
    pub infinix: Pubkey,
    pub current_auction_id: u64,
    pub nonce: u64,
    pub started_at: u64,
    pub restricted_until: u64,
    pub available_until: u64,
    pub details: RebalanceDetails,
}

impl Rebalance {
    pub const SIZE: usize = 8 + RebalanceDetails::INIT_SPACE;
}

#[account(zero_copy)]
#[derive(Default, InitSpace)]
pub struct Auction {
    pub bump: u8,
    pub _padding: [u8; 7],
    pub id: u64,
    pub nonce: u64,
    pub _padding2: [u8; 8],
    pub infinix: Pubkey,
    pub sell_mint: Pubkey,
    pub buy_mint: Pubkey,
    pub sell_limit: u128,
    pub buy_limit: u128,
    pub start: u64,
    pub end: u64,
    pub prices: PricesInAuction,
}

impl Auction {
    pub const SIZE: usize = 8 + Auction::INIT_SPACE;
}

#[account()]
#[derive(Default, InitSpace)]
pub struct AuctionEnds {
    pub bump: u8,
    pub rebalance_nonce: u64,
    pub token_mint_1: Pubkey,
    pub token_mint_2: Pubkey,
    pub end_time: u64,
}

impl AuctionEnds {
    pub const SIZE: usize = 8 + Auction::INIT_SPACE;
}
