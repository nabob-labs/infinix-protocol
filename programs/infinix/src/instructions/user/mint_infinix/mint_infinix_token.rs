use crate::state::{Infinix, InfinixBasket, UserPendingBasket};
use crate::utils::structs::InfinixStatus;
use anchor_lang::prelude::*;
use anchor_spl::token_interface;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use infinix_admin::state::DAOFeeConfig;
use infinix_admin::ID as INFINIX_ADMIN_PROGRAM_ID;
use shared::constants::{
    PendingBasketType, INFINIX_BASKET_SEEDS, INFINIX_FEE_CONFIG_SEEDS, USER_PENDING_BASKET_SEEDS,
};
use shared::errors::ErrorCode;
use shared::{
    check_condition,
    constants::{DAO_FEE_CONFIG_SEEDS, INFINIX_SEEDS},
};

#[derive(Accounts)]
pub struct MintInfinixToken<'info> {
    pub system_program: Program<'info, System>,
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

impl MintInfinixToken<'_> {
    pub fn validate(&self, infinix: &Infinix) -> Result<()> {
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

        Ok(())
    }
}

pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, MintInfinixToken<'info>>,
    raw_shares: u64,
    min_raw_shares: Option<u64>,
) -> Result<()> {
    let infinix_bump = {
        let infinix = &mut ctx.accounts.infinix.load_mut()?;
        ctx.accounts.validate(infinix)?;
        infinix.bump
    };

    let token_mint_key = ctx.accounts.infinix_token_mint.key();
    let current_time = Clock::get()?.unix_timestamp;

    let infinix_basket = &mut ctx.accounts.infinix_basket.load_mut()?;

    let token_amounts_user = &mut ctx.accounts.user_pending_basket.load_mut()?;

    // Get the related infinix fees
    let fee_details = ctx
        .accounts
        .dao_fee_config
        .get_fee_details(&ctx.accounts.infinix_fee_config)?;

    {
        let infinix = &mut ctx.accounts.infinix.load_mut()?;

        // infinix is poked via the to_assets function, so don't need to poke it here
        token_amounts_user.to_assets(
            raw_shares,
            ctx.accounts.infinix_token_mint.supply,
            infinix_basket,
            infinix,
            PendingBasketType::MintProcess,
            current_time,
            fee_details.scaled_fee_numerator,
            fee_details.scaled_fee_denominator,
            fee_details.scaled_fee_floor,
            vec![],
        )?;
    }

    // Mint infinix token to user based on shares
    let fee_shares = ctx.accounts.infinix.load_mut()?.calculate_fees_for_minting(
        raw_shares,
        fee_details.scaled_fee_numerator,
        fee_details.scaled_fee_denominator,
        fee_details.scaled_fee_floor,
    )?;

    let raw_infinix_token_amount_to_mint = raw_shares
        .checked_sub(fee_shares.0)
        .ok_or(ErrorCode::MathOverflow)?;

    let signer_seeds = &[INFINIX_SEEDS, token_mint_key.as_ref(), &[infinix_bump]];

    if let Some(min_raw_shares) = min_raw_shares {
        check_condition!(
            raw_infinix_token_amount_to_mint >= min_raw_shares,
            SlippageExceeded
        );
    }

    token_interface::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_interface::MintTo {
                mint: ctx.accounts.infinix_token_mint.to_account_info(),
                to: ctx.accounts.user_infinix_token_account.to_account_info(),
                authority: ctx.accounts.infinix.to_account_info(),
            },
            &[signer_seeds],
        ),
        raw_infinix_token_amount_to_mint,
    )?;

    Ok(())
}
