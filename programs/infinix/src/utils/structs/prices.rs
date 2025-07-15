use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

#[derive(
    AnchorSerialize,
    AnchorDeserialize,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    InitSpace,
    Zeroable,
    Pod,
)]
#[repr(C)]
pub struct PricesInAuction {
    pub start: u128,
    pub end: u128,
}

#[derive(
    AnchorSerialize,
    AnchorDeserialize,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    InitSpace,
    Zeroable,
    Pod,
)]
#[repr(C)]
pub struct PricesInRebalance {
    pub low: u128,
    pub high: u128,
}
