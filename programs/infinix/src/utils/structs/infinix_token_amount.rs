use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use shared::constants::MAX_INFINIX_TOKEN_AMOUNT;

#[derive(
    AnchorSerialize,
    AnchorDeserialize,
    Clone,
    Copy,
    Default,
    InitSpace,
    Zeroable,
    Pod,
    PartialEq,
    Debug,
)]
#[repr(C)]
pub struct InfinixTokenAmount {
    pub mint: Pubkey,
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace)]
#[repr(C)]
pub struct InfinixTokenBasket {
    pub token_amounts: [InfinixTokenAmount; MAX_INFINIX_TOKEN_AMOUNT],
}

impl Default for InfinixTokenBasket {
    fn default() -> Self {
        Self {
            token_amounts: [InfinixTokenAmount::default(); MAX_INFINIX_TOKEN_AMOUNT],
        }
    }
}

unsafe impl Pod for InfinixTokenBasket {}
unsafe impl Zeroable for InfinixTokenBasket {}
