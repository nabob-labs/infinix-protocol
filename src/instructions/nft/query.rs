//! NFT查询指令模块

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::{
    core::{constants::*, events::*, types::*, validation::*},
    errors::*,
    services::*,
    utils::*,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct QueryNftParams {
    pub query_type: String,
    pub exec_params: ExecutionParams,
}

#[derive(Accounts)]
pub struct QueryNft<'info> {
    #[account(
        constraint = nft.asset_type == AssetType::NFT @ AssetError::InvalidAssetType
    )]
    pub nft: Account<'info, Asset>,
    
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn validate_query_nft_params(params: &QueryNftParams) -> Result<()> {
    require!(!params.query_type.is_empty(), AssetError::InvalidQueryType);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

pub fn check_query_authority_permission(
    authority: &Signer,
    nft: &Account<Asset>,
) -> Result<()> {
    require!(nft.is_active(), AssetError::AssetNotActive);
    Ok(())
}

pub fn query_nft(ctx: Context<QueryNft>, params: QueryNftParams) -> Result<()> {
    validate_query_nft_params(&params)?;
    check_query_authority_permission(&ctx.accounts.authority, &ctx.accounts.nft)?;
    
    let nft = &ctx.accounts.nft;
    let authority = &ctx.accounts.authority;
    let service = NftService::new();
    
    service.query(nft, &params.query_type, &params.exec_params)?;
    
    emit!(AssetQueried {
        basket_id: nft.id,
        query_type: params.query_type,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::NFT,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 