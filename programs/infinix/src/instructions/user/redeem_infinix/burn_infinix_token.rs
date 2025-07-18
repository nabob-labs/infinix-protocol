use crate::state::{Infinix, InfinixBasket, UserPendingBasket};
use crate::utils::structs::InfinixStatus;
use crate::utils::MinimumOutForTokenAmount;
use anchor_lang::prelude::*;
use anchor_spl::token_interface;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use infinix_admin::state::DAOFeeConfig;
use infinix_admin::ID as INFINIX_ADMIN_PROGRAM_ID;
use shared::constants::{
    DAO_FEE_CONFIG_SEEDS, INFINIX_BASKET_SEEDS, INFINIX_FEE_CONFIG_SEEDS, USER_PENDING_BASKET_SEEDS,
};
use shared::errors::ErrorCode;
use shared::{check_condition, constants::PendingBasketType};

#[derive(Accounts)]
pub struct BurnInfinixToken<'info> {
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [DAO_FEE_CONFIG_SEEDS],
        bump,
        seeds::program = INFINIX_ADMIN_PROGRAM_ID,
    )]
    pub dao_fee_config: Account<'info, DAOFeeConfig>,

    /// CHECK: Could be empty or could be set, if set we use that one, else we use dao fee config
    #[account(
        seeds = [INFINIX_FEE_CONFIG_SEEDS, infinix.key().as_ref()],
        bump,
        seeds::program = INFINIX_ADMIN_PROGRAM_ID,
    )]
    pub infinix_fee_config: UncheckedAccount<'info>,

    #[account(mut)]
    pub infinix: AccountLoader<'info, Infinix>,

    #[account(mut)]
    pub infinix_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut,
        seeds = [INFINIX_BASKET_SEEDS, infinix.key().as_ref()],
        bump
    )]
    pub infinix_basket: AccountLoader<'info, InfinixBasket>,

    #[account(mut,
        seeds = [USER_PENDING_BASKET_SEEDS, infinix.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub user_pending_basket: AccountLoader<'info, UserPendingBasket>,

    #[account(mut,
        associated_token::mint = infinix_token_mint,
        associated_token::authority = user,
        associated_token::token_program = infinix_token_mint.to_account_info().owner,
    )]
    pub user_infinix_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
}

impl BurnInfinixToken<'_> {
    pub fn validate(&self, infinix: &Infinix) -> Result<()> {
        infinix.validate_infinix(
            &self.infinix.key(),
            None,
            None,
            Some(vec![InfinixStatus::Initialized, InfinixStatus::Killed]),
        )?;

        check_condition!(
            self.infinix_token_mint.key() == infinix.infinix_token_mint,
            InvalidInfinixTokenMint
        );

        Ok(())
    }
}

pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, BurnInfinixToken<'info>>,
    raw_shares: u64,
    minimum_out_for_token_amounts: Vec<MinimumOutForTokenAmount>,
) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;

    {
        let infinix = ctx.accounts.infinix.load()?;

        ctx.accounts.validate(&infinix)?;
    }

    // Get the related infinix fees
    let fee_details = ctx
        .accounts
        .dao_fee_config
        .get_fee_details(&ctx.accounts.infinix_fee_config)?;

    {
        let token_amounts_user = &mut ctx.accounts.user_pending_basket.load_mut()?;
        let infinix = &mut ctx.accounts.infinix.load_mut()?;
        let infinix_basket = &mut ctx.accounts.infinix_basket.load_mut()?;

        // infinix is poked via the to_assets function, so don't need to poke it here
        token_amounts_user.to_assets(
            raw_shares,
            ctx.accounts.infinix_token_mint.supply,
            infinix_basket,
            infinix,
            PendingBasketType::RedeemProcess,
            current_time,
            fee_details.scaled_fee_numerator,
            fee_details.scaled_fee_denominator,
            fee_details.scaled_fee_floor,
            minimum_out_for_token_amounts,
        )?;
    }

    // Burn infinix token from user's infinix token account
    token_interface::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token_interface::Burn {
                mint: ctx.accounts.infinix_token_mint.to_account_info(),
                from: ctx.accounts.user_infinix_token_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        raw_shares,
    )?;

    Ok(())
}
