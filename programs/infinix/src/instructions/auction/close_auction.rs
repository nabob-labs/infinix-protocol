use crate::state::AuctionEnds;
use crate::utils::structs::{InfinixStatus, Role};
use crate::{
    events::AuctionClosed,
    state::{Actor, Auction, Infinix},
};
use anchor_lang::prelude::*;
use shared::constants::ACTOR_SEEDS;
use shared::errors::ErrorCode;

#[derive(Accounts)]
pub struct CloseAuction<'info> {
    pub system_program: Program<'info, System>,

    #[account(mut)]
    pub auction_actor: Signer<'info>,

    #[account(
        seeds = [ACTOR_SEEDS, auction_actor.key().as_ref(), infinix.key().as_ref()],
        bump = actor.bump,
    )]
    pub actor: Account<'info, Actor>,

    #[account(mut)]
    pub infinix: AccountLoader<'info, Infinix>,

    #[account(mut)]
    pub auction: AccountLoader<'info, Auction>,

    #[account(mut)]
    pub auction_ends: Account<'info, AuctionEnds>,
}

impl CloseAuction<'_> {
    pub fn validate(&self, infinix: &Infinix, auction: &Auction) -> Result<()> {
        infinix.validate_infinix(
            &self.infinix.key(),
            Some(&self.actor),
            Some(vec![
                Role::RebalanceManager,
                Role::AuctionLauncher,
                Role::Owner,
            ]),
            Some(vec![
                InfinixStatus::Initialized,
                InfinixStatus::Initializing,
            ]),
        )?;

        auction.validate_auction(&self.auction.key(), &self.infinix.key())?;

        self.auction_ends.validate_auction_ends(
            &self.auction_ends.key(),
            auction,
            &self.infinix.key(),
        )?;

        Ok(())
    }
}

pub fn handler(ctx: Context<CloseAuction>) -> Result<()> {
    let infinix = &mut ctx.accounts.infinix.load_mut()?;
    let auction = &mut ctx.accounts.auction.load_mut()?;

    ctx.accounts.validate(infinix, auction)?;

    let current_time = Clock::get()?.unix_timestamp as u64;

    if auction.end > current_time {
        auction.end = current_time
            .checked_sub(1)
            .ok_or(error!(ErrorCode::MathOverflow))?;
        ctx.accounts.auction_ends.end_time = auction.end;
    }

    emit!(AuctionClosed {
        auction_id: auction.id,
    });

    Ok(())
}
