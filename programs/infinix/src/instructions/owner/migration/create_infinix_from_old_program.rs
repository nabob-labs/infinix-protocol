use crate::{
    state::{Actor, Infinix, InfinixBasket},
    utils::InfinixStatus,
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use shared::{
    check_condition,
    constants::{ACTOR_SEEDS, INFINIX_BASKET_SEEDS, INFINIX_SEEDS},
    errors::ErrorCode,
};

#[derive(Accounts)]
pub struct CreateInfinixFromOldProgram<'info> {
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account()]
    pub old_infinix: Signer<'info>,

    #[account(
        init,
        payer = owner,
        space = Infinix::SIZE,
        seeds = [INFINIX_SEEDS, infinix_token_mint.key().as_ref()],
        bump,
    )]
    pub new_infinix: AccountLoader<'info, Infinix>,

    #[account(
        init,
        payer = owner,
        space = Actor::SIZE,
        seeds = [ACTOR_SEEDS, owner.key().as_ref(), new_infinix.key().as_ref()],
        bump
    )]
    pub actor: Box<Account<'info, Actor>>,

    /// CHECK: Seeds are checked and the account data is checked in cpi to new infinix program
    #[account(
        init,
        payer = owner,
        space = InfinixBasket::SIZE,
        seeds = [INFINIX_BASKET_SEEDS, new_infinix.key().as_ref()],
        bump,
    )]
    pub new_infinix_basket: AccountLoader<'info, InfinixBasket>,

    pub infinix_token_mint: Box<InterfaceAccount<'info, Mint>>,
}

impl CreateInfinixFromOldProgram<'_> {
    /// Validate the instruction.
    pub fn validate(&self, old_infinix: &Infinix) -> Result<()> {
        check_condition!(
            old_infinix.status == InfinixStatus::Migrating as u8,
            InvalidInfinixStatus
        );

        check_condition!(
            self.infinix_token_mint.key() == old_infinix.infinix_token_mint,
            InvalidInfinixTokenMint
        );

        Ok(())
    }
}

#[allow(unused_variables)]
pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, CreateInfinixFromOldProgram<'info>>,
) -> Result<()> {
    // If by mistake it's included in the program, if we don't see dev flag, we return ok
    #[cfg(not(feature = "test"))]
    return Ok(());

    #[allow(unreachable_code)]
    let infinix_data = &ctx.accounts.old_infinix.data.borrow();
    let old_infinix: &Infinix = bytemuck::from_bytes(&infinix_data[8..]);

    {
        ctx.accounts.validate(old_infinix)?;
    }

    {
        let infinix = &mut ctx.accounts.new_infinix.load_init()?;

        infinix.bump = ctx.bumps.new_infinix;
        infinix.infinix_token_mint = ctx.accounts.infinix_token_mint.key();
        infinix.set_tvl_fee(old_infinix.tvl_fee)?;
        infinix.mint_fee = old_infinix.mint_fee;
        infinix.last_poke = old_infinix.last_poke;
        infinix.auction_length = old_infinix.auction_length;
        infinix.mandate = old_infinix.mandate;

        // We can set these to 0, as the old_program, before calling this function confirms us that the
        // values are less then D9, or max the infinix owner is willing to loss as fees.
        infinix.dao_pending_fee_shares = 0;
        infinix.fee_recipients_pending_fee_shares = 0;
        infinix.fee_recipients_pending_fee_shares_to_be_minted = 0;

        infinix.status = InfinixStatus::Migrating as u8;
    }

    InfinixBasket::process_init_if_needed(
        &mut ctx.accounts.new_infinix_basket,
        ctx.bumps.new_infinix_basket,
        &ctx.accounts.new_infinix.key(),
        &vec![],
    )?;

    Ok(())
}
