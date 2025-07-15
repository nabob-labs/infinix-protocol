use anchor_lang::prelude::*;

#[derive(
    AnchorSerialize, AnchorDeserialize, Default, Clone, Copy, PartialEq, Eq, Debug, InitSpace,
)]
pub enum InfinixStatus {
    #[default]
    Initializing = 0,
    Initialized = 1,
    Killed = 2,
    Migrating = 3,
}

impl From<u8> for InfinixStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => InfinixStatus::Initializing,
            1 => InfinixStatus::Initialized,
            2 => InfinixStatus::Killed,
            3 => InfinixStatus::Migrating,
            _ => panic!("Invalid enum value"),
        }
    }
}

impl InfinixStatus {
    pub fn try_from(value: u8) -> Option<Self> {
        match value {
            0 => Some(InfinixStatus::Initializing),
            1 => Some(InfinixStatus::Initialized),
            2 => Some(InfinixStatus::Killed),
            3 => Some(InfinixStatus::Migrating),
            _ => None,
        }
    }
}
