//! 股票报价指令
//!
//! 本模块实现了股票的报价功能，包括参数验证、权限检查、服务层调用和事件发射。
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
//! - 股票报价操作
//! - 投资组合估值
//! - 自动化交易

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, QuoteParams};
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetQuoted;
use crate::errors::AssetError;

/// 股票报价指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct QuoteStockParams {
    /// 报价参数
    pub quote_params: QuoteParams,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 股票报价指令账户上下文
#[derive(Accounts)]
pub struct QuoteStock<'info> {
    /// 股票资产账户，需可变
    #[account(
        mut,
        constraint = stock.asset_type == AssetType::Stock @ AssetError::InvalidAssetType
    )]
    pub stock: Account<'info, BasketIndexState>,
    /// 报价权限签名者
    #[account(
        constraint = authority.key() == stock.quote_authority @ AssetError::InsufficientAuthority
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

/// 股票报价指令实现
pub fn quote_stock(
    ctx: Context<QuoteStock>,
    params: QuoteStockParams,
) -> Result<()> {
    validate_quote_stock_params(&params)?;
    check_quote_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    let service = StockService::new();
    service.quote(stock, &params.quote_params, &params.exec_params)?;
    emit!(AssetQuoted {
        basket_id: stock.id,
        quote_params: params.quote_params.clone(),
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Stock,
        exec_params: params.exec_params,
    });
    Ok(())
}
fn validate_quote_stock_params(params: &QuoteStockParams) -> Result<()> {
    // 这里假设QuoteParams有自己的校验逻辑
    require!(params.exec_params.slippage_tolerance > 0.0, AssetError::InvalidParams);
    require!(params.exec_params.slippage_tolerance <= 1.0, AssetError::InvalidParams);
    require!(params.exec_params.max_retries > 0, AssetError::InvalidParams);
    require!(params.exec_params.max_retries <= 10, AssetError::InvalidParams);
    Ok(())
}
fn check_quote_authority_permission(
    authority: &Signer,
    stock: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == stock.quote_authority,
        AssetError::InsufficientAuthority
    );
    Ok(())
} 