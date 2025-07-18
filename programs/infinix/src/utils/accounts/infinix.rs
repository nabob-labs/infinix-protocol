use crate::utils::structs::{InfinixStatus, Role};
use crate::{
    events::TVLFeeSet,
    state::{Actor, Infinix},
};
use anchor_lang::prelude::*;
use shared::constants::{DAY_IN_SECONDS, YEAR_IN_SECONDS};
use shared::utils::{Decimal, Rounding, TokenResult};
use shared::{
    check_condition,
    constants::{INFINIX_SEEDS, MAX_TVL_FEE},
    errors::ErrorCode,
};

impl Infinix {
    #[allow(clippy::too_many_arguments)]
    #[cfg(not(tarpaulin_include))]
    pub fn validate_infinix(
        &self,
        infinix_pubkey: &Pubkey,
        actor: Option<&Account<'_, Actor>>,
        required_roles: Option<Vec<Role>>,
        expected_statuses: Option<Vec<InfinixStatus>>,
    ) -> Result<()> {
        let infinix_token_mint = self.infinix_token_mint.key();
        check_condition!(
            (*infinix_pubkey, self.bump)
                == Pubkey::find_program_address(
                    &[INFINIX_SEEDS, infinix_token_mint.as_ref()],
                    &crate::id()
                ),
            InvalidPda
        );

        if let (Some(actor), Some(required_roles)) = (actor, required_roles) {
            Infinix::validate_permission_for_action(actor, required_roles)?;
        }

        if let Some(expected_statuses) = expected_statuses {
            check_condition!(
                expected_statuses.contains(&InfinixStatus::from(self.status)),
                InvalidInfinixStatus
            );
        }
        Ok(())
    }

    #[cfg(not(tarpaulin_include))]
    fn validate_permission_for_action(
        actor: &Account<'_, Actor>,
        required_roles: Vec<Role>,
    ) -> Result<()> {
        let mut has_one_of_the_roles = false;

        for required_role in required_roles {
            if Role::has_role(actor.roles, required_role) {
                has_one_of_the_roles = true;
                break;
            }
        }

        check_condition!(has_one_of_the_roles, InvalidRole);

        Ok(())
    }

    pub fn set_tvl_fee(&mut self, scaled_new_fee_annually: u128) -> Result<()> {
        check_condition!(scaled_new_fee_annually <= MAX_TVL_FEE, TVLFeeTooHigh);

        if scaled_new_fee_annually == 0 {
            self.tvl_fee = 0;

            emit!(TVLFeeSet { new_fee: 0 });

            return Ok(());
        }

        let one_minus_fee = Decimal::ONE_E18.sub(&Decimal::from_scaled(scaled_new_fee_annually))?;

        let result = one_minus_fee.nth_root(YEAR_IN_SECONDS)?;

        let scaled_tvl_fee = Decimal::ONE_E18.sub(&result)?;

        check_condition!(
            scaled_new_fee_annually == 0 || scaled_tvl_fee != Decimal::ZERO,
            TVLFeeTooLow
        );

        self.tvl_fee = scaled_tvl_fee.to_scaled(Rounding::Floor)?;

        emit!(TVLFeeSet {
            new_fee: self.tvl_fee
        });

        Ok(())
    }

    pub fn calculate_fees_for_minting(
        &mut self,
        raw_user_shares: u64,
        scaled_dao_fee_numerator: u128,
        scaled_dao_fee_denominator: u128,
        scaled_dao_fee_floor: u128,
    ) -> Result<TokenResult> {
        let scaled_user_shares = Decimal::from_token_amount(raw_user_shares)?;
        let scaled_mint_fee = Decimal::from_scaled(self.mint_fee);

        let scaled_dao_fee_numerator = Decimal::from_scaled(scaled_dao_fee_numerator);
        let scaled_dao_fee_denominator = Decimal::from_scaled(scaled_dao_fee_denominator);
        let scaled_dao_fee_floor = Decimal::from_scaled(scaled_dao_fee_floor);

        // {share} = {share} * D18{1} / D18
        let mut scaled_total_fee_shares = scaled_user_shares
            .mul(&scaled_mint_fee)?
            .add(&Decimal::ONE_E18)?
            .sub(&Decimal::ONE)?
            .div(&Decimal::ONE_E18)?;

        let mut scaled_dao_fee_shares = scaled_total_fee_shares
            .mul(&scaled_dao_fee_numerator)?
            .add(&scaled_dao_fee_denominator)?
            .sub(&Decimal::ONE)?
            .div(&scaled_dao_fee_denominator)?;

        let scaled_min_dao_shares = scaled_user_shares
            .mul(&scaled_dao_fee_floor)?
            .add(&Decimal::ONE_E18)?
            .sub(&Decimal::ONE)?
            .div(&Decimal::ONE_E18)?;

        if scaled_dao_fee_shares < scaled_min_dao_shares {
            scaled_dao_fee_shares = scaled_min_dao_shares;
        }

        // 100% to DAO, if necessary
        if scaled_total_fee_shares < scaled_dao_fee_shares {
            scaled_total_fee_shares = scaled_dao_fee_shares.clone();
        }

        self.dao_pending_fee_shares = self
            .dao_pending_fee_shares
            .checked_add(scaled_dao_fee_shares.to_scaled(Rounding::Floor)?)
            .ok_or(ErrorCode::MathOverflow)?;

        self.fee_recipients_pending_fee_shares = self
            .fee_recipients_pending_fee_shares
            .checked_add(
                scaled_total_fee_shares
                    .sub(&scaled_dao_fee_shares)?
                    .to_scaled(Rounding::Floor)?,
            )
            .ok_or(ErrorCode::MathOverflow)?;

        scaled_total_fee_shares.to_token_amount(Rounding::Ceiling)
    }

    pub fn get_account_fee_until(&self, current_time: i64) -> Result<u64> {
        let current_time = current_time as u64;
        let account_fee_until = current_time
            .checked_div(DAY_IN_SECONDS)
            .ok_or(ErrorCode::MathOverflow)?
            .checked_mul(DAY_IN_SECONDS)
            .ok_or(ErrorCode::MathOverflow)?;

        Ok(account_fee_until)
    }

    pub fn poke(
        &mut self,
        raw_infinix_token_supply: u64,
        current_time: i64,
        scaled_dao_fee_numerator: u128,
        scaled_dao_fee_denominator: u128,
        scaled_dao_fee_floor: u128,
    ) -> Result<()> {
        let account_fee_until = self.get_account_fee_until(current_time)?;

        if account_fee_until.saturating_sub(self.last_poke) == 0 {
            return Ok(());
        }

        let (scaled_fee_recipients_pending_fee, scaled_dao_pending_fee_shares) = self
            .get_pending_fee_shares(
                raw_infinix_token_supply,
                account_fee_until,
                scaled_dao_fee_numerator,
                scaled_dao_fee_denominator,
                scaled_dao_fee_floor,
            )?;

        self.dao_pending_fee_shares = self
            .dao_pending_fee_shares
            .checked_add(scaled_dao_pending_fee_shares.to_scaled(Rounding::Floor)?)
            .ok_or(ErrorCode::MathOverflow)?;

        self.fee_recipients_pending_fee_shares = self
            .fee_recipients_pending_fee_shares
            .checked_add(scaled_fee_recipients_pending_fee.to_scaled(Rounding::Floor)?)
            .ok_or(ErrorCode::MathOverflow)?;

        self.last_poke = account_fee_until;
        Ok(())
    }

    pub fn get_total_supply(&self, raw_folio_token_supply: u64) -> Result<Decimal> {
        let scaled_total_supply = Decimal::from_token_amount(raw_folio_token_supply)?;

        scaled_total_supply
            .add(&Decimal::from_scaled(self.dao_pending_fee_shares))?
            .add(&Decimal::from_scaled(
                self.fee_recipients_pending_fee_shares,
            ))?
            .add(&Decimal::from_scaled(
                self.fee_recipients_pending_fee_shares_to_be_minted,
            ))
    }

    pub fn get_pending_fee_shares(
        &self,
        raw_infinix_token_supply: u64,
        account_until: u64,
        scaled_dao_fee_numerator: u128,
        scaled_dao_fee_denominator: u128,
        scaled_dao_fee_floor: u128,
    ) -> Result<(Decimal, Decimal)> {
        let scaled_total_supply_with_pending_fees =
            self.get_total_supply(raw_infinix_token_supply)?;

        let elapsed = account_until.saturating_sub(self.last_poke);

        // convert annual percentage to per-second for comparison with stored tvlFee
        // = 1 - (1 - feeFloor) ^ (1 / 31536000)
        // D18{1/s} = D18{1} - D18{1} * D18{1} ^ D18{1/s}
        let scaled_one_minus_fee_floor =
            Decimal::ONE_E18.sub(&Decimal::from_scaled(scaled_dao_fee_floor))?;

        let scaled_fee_floor =
            Decimal::ONE_E18.sub(&scaled_one_minus_fee_floor.nth_root(YEAR_IN_SECONDS)?)?;

        // Use higher of fee floor or TVL fee  D18{1/s}
        let scaled_tvl_fee = Decimal::from_scaled(self.tvl_fee);
        let scaled_tvl_fee_to_use = if scaled_fee_floor > scaled_tvl_fee {
            scaled_fee_floor.clone()
        } else {
            scaled_tvl_fee
        };

        let scaled_one_minus_tvl_fee = Decimal::ONE_E18.sub(&scaled_tvl_fee_to_use)?;
        let scaled_denominator = scaled_one_minus_tvl_fee.pow(elapsed)?;
        let scaled_fee_shares = scaled_total_supply_with_pending_fees
            .mul(&Decimal::ONE_E18)?
            .div(&scaled_denominator)?
            .sub(&scaled_total_supply_with_pending_fees)?;

        let scaled_correction = scaled_fee_floor
            .mul(&Decimal::ONE_E18)?
            .add(&scaled_tvl_fee_to_use)?
            .sub(&Decimal::ONE)?
            .div(&scaled_tvl_fee_to_use)?;

        let scaled_dao_ratio = Decimal::from_scaled(scaled_dao_fee_numerator)
            .mul(&Decimal::ONE_E18)?
            .add(&Decimal::from_scaled(scaled_dao_fee_denominator))?
            .sub(&Decimal::ONE)?
            .div(&Decimal::from_scaled(scaled_dao_fee_denominator))?;

        let scaled_dao_shares = if scaled_correction > scaled_dao_ratio {
            scaled_fee_shares
                .mul(&scaled_correction)?
                .add(&Decimal::ONE_E18)?
                .sub(&Decimal::ONE)?
                .div(&Decimal::ONE_E18)?
        } else {
            scaled_fee_shares
                .mul(&Decimal::from_scaled(scaled_dao_fee_numerator))?
                .add(&Decimal::from_scaled(scaled_dao_fee_denominator))?
                .sub(&Decimal::ONE)?
                .div(&Decimal::from_scaled(scaled_dao_fee_denominator))?
        };

        let scaled_fee_recipient_shares = scaled_fee_shares.sub(&scaled_dao_shares)?;

        Ok((scaled_fee_recipient_shares, scaled_dao_shares))
    }
}
