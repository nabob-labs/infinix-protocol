use crate::state::{Actor, Infinix};
use crate::utils::structs::Role;
use anchor_lang::prelude::*;
use shared::constants::ACTOR_SEEDS;

#[derive(Accounts)]
pub struct InitOrUpdateActor<'info> {
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    #[account(mut)]
    pub infinix_owner: Signer<'info>,

    #[account(
        seeds = [ACTOR_SEEDS, infinix_owner.key().as_ref(), infinix_owner_actor.infinix.key().as_ref()],
        bump = infinix_owner_actor.bump,
    )]
    pub infinix_owner_actor: Box<Account<'info, Actor>>,

    /// CHECK: Wallet, DAO, multisig that will be the new actor
    #[account()]
    pub new_actor_authority: UncheckedAccount<'info>,

    /*
    Init if needed because we use the same functionality to add roles to the actor
     */
    #[account(init_if_needed,
        payer = infinix_owner,
        space = Actor::SIZE,
        seeds = [ACTOR_SEEDS, new_actor_authority.key().as_ref(), infinix_owner_actor.infinix.key().as_ref()],
        bump
    )]
    pub new_actor: Box<Account<'info, Actor>>,

    #[account()]
    pub infinix: AccountLoader<'info, Infinix>,
}

impl InitOrUpdateActor<'_> {
    pub fn validate(&self) -> Result<()> {
        let infinix = &self.infinix.load()?;

        infinix.validate_infinix(
            &self.infinix.key(),
            Some(&self.infinix_owner_actor),
            Some(vec![Role::Owner]),
            None, // Can CRUD actors no matter the status
        )?;

        Ok(())
    }
}

pub fn handler(ctx: Context<InitOrUpdateActor>, role: Role) -> Result<()> {
    ctx.accounts.validate()?;

    let new_actor = &mut ctx.accounts.new_actor;

    let new_actor_bump = new_actor.bump;

    new_actor.process_init_if_needed(
        new_actor_bump,
        ctx.bumps.new_actor,
        &ctx.accounts.new_actor_authority.key(),
        &ctx.accounts.infinix_owner_actor.infinix,
    )?;

    Role::add_role(&mut new_actor.roles, role);

    Ok(())
}
