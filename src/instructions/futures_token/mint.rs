//! Futures Token铸造指令模块
//! 
//! 本模块提供Futures Token资产的铸造功能，包括：
//! - 参数验证：验证铸造参数的有效性和边界条件
//! - 权限检查：验证铸造权限和授权状态
//! - 服务层调用：委托给FuturesTokenService执行核心业务逻辑
//! - 事件发射：发射Futures Token铸造事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于Futures Token铸造功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给FuturesTokenService
//! - 事件驱动：完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

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

/// Futures Token铸造参数结构体
/// 
/// 包含Futures Token铸造所需的所有参数：
/// - amount: 铸造数量
/// - futures_pool_id: 期货池ID
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct MintFuturesParams {
    /// 铸造数量
    pub amount: u64,
    /// 期货池ID
    pub futures_pool_id: Pubkey,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// Futures Token铸造账户上下文
/// 
/// 定义Futures Token铸造指令所需的账户结构：
/// - futures_token: Futures Token账户（可变，Futures Token类型约束）
/// - authority: 铸造权限账户（mint_authority约束）
/// - futures_pool: 期货池账户
/// - system_program: 系统程序
/// - token_program: 代币程序
/// - associated_token_account: 关联代币账户
/// - recipient_token_account: 接收者代币账户
#[derive(Accounts)]
pub struct MintFutures<'info> {
    /// Futures Token账户（可变，Futures Token类型约束）
    #[account(
        mut,
        constraint = futures_token.asset_type == AssetType::FuturesToken @ AssetError::InvalidAssetType
    )]
    pub futures_token: Account<'info, Asset>,
    
    /// 铸造权限账户（mint_authority约束）
    #[account(
        constraint = authority.key() == futures_token.mint_authority @ AssetError::InvalidMintAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 期货池账户
    /// CHECK: 由程序验证
    pub futures_pool: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
    
    /// 关联代币账户
    /// CHECK: 由代币程序验证
    pub associated_token_account: UncheckedAccount<'info>,
    
    /// 接收者代币账户
    #[account(mut)]
    pub recipient_token_account: Account<'info, TokenAccount>,
}

/// 验证Futures Token铸造参数
/// 
/// 检查Futures Token铸造参数的有效性和边界条件：
/// - 铸造数量验证
/// - 期货池ID验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: Futures Token铸造参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_mint_futures_params(params: &MintFuturesParams) -> Result<()> {
    // 验证铸造数量
    require!(
        params.amount > 0,
        AssetError::InvalidAmount
    );
    
    // 验证期货池ID
    require!(
        params.futures_pool_id != Pubkey::default(),
        AssetError::InvalidFuturesPoolId
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查Futures Token铸造权限
/// 
/// 验证Futures Token铸造权限和授权状态：
/// - 检查铸造权限
/// - 验证授权状态
/// 
/// # 参数
/// - authority: 权限账户
/// - futures_token: Futures Token账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_mint_authority_permission(
    authority: &Signer,
    futures_token: &Account<Asset>,
) -> Result<()> {
    // 检查铸造权限
    require!(
        authority.key() == futures_token.mint_authority,
        AssetError::InvalidMintAuthority
    );
    
    // 验证Futures Token状态
    require!(
        futures_token.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// Futures Token铸造指令
/// 
/// 执行Futures Token铸造操作，包括：
/// - 参数验证：验证铸造参数的有效性
/// - 权限检查：验证铸造权限
/// - 服务层调用：委托给FuturesTokenService执行铸造逻辑
/// - 事件发射：发射Futures Token铸造事件
/// 
/// # 参数
/// - ctx: Futures Token铸造账户上下文
/// - params: Futures Token铸造参数
/// 
/// # 返回
/// - Result<()>: 铸造操作结果
pub fn mint_futures_token(
    ctx: Context<MintFutures>,
    params: MintFuturesParams,
) -> Result<()> {
    // 参数验证
    validate_mint_futures_params(&params)?;
    
    // 权限检查
    check_mint_authority_permission(&ctx.accounts.authority, &ctx.accounts.futures_token)?;
    
    let futures_token = &mut ctx.accounts.futures_token;
    let authority = &ctx.accounts.authority;
    
    // 创建Futures Token服务实例
    let service = FuturesTokenService::new();
    
    // 执行Futures Token铸造
    service.mint(
        futures_token,
        params.amount,
        params.futures_pool_id,
        &params.exec_params,
    )?;
    
    // 发射Futures Token铸造事件
    emit!(AssetMinted {
        basket_id: futures_token.id,
        amount: params.amount,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::FuturesToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 