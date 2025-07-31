//! 股票熔断机制指令
//!
//! 本模块实现了股票的熔断机制管理功能，包括价格熔断、交易熔断、波动熔断等。
//!
//! ## 功能特点
//!
//! - **多种熔断类型**: 支持价格熔断、交易熔断、波动熔断
//! - **灵活管理方式**: 支持自动、手动、定期触发
//! - **熔断信息管理**: 完整的熔断记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 价格异常熔断
//! - 交易量异常熔断
//! - 波动率异常熔断
//! - 熔断恢复管理

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetCircuitBreakerTriggered;
use crate::errors::AssetError;

/// 熔断类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CircuitBreakerType {
    /// 价格熔断
    Price,
    /// 交易熔断
    Trading,
    /// 波动熔断
    Volatility,
    /// 系统熔断
    System,
}

/// 熔断管理方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CircuitBreakerMethod {
    /// 自动触发
    Automatic,
    /// 手动触发
    Manual,
    /// 定期触发
    Periodic,
    /// 事件触发
    EventTriggered,
}

/// 熔断信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CircuitBreakerInfo {
    /// 熔断类型
    pub breaker_type: CircuitBreakerType,
    /// 触发价格
    pub trigger_price: Option<f64>,
    /// 触发交易量
    pub trigger_volume: Option<u64>,
    /// 触发波动率
    pub trigger_volatility: Option<f64>,
    /// 熔断开始时间
    pub start_time: i64,
    /// 熔断结束时间
    pub end_time: i64,
    /// 说明
    pub description: String,
}

/// 熔断结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CircuitBreakerResult {
    /// 熔断ID
    pub breaker_id: u64,
    /// 熔断类型
    pub breaker_type: CircuitBreakerType,
    /// 管理方式
    pub method: CircuitBreakerMethod,
    /// 状态
    pub status: bool,
    /// 时间戳
    pub timestamp: i64,
}

/// 熔断机制指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CircuitBreakerParams {
    /// 熔断类型
    pub breaker_type: CircuitBreakerType,
    /// 管理方式
    pub method: CircuitBreakerMethod,
    /// 熔断信息
    pub info: CircuitBreakerInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 熔断机制指令账户上下文
#[derive(Accounts)]
pub struct CircuitBreaker<'info> {
    /// 股票资产账户，需可变
    #[account(
        mut,
        constraint = stock.asset_type == AssetType::Stock @ AssetError::InvalidAssetType
    )]
    pub stock: Account<'info, BasketIndexState>,
    
    /// 熔断权限签名者
    #[account(
        constraint = authority.key() == stock.circuit_breaker_authority @ AssetError::InsufficientAuthority
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

/// 熔断机制指令实现
pub fn circuit_breaker(
    ctx: Context<CircuitBreaker>,
    params: CircuitBreakerParams,
) -> Result<CircuitBreakerResult> {
    validate_circuit_breaker_params(&params)?;
    check_circuit_breaker_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    let service = StockService::new();
    let result = service.circuit_breaker(
        stock,
        &params.breaker_type,
        &params.method,
        &params.info,
        &params.exec_params,
    )?;
    emit!(AssetCircuitBreakerTriggered {
        basket_id: stock.id,
        breaker_id: result.breaker_id,
        breaker_type: params.breaker_type,
        status: result.status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Stock,
        exec_params: params.exec_params,
    });
    Ok(result)
}
fn validate_circuit_breaker_params(params: &CircuitBreakerParams) -> Result<()> {
    require!(params.info.start_time > 0, AssetError::InvalidCircuitBreakerTime);
    require!(params.info.end_time > 0, AssetError::InvalidCircuitBreakerTime);
    require!(params.info.end_time > params.info.start_time, AssetError::InvalidCircuitBreakerTime);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}
fn check_circuit_breaker_authority_permission(
    authority: &Signer,
    stock: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == stock.circuit_breaker_authority,
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