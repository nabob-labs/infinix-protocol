use crate::{
    state::{Infinix, InfinixBasket},
    utils::{InfinixStatus, InfinixTokenAmount},
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount};
use infinix_admin::{state::ProgramRegistrar, ID as INFINIX_ADMIN_PROGRAM_ID};
use shared::{
    check_condition,
    constants::{INFINIX_BASKET_SEEDS, PROGRAM_REGISTRAR_SEEDS},
    errors::ErrorCode,
};

#[derive(Accounts)]

pub struct UpdateBasketInNewInfinixProgram<'info> {
    #[account()]
    pub old_infinix: Signer<'info>,
    #[account(mut)]
    pub new_infinix: AccountLoader<'info, Infinix>,

    /// CHECK: Seeds are checked and the account data is checked in cpi to new infinix program
    #[account(
        seeds = [INFINIX_BASKET_SEEDS, old_infinix.key().as_ref()],
        bump,
        seeds::program = old_infinix.owner,
        owner = *old_infinix.owner,
    )]
    pub old_infinix_basket: UncheckedAccount<'info>,

    /// CHECK: Seeds are checked and the account data is checked in cpi to new infinix program
    #[account(
        mut,
        seeds = [INFINIX_BASKET_SEEDS, new_infinix.key().as_ref()],
        bump,
    )]
    pub new_infinix_basket: AccountLoader<'info, InfinixBasket>,

    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    // Expected to be the ATA of the new infinix with the token mint that is being migrated.
    #[account(
        associated_token::authority = new_infinix,
        associated_token::mint = token_mint,
    )]
    pub infinix_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        seeds = [PROGRAM_REGISTRAR_SEEDS],
        bump = program_registrar.bump,
        seeds::program = INFINIX_ADMIN_PROGRAM_ID,
    )]
    pub program_registrar: Box<Account<'info, ProgramRegistrar>>,
}

impl UpdateBasketInNewInfinixProgram<'_> {
    /// Validate the instruction.
    pub fn validate(&self, old_infinix: &Infinix, new_infinix: &Infinix) -> Result<()> {
        check_condition!(
            old_infinix.status == InfinixStatus::Migrating as u8,
            InvalidInfinixStatus
        );
        check_condition!(
            new_infinix.infinix_token_mint == old_infinix.infinix_token_mint,
            InvalidInfinixTokenMint
        );

        check_condition!(
            self.program_registrar
                .is_in_registrar(*self.old_infinix.owner),
            ProgramNotInRegistrar
        );

        Ok(())
    }
}

#[allow(unused_variables)]
pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, UpdateBasketInNewInfinixProgram<'info>>,
) -> Result<()> {
    // If by mistake it's included in the program, if we don't see dev flag, we return ok
    #[cfg(not(feature = "test"))]
    return Ok(());

    #[allow(unreachable_code)]
    let infinix_data = &ctx.accounts.old_infinix.data.borrow();
    let old_infinix: &Infinix = bytemuck::from_bytes(&infinix_data[8..]);
    let new_infinix = &mut ctx.accounts.new_infinix.load_mut()?;

    {
        ctx.accounts.validate(old_infinix, new_infinix)?;
    }

    let new_infinix_basket = &mut ctx.accounts.new_infinix_basket.load_mut()?;

    let mint_pk = ctx.accounts.token_mint.key();

    let token_balance_in_old_infinix_basket: u64;
    let token_left_in_old_infinix_basket_after_removal_of_mint_pk: usize;

    {
        let old_infinix_basket_data = &ctx.accounts.old_infinix_basket.data.borrow();
        let old_infinix_basket: &InfinixBasket = bytemuck::from_bytes(&old_infinix_basket_data[8..]);

        token_balance_in_old_infinix_basket =
            old_infinix_basket.get_token_amount_in_infinix_basket(&mint_pk)?;

        token_left_in_old_infinix_basket_after_removal_of_mint_pk = old_infinix_basket
            .basket
            .token_amounts
            .iter()
            .filter(|token_amount| {
                // We already know that the removal from infinix-basket happens only after the cpi to new folio program, is made.
                token_amount.mint != Pubkey::default() && token_amount.mint != mint_pk
            })
            .count();
    }

    new_infinix_basket.add_tokens_to_basket(&vec![InfinixTokenAmount {
        mint: mint_pk,
        amount: token_balance_in_old_infinix_basket,
    }])?;

    check_condition!(
        ctx.accounts.infinix_token_account.amount >= token_balance_in_old_infinix_basket,
        InvalidTokenBalance
    );

    if token_left_in_old_infinix_basket_after_removal_of_mint_pk > 0 {
        // We set the status of new infinix to migrating to prevent any minting or redeeming
        new_infinix.status = InfinixStatus::Migrating as u8;
    } else {
        // If there are no tokens left in the old infinix basket, we set the status of new infinix to initialized
        new_infinix.status = InfinixStatus::Initialized as u8;
    }

    Ok(())
}
