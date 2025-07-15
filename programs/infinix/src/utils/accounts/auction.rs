use std::cell::RefMut;
use crate::state::{Auction, AuctionEnds, Infinix, InfinixBasket, Rebalance};
use crate::utils::structs::AuctionStatus;
use crate::utils::{BasketRange, OpenAuctionConfig, PricesInAuction};
use anchor_lang::prelude::*;
use shared::constants::{MAX_RATE, MAX_TTL};
use shared::errors::ErrorCode;
use shared::utils::math_util::Decimal;
use shared::utils::Rounding;
use shared::{check_condition, constants::AUCTION_SEEDS};

impl Auction {
    pub fn validate_auction(&self, auction_pubkey: &Pubkey, infinix_pubkey: &Pubkey) -> Result<()> {
        let auction_id = self.id.to_be_bytes();

        check_condition!(
            (*auction_pubkey, self.bump)
                == Pubkey::find_program_address(
                    &[
                        AUCTION_SEEDS,
                        infinix_pubkey.as_ref(),
                        self.nonce.to_be_bytes().as_ref(),
                        auction_id.as_ref(),
                    ],
                    &crate::id()
                ),
            InvalidPda
        );

        Ok(())
    }

    pub fn validate_auction_approve(
        scaled_sell_limit: &BasketRange,
        scaled_buy_limit: &BasketRange,
        scaled_prices: &PricesInAuction,
        ttl: u64,
    ) -> Result<()> {
        check_condition!(
            scaled_sell_limit.high <= MAX_RATE
                && scaled_sell_limit.low <= scaled_sell_limit.spot
                && scaled_sell_limit.high >= scaled_sell_limit.spot,
            InvalidSellLimit
        );

        check_condition!(
            scaled_buy_limit.spot != 0
                && scaled_buy_limit.high <= MAX_RATE
                && scaled_buy_limit.low <= scaled_buy_limit.spot
                && scaled_buy_limit.high >= scaled_buy_limit.spot,
            InvalidBuyLimit
        );

        check_condition!(scaled_prices.start >= scaled_prices.end, InvalidPrices);

        check_condition!(ttl <= MAX_TTL, InvalidTtl);
        Ok(())
    }

    pub fn open_auction(
        &mut self,
        infinix: &mut RefMut<'_, Infinix>,
        infinix_basket: &mut InfinixBasket,
        auction_ends: &mut AuctionEnds,
        raw_infinix_token_supply: u64,
        rebalance: &mut RefMut<'_, Rebalance>,
        sell_mint: &Pubkey,
        buy_mint: &Pubkey,
        current_time: u64,
        auction_buffer: u64,
        config: Option<OpenAuctionConfig>,
        is_permissionless: bool,
    ) -> Result<()> {
        Ok(())
    }

    pub fn try_get_status(&self, current_time: u64) -> Option<AuctionStatus> {
        todo!()
    }

    pub fn calculate_k(&self) -> Result<u128> {
        todo!()
    }

    pub fn auction_length(&self) -> Result<u64> {
        todo!()
    }

    pub fn get_price(&self, current_time: u64) -> Result<u128> {
        todo!()
    }

    pub fn get_bid(
        &self,
        infinix: &Infinix,
        infinix_basket: &InfinixBasket,
        raw_infinix_token_supply: u64,
        current_time: u64,
        raw_sell_amount: u64,
        raw_max_buy_amount: u64,
    ) -> Result<(u64, u64, Decimal, Decimal)> {
        let scaled_price = Decimal::from_scaled(self.get_price(current_time)?);
        //let scaled_infinix_token_total_supply = infinix.get_total_supply(raw_infinix_token_supply)?;
        //let raw_sell_balance = infinix_basket.get
        todo!()
    }
}
