use crate::state::{Actor, Infinix};
use crate::utils::structs::Role;
use anchor_lang::prelude::*;
use shared::constants::ACTOR_SEEDS;

#[derive(Accounts)]
pub struct RemoveActor<'info> {
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    #[account(mut)]
    pub infinix_owner: Signer<'info>,

    /// CHECK: Wallet, DAO, multisig
    #[account()]
    pub actor_authority: UncheckedAccount<'info>,

    #[account(
        seeds = [ACTOR_SEEDS, infinix_owner.key().as_ref(), infinix_owner_actor.infinix.key().as_ref()],
        bump = infinix_owner_actor.bump,
    )]
    pub infinix_owner_actor: Box<Account<'info, Actor>>,

    #[account(mut,
        seeds = [ACTOR_SEEDS, actor_authority.key().as_ref(), infinix_owner_actor.infinix.key().as_ref()],
        bump = actor_to_remove.bump,
    )]
    pub actor_to_remove: Box<Account<'info, Actor>>,

    #[account()]
    pub infinix: AccountLoader<'info, Infinix>,
}

impl RemoveActor<'_> {
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

pub fn handler(ctx: Context<RemoveActor>, role: Role, close_actor: bool) -> Result<()> {
    ctx.accounts.validate()?;

    let actor_to_remove = &mut ctx.accounts.actor_to_remove;

    if !close_actor {
        Role::remove_role(&mut actor_to_remove.roles, role);
    } else {
        // To prevent re-init attacks, we reset the actor with default values
        actor_to_remove.reset();

        actor_to_remove.close(ctx.accounts.infinix_owner.to_account_info())?;
    }

    Ok(())
}
