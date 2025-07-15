use crate::ID as INFINIX_PROGRAM_ID;
use anchor_lang::{prelude::*, solana_program::instruction::Instruction};
use shared::check_condition;
use shared::errors::ErrorCode;

const CALLBACK_PROGRAM_ID_INDEX: usize = 0;

#[cfg(not(tarpaulin_include))]
pub fn cpi_call(remaining_accounts: &[AccountInfo], data: Vec<u8>) -> Result<()> {
    if !remaining_accounts.is_empty() {
        let callback_program = &remaining_accounts[CALLBACK_PROGRAM_ID_INDEX];

        check_condition!(
            callback_program.key() != INFINIX_PROGRAM_ID,
            InvalidCallbackProgram
        );

        let callback_accounts = &remaining_accounts[CALLBACK_PROGRAM_ID_INDEX + 1..];

        let mut callback_accounts_metas: Vec<anchor_lang::prelude::AccountMeta> = vec![];

        for account in callback_accounts {
            check_condition!(account.key() != INFINIX_PROGRAM_ID, InvalidCallbackProgram);

            if account.is_writable {
                callback_accounts_metas.push(AccountMeta::new(*account.key, account.is_signer));
            } else {
                callback_accounts_metas
                    .push(AccountMeta::new_readonly(*account.key, account.is_signer));
            }
        }

        anchor_lang::solana_program::program::invoke(
            &Instruction {
                program_id: callback_program.key(),
                accounts: callback_accounts_metas,
                data: data.clone(),
            },
            callback_accounts,
        )?;
    }
    Ok(())
}
