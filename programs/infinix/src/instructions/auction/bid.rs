use crate::state::{AuctionEnds, Rebalance};
use crate::utils::structs::InfinixStatus;
use crate::utils::{AuctionStatus, InfinixTokenAmount};
use crate::{
    cpi_call,
    events::AuctionBid,
    state::{Auction, Infinix, InfinixBasket},
};
use anchor_lang::prelude::*;
use anchor_spl::token::transfer_checked;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};
use infinix_admin::state::DAOFeeConfig;
use infinix_admin::ID as INFINIX_ADMIN_PROGRAM_ID;
use shared::constants::REBALANCE_SEEDS;
use shared::utils::TokenUtil;
use shared::{
    check_condition,
    constants::{
        DAO_FEE_CONFIG_SEEDS, INFINIX_BASKET_SEEDS, INFINIX_FEE_CONFIG_SEEDS, INFINIX_SEEDS,
    },
    errors::ErrorCode,
};

#[derive(Accounts)]
pub struct Bid<'info> {
    pub system_program: Program<'info, System>,
    pub buy_token_program: Interface<'info, TokenInterface>,
    pub sell_token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(mut)]
    pub bidder: Signer<'info>,

    #[account(mut)]
    pub infinix: AccountLoader<'info, Infinix>,

    #[account(mut,
    seeds = [INFINIX_BASKET_SEEDS, infinix.key().as_ref()],
    bump
    )]
    pub infinix_basket: AccountLoader<'info, InfinixBasket>,

    #[account()]
    pub infinix_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    pub auction: AccountLoader<'info, Auction>,

    #[account()]
    pub auction_sell_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account()]
    pub auction_buy_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut,
    associated_token::mint = auction_sell_token_mint,
    associated_token::authority = infinix,
    associated_token::token_program = sell_token_program,
    )]
    pub infinix_sell_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut,
    associated_token::mint = auction_buy_token_mint,
    associated_token::authority = infinix,
    associated_token::token_program = buy_token_program,
    )]
    pub infinix_buy_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut,
    associated_token::mint = auction_sell_token_mint,
    associated_token::authority = bidder,
    associated_token::token_program = sell_token_program,
    )]
    pub bidder_sell_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut,
    associated_token::mint = auction_buy_token_mint,
    associated_token::authority = bidder,
    associated_token::token_program = buy_token_program,
    )]
    pub bidder_buy_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        seeds = [REBALANCE_SEEDS, infinix.key().as_ref()],
        bump = rebalance.load()?.bump,
    )]
    pub rebalance: AccountLoader<'info, Rebalance>,

    #[account(mut)]
    pub auction_ends: Account<'info, AuctionEnds>,

    #[account(
        seeds = [DAO_FEE_CONFIG_SEEDS],
        bump,
        seeds::program =INFINIX_ADMIN_PROGRAM_ID,
    )]
    pub dao_fee_config: Account<'info, DAOFeeConfig>,

    /// CHECK: Could be empty or could be set, if set we use that one, else we use dao fee config
    #[account(
        seeds = [INFINIX_FEE_CONFIG_SEEDS, infinix.key().as_ref()],
        bump,
        seeds::program = INFINIX_ADMIN_PROGRAM_ID,
    )]
    pub infinix_fee_config: UncheckedAccount<'info>,
    /*
    Remaining accounts will be the accounts required for the "custom" CPI provided by the bidder.
     */
}

impl Bid<'_> {
    pub fn validate(
        &self,
        infinix: &Infinix,
        current_time: u64,
        auction: &Auction,
        rebalance: &Rebalance,
    ) -> Result<()> {
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

        check_condition!(
            self.auction_sell_token_mint.key() == auction.sell_mint,
            InvalidAuctionSellTokenMint
        );

        check_condition!(
            self.auction_buy_token_mint.key() == auction.buy_mint,
            InvalidAuctionBuyTokenMint
        );

        // Validate that the buy token is a supported SPL token (only need to check the token account here)
        check_condition!(
            TokenUtil::is_supported_spl_token(
                None,
                Some(&self.bidder_buy_token_account.to_account_info())
            )?,
            UnsupportedSPLToken
        );

        self.auction_ends.validate_auction_ends(
            &self.auction_ends.key(),
            auction,
            &self.infinix.key(),
        )?;

        check_condition!(
            rebalance.nonce == self.auction_ends.rebalance_nonce,
            InvalidRebalanceNonceAuctionEnded
        );
        check_condition!(
            rebalance.nonce == auction.nonce,
            InvalidRebalanceNonceAuctionEnded
        );

        let auction_status = auction.try_get_status(current_time);

        check_condition!(
            auction_status == Some(AuctionStatus::Open),
            AuctionNotOngoing
        );

        Ok(())
    }
}

pub fn handler(
    ctx: Context<Bid>,
    raw_sell_amount: u64,
    raw_max_buy_amount: u64,
    with_callback: bool,
    callback_data: Vec<u8>,
) -> Result<()> {
    let infinix_token_mint_key = &ctx.accounts.infinix_token_mint.key();
    let auction = &mut ctx.accounts.auction.load_init()?;
    let rebalance = &mut ctx.accounts.rebalance.load()?;
    let infinix_basket = &mut ctx.accounts.infinix_basket.load_mut()?;
    let current_time = Clock::get()?.unix_timestamp;
    let raw_infinix_token_supply = ctx.accounts.infinix_token_mint.supply;

    let infinix_bump: u8;

    let (raw_sell_amount, raw_bought_amount, _price, scaled_infinix_token_total_supply) = {
        let infinix = &mut ctx.accounts.infinix.load_init()?;
        ctx.accounts
            .validate(infinix, current_time as u64, auction, rebalance)?;

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
        infinix_bump = infinix.bump;

        auction.get_bid(
            infinix,
            infinix_basket,
            raw_infinix_token_supply,
            current_time as u64,
            raw_sell_amount,
            raw_max_buy_amount,
        )?
    };

    infinix_basket.remove_tokens_from_basket(&vec![InfinixTokenAmount {
        mint: auction.sell_mint,
        amount: raw_sell_amount,
    }])?;

    let sell_basket_presence: u128;
    {
        let sell_balance = infinix_basket.get_token_amount_in_infinix_basket(&auction.sell_mint)?;
        if sell_balance == 0 {
            infinix_basket.remove_token_mint_from_basket(auction.sell_mint)?;
        }

        sell_basket_presence = infinix_basket.get_token_presence_per_share_in_basket(
            &auction.sell_mint,
            &scaled_infinix_token_total_supply,
        )?;

        check_condition!(
            sell_basket_presence >= auction.sell_limit,
            BidInvariantViolated
        );
    }
    let signer_seeds = &[
        INFINIX_SEEDS,
        infinix_token_mint_key.as_ref(),
        &[infinix_bump],
    ];

    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.sell_token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.infinix_sell_token_account.to_account_info(),
                to: ctx.accounts.bidder_sell_token_account.to_account_info(),
                authority: ctx.accounts.infinix.to_account_info(),
                mint: ctx.accounts.auction_sell_token_mint.to_account_info(),
            },
            &[signer_seeds],
        ),
        raw_sell_amount,
        ctx.accounts.auction_sell_token_mint.decimals,
    )?;

    emit!(AuctionBid {
        auction_id: auction.id,
        sell_amount: raw_sell_amount,
        bought_amount: raw_bought_amount,
    });

    if with_callback {
        ctx.accounts.infinix_buy_token_account.reload()?;

        let raw_infinix_buy_balance_before = ctx.accounts.infinix_buy_token_account.amount;

        cpi_call(ctx.remaining_accounts, callback_data)?;

        ctx.accounts.infinix_buy_token_account.reload()?;

        check_condition!(
            ctx.accounts
                .infinix_buy_token_account
                .amount
                .checked_sub(raw_infinix_buy_balance_before)
                .ok_or(ErrorCode::MathOverflow)?
                >= raw_bought_amount,
            InsufficientBid
        );
    } else {
        token_interface::transfer_checked(
            CpiContext::new(
                ctx.accounts.buy_token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.bidder_buy_token_account.to_account_info(),
                    to: ctx.accounts.infinix_buy_token_account.to_account_info(),
                    authority: ctx.accounts.bidder.to_account_info(),
                    mint: ctx.accounts.auction_buy_token_mint.to_account_info(),
                },
            ),
            raw_bought_amount,
            ctx.accounts.auction_buy_token_mint.decimals,
        )?;
    }

    infinix_basket.add_tokens_to_basket(&vec![InfinixTokenAmount {
        mint: auction.buy_mint,
        amount: raw_sell_amount,
    }])?;

    let buy_basket_presence = infinix_basket.get_token_presence_per_share_in_basket(
        &auction.buy_mint,
        &scaled_infinix_token_total_supply,
    )?;

    let current_time = current_time as u64;

    if sell_basket_presence == auction.sell_limit || buy_basket_presence >= auction.buy_limit {
        auction.end = current_time - 1;
        ctx.accounts.auction_ends.end_time = current_time - 1;
    }

    Ok(())
}
