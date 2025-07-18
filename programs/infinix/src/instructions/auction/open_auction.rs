use crate::state::{AuctionEnds, InfinixBasket, Rebalance};
use crate::utils::structs::{InfinixStatus, Role};
use crate::utils::{OpenAuctionConfig, PricesInAuction};
use crate::{
    events::AuctionOpened,
    state::{Actor, Auction, Infinix},
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use infinix_admin::state::DAOFeeConfig;
use infinix_admin::ID as INFINIX_ADMIN_PROGRAM_ID;
use shared::check_condition;
use shared::constants::{
    ACTOR_SEEDS, AUCTION_ENDS_SEEDS, AUCTION_SEEDS, DAO_FEE_CONFIG_SEEDS, INFINIX_BASKET_SEEDS,
    INFINIX_FEE_CONFIG_SEEDS, REBALANCE_SEEDS,
};
use shared::errors::ErrorCode;

#[derive(Accounts)]
#[instruction(token_1: Pubkey, token_2: Pubkey)]
pub struct OpenAuction<'info> {
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub auction_launcher: Signer<'info>,
    #[account(
        seeds = [ACTOR_SEEDS, auction_launcher.key().as_ref(), infinix.key().as_ref()],
        bump = actor.bump,
    )]
    pub actor: Account<'info, Actor>,
    #[account(mut)]
    pub infinix: AccountLoader<'info, Infinix>,
    #[account(
        init,
        payer = auction_launcher,
        seeds = [AUCTION_SEEDS, infinix.key().as_ref(), rebalance.load()?.nonce.to_le_bytes().as_ref(), rebalance.load()?.get_next_auction_id().to_le_bytes().as_ref()],
        bump,
        space = Auction::SIZE,
    )]
    pub auction: AccountLoader<'info, Auction>,
    pub buy_mint: InterfaceAccount<'info, Mint>,
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
        payer = auction_launcher,
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

impl OpenAuction<'_> {
    pub fn validate(
        &self,
        infinix: &Infinix,
        rebalance: &Rebalance,
        token_1: Pubkey,
        token_2: Pubkey,
    ) -> Result<u8> {
        infinix.validate_infinix(
            &self.infinix.key(),
            Some(&self.actor),
            Some(vec![Role::AuctionLauncher]),
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