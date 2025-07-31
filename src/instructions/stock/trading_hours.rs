//! 股票交易时间指令
//!
//! 本模块实现了股票的交易时间管理功能，包括开盘、收盘、休市、特殊时段等。
//!
//! ## 功能特点
//!
//! - **多种时间类型**: 支持开盘、收盘、休市、特殊时段
//! - **灵活管理方式**: 支持自动、手动、定期调整
//! - **交易时间信息管理**: 完整的交易时间记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 开盘时间管理
//! - 收盘时间管理
//! - 休市安排
//! - 特殊时段管理

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetTradingHoursUpdated;
use crate::errors::AssetError;

/// 交易时间类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum TradingHoursType {
    /// 开盘
    Open,
    /// 收盘
    Close,
    /// 休市
    Holiday,
    /// 特殊时段
    Special,
}

/// 交易时间管理方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum TradingHoursMethod {
    /// 自动调整
    Automatic,
    /// 手动调整
    Manual,
    /// 定期调整
    Periodic,
    /// 事件触发
    EventTriggered,
}

/// 交易时间信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TradingHoursInfo {
    /// 时间类型
    pub hours_type: TradingHoursType,
    /// 开始时间
    pub start_time: i64,
    /// 结束时间
    pub end_time: i64,
    /// 说明
    pub description: String,
}

/// 交易时间结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TradingHoursResult {
    /// 记录ID
    pub record_id: u64,
    /// 时间类型
    pub hours_type: TradingHoursType,
    /// 管理方式
    pub method: TradingHoursMethod,
    /// 状态
    pub status: bool,
    /// 时间戳
    pub timestamp: i64,
}

/// 交易时间指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TradingHoursParams {
    /// 时间类型
    pub hours_type: TradingHoursType,
    /// 管理方式
    pub method: TradingHoursMethod,
    /// 时间信息
    pub info: TradingHoursInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 交易时间指令账户上下文
#[derive(Accounts)]
pub struct TradingHours<'info> {
    /// 股票资产账户，需可变
    #[account(
        mut,
        constraint = stock.asset_type == AssetType::Stock @ AssetError::InvalidAssetType
    )]
    pub stock: Account<'info, BasketIndexState>,
    
    /// 交易时间权限签名者
    #[account(
        constraint = authority.key() == stock.trading_hours_authority @ AssetError::InsufficientAuthority
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

/// 交易时间指令实现
pub fn trading_hours(
    ctx: Context<TradingHours>,
    params: TradingHoursParams,
) -> Result<TradingHoursResult> {
    validate_trading_hours_params(&params)?;
    check_trading_hours_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    let service = StockService::new();
    let result = service.trading_hours(
        stock,
        &params.hours_type,
        &params.method,
        &params.info,
        &params.exec_params,
    )?;
    emit!(AssetTradingHoursUpdated {
        basket_id: stock.id,
        record_id: result.record_id,
        hours_type: params.hours_type,
        status: result.status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Stock,
        exec_params: params.exec_params,
    });
    Ok(result)
}
fn validate_trading_hours_params(params: &TradingHoursParams) -> Result<()> {
    require!(params.info.start_time > 0, AssetError::InvalidTradingHoursTime);
    require!(params.info.end_time > 0, AssetError::InvalidTradingHoursTime);
    require!(params.info.end_time > params.info.start_time, AssetError::InvalidTradingHoursTime);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}
fn check_trading_hours_authority_permission(
    authority: &Signer,
    stock: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == stock.trading_hours_authority,
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