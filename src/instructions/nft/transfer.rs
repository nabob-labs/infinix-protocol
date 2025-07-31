//! NFT转账指令模块

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::{
    core::{constants::*, events::*, types::*, validation::*},
    errors::*,
    services::*,
    utils::*,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct TransferNftParams {
    pub recipient: Pubkey,
    pub exec_params: ExecutionParams,
}

#[derive(Accounts)]
pub struct TransferNft<'info> {
    #[account(
        mut,
        constraint = nft.asset_type == AssetType::NFT @ AssetError::InvalidAssetType
    )]
    pub nft: Account<'info, Asset>,
    
    #[account(
        constraint = authority.key() == nft.owner @ AssetError::InvalidOwner
    )]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn validate_transfer_nft_params(params: &TransferNftParams) -> Result<()> {
    require!(params.recipient != Pubkey::default(), AssetError::InvalidRecipient);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

pub fn check_transfer_authority_permission(
    authority: &Signer,
    nft: &Account<Asset>,
) -> Result<()> {
    require!(authority.key() == nft.owner, AssetError::InvalidOwner);
    require!(nft.is_active(), AssetError::AssetNotActive);
    Ok(())
}

pub fn transfer_nft(ctx: Context<TransferNft>, params: TransferNftParams) -> Result<()> {
    validate_transfer_nft_params(&params)?;
    check_transfer_authority_permission(&ctx.accounts.authority, &ctx.accounts.nft)?;
    
    let nft = &mut ctx.accounts.nft;
    let authority = &ctx.accounts.authority;
    let service = NftService::new();
    
    service.transfer(nft, params.recipient, &params.exec_params)?;
    
    emit!(AssetTransferred {
        basket_id: nft.id,
        recipient: params.recipient,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::NFT,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 