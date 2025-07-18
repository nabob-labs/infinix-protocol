use crate::events::BasketTokenAdded;
use crate::state::{Actor, Infinix, InfinixBasket};
use crate::utils::structs::{InfinixStatus, Role};
use crate::utils::InfinixTokenAmount;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::{get_associated_token_address_with_program_id, AssociatedToken};
use anchor_spl::token_2022;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};
use shared::check_condition;
use shared::constants::{ACTOR_SEEDS, INFINIX_BASKET_SEEDS, INFINIX_SEEDS};
use shared::errors::ErrorCode;
use shared::utils::account_util::next_account;
use shared::utils::{next_token_program, TokenUtil};

const EXPECTED_REMAINING_ACCOUNTS_LENGTH: usize = 4;

#[derive(Accounts)]
pub struct AddToBasket<'info> {
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(mut)]
    pub infinix_owner: Signer<'info>,

    #[account(
        seeds = [ACTOR_SEEDS, infinix_owner.key().as_ref(), infinix.key().as_ref()],
        bump = actor.bump,
    )]
    pub actor: Account<'info, Actor>,

    #[account(mut)]
    pub infinix: AccountLoader<'info, Infinix>,

    #[account(init_if_needed,
        payer = infinix_owner,
        space = InfinixBasket::SIZE,
        seeds = [INFINIX_BASKET_SEEDS, infinix.key().as_ref()],
        bump
    )]
    pub infinix_basket: AccountLoader<'info, InfinixBasket>,

    #[account(mut)]
    pub infinix_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut,
        associated_token::mint = infinix_token_mint,
        associated_token::authority = infinix_owner,
        associated_token::token_program = infinix_token_mint.to_account_info().owner,
    )]
    pub owner_infinix_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
}

impl AddToBasket<'_> {
    pub fn validate(&self, infinix: &Infinix, raw_initial_shares: Option<u64>) -> Result<()> {
        infinix.validate_infinix(
            &self.infinix.key(),
            Some(&self.actor),
            Some(vec![Role::Owner]),
            Some(vec![InfinixStatus::Initializing, InfinixStatus::Initialized]),
        )?;

        if raw_initial_shares.is_some() {
            check_condition!(
                *self.infinix_token_mint.to_account_info().owner == self.token_program.key(),
                InvalidTokenMintProgram
            );
        }

        Ok(())
    }
}

fn mint_initial_shares<'info>(
    ctx: &Context<'_, '_, 'info, 'info, AddToBasket<'info>>,
    raw_initial_shares: Option<u64>,
) -> Result<()> {
    let token_mint_key = ctx.accounts.infinix_token_mint.key();

    {
        let infinix = ctx.accounts.infinix.load()?;

        // Can only mint the initial shares once
        if infinix.status == InfinixStatus::Initializing as u8 {
            let bump = infinix.bump;
            let signer_seeds = &[INFINIX_SEEDS, token_mint_key.as_ref(), &[bump]];

            let cpi_accounts = token_2022::MintTo {
                mint: ctx.accounts.infinix_token_mint.to_account_info(),
                to: ctx.accounts.owner_infinix_token_account.to_account_info(),
                authority: ctx.accounts.infinix.to_account_info(),
            };

            let token_program = ctx.accounts.token_program.to_account_info();

            token_interface::mint_to(
                CpiContext::new_with_signer(token_program, cpi_accounts, &[signer_seeds]),
                raw_initial_shares.ok_or(ErrorCode::MathOverflow)?,
            )?;
        }
    }

    {
        let mut infinix = ctx.accounts.infinix.load_mut()?;

        infinix.status = InfinixStatus::Initialized as u8;
    }

    Ok(())
}

pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, AddToBasket<'info>>,
    raw_amounts: Vec<u64>,
    raw_initial_shares: Option<u64>,
) -> Result<()> {
    {
        let infinix = ctx.accounts.infinix.load()?;
        ctx.accounts.validate(&infinix, raw_initial_shares)?;
    }

    let infinix_key = ctx.accounts.infinix.key();

    let remaining_accounts = &ctx.remaining_accounts;
    let mut remaining_accounts_iter = remaining_accounts.iter();

    let infinix_owner = ctx.accounts.infinix_owner.to_account_info();

    let mut infinix_token_amounts: Vec<InfinixTokenAmount> = vec![];

    check_condition!(
        remaining_accounts.len() % EXPECTED_REMAINING_ACCOUNTS_LENGTH == 0,
        InvalidNumberOfRemainingAccounts
    );

    check_condition!(
        remaining_accounts.len() / EXPECTED_REMAINING_ACCOUNTS_LENGTH == raw_amounts.len(),
        InvalidNumberOfRemainingAccounts
    );

    for raw_amount in raw_amounts {
        let token_program = next_token_program(&mut remaining_accounts_iter)?;
        let token_mint = next_account(
            &mut remaining_accounts_iter,
            false,
            false,
            &token_program.key(),
        )?;
        let sender_token_account = next_account(
            &mut remaining_accounts_iter,
            false,
            true,
            &token_program.key(),
        )?;
        let recipient_token_account = next_account(
            &mut remaining_accounts_iter,
            false,
            true,
            &token_program.key(),
        )?;

        // Validate the recipient token account is the ATA of the infinix
        check_condition!(
            recipient_token_account.key()
                == get_associated_token_address_with_program_id(
                    &infinix_key,
                    token_mint.key,
                    &token_program.key(),
                ),
            InvalidRecipientTokenAccount
        );

        // Validate that the token mint is a supported SPL token
        check_condition!(
            TokenUtil::is_supported_spl_token(
                Some(&token_mint.to_account_info()),
                Some(&sender_token_account.to_account_info())
            )?,
            UnsupportedSPLToken
        );

        // Get decimals from token mint
        let data = token_mint.try_borrow_data()?;
        let mint = Mint::try_deserialize(&mut &data[..])?;

        let cpi_accounts = TransferChecked {
            from: sender_token_account.to_account_info(),
            to: recipient_token_account.to_account_info(),
            authority: infinix_owner.clone(),
            mint: token_mint.to_account_info(),
        };

        token_interface::transfer_checked(
            CpiContext::new(token_program.to_account_info(), cpi_accounts),
            raw_amount,
            mint.decimals,
        )?;

        infinix_token_amounts.push(InfinixTokenAmount {
            mint: token_mint.key(),
            amount: raw_amount,
        });

        emit!(BasketTokenAdded {
            token: token_mint.key(),
        });
    }

    InfinixBasket::process_init_if_needed(
        &mut ctx.accounts.infinix_basket,
        ctx.bumps.infinix_basket,
        &infinix_key,
        &infinix_token_amounts,
    )?;

    if raw_initial_shares.is_some() {
        mint_initial_shares(&ctx, raw_initial_shares)?;
    }

    Ok(())
}
