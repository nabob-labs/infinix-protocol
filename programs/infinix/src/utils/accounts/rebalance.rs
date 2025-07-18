use crate::utils::{RebalanceDetails, RebalanceDetailsToken, RebalancePriceAndLimits};
use anchor_lang::prelude::*;
use shared::check_condition;
use shared::constants::{MAX_RATE, MAX_TOKEN_PRICE, MAX_TOKEN_PRICE_RANGE, MAX_TTL};
use shared::errors::ErrorCode;
use std::collections::HashSet;

use crate::state::Rebalance;

impl Rebalance {
    #[cfg(not(tarpaulin_include))]
    pub fn process_init_if_needed(
        account_loader_rebalance: &mut AccountLoader<Rebalance>,
        context_bump: u8,
        infinix: &Pubkey,
    ) -> Result<()> {
        let account_info_rebalance = account_loader_rebalance.to_account_info();

        let data = account_info_rebalance.try_borrow_mut_data()?;
        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);

        let discriminator = u64::from_le_bytes(disc_bytes);

        drop(data);

        if discriminator == 0 {
            // Not initialized yet
            let rebalance = &mut account_loader_rebalance.load_init()?;

            rebalance.bump = context_bump;
            rebalance.infinix = *infinix;
            rebalance.nonce = 0;
        } else {
            let rebalance = &mut account_loader_rebalance.load_mut()?;

            check_condition!(rebalance.bump == context_bump, InvalidBump);
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        self.all_rebalance_details_added = 0;
        self.current_auction_id = 0;
        self.details = RebalanceDetails::default();
    }

    pub fn start_rebalance(
        &mut self,
        current_time: u64,
        auction_launcher_window: u64,
        ttl: u64,
        mints: &[AccountInfo],
        prices_and_limits: Vec<RebalancePriceAndLimits>,
        all_rebalance_details_added: bool,
    ) -> Result<()> {
        check_condition!(ttl <= MAX_TTL, RebalanceTTLExceeded);
        check_condition!(
            ttl >= auction_launcher_window,
            RebalanceAuctionLauncherWindowTooLong
        );
        self.nonce = self
            .nonce
            .checked_add(1)
            .ok_or(error!(ErrorCode::MathOverflow))?;

        self.started_at = current_time;
        self.restricted_until = current_time + auction_launcher_window;
        self.available_until = current_time + ttl;

        self.clear();

        self.add_rebalance_details(mints, prices_and_limits, all_rebalance_details_added)?;
        Ok(())
    }

    pub fn add_rebalance_details(
        &mut self,
        mints: &[AccountInfo],
        prices_and_limits: Vec<RebalancePriceAndLimits>,
        all_rebalance_details_added: bool,
    ) -> Result<()> {
        check_condition!(
            self.open_for_detail_update(),
            RebalanceNotOpenForDetailUpdates
        );
        let mut hash_set = HashSet::new();

        check_condition!(
            mints.len() == prices_and_limits.len(),
            RebalanceMintsAndPricesAndLimitsLengthMismatch
        );

        let is_price_deferred = if self.details.tokens[0].mint != Pubkey::default() {
            self.details.tokens[0].prices.low == 0
        } else {
            !prices_and_limits.is_empty() && prices_and_limits[0].prices.low == 0
        };

        let mut mint_to_process_index = 0;
        for rebalance in self.details.tokens.iter_mut() {
            if mint_to_process_index >= mints.len() {
                break;
            }

            let is_not_empty = rebalance.mint != Pubkey::default();
            if is_not_empty {
                hash_set.insert(rebalance.mint);
                continue;
            }

            let mint = mints[mint_to_process_index].key();
            let limit = prices_and_limits[mint_to_process_index].limits;
            let prices = prices_and_limits[mint_to_process_index].prices;
            check_condition!(
                limit.high <= MAX_RATE && limit.low <= limit.spot && limit.spot <= limit.high,
                InvalidRebalanceLimit
            );

            check_condition!(
                limit.low != 0 || limit.high == 0,
                InvalidRebalanceLimitAllZeroOrAllGreaterThanZero
            );

            check_condition!(!hash_set.contains(&mint.key()), RebalanceTokenAlreadyAdded);

            if is_price_deferred {
                // If price for index 0 is deferred, then it is deferred for all of them.
                check_condition!(prices.low == 0 && prices.high == 0, InvalidPrices);
            } else {
                check_condition!(
                    prices.low != 0
                        && prices.high != 0
                        && prices.low <= prices.high
                        && prices.high <= MAX_TOKEN_PRICE
                        && prices.high / prices.low <= MAX_TOKEN_PRICE_RANGE,
                    InvalidPrices
                );
            }

            rebalance.mint = mint.key();
            rebalance.prices = prices;
            rebalance.limits = limit;
            hash_set.insert(mint.key());

            mint_to_process_index += 1;
        }

        self.all_rebalance_details_added = if all_rebalance_details_added { 1 } else { 0 };

        Ok(())
    }

    #[inline]
    pub fn open_for_detail_update(&self) -> bool {
        self.all_rebalance_details_added == 0
    }

    #[inline]
    pub fn rebalance_ready(&self) -> bool {
        self.all_rebalance_details_added == 1
    }

    pub fn get_token_details_pair(
        &self,
        sell_mint: &Pubkey,
        buy_mint: &Pubkey,
    ) -> (
        Option<&RebalanceDetailsToken>,
        Option<&RebalanceDetailsToken>,
    ) {
        let mut sell_details = None;
        let mut buy_details = None;

        for detail in self.details.tokens.iter() {
            if detail.mint == *sell_mint {
                sell_details = Some(detail);
            } else if detail.mint == *buy_mint {
                buy_details = Some(detail);
            }
        }

        (sell_details, buy_details)
    }

    pub fn get_token_details_pair_mut(
        &mut self,
        sell_mint: &Pubkey,
        buy_mint: &Pubkey,
    ) -> (
        Option<&mut RebalanceDetailsToken>,
        Option<&mut RebalanceDetailsToken>,
    ) {
        let mut sell_details = None;
        let mut buy_details = None;

        for detail in self.details.tokens.iter_mut() {
            if detail.mint == *sell_mint {
                sell_details = Some(detail);
            } else if detail.mint == *buy_mint {
                buy_details = Some(detail);
            }
        }

        (sell_details, buy_details)
    }

    pub fn get_next_auction_id(&self) -> u64 {
        self.current_auction_id + 1
    }
}
