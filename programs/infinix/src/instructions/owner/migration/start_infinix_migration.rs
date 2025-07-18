use crate::utils::{Metaplex, NewInfinixProgram, UpdateAuthority};
use crate::ID as INFINIX_PROGRAM_ID;
use crate::{
    state::{Actor, Infinix},
    utils::{InfinixStatus, Role},
};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions;
use anchor_spl::token::Token;
use anchor_spl::token_2022_extensions::token_metadata::token_metadata_update_authority;
use anchor_spl::token_interface::spl_pod::optional_keys::OptionalNonZeroPubkey;
use anchor_spl::token_interface::TokenMetadataUpdateAuthority;
use anchor_spl::{
    token_2022::spl_token_2022::instruction::AuthorityType,
    token_interface::{self, Mint, TokenInterface},
};
use infinix_admin::{state::ProgramRegistrar, ID as INFINIX_ADMIN_PROGRAM_ID};
use shared::constants::METADATA_SEEDS;
use shared::errors::ErrorCode;
use shared::{
    check_condition,
    constants::{ACTOR_SEEDS, INFINIX_SEEDS, PROGRAM_REGISTRAR_SEEDS},
};

#[derive(Accounts)]
pub struct StartInfinixMigration<'info> {
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: Instructions sysvar
    #[account(address = instructions::ID)]
    pub instructions_sysvar: UncheckedAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,

    #[account(mut)]
    pub infinix_owner: Signer<'info>,

    #[account(
        seeds = [PROGRAM_REGISTRAR_SEEDS],
        bump = program_registrar.bump,
        seeds::program = INFINIX_ADMIN_PROGRAM_ID,
    )]
    pub program_registrar: Box<Account<'info, ProgramRegistrar>>,

    /// CHECK: Folio program used for new infinix
    #[account(executable)]
    pub new_infinix_program: UncheckedAccount<'info>,

    /// CHECK: The new infinix
    #[account(mut)]
    pub new_infinix: UncheckedAccount<'info>,

    #[account(
        seeds = [ACTOR_SEEDS, infinix_owner.key().as_ref(), old_infinix.key().as_ref()],
        bump = actor.bump,
    )]
    pub actor: Account<'info, Actor>,

    #[account(mut)]
    pub old_infinix: AccountLoader<'info, Infinix>,

    #[account(mut,
    mint::authority = old_infinix,
    mint::freeze_authority = old_infinix,
    )]
    pub infinix_token_mint: Box<InterfaceAccount<'info, Mint>>,

    /// CHECK: it is checked in the cpi to the new infinix program
    #[account(mut)]
    pub new_infinix_basket: UncheckedAccount<'info>,

    /// CHECK: it is checked in the cpi to the new infinix program
    #[account(mut)]
    pub new_actor: UncheckedAccount<'info>,
    // Any remaining accounts that are required in the new infinix program
    // When calling `create_infinix_from_old_program`

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

impl StartInfinixMigration<'_> {
    pub fn validate(&self, old_infinix: &Infinix, max_allowed_pending_fees: u128) -> Result<()> {
        // Validate old infinix, make sure the owner is the one calling the instruction
        old_infinix.validate_infinix(
            &self.old_infinix.key(),
            Some(&self.actor),
            Some(vec![Role::Owner]),
            Some(vec![InfinixStatus::Initialized, InfinixStatus::Killed]),
        )?;

        check_condition!(
            old_infinix.infinix_token_mint == self.infinix_token_mint.key(),
            InvalidInfinixTokenMint
        );

        let last_infinix_poke = old_infinix.last_poke;
        let current_timestamp = Clock::get()?.unix_timestamp;

        let account_fee_until = old_infinix.get_account_fee_until(current_timestamp)?;
        check_condition!(
            // Last folio poke can only be greater than the account_fee_until, when a new infinix was created the same day and the migration is being tried the same day.
            // As on creation, of the folio we want to set last_poke to current time.
            // In all other cases, the last_poke should match the account_fee_until.
            account_fee_until <= last_infinix_poke,
            MigrationFailedInfinixNotPoked
        );

        // Folio owners can decide, up-to what pending amount they want to loss.
        // As these amounts become unmintable and non-distributable in the new program.
        check_condition!(
            old_infinix
                .dao_pending_fee_shares
                .lt(&max_allowed_pending_fees),
            MigrationFailedDaoPendingFeeSharesTooHigh
        );

        check_condition!(
            old_infinix
                .fee_recipients_pending_fee_shares
                .lt(&max_allowed_pending_fees),
            MigrationFailedFeeRecipientsPendingFeeSharesTooHigh
        );

        check_condition!(
            old_infinix
                .fee_recipients_pending_fee_shares_to_be_minted
                .lt(&max_allowed_pending_fees),
            MigrationFailedFeeRecipientsPendingFeeShareToBeMintedTooHigh
        );

        /*
        New Folio Validation
         */
        // Make sure the new infinix program is in the registrar
        check_condition!(
            self.program_registrar
                .is_in_registrar(self.new_infinix_program.key()),
            ProgramNotInRegistrar
        );

        check_condition!(
            self.program_registrar.is_in_registrar(crate::ID),
            ProgramNotInRegistrar
        );

        check_condition!(
            self.new_infinix_program.key() != INFINIX_PROGRAM_ID,
            CantMigrateToSameProgram
        );

        Ok(())
    }
}

pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, StartInfinixMigration<'info>>,
    max_allowed_pending_fees: u128,
) -> Result<()> {
    let old_infinix_bump: u8;
    {
        let old_infinix = &mut ctx.accounts.old_infinix.load_mut()?;

        old_infinix_bump = old_infinix.bump;

        ctx.accounts.validate(old_infinix, max_allowed_pending_fees)?;
    }

    // Transfer the mint and freeze authority to the new infinix
    let token_mint_key = ctx.accounts.infinix_token_mint.key();

    let infinix_signer_seeds = &[INFINIX_SEEDS, token_mint_key.as_ref(), &[old_infinix_bump]];
    let infinix_signer = &[&infinix_signer_seeds[..]];

    if ctx.accounts.token_program.key() == Token::id() {
        // Update Metadata authority to the new infinix
        Metaplex::update_metadata_authority(
            &UpdateAuthority {
                metadata: ctx.accounts.metadata.to_account_info(),
                mint: ctx.accounts.infinix_token_mint.to_account_info(),
                mint_authority: ctx.accounts.old_infinix.to_account_info(),
                payer: ctx.accounts.infinix_owner.to_account_info(),
                update_authority: ctx.accounts.old_infinix.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
                token_metadata_program: ctx.accounts.token_metadata_program.to_account_info(),
                sysvar_instructions: ctx.accounts.instructions_sysvar.to_account_info(),
            },
            ctx.accounts.new_infinix.key(),
            infinix_signer,
        )?;
    } else {
        // The metadata is with token 2022 program
        token_metadata_update_authority(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                TokenMetadataUpdateAuthority {
                    program_id: ctx.accounts.token_program.to_account_info(),
                    metadata: ctx.accounts.infinix_token_mint.to_account_info(),
                    current_authority: ctx.accounts.old_infinix.to_account_info(),
                    new_authority: ctx.accounts.new_infinix.to_account_info(),
                },
                infinix_signer,
            ),
            OptionalNonZeroPubkey(ctx.accounts.new_infinix.key()),
        )?;

        // Token 2022, does not allow updates for Metadata pointer authority.
    }

    token_interface::set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_interface::SetAuthority {
                current_authority: ctx.accounts.old_infinix.to_account_info(),
                account_or_mint: ctx.accounts.infinix_token_mint.to_account_info(),
            },
            infinix_signer,
        ),
        AuthorityType::MintTokens,
        Some(ctx.accounts.new_infinix.key()),
    )?;

    token_interface::set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_interface::SetAuthority {
                current_authority: ctx.accounts.old_infinix.to_account_info(),
                account_or_mint: ctx.accounts.infinix_token_mint.to_account_info(),
            },
            infinix_signer,
        ),
        AuthorityType::FreezeAccount,
        Some(ctx.accounts.new_infinix.key()),
    )?;

    // Update the infinix status to migrating
    {
        let old_infinix = &mut ctx.accounts.old_infinix.load_mut()?;
        old_infinix.status = InfinixStatus::Migrating as u8;
    }

    NewInfinixProgram::create_infinix_from_old_program(
        &ctx.accounts.new_infinix_program,
        &ctx.accounts.system_program,
        &ctx.accounts.infinix_owner,
        &ctx.accounts.old_infinix.to_account_info(),
        &ctx.accounts.new_infinix,
        &ctx.accounts.new_actor,
        &ctx.accounts.new_infinix_basket,
        &ctx.accounts.infinix_token_mint.to_account_info(),
        ctx.remaining_accounts,
        infinix_signer,
    )?;

    // No need to transfer tokens of the infinix token mint, as the infinix is minting / burning, never holding them.

    Ok(())
}
