use crate::{
    events::InfinixCreated,
    state::{Actor, Infinix},
    utils::structs::{InfinixStatus, Role},
    utils::{FixedSizeString, MAX_PADDED_STRING_LENGTH},
};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::rent::{
    DEFAULT_EXEMPTION_THRESHOLD, DEFAULT_LAMPORTS_PER_BYTE_YEAR,
};
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{Token2022, ID as TOKEN_2022_PROGRAM_ID},
    token_interface::{
        spl_pod::optional_keys::OptionalNonZeroPubkey, token_metadata_initialize, Mint,
        TokenMetadataInitialize,
    },
};
use shared::{
    check_condition,
    constants::{
        ACTOR_SEEDS, INFINIX_SEEDS, MAX_AUCTION_LENGTH, MAX_MINT_FEE, MAX_TVL_FEE, MIN_AUCTION_LENGTH,
    },
    errors::ErrorCode,
};
use spl_token_metadata_interface::state::TokenMetadata;
use spl_type_length_value::variable_len_pack::VariableLenPack;

#[derive(Accounts)]
pub struct InitInfinix2022<'info> {
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    #[account(address = TOKEN_2022_PROGRAM_ID)]
    pub token_program: Program<'info, Token2022>,

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

    #[account(
        init,
        payer = infinix_owner,
        mint::decimals = 9,
        mint::authority = infinix,
        mint::freeze_authority = infinix,
        extensions::metadata_pointer::authority = infinix,
        extensions::metadata_pointer::metadata_address = infinix_token_mint,
    )]
    pub infinix_token_mint: InterfaceAccount<'info, Mint>,

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
}

impl InitInfinix2022<'_> {
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

pub fn handler(
    ctx: Context<InitInfinix2022>,
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
    let bump = ctx.bumps.infinix;

    {
        let infinix = &mut ctx.accounts.infinix.load_init()?;

        infinix.bump = bump;
        infinix.infinix_token_mint = infinix_token_mint_key;
        infinix.set_tvl_fee(scaled_tvl_fee)?;
        infinix.mint_fee = scaled_mint_fee;
        infinix.status = InfinixStatus::Initializing as u8;
        infinix.last_poke = Clock::get()?.unix_timestamp as u64;
        infinix.dao_pending_fee_shares = 0;
        infinix.fee_recipients_pending_fee_shares = 0;
        infinix.auction_length = auction_length;
        infinix.mandate = FixedSizeString::new(&mandate);
    }

    // Setup the actor account
    let actor = &mut ctx.accounts.actor;
    actor.bump = ctx.bumps.actor;
    actor.authority = ctx.accounts.infinix_owner.key();
    actor.infinix = ctx.accounts.infinix.key();
    Role::add_role(&mut actor.roles, Role::Owner);

    // Create the metadata via spl 2022
    let token_metadata = TokenMetadata {
        name: name.clone(),
        symbol: symbol.clone(),
        uri: uri.clone(),
        update_authority: OptionalNonZeroPubkey(ctx.accounts.infinix.key()),
        ..Default::default()
    };

    // Add 4 extra bytes for size of MetadataExtension (2 bytes for type, 2 bytes for length)
    let data_len = 4 + token_metadata.get_packed_len()?;

    // Calculate lamports required for the additional metadata
    let lamports =
        data_len as u64 * DEFAULT_LAMPORTS_PER_BYTE_YEAR * DEFAULT_EXEMPTION_THRESHOLD as u64;

    // Transfer additional lamports to mint account for metadata storage
    transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.infinix_owner.to_account_info(),
                to: ctx.accounts.infinix_token_mint.to_account_info(),
            },
        ),
        lamports,
    )?;

    // Initialize the token metadata using the Anchor CPI implementation
    let signer_seeds = &[INFINIX_SEEDS, infinix_token_mint_key.as_ref(), &[bump]];
    token_metadata_initialize(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TokenMetadataInitialize {
                program_id: ctx.accounts.token_program.to_account_info(),
                mint: ctx.accounts.infinix_token_mint.to_account_info(),
                metadata: ctx.accounts.infinix_token_mint.to_account_info(),
                mint_authority: ctx.accounts.infinix.to_account_info(),
                update_authority: ctx.accounts.infinix.to_account_info(),
            },
            &[signer_seeds],
        ),
        name,
        symbol,
        uri,
    )?;

    emit!(InfinixCreated {
        infinix_token_mint: ctx.accounts.infinix_token_mint.key(),
    });

    Ok(())
}
