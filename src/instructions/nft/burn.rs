//! NFT销毁指令模块

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::{
    core::{constants::*, events::*, types::*, validation::*},
    errors::*,
    services::*,
    utils::*,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BurnNftParams {
    pub exec_params: ExecutionParams,
}

#[derive(Accounts)]
pub struct BurnNft<'info> {
    #[account(
        mut,
        constraint = nft.asset_type == AssetType::NFT @ AssetError::InvalidAssetType
    )]
    pub nft: Account<'info, Asset>,
    
    #[account(
        constraint = authority.key() == nft.burn_authority @ AssetError::InvalidBurnAuthority
    )]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn validate_burn_nft_params(params: &BurnNftParams) -> Result<()> {
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

pub fn check_burn_authority_permission(
    authority: &Signer,
    nft: &Account<Asset>,
) -> Result<()> {
    require!(
        authority.key() == nft.burn_authority,
        AssetError::InvalidBurnAuthority
    );
    require!(nft.is_active(), AssetError::AssetNotActive);
    Ok(())
}

pub fn burn_nft(ctx: Context<BurnNft>, params: BurnNftParams) -> Result<()> {
    validate_burn_nft_params(&params)?;
    check_burn_authority_permission(&ctx.accounts.authority, &ctx.accounts.nft)?;
    
    let nft = &mut ctx.accounts.nft;
    let authority = &ctx.accounts.authority;
    let service = NftService::new();
    
    service.burn(nft, &params.exec_params)?;
    
    emit!(AssetBurned {
        basket_id: nft.id,
        amount: 1,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::NFT,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 