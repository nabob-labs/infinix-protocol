//! 股票兑换指令
//!
//! 本模块实现了股票的兑换功能，包括参数验证、权限检查、服务层调用和事件发射。
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
//! - 股票兑换操作
//! - 投资组合调整
//! - 自动化交易

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, SwapParams};
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetSwapped;
use crate::errors::AssetError;

/// 股票兑换指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SwapStockParams {
    /// 兑换参数
    pub swap_params: SwapParams,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 股票兑换指令账户上下文
#[derive(Accounts)]
pub struct SwapStock<'info> {
    /// 股票资产账户，需可变
    #[account(
        mut,
        constraint = stock.asset_type == AssetType::Stock @ AssetError::InvalidAssetType
    )]
    pub stock: Account<'info, BasketIndexState>,
    /// 兑换权限签名者
    #[account(
        constraint = authority.key() == stock.swap_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    /// 资金账户
    #[account(mut)]
    pub fund_account: Account<'info, TokenAccount>,
    /// 系统程序
    pub system_program: Program<'info, System>,
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 股票兑换指令实现
pub fn swap_stock(
    ctx: Context<SwapStock>,
    params: SwapStockParams,
) -> Result<()> {
    validate_swap_stock_params(&params)?;
    check_swap_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    let service = StockService::new();
    service.swap(stock, &params.swap_params, &params.exec_params)?;
    emit!(AssetSwapped {
        basket_id: stock.id,
        swap_params: params.swap_params.clone(),
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Stock,
        exec_params: params.exec_params,
    });
    Ok(())
}
fn validate_swap_stock_params(params: &SwapStockParams) -> Result<()> {
    // 这里假设SwapParams有自己的校验逻辑
    require!(params.exec_params.slippage_tolerance > 0.0, AssetError::InvalidParams);
    require!(params.exec_params.slippage_tolerance <= 1.0, AssetError::InvalidParams);
    require!(params.exec_params.max_retries > 0, AssetError::InvalidParams);
    require!(params.exec_params.max_retries <= 10, AssetError::InvalidParams);
    Ok(())
}
fn check_swap_authority_permission(
    authority: &Signer,
    stock: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == stock.swap_authority,
        AssetError::InsufficientAuthority
    );
    Ok(())
} 