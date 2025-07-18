use std::collections::BTreeSet;

use crate::events::FeeRecipientSet;
use crate::state::FeeRecipients;
use crate::utils::structs::FeeRecipient;
use anchor_lang::prelude::*;
use anchor_spl::token::set_authority;
use shared::constants::MAX_FEE_RECIPIENTS_PORTION;
use shared::errors::ErrorCode;
use shared::{check_condition, constants::MAX_FEE_RECIPIENTS};

impl FeeRecipients {
    #[cfg(not(tarpaulin_include))]
    pub fn process_init_if_needed(
        account_loader_fee_recipients: &mut AccountLoader<FeeRecipients>,
        context_bump: u8,
        infinix: &Pubkey,
    ) -> Result<bool> {
        let account_info_fee_recipients = account_loader_fee_recipients.to_account_info();

        let data = account_info_fee_recipients.try_borrow_mut_data()?;
        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);

        let discriminator = u64::from_le_bytes(disc_bytes);

        drop(data);

        if discriminator == 0 {
            let fee_recipients = &mut account_loader_fee_recipients.load_init()?;
            fee_recipients.bump = context_bump;
            fee_recipients.infinix = *infinix;
            fee_recipients.distribution_index = 0;
            fee_recipients.fee_recipients = [FeeRecipient::default(); MAX_FEE_RECIPIENTS];

            return Ok(true);
        } else {
            let account_bump = account_loader_fee_recipients.load()?.bump;
            check_condition!(account_bump == context_bump, InvalidBump);
        }

        Ok(false)
    }

    pub fn update_fee_recipients(
        &mut self,
        fee_recipients_to_add: Vec<FeeRecipient>,
        fee_recipients_to_remove: Vec<Pubkey>,
    ) -> Result<()> {
        let mut new_recipients = [FeeRecipient::default(); MAX_FEE_RECIPIENTS];
        let mut add_index = 0;

        for fee_recipient in self.fee_recipients.iter() {
            if !fee_recipients_to_remove.contains(&fee_recipient.recipient)
                && fee_recipient.recipient != Pubkey::default()
            {
                new_recipients[add_index] = *fee_recipient;
                add_index += 1;
            }
        }

        let mut filtered_fee_recipients_to_add: Vec<FeeRecipient> = vec![];
        for fee_recipient_to_add in fee_recipients_to_add {
            if !fee_recipients_to_remove.contains(&fee_recipient_to_add.recipient) {
                filtered_fee_recipients_to_add.push(fee_recipient_to_add);
            }
        }

        for new_recipient in filtered_fee_recipients_to_add {
            check_condition!(add_index < MAX_FEE_RECIPIENTS, InvalidFeeRecipientCount);
            new_recipients[add_index] = new_recipient;
            add_index += 1;

            emit!(FeeRecipientSet {
                recipient: new_recipient.recipient,
                portion: new_recipient.portion,
            });
        }

        self.fee_recipients = new_recipients;
        self.validate_fee_recipient_total_portions_and_check_for_duplicates()
    }

    pub fn validate_fee_recipient_total_portions_and_check_for_duplicates(&self) -> Result<()> {
        check_condition!(
            self.fee_recipients.iter().map(|r| r.portion).sum::<u128>()
                == MAX_FEE_RECIPIENTS_PORTION,
            InvalidFeeRecipientPortion
        );

        let mut seen = BTreeSet::new();
        if !self
            .fee_recipients
            .iter()
            .filter(|r| r.recipient != Pubkey::default())
            .map(|r| r.recipient)
            .all(|pubkey| seen.insert(pubkey))
        {
            return err!(ErrorCode::InvalidFeeRecipientContainsDuplicates);
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        let default_pubkey = Pubkey::default();

        self.fee_recipients.iter().all(|r|r.recipient == default_pubkey)
    }
}