use crate::events::InfinixKilled;
use crate::state::{Actor, Infinix};
use crate::utils::structs::{InfinixStatus, Role};
use anchor_lang::prelude::*;
use shared::constants::ACTOR_SEEDS;

#[derive(Accounts)]
pub struct KillInfinix<'info> {
    pub system_program: Program<'info, System>,

    #[account(mut)]
    pub infinix_owner: Signer<'info>,

    #[account(
        seeds = [ACTOR_SEEDS, infinix_owner.key().as_ref(), infinix.key().as_ref()],
        bump = actor.bump,
    )]
    pub actor: Account<'info, Actor>,

    #[account(mut)]
    pub infinix: AccountLoader<'info, Infinix>,
}

impl KillInfinix<'_> {
    pub fn validate(&self, infinix: &Infinix) -> Result<()> {
        infinix.validate_infinix(
            &self.infinix.key(),
            Some(&self.actor),
            Some(vec![Role::Owner]),
            Some(vec![InfinixStatus::Initialized, InfinixStatus::Initializing]),
        )?;

        Ok(())
    }
}

pub fn handler(ctx: Context<KillInfinix>) -> Result<()> {
    let infinix = &mut ctx.accounts.infinix.load_mut()?;

    ctx.accounts.validate(infinix)?;

    infinix.status = InfinixStatus::Killed as u8;

    emit!(InfinixKilled {});

    Ok(())
}
