//! 股票保证金交易指令
//!
//! 本模块实现了股票的保证金交易管理功能，包括保证金买入、保证金卖出、风险管理等。
//!
//! ## 功能特点
//!
//! - **多种交易类型**: 支持保证金买入、保证金卖出、杠杆交易
//! - **灵活管理方式**: 支持自动、手动、定期管理
//! - **保证金交易信息管理**: 完整的保证金交易记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 保证金买入
//! - 保证金卖出
//! - 杠杆交易
//! - 风险管理

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetMarginTradingExecuted;
use crate::errors::AssetError;

/// 保证金交易类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum MarginTradingType {
    /// 保证金买入
    Buy,
    /// 保证金卖出
    Sell,
    /// 杠杆交易
    Leverage,
}

/// 保证金交易管理方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum MarginTradingMethod {
    /// 自动管理
    Automatic,
    /// 手动管理
    Manual,
    /// 定期管理
    Periodic,
    /// 事件触发
    EventTriggered,
}

/// 保证金交易信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MarginTradingInfo {
    /// 交易类型
    pub trading_type: MarginTradingType,
    /// 保证金比例
    pub margin_ratio: f64,
    /// 杠杆倍数
    pub leverage: f64,
    /// 交易数量
    pub amount: u64,
    /// 交易开始时间
    pub start_time: i64,
    /// 交易结束时间
    pub end_time: i64,
    /// 说明
    pub description: String,
}

/// 保证金交易结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MarginTradingResult {
    /// 交易ID
    pub trading_id: u64,
    /// 交易类型
    pub trading_type: MarginTradingType,
    /// 管理方式
    pub method: MarginTradingMethod,
    /// 状态
    pub status: bool,
    /// 时间戳
    pub timestamp: i64,
}

/// 保证金交易指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MarginTradingParams {
    /// 交易类型
    pub trading_type: MarginTradingType,
    /// 管理方式
    pub method: MarginTradingMethod,
    /// 交易信息
    pub info: MarginTradingInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 保证金交易指令账户上下文
#[derive(Accounts)]
pub struct MarginTrading<'info> {
    /// 股票资产账户，需可变
    #[account(
        mut,
        constraint = stock.asset_type == AssetType::Stock @ AssetError::InvalidAssetType
    )]
    pub stock: Account<'info, BasketIndexState>,
    
    /// 保证金交易权限签名者
    #[account(
        constraint = authority.key() == stock.margin_trading_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 管理账户
    #[account(mut)]
    pub manage_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 保证金交易指令实现
pub fn margin_trading(
    ctx: Context<MarginTrading>,
    params: MarginTradingParams,
) -> Result<MarginTradingResult> {
    validate_margin_trading_params(&params)?;
    check_margin_trading_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    let service = StockService::new();
    let result = service.margin_trading(
        stock,
        &params.trading_type,
        &params.method,
        &params.info,
        &params.exec_params,
    )?;
    emit!(AssetMarginTradingExecuted {
        basket_id: stock.id,
        trading_id: result.trading_id,
        trading_type: params.trading_type,
        status: result.status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Stock,
        exec_params: params.exec_params,
    });
    Ok(result)
}
fn validate_margin_trading_params(params: &MarginTradingParams) -> Result<()> {
    require!(params.info.margin_ratio > 0.0, AssetError::InvalidMarginRatio);
    require!(params.info.leverage > 0.0, AssetError::InvalidLeverage);
    require!(params.info.amount > 0, AssetError::InvalidMarginTradingAmount);
    require!(params.info.start_time > 0, AssetError::InvalidMarginTradingTime);
    require!(params.info.end_time > 0, AssetError::InvalidMarginTradingTime);
    require!(params.info.end_time > params.info.start_time, AssetError::InvalidMarginTradingTime);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}
fn check_margin_trading_authority_permission(
    authority: &Signer,
    stock: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == stock.margin_trading_authority,
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