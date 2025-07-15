use crate::{check_condition, errors::ErrorCode};
use anchor_lang::{
    prelude::*,
    solana_program::{program::invoke, program::invoke_signed, system_instruction},
};

#[cfg(not(tarpaulin_include))]
pub fn next_account<'b>(
    iter: &mut std::slice::Iter<'b, AccountInfo<'b>>,
    must_be_signer: bool,
    must_be_writable: bool,
    expected_owner: &Pubkey,
) -> Result<&'b AccountInfo<'b>> {
    let account = iter.next().ok_or(ErrorCode::MissingRemainingAccount)?;

    check_condition!(account.is_signer == must_be_signer, AccountNotSigner);
    check_condition!(account.is_writable == must_be_writable, AccountNotWritable);

    // Only check owner if account is initialized
    if !account.data_is_empty() {
        check_condition!(account.owner == expected_owner, InvalidAccountOwner);
    }

    Ok(account)
}

#[cfg(not(tarpaulin_include))]
pub fn next_token_program<'b>(
    iter: &mut std::slice::Iter<'b, AccountInfo<'b>>,
) -> Result<&'b AccountInfo<'b>> {
    let account = iter.next().ok_or(ErrorCode::MissingRemainingAccount)?;

    check_condition!(
        account.key() == anchor_spl::token::ID || account.key() == anchor_spl::token_2022::ID,
        InvalidTokenProgram
    );
    check_condition!(account.executable, AccountNotExecutable);
    Ok(account)
}

#[cfg(not(tarpaulin_include))]
pub fn init_pda_account_rent<'info>(
    account_to_init: &AccountInfo<'info>,
    space: usize,
    payer: &AccountInfo<'info>,
    owner_program_id: &Pubkey,
    system_program: &AccountInfo<'info>,
    pda_signers_seeds: &[&[&[u8]]],
) -> Result<()> {
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(space);

    let current_lamports_balance = account_to_init.lamports();

    if current_lamports_balance == 0 {
        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                account_to_init.key,
                rent_lamports,
                space as u64,
                owner_program_id,
            ),
            &[
                payer.clone(),
                account_to_init.clone(),
                system_program.clone(),
            ],
            pda_signers_seeds,
        )?;
    } else {
        let lamports_needed = rent
            .minimum_balance(space)
            .saturating_sub(current_lamports_balance);

        if lamports_needed > 0 {
            invoke(
                &system_instruction::transfer(payer.key, account_to_init.key, lamports_needed),
                &[payer.clone(), account_to_init.clone()],
            )?;
        }

        invoke_signed(
            &system_instruction::allocate(account_to_init.key, space as u64),
            &[account_to_init.clone(), system_program.clone()],
            pda_signers_seeds,
        )?;

        invoke_signed(
            &system_instruction::assign(account_to_init.key, owner_program_id),
            &[account_to_init.clone(), system_program.clone()],
            pda_signers_seeds,
        )?;
    }

    Ok(())
}
