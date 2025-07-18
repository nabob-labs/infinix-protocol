use anchor_lang::{
    prelude::*,
    solana_program::{hash, instruction::Instruction, program::invoke_signed},
};
use shared::check_condition;
use shared::errors::ErrorCode;

pub struct NewInfinixProgram {}

impl NewInfinixProgram {
    const INSTRUCTION_DISCRIMINATOR_SIZE: usize = 8;
    const CREATE_INFINIX_FROM_OLD_PROGRAM_FUNCTION_NAME: &'static str =
        "create_infinix_from_old_program";
    const UPDATE_BASKET_IN_NEW_INFINIX_PROFRAM_FUNCTION_NAME: &'static str =
        "update_basket_in_new_infinix_program";

    fn get_instruction_discriminator(instruction_name: &str) -> [u8; 8] {
        let preimage = format!("global:{}", instruction_name);
        let mut hasher = hash::Hasher::default();
        hasher.hash(preimage.as_bytes());
        let hash_result = hasher.result();
        let mut discriminator = [0u8; Self::INSTRUCTION_DISCRIMINATOR_SIZE];

        discriminator.copy_from_slice(&hash_result.to_bytes()[..8]);

        discriminator
    }

    #[cfg(not(tarpaulin_include))]
    pub fn create_infinix_from_old_program<'info>(
        new_infinix_program: &AccountInfo<'info>,
        system_program: &AccountInfo<'info>,
        owner: &AccountInfo<'info>,
        old_infinix: &AccountInfo<'info>,
        new_infinix: &AccountInfo<'info>,
        actor: &AccountInfo<'info>,
        new_infinix_basket: &AccountInfo<'info>,
        infinix_token_mint: &AccountInfo<'info>,
        remaining_accounts: &[AccountInfo<'info>],
        signer_seeds: &[&[&[u8]]],
    ) -> Result<()> {
        let mut account_metas = vec![
            AccountMeta::new_readonly(system_program.key(), false),
            AccountMeta::new(owner.key(), true),
            AccountMeta::new_readonly(old_infinix.key(), true),
            AccountMeta::new(new_infinix.key(), false),
            AccountMeta::new(actor.key(), false),
            AccountMeta::new(new_infinix_basket.key(), false),
            AccountMeta::new_readonly(infinix_token_mint.key(), false),
        ];

        let mut requited_account_infos: Vec<AccountInfo> = vec![
            new_infinix_program.to_account_info(),
            system_program.to_account_info(),
            owner.to_account_info(),
            old_infinix.to_account_info(),
            new_infinix.to_account_info(),
            actor.to_account_info(),
            new_infinix_basket.to_account_info(),
            infinix_token_mint.to_account_info(),
        ];

        for account_info in remaining_accounts {
            if account_info.is_writable {
                account_metas.push(AccountMeta::new(account_info.key(), false));
            } else {
                account_metas.push(AccountMeta::new_readonly(account_info.key(), false));
            }
            requited_account_infos.push(account_info.clone());

            check_condition!(account_info.key() != crate::id(), InvalidCallbackProgram);
        }

        let data = NewInfinixProgram::get_instruction_discriminator(
            Self::CREATE_INFINIX_FROM_OLD_PROGRAM_FUNCTION_NAME,
        )
        .to_vec();

        invoke_signed(
            &Instruction {
                program_id: new_infinix_program.key(),
                accounts: account_metas,
                data: data.clone(),
            },
            &requited_account_infos,
            signer_seeds,
        )?;

        Ok(())
    }

    pub fn update_infinix_basket_in_new_infinix_program<'info>(
        old_infinix: &AccountInfo<'info>,
        new_infinix: &AccountInfo<'info>,
        old_infinix_basket: &AccountInfo<'info>,
        new_infinix_basket: &AccountInfo<'info>,
        token_mint: &AccountInfo<'info>,
        infinix_token_account: &AccountInfo<'info>,
        program_registrar: &AccountInfo<'info>,
        new_infinix_program: &AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]],
    ) -> Result<()> {
        let account_metas = vec![
            AccountMeta::new_readonly(old_infinix.key(), true),
            AccountMeta::new(new_infinix.key(), false),
            AccountMeta::new_readonly(old_infinix_basket.key(), false),
            AccountMeta::new(new_infinix_basket.key(), false),
            AccountMeta::new_readonly(token_mint.key(), false),
            AccountMeta::new_readonly(infinix_token_account.key(), false),
            AccountMeta::new_readonly(program_registrar.key(), false),
        ];

        let data = NewInfinixProgram::get_instruction_discriminator(
            Self::UPDATE_BASKET_IN_NEW_INFINIX_PROFRAM_FUNCTION_NAME,
        )
        .to_vec();

        invoke_signed(
            &Instruction {
                program_id: new_infinix_program.key(),
                accounts: account_metas,
                data: data.clone(),
            },
            &[
                old_infinix.to_account_info(),
                new_infinix.to_account_info(),
                old_infinix_basket.to_account_info(),
                new_infinix_basket.to_account_info(),
                token_mint.to_account_info(),
                infinix_token_account.to_account_info(),
                program_registrar.to_account_info(),
            ],
            signer_seeds,
        )?;

        Ok(())
    }
}
