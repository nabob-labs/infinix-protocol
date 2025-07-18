use std::cell::RefMut;

use crate::utils::token_amount::{MinimumOutForTokenAmount, TokenAmount};
use crate::utils::InfinixTokenAmount;
use anchor_lang::prelude::*;
use shared::check_condition;
use shared::constants::{PendingBasketType, MAX_USER_PENDING_BASKET_TOKEN_AMOUNTS};
use shared::errors::ErrorCode;
use shared::errors::ErrorCode::InvalidAddedTokenMints;
use shared::errors::ErrorCode::*;
use shared::utils::math_util::Decimal;
use shared::utils::Rounding;

use crate::state::{Infinix, InfinixBasket, UserPendingBasket};

impl UserPendingBasket {
    #[cfg(not(tarpaulin_include))]
    pub fn process_init_if_needed(
        account_loader_user_pending_basket: &mut AccountLoader<UserPendingBasket>,
        context_bump: u8,
        owner: &Pubkey,
        infinix: &Pubkey,
        added_token_amounts: &Vec<TokenAmount>,
        can_add_new_mints: bool,
    ) -> Result<()> {
        let account_info_user_pending_basket = account_loader_user_pending_basket.to_account_info();

        let data = account_info_user_pending_basket.try_borrow_mut_data()?;
        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);

        let discriminator = u64::from_le_bytes(disc_bytes);

        drop(data);

        if discriminator == 0 {
            // Not initialized yet
            let user_pending_basket = &mut account_loader_user_pending_basket.load_init()?;

            user_pending_basket.bump = context_bump;
            user_pending_basket.owner = *owner;
            user_pending_basket.infinix = *infinix;
            user_pending_basket.basket.token_amounts =
                [TokenAmount::default(); MAX_USER_PENDING_BASKET_TOKEN_AMOUNTS];

            user_pending_basket.add_token_amounts_to_infinix(
                added_token_amounts,
                can_add_new_mints,
                PendingBasketType::MintProcess,
            )?;
        } else {
            let user_pending_basket = &mut account_loader_user_pending_basket.load_mut()?;

            check_condition!(user_pending_basket.bump == context_bump, InvalidBump);

            user_pending_basket.add_token_amounts_to_infinix(
                added_token_amounts,
                can_add_new_mints,
                PendingBasketType::MintProcess,
            )?;
        }

        Ok(())
    }

    pub fn add_token_amounts_to_infinix(
        &mut self,
        token_amounts: &Vec<TokenAmount>,
        can_add_new_mints: bool,
        pending_basket_type: PendingBasketType,
    ) -> Result<()> {
        match pending_basket_type {
            PendingBasketType::MintProcess => {
                for token_amount in token_amounts {
                    if let Some(slot_for_update) = self
                        .basket
                        .token_amounts
                        .iter_mut()
                        .find(|ta| ta.mint == token_amount.mint)
                    {
                        slot_for_update.amount_for_minting = token_amount
                            .amount_for_minting
                            .checked_add(slot_for_update.amount_for_minting)
                            .ok_or(ErrorCode::MathOverflow)?;
                    } else if can_add_new_mints {
                        if let Some(slot) = self
                            .basket
                            .token_amounts
                            .iter_mut()
                            .find(|ta| ta.mint == Pubkey::default())
                        {
                            slot.mint = token_amount.mint;
                            slot.amount_for_minting = token_amount.amount_for_minting;
                        } else {
                            // No available slot found, return an error
                            return Err(error!(InvalidAddedTokenMints));
                        }
                    } else {
                        return Err(error!(InvalidAddedTokenMints));
                    }
                }
            }
            PendingBasketType::RedeemProcess => {}
        }

        Ok(())
    }

    pub fn remove_token_amounts_from_infinix(
        &mut self,
        token_amounts: &Vec<TokenAmount>,
        needs_to_validate_mint_existence: bool,
        pending_basket_type: PendingBasketType,
    ) -> Result<()> {
        for token_amount in token_amounts {
            if let Some(slot_for_update) = self
                .basket
                .token_amounts
                .iter_mut()
                .find(|ta| ta.mint == token_amount.mint)
            {
                match pending_basket_type {
                    PendingBasketType::MintProcess => {
                        // Will crash if trying to remove more than actual balance
                        slot_for_update.amount_for_minting = slot_for_update
                            .amount_for_minting
                            .checked_sub(token_amount.amount_for_minting)
                            .ok_or(InvalidShareAmountProvided)?;
                    }
                    PendingBasketType::RedeemProcess => {
                        slot_for_update.amount_for_redeeming = slot_for_update
                            .amount_for_redeeming
                            .checked_sub(token_amount.amount_for_redeeming)
                            .ok_or(InvalidShareAmountProvided)?;
                    }
                }

                // Clear the slot if both amounts are 0.
                // This is an optimization to make sure that the User does not have to empty out the complete user pending basket if tokens in the folio change multiple times.
                if slot_for_update.amount_for_minting == 0
                    && slot_for_update.amount_for_redeeming == 0
                {
                    slot_for_update.mint = Pubkey::default();
                }
            } else if needs_to_validate_mint_existence {
                return Err(error!(InvalidRemovedTokenMints));
            }
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.basket
            .token_amounts
            .iter()
            .all(|ta| ta.amount_for_minting == 0 && ta.amount_for_redeeming == 0)
    }

    pub fn reset(&mut self) {
        self.basket.token_amounts = [TokenAmount::default(); MAX_USER_PENDING_BASKET_TOKEN_AMOUNTS];
    }

    #[allow(clippy::too_many_arguments)]
    #[cfg(not(tarpaulin_include))]
    pub fn to_assets(
        &mut self,
        raw_shares: u64,
        raw_infinix_token_supply: u64,
        infinix_basket: &mut RefMut<'_, InfinixBasket>,
        infinix: &mut RefMut<'_, Infinix>,
        pending_basket_type: PendingBasketType,
        current_time: i64,
        scaled_dao_fee_numerator: u128,
        scaled_dao_fee_denominator: u128,
        scaled_dao_fee_floor: u128,
        minimum_out_for_token_amounts: Vec<MinimumOutForTokenAmount>,
    ) -> Result<()> {
        infinix.poke(
            raw_infinix_token_supply,
            current_time,
            scaled_dao_fee_numerator,
            scaled_dao_fee_denominator,
            scaled_dao_fee_floor,
        )?;

        let scaled_total_supply_infinix_token =
            infinix.get_total_supply(raw_infinix_token_supply)?;
        let raw_shares = Decimal::from_token_amount(raw_shares)?;

        for infinix_token_account in infinix_basket.basket.token_amounts.iter_mut() {
            if infinix_token_account.mint == Pubkey::default() {
                continue;
            }

            let raw_user_amount = &mut self
                .basket
                .token_amounts
                .iter_mut()
                .find(|ta| ta.mint == infinix_token_account.mint)
                .ok_or(ErrorCode::MintMismatch)?;

            let scaled_infinix_token_balance =
                Decimal::from_token_amount(infinix_token_account.amount)?;

            let minimum_amount_out = minimum_out_for_token_amounts
                .iter()
                .find(|m| m.mint == infinix_token_account.mint)
                .map(|a| a.minimum_out);

            match pending_basket_type {
                PendingBasketType::MintProcess => {
                    UserPendingBasket::to_assets_for_minting(
                        raw_user_amount,
                        infinix_token_account,
                        &scaled_total_supply_infinix_token,
                        &scaled_infinix_token_balance,
                        &raw_shares,
                    )?;
                }
                PendingBasketType::RedeemProcess => {
                    UserPendingBasket::to_assets_for_redeeming(
                        raw_user_amount,
                        infinix_token_account,
                        &scaled_total_supply_infinix_token,
                        &scaled_infinix_token_balance,
                        &raw_shares,
                        minimum_amount_out,
                    )?;
                }
            }
        }
        Ok(())
    }

    pub fn to_assets_for_minting(
        raw_user_amount: &mut TokenAmount,
        infinix_token_amount: &mut InfinixTokenAmount,
        scaled_total_supply_infinix_token: &Decimal,
        scaled_infinix_token_balance: &Decimal,
        raw_shares: &Decimal,
    ) -> Result<()> {
        let scaled_calculated_shares =
            Decimal::from_token_amount(raw_user_amount.amount_for_minting)?
                .mul(scaled_total_supply_infinix_token)?
                .div(scaled_infinix_token_balance)?;

        check_condition!(
            scaled_calculated_shares >= *raw_shares,
            InvalidShareAmountProvided
        );

        // {tok} = {share} * {tok} / {share}
        let raw_user_amount_taken = raw_shares
            .mul(scaled_infinix_token_balance)?
            .div(scaled_total_supply_infinix_token)?
            .to_token_amount(Rounding::Ceiling)?;

        // Remove from pending amounts from the user's pending basket
        raw_user_amount.amount_for_minting = raw_user_amount
            .amount_for_minting
            .checked_sub(raw_user_amount_taken.0)
            .ok_or(ErrorCode::MathOverflow)?;

        // Add the amount to infinix token amount
        infinix_token_amount.amount = infinix_token_amount
            .amount
            .checked_add(raw_user_amount_taken.0)
            .ok_or(ErrorCode::MathOverflow)?;

        Ok(())
    }

    pub fn to_assets_for_redeeming(
        raw_user_amount: &mut TokenAmount,
        infinix_token_amount: &mut InfinixTokenAmount,
        scaled_total_supply_infinix_token: &Decimal,
        scaled_infinix_token_balance: &Decimal,
        raw_shares: &Decimal,
        minimum_amount_out: Option<u64>,
    ) -> Result<()> {
        let raw_amount_to_give_to_user = raw_shares
            .mul(scaled_infinix_token_balance)?
            .div(scaled_total_supply_infinix_token)?
            .to_token_amount(Rounding::Floor)?;

        // Add to pending amounts in the user's pending basket
        raw_user_amount.amount_for_redeeming = raw_user_amount
            .amount_for_redeeming
            .checked_add(raw_amount_to_give_to_user.0)
            .ok_or(ErrorCode::MathOverflow)?;

        // Remove the amount from infinix token amount
        infinix_token_amount.amount = infinix_token_amount
            .amount
            .checked_sub(raw_amount_to_give_to_user.0)
            .ok_or(ErrorCode::MathOverflow)?;

        if let Some(minimum_amount_out) = minimum_amount_out {
            check_condition!(
                raw_amount_to_give_to_user.0 >= minimum_amount_out,
                MinimumAmountOutNotMet
            );
        }

        Ok(())
    }
}
