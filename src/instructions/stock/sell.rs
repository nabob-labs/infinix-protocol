//! 股票卖出指令
//!
//! 本模块实现了股票的卖出功能，包括参数验证、权限检查、服务层调用和事件发射。
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
//! - 股票卖出操作
//! - 投资组合调整
//! - 自动化交易

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetSold;
use crate::errors::AssetError;

/// 股票卖出指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SellStockParams {
    /// 卖出数量
    pub amount: u64,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 股票卖出指令账户上下文
#[derive(Accounts)]
pub struct SellStock<'info> {
    /// 股票资产账户，需可变
    #[account(
        mut,
        constraint = stock.asset_type == AssetType::Stock @ AssetError::InvalidAssetType
    )]
    pub stock: Account<'info, BasketIndexState>,
    /// 卖出权限签名者
    #[account(
        constraint = authority.key() == stock.sell_authority @ AssetError::InsufficientAuthority
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

/// 股票卖出指令实现
pub fn sell_stock(
    ctx: Context<SellStock>,
    params: SellStockParams,
) -> Result<()> {
    validate_sell_stock_params(&params)?;
    check_sell_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    let service = StockService::new();
    service.sell(stock, params.amount, &params.exec_params)?;
    emit!(AssetSold {
        basket_id: stock.id,
        amount: params.amount,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Stock,
        exec_params: params.exec_params,
    });
    Ok(())
}
fn validate_sell_stock_params(params: &SellStockParams) -> Result<()> {
    require!(params.amount > 0, AssetError::InvalidAmount);
    require!(params.amount <= u64::MAX, AssetError::InvalidAmount);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}
fn check_sell_authority_permission(
    authority: &Signer,
    stock: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == stock.sell_authority,
        AssetError::InsufficientAuthority
    );
    Ok(())
}
fn validate_execution_params(exec_params: &ExecutionParams) -> Result<()> {
    require!(exec_params.slippage_tolerance > 0.0, AssetError::InvalidParams);
    require!(exec_params.slippage_tolerance <= 1.0, AssetError::InvalidParams);
    require!(exec_params.max_retries > 0, AssetError::InvalidParams);
    require!(exec_params.max_retries <= 10, AssetError::InvalidParams);
    Ok(())
} 