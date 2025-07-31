//! NFT碎片化指令模块
//! 
//! 本模块提供NFT资产的碎片化功能，包括：
//! - 参数验证：验证碎片化参数的有效性和边界条件
//! - 权限检查：验证碎片化权限和授权状态
//! - 服务层调用：委托给NftService执行核心业务逻辑
//! - 事件发射：发射NFT碎片化事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于NFT碎片化功能
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

/// NFT碎片化参数结构体
/// 
/// 包含NFT碎片化所需的所有参数：
/// - total_supply: 碎片化代币总供应量
/// - symbol: 碎片化代币符号
/// - name: 碎片化代币名称
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct FractionalizeNftParams {
    /// 碎片化代币总供应量
    pub total_supply: u64,
    /// 碎片化代币符号
    pub symbol: String,
    /// 碎片化代币名称
    pub name: String,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// NFT碎片化账户上下文
/// 
/// 定义NFT碎片化指令所需的账户结构：
/// - nft: NFT账户（可变，NFT类型约束）
/// - authority: 碎片化权限账户（owner约束）
/// - fractional_token_mint: 碎片化代币铸造账户
/// - fractional_token_account: 碎片化代币账户
/// - nft_token_account: NFT代币账户
/// - system_program: 系统程序
/// - token_program: 代币程序
/// - associated_token_program: 关联代币程序
#[derive(Accounts)]
pub struct FractionalizeNft<'info> {
    /// NFT账户（可变，NFT类型约束）
    #[account(
        mut,
        constraint = nft.asset_type == AssetType::NFT @ AssetError::InvalidAssetType
    )]
    pub nft: Account<'info, Asset>,
    
    /// 碎片化权限账户（owner约束）
    #[account(
        constraint = authority.key() == nft.owner @ AssetError::InvalidOwner
    )]
    pub authority: Signer<'info>,
    
    /// 碎片化代币铸造账户
    #[account(mut)]
    pub fractional_token_mint: Account<'info, Mint>,
    
    /// 碎片化代币账户
    #[account(mut)]
    pub fractional_token_account: Account<'info, TokenAccount>,
    
    /// NFT代币账户
    #[account(
        mut,
        constraint = nft_token_account.owner == authority.key() @ AssetError::InvalidTokenAccount
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
    
    /// 关联代币程序
    /// CHECK: 由关联代币程序验证
    pub associated_token_program: UncheckedAccount<'info>,
}

/// 验证NFT碎片化参数
/// 
/// 检查NFT碎片化参数的有效性和边界条件：
/// - 总供应量验证
/// - 符号验证
/// - 名称验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: NFT碎片化参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_fractionalize_nft_params(params: &FractionalizeNftParams) -> Result<()> {
    // 验证总供应量
    require!(
        params.total_supply > 0,
        AssetError::InvalidTotalSupply
    );
    
    require!(
        params.total_supply <= MAX_FRACTIONAL_SUPPLY,
        AssetError::TotalSupplyTooLarge
    );
    
    // 验证符号
    require!(
        params.symbol.len() <= MAX_SYMBOL_LENGTH,
        AssetError::SymbolTooLong
    );
    
    // 验证名称
    require!(
        params.name.len() <= MAX_NAME_LENGTH,
        AssetError::NameTooLong
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查NFT碎片化权限
/// 
/// 验证NFT碎片化权限和授权状态：
/// - 检查所有权
/// - 验证NFT状态
/// - 检查代币余额
/// 
/// # 参数
/// - authority: 权限账户
/// - nft: NFT账户
/// - nft_token_account: NFT代币账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_fractionalize_authority_permission(
    authority: &Signer,
    nft: &Account<Asset>,
    nft_token_account: &Account<TokenAccount>,
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
        nft_token_account.amount > 0,
        AssetError::InsufficientBalance
    );
    
    Ok(())
}

/// NFT碎片化指令
/// 
/// 执行NFT碎片化操作，包括：
/// - 参数验证：验证碎片化参数的有效性
/// - 权限检查：验证碎片化权限
/// - 服务层调用：委托给NftService执行碎片化逻辑
/// - 事件发射：发射NFT碎片化事件
/// 
/// # 参数
/// - ctx: NFT碎片化账户上下文
/// - params: NFT碎片化参数
/// 
/// # 返回
/// - Result<()>: 碎片化操作结果
pub fn fractionalize_nft(
    ctx: Context<FractionalizeNft>,
    params: FractionalizeNftParams,
) -> Result<()> {
    // 参数验证
    validate_fractionalize_nft_params(&params)?;
    
    // 权限检查
    check_fractionalize_authority_permission(
        &ctx.accounts.authority,
        &ctx.accounts.nft,
        &ctx.accounts.nft_token_account,
    )?;
    
    let nft = &mut ctx.accounts.nft;
    let authority = &ctx.accounts.authority;
    
    // 创建NFT服务实例
    let service = NftService::new();
    
    // 执行NFT碎片化
    service.fractionalize(
        nft,
        params.total_supply,
        &params.symbol,
        &params.name,
        &params.exec_params,
    )?;
    
    // 发射NFT碎片化事件
    emit!(AssetFractionalized {
        basket_id: nft.id,
        total_supply: params.total_supply,
        symbol: params.symbol,
        name: params.name,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::NFT,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 