use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
use shared::constants::MAX_USER_PENDING_BASKET_TOKEN_AMOUNTS;

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
pub struct TokenAmount {
    pub mint: Pubkey,
    pub amount_for_minting: u64,
    pub amount_for_redeeming: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace)]
#[repr(C)]
pub struct UserTokenBasket {
    pub token_amounts: [TokenAmount; MAX_USER_PENDING_BASKET_TOKEN_AMOUNTS],
}

impl Default for UserTokenBasket {
    fn default() -> Self {
        Self {
            token_amounts: [TokenAmount::default(); MAX_USER_PENDING_BASKET_TOKEN_AMOUNTS],
        }
    }
}

unsafe impl Pod for UserTokenBasket {}
unsafe impl Zeroable for UserTokenBasket {}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct MinimumOutForTokenAmount {
    pub mint: Pubkey,
    pub minimum_out: u64,
}
