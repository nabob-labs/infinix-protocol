//! 股票资产mint指令
//!
//! 本模块实现了股票资产的增发功能，包括参数验证、权限检查、服务层调用和事件发射。
//!
//! ## 功能特点
//!
//! - **参数验证**: 严格的输入参数验证和边界检查
//! - **权限控制**: 细粒度的权限验证和管理
//! - **服务层抽象**: 核心业务逻辑委托给StockService
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 股票的初始发行
//! - 股票的增发操作
//! - 股票的流动性管理
//! - 股票的治理操作

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetMinted;
use crate::errors::AssetError;

/// 股票增发指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MintStockParams {
    /// 增发数量
    pub amount: u64,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 股票增发指令账户上下文
#[derive(Accounts)]
pub struct MintStock<'info> {
    /// 股票资产账户，需可变
    #[account(
        mut,
        constraint = stock.asset_type == AssetType::Stock @ AssetError::InvalidAssetType
    )]
    pub stock: Account<'info, BasketIndexState>,
    
    /// 增发权限签名者
    #[account(
        constraint = authority.key() == stock.mint_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
    
    /// 关联代币账户
    #[account(mut)]
    pub associated_token_account: Account<'info, TokenAccount>,
    
    /// 接收方代币账户
    #[account(mut)]
    pub recipient_token_account: Account<'info, TokenAccount>,
}

/// 股票增发指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 增发参数，包含数量和执行参数
///
/// ## 返回值
/// - `Result<()>`: 操作结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidAmount`: 无效的增发数量
/// - `InvalidParams`: 无效的参数
pub fn mint_stock(
    ctx: Context<MintStock>,
    params: MintStockParams,
) -> Result<()> {
    // 参数验证
    validate_mint_stock_params(&params)?;
    
    // 权限检查
    check_mint_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    
    // 获取账户引用
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = StockService::new();
    
    // 调用服务层执行增发操作
    service.mint(stock, params.amount, &params.exec_params)?;
    
    // 发射事件
    emit!(AssetMinted {
        basket_id: stock.id,
        amount: params.amount,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Stock,
        exec_params: params.exec_params,
    });
    
    Ok(())
}

/// 验证增发参数
fn validate_mint_stock_params(params: &MintStockParams) -> Result<()> {
    require!(params.amount > 0, AssetError::InvalidAmount);
    require!(params.amount <= u64::MAX, AssetError::InvalidAmount);
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查增发权限
fn check_mint_authority_permission(
    authority: &Signer,
    stock: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == stock.mint_authority,
        AssetError::InsufficientAuthority
    );
    
    Ok(())
}

/// 验证执行参数
fn validate_execution_params(exec_params: &ExecutionParams) -> Result<()> {
    require!(exec_params.slippage_tolerance > 0.0, AssetError::InvalidParams);
    require!(exec_params.slippage_tolerance <= 1.0, AssetError::InvalidParams);
    require!(exec_params.max_retries > 0, AssetError::InvalidParams);
    require!(exec_params.max_retries <= 10, AssetError::InvalidParams);
    
    Ok(())
} 