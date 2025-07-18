use crate::{
    events::InfinixCreated,
    state::{Actor, Infinix},
    utils::{FixedSizeString, MAX_PADDED_STRING_LENGTH},
    CreateMetadataAccount, Metaplex,
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::ID as TOKEN_PROGRAM_ID,
    token_interface::{Mint, TokenInterface},
};
use shared::{
    check_condition,
    constants::{
        ACTOR_SEEDS, INFINIX_SEEDS, MAX_AUCTION_LENGTH, MAX_MINT_FEE, MAX_TVL_FEE, METADATA_SEEDS,
        MIN_AUCTION_LENGTH,
    },
};

use crate::utils::structs::{InfinixStatus, Role};
use shared::errors::ErrorCode;

#[derive(Accounts)]
pub struct InitInfinix<'info> {
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    // Infinix with Token2022, creation is possible via the init_infinix_2022 instruction
    #[account(address = TOKEN_PROGRAM_ID)]
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(mut)]
    pub infinix_owner: Signer<'info>,

    #[account(init,
        payer = infinix_owner,
        space = Infinix::SIZE,
        seeds = [INFINIX_SEEDS, infinix_token_mint.key().as_ref()],
        bump
    )]
    pub infinix: AccountLoader<'info, Infinix>,

    #[account(init,
    payer = infinix_owner,
    mint::decimals = 9,
    mint::authority = infinix,
    mint::freeze_authority = infinix,
    )]
    pub infinix_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = infinix_owner,
        space = Actor::SIZE,
        seeds = [ACTOR_SEEDS, infinix_owner.key().as_ref(), infinix.key().as_ref()],
        bump
    )]
    pub actor: Box<Account<'info, Actor>>,

    /*
        Because of solana's limits with stack size, etc.

        the fee_recipients will be created in the update function (if needed)
        the infinix_basket will be created in the init tokens (if needed)
    */

    /*
    Metaplex accounts for metadata
     */
    /// CHECK: Token metadata program
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: UncheckedAccount<'info>,

    /// CHECK: Metadata account
    #[account(
        mut,
        seeds = [
            METADATA_SEEDS,
            mpl_token_metadata::ID.as_ref(),
            infinix_token_mint.key().as_ref()
        ],
        seeds::program = mpl_token_metadata::ID,
        bump
    )]
    pub metadata: UncheckedAccount<'info>,
}

impl InitInfinix<'_> {
    pub fn validate(
        &self,
        scaled_tvl_fee: u128,
        scaled_mint_fee: u128,
        auction_length: u64,
        mandate: &str,
    ) -> Result<()> {
        check_condition!(scaled_tvl_fee <= MAX_TVL_FEE, TVLFeeTooHigh);

        check_condition!(scaled_mint_fee <= MAX_MINT_FEE, InvalidMintFee);

        check_condition!(
            (MIN_AUCTION_LENGTH..=MAX_AUCTION_LENGTH).contains(&auction_length),
            InvalidAuctionLength
        );

        check_condition!(
            mandate.len() <= MAX_PADDED_STRING_LENGTH,
            InvalidMandateLength
        );

        Ok(())
    }
}

impl<'info> CreateMetadataAccount<'info> {
    pub fn from_init_infinix(
        ctx: &Context<InitInfinix<'info>>,
    ) -> Result<CreateMetadataAccount<'info>> {
        Ok(CreateMetadataAccount {
            system_program: ctx.accounts.system_program.to_account_info(),
            rent: ctx.accounts.rent.to_account_info(),
            mint: ctx.accounts.infinix_token_mint.to_account_info(),
            mint_authority: ctx.accounts.infinix.to_account_info(),
            payer: ctx.accounts.infinix_owner.to_account_info(),
            update_authority: ctx.accounts.infinix.to_account_info(),
            metadata: ctx.accounts.metadata.to_account_info(),
            token_metadata_program: ctx.accounts.token_metadata_program.to_account_info(),
        })
    }
}

pub fn handler(
    ctx: Context<InitInfinix>,
    scaled_tvl_fee: u128,
    scaled_mint_fee: u128,
    auction_length: u64,
    name: String,
    symbol: String,
    uri: String,
    mandate: String,
) -> Result<()> {
    ctx.accounts
        .validate(scaled_tvl_fee, scaled_mint_fee, auction_length, &mandate)?;

    let infinix_token_mint_key = ctx.accounts.infinix_token_mint.key();
    {
        let infinix = &mut ctx.accounts.infinix.load_init()?;

        infinix.bump = ctx.bumps.infinix;
        infinix.infinix_token_mint = infinix_token_mint_key;
        infinix.set_tvl_fee(scaled_tvl_fee)?;
        infinix.mint_fee = scaled_mint_fee;
        infinix.status = InfinixStatus::Initializing as u8;
        infinix.last_poke = Clock::get()?.unix_timestamp as u64;
        infinix.dao_pending_fee_shares = 0;
        infinix.fee_recipients_pending_fee_shares = 0;
        infinix.auction_length = auction_length;
        infinix.mandate = FixedSizeString::new(&mandate);
        infinix.fee_recipients_pending_fee_shares_to_be_minted = 0;
    }

    let actor = &mut ctx.accounts.actor;
    actor.bump = ctx.bumps.actor;
    actor.authority = ctx.accounts.infinix_owner.key();
    actor.infinix = ctx.accounts.infinix.key();
    Role::add_role(&mut actor.roles, Role::Owner);

    let bump = ctx.bumps.infinix;
    let signer_seeds = &[INFINIX_SEEDS, infinix_token_mint_key.as_ref(), &[bump]];

    Metaplex::create_metadata_account(
        &CreateMetadataAccount::from_init_infinix(&ctx)?,
        name,
        symbol,
        uri,
        &[&signer_seeds[..]],
    )?;

    emit!(InfinixCreated {
        infinix_token_mint: ctx.accounts.infinix_token_mint.key(),
    });

    Ok(())
}
