use anchor_lang::prelude::*;

#[account]
#[derive(Default, InitSpace)]
pub struct DAOFeeConfig {
    pub bump: u8,
    pub fee_recipient: Pubkey,
    pub default_fee_numerator: u128,
    pub default_fee_floor: u128,
}

impl DAOFeeConfig {
    pub const SIZE: usize = 8 + DAOFeeConfig::INIT_SPACE;
}

#[account]
#[derive(Default, InitSpace)]
pub struct InfinixFeeConfig {
    pub bump: u8,
    pub fee_numerator: u128,
    pub fee_floor: u128,
}

impl InfinixFeeConfig {
    pub const SIZE: usize = 8 + InfinixFeeConfig::INIT_SPACE;
}

#[account]
#[derive(Default, InitSpace)]
pub struct ProgramRegistrar {
    pub bump: u8,

    pub accepted_programs: [Pubkey; ProgramRegistrar::MAX_ACCEPTED_PROGRAMS],
}

impl ProgramRegistrar {
    pub const SIZE: usize = 8 + ProgramRegistrar::INIT_SPACE;

    pub const MAX_ACCEPTED_PROGRAMS: usize = 8;
}
