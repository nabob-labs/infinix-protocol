use crate::utils::RebalanceDetails;
use anchor_lang::prelude::*;

#[event]
pub struct InfinixCreated {
    pub infinix_token_mint: Pubkey,
}

#[event]
pub struct InfinixKilled {}

#[event]
pub struct BasketTokenAdded {
    pub token: Pubkey,
}

#[event]
pub struct BasketTokenRemoved {
    pub token: Pubkey,
}

#[event]
pub struct TVLFeeSet {
    pub new_fee: u128,
}

#[event]
pub struct MintFeeSet {
    pub new_fee: u128,
}

#[event]
pub struct FeeRecipientSet {
    pub recipient: Pubkey,
    pub portion: u128,
}

#[event]
pub struct TVLFeePaid {
    pub recipient: Pubkey,
    pub amount: u64,
}

#[event]
pub struct ProtocolFeePaid {
    pub recipient: Pubkey,
    pub amount: u64,
}

#[event]
pub struct AuctionOpened {
    pub auction_id: u64,
    pub nonce: u64,
    pub start_price: u128,
    pub end_price: u128,
    pub start: u64,
    pub end: u64,
}

#[event]
pub struct RebalanceStarted {
    pub nonce: u64,
    pub infinix: Pubkey,
    pub started_at: u64,
    pub restricted_until: u64,
    pub available_until: u64,
    pub details: RebalanceDetails,
}

#[event]
pub struct AuctionClosed {
    pub auction_id: u64,
}

#[event]
pub struct AuctionBid {
    pub auction_id: u64,
    pub sell_amount: u64,
    pub bought_amount: u64,
}

#[event]
pub struct AuctionLengthSet {
    pub new_auction_length: u64,
}

#[event]
pub struct MandateSet {
    pub new_mandate: Pubkey,
}
