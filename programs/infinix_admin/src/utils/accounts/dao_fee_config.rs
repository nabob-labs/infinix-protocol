use anchor_lang::prelude::*;
use shared::check_condition;
use shared::constants::{FEE_DENOMINATOR, MAX_DAO_FEE, MAX_FEE_FLOOR};
use shared::errors::ErrorCode;

use crate::state::{DAOFeeConfig, InfinixFeeConfig};

pub struct FeeDetails {
    pub fee_recipient: Pubkey,
    pub scaled_fee_denominator: u128,
    pub scaled_fee_numerator: u128,
    pub scaled_fee_floor: u128,
}

impl DAOFeeConfig {
    #[cfg(not(tarpaulin_include))]
    pub fn init_or_update_dao_fee_config(
        dao_fee_config: &mut Account<DAOFeeConfig>,
        context_bump: u8,
        fee_recipient: Option<Pubkey>,
        scaled_default_fee_numerator: Option<u128>,
        scaled_default_fee_floor: Option<u128>,
    ) -> Result<()> {
        let account_info_dao_fee_config = dao_fee_config.to_account_info();

        let data = account_info_dao_fee_config.try_borrow_mut_data()?;
        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);

        let discriminator = u64::from_be_bytes(disc_bytes);

        drop(data);

        if discriminator == 0 {
            dao_fee_config.bump = context_bump;
            dao_fee_config.fee_recipient = fee_recipient.ok_or(ErrorCode::InvalidFeeRecipient)?;
            dao_fee_config.default_fee_numerator = scaled_default_fee_numerator.unwrap_or(MAX_DAO_FEE);
            dao_fee_config.default_fee_floor = scaled_default_fee_floor.unwrap_or(MAX_FEE_FLOOR);
        } else {
            check_condition!(dao_fee_config.bump == context_bump, InvalidBump);

            if let Some(fee_recipient) = fee_recipient {
                dao_fee_config.fee_recipient = fee_recipient;
            }

            if let Some(scaled_default_fee_numerator) = scaled_default_fee_numerator {
                dao_fee_config.default_fee_numerator = scaled_default_fee_numerator;
            }

            if let Some(scaled_fee_floor) = scaled_default_fee_floor {
                dao_fee_config.default_fee_floor = scaled_fee_floor;
            }

        }

        Ok(())
    }

    #[cfg(not(tarpaulin_include))]
    pub fn get_fee_details(&self, infinix_fee_config: &AccountInfo) -> Result<FeeDetails> {
        let mut fee_details = FeeDetails {
            fee_recipient: self.fee_recipient,
            scaled_fee_denominator: FEE_DENOMINATOR,
            scaled_fee_numerator: self.default_fee_numerator,
            scaled_fee_floor: self.default_fee_floor,
        };

        if !infinix_fee_config.data_is_empty() {
            let infinix_fee_config_data = infinix_fee_config.try_borrow_mut_data()?;

            let mut disc_bytes = [0u8; 8];
            disc_bytes.copy_from_slice(&infinix_fee_config_data[..8]);
            let discriminator = u64::from_be_bytes(disc_bytes);

            if discriminator == 0 {
                return Ok(fee_details);
            }

            let infinix_fee_config = InfinixFeeConfig::try_deserialize(&mut &infinix_fee_config_data[..])?;

            fee_details.scaled_fee_denominator = infinix_fee_config.fee_numerator;
            fee_details.scaled_fee_floor = infinix_fee_config.fee_floor;
        }

        Ok(fee_details)
    }
}