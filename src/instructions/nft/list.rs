//! NFT上架指令模块
//! 
//! 本模块提供NFT资产的上架功能，包括：
//! - 参数验证：验证上架参数的有效性和边界条件
//! - 权限检查：验证上架权限和授权状态
//! - 服务层调用：委托给NftService执行核心业务逻辑
//! - 事件发射：发射NFT上架事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于NFT上架功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给NftService
//! - 事件驱动：完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::{
    core::{
        constants::*,
        events::*,
        types::*,
        validation::*,
    },
    errors::*,
    services::*,
    utils::*,
};

/// NFT上架参数结构体
/// 
/// 包含NFT上架所需的所有参数：
/// - price: 上架价格
/// - currency: 计价货币
/// - listing_duration: 上架持续时间
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct ListNftParams {
    /// 上架价格
    pub price: u64,
    /// 计价货币
    pub currency: Pubkey,
    /// 上架持续时间（秒）
    pub listing_duration: u64,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// NFT上架账户上下文
/// 
/// 定义NFT上架指令所需的账户结构：
/// - nft: NFT账户（可变，NFT类型约束）
/// - authority: 上架权限账户（owner约束）
/// - token_account: NFT代币账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct ListNft<'info> {
    /// NFT账户（可变，NFT类型约束）
    #[account(
        mut,
        constraint = nft.asset_type == AssetType::NFT @ AssetError::InvalidAssetType
    )]
    pub nft: Account<'info, Asset>,
    
    /// 上架权限账户（owner约束）
    #[account(
        constraint = authority.key() == nft.owner @ AssetError::InvalidOwner
    )]
    pub authority: Signer<'info>,
    
    /// NFT代币账户
    #[account(
        mut,
        constraint = token_account.owner == authority.key() @ AssetError::InvalidTokenAccount
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证NFT上架参数
/// 
/// 检查NFT上架参数的有效性和边界条件：
/// - 价格验证
/// - 计价货币验证
/// - 上架持续时间验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: NFT上架参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_list_nft_params(params: &ListNftParams) -> Result<()> {
    // 验证价格
    require!(
        params.price > 0,
        AssetError::InvalidPrice
    );
    
    // 验证计价货币
    require!(
        params.currency != Pubkey::default(),
        AssetError::InvalidCurrency
    );
    
    // 验证上架持续时间
    require!(
        params.listing_duration >= MIN_LISTING_DURATION,
        AssetError::ListingDurationTooShort
    );
    
    require!(
        params.listing_duration <= MAX_LISTING_DURATION,
        AssetError::ListingDurationTooLong
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查NFT上架权限
/// 
/// 验证NFT上架权限和授权状态：
/// - 检查所有权
/// - 验证NFT状态
/// - 检查代币余额
/// 
/// # 参数
/// - authority: 权限账户
/// - nft: NFT账户
/// - token_account: 代币账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_list_authority_permission(
    authority: &Signer,
    nft: &Account<Asset>,
    token_account: &Account<TokenAccount>,
) -> Result<()> {
    // 检查所有权
    require!(
        authority.key() == nft.owner,
        AssetError::InvalidOwner
    );
    
    // 验证NFT状态
    require!(
        nft.is_active(),
        AssetError::AssetNotActive
    );
    
    // 检查代币余额
    require!(
        token_account.amount > 0,
        AssetError::InsufficientBalance
    );
    
    Ok(())
}

/// NFT上架指令
/// 
/// 执行NFT上架操作，包括：
/// - 参数验证：验证上架参数的有效性
/// - 权限检查：验证上架权限
/// - 服务层调用：委托给NftService执行上架逻辑
/// - 事件发射：发射NFT上架事件
/// 
/// # 参数
/// - ctx: NFT上架账户上下文
/// - params: NFT上架参数
/// 
/// # 返回
/// - Result<()>: 上架操作结果
pub fn list_nft(
    ctx: Context<ListNft>,
    params: ListNftParams,
) -> Result<()> {
    // 参数验证
    validate_list_nft_params(&params)?;
    
    // 权限检查
    check_list_authority_permission(
        &ctx.accounts.authority,
        &ctx.accounts.nft,
        &ctx.accounts.token_account,
    )?;
    
    let nft = &mut ctx.accounts.nft;
    let authority = &ctx.accounts.authority;
    
    // 创建NFT服务实例
    let service = NftService::new();
    
    // 执行NFT上架
    service.list(
        nft,
        params.price,
        params.currency,
        params.listing_duration,
        &params.exec_params,
    )?;
    
    // 发射NFT上架事件
    emit!(AssetListed {
        basket_id: nft.id,
        price: params.price,
        currency: params.currency,
        listing_duration: params.listing_duration,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::NFT,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 