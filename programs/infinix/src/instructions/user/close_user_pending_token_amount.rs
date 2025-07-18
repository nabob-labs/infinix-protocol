use crate::state::{Infinix, UserPendingBasket};
use anchor_lang::prelude::*;
use shared::errors::ErrorCode;
use shared::{check_condition, constants::USER_PENDING_BASKET_SEEDS};

#[derive(Accounts)]
pub struct CloseUserPendingTokenAmount<'info> {
    pub system_program: Program<'info, System>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account()]
    pub infinix: AccountLoader<'info, Infinix>,

    #[account(mut,
        seeds = [USER_PENDING_BASKET_SEEDS, infinix.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub user_pending_basket: AccountLoader<'info, UserPendingBasket>,
}

impl CloseUserPendingTokenAmount<'_> {
    pub fn validate(&self) -> Result<()> {
        self.infinix.load()?.validate_infinix(
            &self.infinix.key(),
            None,
            None,
            // User should always be able to close their pending tokens
            None,
        )?;

        Ok(())
    }
}

pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, CloseUserPendingTokenAmount<'info>>,
) -> Result<()> {
    ctx.accounts.validate()?;

    {
        let user_pending_basket = &mut ctx.accounts.user_pending_basket.load_mut()?;

        check_condition!(user_pending_basket.is_empty(), PendingBasketIsNotEmpty);

        // To prevent re-init attacks, we re-init the actor with default values
        user_pending_basket.reset();
    }

    ctx.accounts
        .user_pending_basket
        .close(ctx.accounts.user.to_account_info())?;

    Ok(())
}
