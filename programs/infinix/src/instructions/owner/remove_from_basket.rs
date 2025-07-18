use crate::state::{Actor, Infinix, InfinixBasket};
use crate::utils::structs::{InfinixStatus, Role};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use shared::{
    check_condition,
    constants::{ACTOR_SEEDS, INFINIX_BASKET_SEEDS},
    errors::ErrorCode,
};

#[derive(Accounts)]
pub struct RemoveFromBasket<'info> {
    pub system_program: Program<'info, System>,

    #[account(mut)]
    pub infinix_owner: Signer<'info>,

    #[account(
        seeds = [ACTOR_SEEDS, infinix_owner.key().as_ref(), infinix.key().as_ref()],
        bump = actor.bump,
    )]
    pub actor: Account<'info, Actor>,

    #[account()]
    pub infinix: AccountLoader<'info, Infinix>,

    #[account(mut,
        seeds = [INFINIX_BASKET_SEEDS, infinix.key().as_ref()],
        bump
    )]
    pub infinix_basket: AccountLoader<'info, InfinixBasket>,

    #[account()]
    pub infinix_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account()]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,
}

impl RemoveFromBasket<'_> {
    pub fn validate(&self, infinix: &Infinix) -> Result<()> {
        infinix.validate_infinix(
            &self.infinix.key(),
            Some(&self.actor),
            Some(vec![Role::Owner]),
            Some(vec![InfinixStatus::Initializing, InfinixStatus::Initialized]),
        )?;

        check_condition!(
            self.infinix_token_mint.key() == infinix.infinix_token_mint,
            InvalidInfinixTokenMint
        );

        Ok(())
    }
}

pub fn handler<'info>(ctx: Context<'_, '_, 'info, 'info, RemoveFromBasket<'info>>) -> Result<()> {
    let infinix = ctx.accounts.infinix.load()?;
    ctx.accounts.validate(&infinix)?;

    let infinix_basket = &mut ctx.accounts.infinix_basket.load_mut()?;

    let _scaled_infinix_token_total_supply =
        infinix.get_total_supply(ctx.accounts.infinix_token_mint.supply)?;

    let mint_to_remove = ctx.accounts.token_mint.key();
    infinix_basket.remove_token_mint_from_basket(mint_to_remove)?;

    Ok(())
}
