use crate::state::{AuctionEnds, InfinixBasket, Rebalance};
use crate::utils::structs::InfinixStatus;
use crate::{
    events::AuctionOpened,
    state::{Auction, Infinix},
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use infinix_admin::state::DAOFeeConfig;
use infinix_admin::ID as INFINIX_ADMIN_PROGRAM_ID;
use shared::check_condition;
use shared::constants::{
    AUCTION_ENDS_SEEDS, AUCTION_SEEDS, DAO_FEE_CONFIG_SEEDS, INFINIX_BASKET_SEEDS,
    INFINIX_FEE_CONFIG_SEEDS, REBALANCE_SEEDS, RESTRICTED_AUCTION_BUFFER,
};
use shared::errors::ErrorCode;
use crate::utils::Role::AuctionLauncher;

#[derive(Accounts)]
#[instruction(token_1: Pubkey, token_2: Pubkey)]
pub struct OpenAuctionPermissionless<'info> {
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub infinix: AccountLoader<'info, Infinix>,
    #[account(
        init,
        payer = user,
        seeds = [AUCTION_SEEDS, infinix.key().as_ref(), rebalance.load()?.nonce.to_le_bytes().as_ref(), rebalance.load()?.get_next_auction_id().to_le_bytes().as_ref()],
        bump,
        space = Auction::SIZE,
    )]
    pub auction: AccountLoader<'info, Auction>,
    #[account()]
    pub buy_mint: InterfaceAccount<'info, Mint>,
    #[account()]
    pub sell_mint: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        seeds = [REBALANCE_SEEDS, infinix.key().as_ref()],
        bump = rebalance.load()?.bump,
    )]
    pub rebalance: AccountLoader<'info, Rebalance>,
    #[account()]
    pub infinix_token_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        seeds = [INFINIX_BASKET_SEEDS, infinix.key().as_ref()],
        bump
    )]
    pub infinix_basket: AccountLoader<'info, InfinixBasket>,
    #[account(
        init_if_needed,
        payer = user,
        seeds = [
            AUCTION_ENDS_SEEDS,
            infinix.key().as_ref(),
            &rebalance.load()?.nonce.to_le_bytes(),
            token_1.to_bytes().as_ref(),
            token_2.to_bytes().as_ref(),
        ],
        bump,
        space = AuctionEnds::SIZE,
    )]
    pub auction_ends: Account<'info, AuctionEnds>,
    #[account(
        seeds = [DAO_FEE_CONFIG_SEEDS],
        bump,
        seeds::program = INFINIX_ADMIN_PROGRAM_ID,
    )]
    pub dao_fee_config: Account<'info, DAOFeeConfig>,
    #[account(
        seeds = [INFINIX_FEE_CONFIG_SEEDS, infinix.key().as_ref()],
        bump,
        seeds::program = INFINIX_ADMIN_PROGRAM_ID,
    )]
    pub infinix_fee_config: UncheckedAccount<'info>,
}

impl OpenAuctionPermissionless<'_> {
    pub fn validate(
        &self,
        infinix: &Infinix,
        rebalance: &Rebalance,
        token_1: Pubkey,
        token_2: Pubkey,
    ) -> Result<u8> {
        infinix.validate_infinix(
            &self.infinix.key(),
            None,
            None,
            Some(vec![InfinixStatus::Initialized]),
        )?;

        check_condition!(
            self.infinix_token_mint.key() == infinix.infinix_token_mint,
            InvalidInfinixTokenMint
        );
        let bump = self
            .auction_ends
            .validate_auction_ends_with_keys_and_get_bump(
                &self.auction_ends.key(),
                &self.infinix.key(),
                self.sell_mint.key(),
                self.buy_mint.key(),
                rebalance.nonce,
            )?;
        let (token_1_expected, token_2_expected) =
            AuctionEnds::keys_pair_in_order(self.sell_mint.key(), self.buy_mint.key());
        check_condition!(token_1 == token_1_expected, InvalidTokenMint);
        check_condition!(token_2 == token_2_expected, InvalidTokenMint);

        Ok(bump)
    }
}

pub fn handler(
    ctx: Context<OpenAuctionPermissionless>,
    token_1: Pubkey,
    token_2: Pubkey,
) -> Result<()> {
    let infinix = &mut ctx.accounts.infinix.load_mut()?;
    let auction = &mut ctx.accounts.auction.load_init()?;
    auction.bump = ctx.bumps.auction;
    let rebalance = &mut ctx.accounts.rebalance.load_mut()?;
    let infinix_basket = &ctx.accounts.infinix_basket.load()?;

    let auction_ends_bump = ctx.accounts.validate(infinix, rebalance, token_1, token_2)?;

    let current_time = Clock::get()?.unix_timestamp;
    {
        // Poke folio
        let fee_details = ctx
            .accounts
            .dao_fee_config
            .get_fee_details(&ctx.accounts.infinix_fee_config)?;

        infinix.poke(
            ctx.accounts.infinix_token_mint.supply,
            current_time,
            fee_details.scaled_fee_numerator,
            fee_details.scaled_fee_denominator,
            fee_details.scaled_fee_floor,
        )?;
    }

    let raw_infinix_token_supply = ctx.accounts.infinix_token_mint.supply;

    let auction_ends = &mut ctx.accounts.auction_ends;
    auction_ends.process_init_if_needed(
        auction_ends_bump,
        ctx.accounts.sell_mint.key(),
        ctx.accounts.buy_mint.key(),
        rebalance.nonce,
    )?;
    let current_time = current_time as u64;

    auction.open_auction(
        infinix,
        infinix_basket,
        auction_ends,
        raw_infinix_token_supply,
        rebalance,
        &ctx.accounts.sell_mint.key(),
        &ctx.accounts.buy_mint.key(),
        current_time,
        RESTRICTED_AUCTION_BUFFER,
        None,
        true,
    )?;

    emit!(AuctionOpened {
        auction_id: auction.id,
        nonce: auction.nonce,
        start_price: auction.prices.start,
        end_price: auction.prices.end,
        start: auction.start,
        end: auction.end,
    });

    Ok(())
}