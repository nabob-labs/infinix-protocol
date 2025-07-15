use anchor_lang::prelude::*;
use anchor_spl::token_2022::spl_token_2022::{
    extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions},
    state::{Account, Mint},
};

pub struct TokenUtil;

impl TokenUtil {
    pub const ALLOWED_MINT_EXTENSION_TYPES: [ExtensionType; 7] = [
        ExtensionType::Uninitialized,
        ExtensionType::InterestBearingConfig,
        ExtensionType::MetadataPointer,
        ExtensionType::TokenMetadata,
        ExtensionType::TokenGroup,
        ExtensionType::TokenGroupMember,
        ExtensionType::GroupPointer,
    ];

    pub const ALLOWED_TOKEN_EXTENSION_TYPES: [ExtensionType; 2] =
        [ExtensionType::Uninitialized, ExtensionType::ImmutableOwner];

    #[cfg(not(tarpaulin_include))]
    fn mint_has_extensions(mint_account_info: &AccountInfo) -> Result<bool> {
        let mint_data = mint_account_info.data.borrow();

        let mint_with_extensions = StateWithExtensions::<Mint>::unpack(&mint_data)?;

        let mint_extension_types = mint_with_extensions.get_extension_types()?;
        let all_extensions_are_allowed = mint_extension_types
            .iter()
            .all(|extension_type| TokenUtil::ALLOWED_MINT_EXTENSION_TYPES.contains(extension_type));
        Ok(!all_extensions_are_allowed)
    }

    #[cfg(not(tarpaulin_include))]
    fn token_has_extensions(token_account_info: &AccountInfo) -> Result<bool> {
        let token_data = token_account_info.data.borrow();
        let token_with_extensions = StateWithExtensions::<Account>::unpack(&token_data)?;
        let token_extension_types = token_with_extensions.get_extension_types()?;

        let all_extensions_are_allowed = token_extension_types.iter().all(|extension_type| {
            TokenUtil::ALLOWED_TOKEN_EXTENSION_TYPES.contains(extension_type)
        });

        Ok(!all_extensions_are_allowed)
    }

    #[cfg(not(tarpaulin_include))]
    pub fn is_supported_spl_token(
        mint_account_info: Option<&AccountInfo>,
        token_account_info: Option<&AccountInfo>,
    ) -> Result<bool> {
        let mint_has_extensions = if let Some(mint_account_info) = mint_account_info {
            TokenUtil::mint_has_extensions(mint_account_info)?
        } else {
            false
        };
        let token_has_extensions = if let Some(token_account_info) = token_account_info {
            TokenUtil::token_has_extensions(token_account_info)?
        } else {
            false
        };

        Ok(!mint_has_extensions && !token_has_extensions)
    }
}
