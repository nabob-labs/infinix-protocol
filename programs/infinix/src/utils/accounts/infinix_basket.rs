use crate::events::{BasketTokenAdded, BasketTokenRemoved};
use crate::state::InfinixBasket;
use crate::InfinixTokenAmount;
use anchor_lang::prelude::*;
use shared::check_condition;
use shared::constants::MAX_INFINIX_TOKEN_AMOUNTS;
use shared::errors::ErrorCode;
use shared::errors::ErrorCode::*;
use shared::utils::{Decimal, Rounding};

impl InfinixBasket {
    #[cfg(not(tarpaulin_include))]
    pub fn process_init_if_needed(
        account_loader_infinix_basket: &mut AccountLoader<InfinixBasket>,
        context_bump: u8,
        infinix: &Pubkey,
        added_infinix_token_amounts: &Vec<InfinixTokenAmount>,
    ) -> Result<()> {
        let account_info_infinix_basket = account_loader_infinix_basket.to_account_info();

        let data = account_info_infinix_basket.try_borrow_mut_data()?;
        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);

        let discriminator = u64::from_le_bytes(disc_bytes);

        drop(data);

        if discriminator == 0 {
            let infinix_basket = &mut account_loader_infinix_basket.load_init()?;

            infinix_basket.bump = context_bump;
            infinix_basket.infinix = *infinix;
            infinix_basket.basket.token_amounts =
                [InfinixTokenAmount::default(); MAX_INFINIX_TOKEN_AMOUNTS];
            infinix_basket.add_tokens_to_basket(added_infinix_token_amounts)?;
        } else {
            let infinix_basket = &mut account_loader_infinix_basket.load_mut()?;

            check_condition!(infinix_basket.bump == context_bump, InvalidBump);

            infinix_basket.add_tokens_to_basket(added_infinix_token_amounts)?;
        }

        Ok(())
    }

    pub fn add_tokens_to_basket(
        &mut self,
        infinix_token_amounts: &Vec<InfinixTokenAmount>,
    ) -> Result<()> {
        for infinix_token_amount in infinix_token_amounts {
            check_condition!(
                infinix_token_amount.mint != Pubkey::default(),
                InvalidAddedTokenMints
            );

            let token_is_present = self
                .basket
                .token_amounts
                .iter_mut()
                .find(|ta| ta.mint == infinix_token_amount.mint);

            if let Some(slot_to_update) = token_is_present {
                slot_to_update.amount = slot_to_update
                    .amount
                    .checked_add(infinix_token_amount.amount)
                    .ok_or(ErrorCode::MathOverflow)?;

                continue;
            }

            let empty_slot = self
                .basket
                .token_amounts
                .iter_mut()
                .find(|ta| ta.mint == Pubkey::default());

            if let Some(slot) = empty_slot {
                slot.mint = infinix_token_amount.mint;
                slot.amount = infinix_token_amount.amount;
                continue;
            }

            return Err(error!(MaxNumberOfTokensReached));
        }
        Ok(())
    }

    pub fn remove_tokens_from_basket(
        &mut self,
        infinix_token_amounts: &Vec<InfinixTokenAmount>,
    ) -> Result<()> {
        for infinix_token_amount in infinix_token_amounts {
            if let Some(slot_to_update) = self
                .basket
                .token_amounts
                .iter_mut()
                .find(|ta| ta.mint == infinix_token_amount.mint)
            {
                slot_to_update.amount = slot_to_update
                    .amount
                    .checked_sub(infinix_token_amount.amount)
                    .ok_or(ErrorCode::MathOverflow)?;
            } else {
                return Err(error!(InvalidRemovedTokenMints));
            }
        }
        Ok(())
    }

    pub fn remove_token_mint_from_basket(&mut self, mint: Pubkey) -> Result<()> {
        if let Some(slot_to_update) = self
            .basket
            .token_amounts
            .iter_mut()
            .find(|ta| ta.mint == mint)
        {
            slot_to_update.amount = 0;
            slot_to_update.mint = Pubkey::default();
            emit!(BasketTokenRemoved { token: mint })
        } else {
            return Err(error!(InvalidRemovedTokenMints));
        }
        Ok(())
    }

    pub fn get_total_number_of_mints(&self) -> u8 {
        self.basket
            .token_amounts
            .iter()
            .filter(|ta| ta.mint != Pubkey::default())
            .count() as u8
    }

    pub fn get_token_amount_in_infinix_basket(&self, mint: &Pubkey) -> Result<u64> {
        let token_amount = self.basket.token_amounts.iter().find(|ta| ta.mint == *mint);

        if let Some(token_amount) = token_amount {
            Ok(token_amount.amount)
        } else {
            Err(error!(TokenMintNotInOldInfinixBasket))
        }
    }

    pub fn get_token_amount_in_infinix_basket_or_zero(&self, mint: &Pubkey) -> u64 {
        self.get_token_amount_in_infinix_basket(mint).unwrap_or(0)
    }

    pub fn get_token_presence_per_share_in_basket(
        &self,
        mint: &Pubkey,
        scaled_infinix_token_total_supply: &Decimal,
    ) -> Result<u128> {
        let total_token_amount = self.get_token_amount_in_infinix_basket_or_zero(mint);
        if total_token_amount == 0 || scaled_infinix_token_total_supply.is_zero() {
            return Ok(0);
        }

        Decimal::from_token_amount(total_token_amount)?
            .mul(&Decimal::ONE_E18)?
            .div(&scaled_infinix_token_total_supply)?
            .to_scaled(Rounding::Ceiling)
    }
}
