use anchor_lang::prelude::*;
use shared::check_condition;
use shared::errors::ErrorCode;

use crate::state::{DAOFeeConfig, InfinixFeeConfig};

impl InfinixFeeConfig {
    #[cfg(not(tarpaulin_include))]
    pub fn init_or_update_infinix_fee_config(
        infinix_fee_config: &mut Account<InfinixFeeConfig>,
        dao_fee_config: &DAOFeeConfig,
        context_bump: u8,
        fee_numerator: Option<u128>,
        fee_floor: Option<u128>,
    ) -> Result<()> {
        let account_info_infinix_fee_config = infinix_fee_config.to_account_info();

        let data = account_info_infinix_fee_config.try_borrow_mut_data()?;
        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);

        let discriminator = u64::from_le_bytes(disc_bytes);

        drop(data);

        if discriminator == 0 {
            infinix_fee_config.bump = context_bump;
            infinix_fee_config.fee_numerator =
                fee_numerator.unwrap_or(dao_fee_config.default_fee_numerator);
            infinix_fee_config.fee_floor = fee_floor.unwrap_or(dao_fee_config.default_fee_floor);
        } else {
            check_condition!(infinix_fee_config.bump == context_bump, InvalidBump);

            if let Some(fee_numerator) = fee_numerator {
                infinix_fee_config.fee_numerator = fee_numerator;
            }

            if let Some(fee_floor) = fee_floor {
                infinix_fee_config.fee_floor = fee_floor;
            }
        }

        Ok(())
    }
}
