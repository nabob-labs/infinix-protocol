//! ETF (Exchange Traded Fund) 套利监控指令
//!
//! 本模块实现了ETF的套利监控功能，包括套利机会检测、套利执行和监控。
//!
//! ## 功能特点
//!
//! - **套利检测**: 检测ETF价格与净值的差异
//! - **套利执行**: 自动执行套利交易
//! - **套利监控**: 实时监控套利机会
//! - **风险控制**: 套利风险控制和管理
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::etf_service::EtfService;
use crate::events::asset_event::AssetArbitrageMonitoring;
use crate::errors::AssetError;

/// 套利类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ArbitrageType {
    /// 溢价套利
    Premium,
    /// 折价套利
    Discount,
    /// 统计套利
    Statistical,
    /// 跨市场套利
    CrossMarket,
}

/// 套利监控方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ArbitrageMonitoringMethod {
    /// 实时监控
    RealTime,
    /// 定期监控
    Periodic,
    /// 事件驱动监控
    EventDriven,
    /// 算法监控
    Algorithmic,
}

/// 套利机会信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ArbitrageOpportunity {
    /// 套利类型
    pub arbitrage_type: ArbitrageType,
    /// 套利价差
    pub spread: f64,
    /// 套利收益率
    pub return_rate: f64,
    /// 套利风险
    pub risk_level: f64,
    /// 套利时间窗口
    pub time_window: u64,
    /// 检测时间戳
    pub timestamp: i64,
}

/// 套利监控结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ArbitrageMonitoringResult {
    /// 套利机会
    pub opportunities: Vec<ArbitrageOpportunity>,
    /// 监控方式
    pub monitoring_method: ArbitrageMonitoringMethod,
    /// 监控成本
    pub monitoring_cost: u64,
    /// 执行建议
    pub execution_recommendations: Vec<String>,
    /// 监控时间戳
    pub timestamp: i64,
}

/// ETF套利监控指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ArbitrageMonitoringParams {
    /// 套利类型
    pub arbitrage_type: ArbitrageType,
    /// 监控方式
    pub monitoring_method: ArbitrageMonitoringMethod,
    /// 监控周期（秒）
    pub monitoring_period: u64,
    /// 最小套利价差
    pub min_spread: f64,
    /// 最大套利风险
    pub max_risk: f64,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// ETF套利监控指令账户上下文
#[derive(Accounts)]
pub struct ArbitrageMonitoring<'info> {
    /// ETF资产账户
    #[account(
        constraint = etf.asset_type == AssetType::Etf @ AssetError::InvalidAssetType
    )]
    pub etf: Account<'info, BasketIndexState>,
    
    /// 套利监控权限签名者
    #[account(
        constraint = authority.key() == etf.arbitrage_monitoring_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 价格预言机程序
    pub oracle_program: Program<'info, OracleProgram>,
    
    /// 市场数据程序
    pub market_data_program: Program<'info, MarketDataProgram>,
    
    /// 历史数据程序
    pub historical_data_program: Program<'info, HistoricalDataProgram>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
}

/// ETF套利监控指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 套利监控参数，包含套利类型、监控方式和周期
///
/// ## 返回值
/// - `Result<ArbitrageMonitoringResult>`: 套利监控结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidParams`: 无效的参数
/// - `ArbitrageMonitoringFailed`: 套利监控失败
pub fn arbitrage_monitoring(
    ctx: Context<ArbitrageMonitoring>,
    params: ArbitrageMonitoringParams,
) -> Result<ArbitrageMonitoringResult> {
    // 参数验证
    validate_arbitrage_monitoring_params(&params)?;
    
    // 权限检查
    check_arbitrage_monitoring_authority_permission(&ctx.accounts.authority, &ctx.accounts.etf)?;
    
    // 获取账户引用
    let etf = &ctx.accounts.etf;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = EtfService::new();
    
    // 调用服务层执行套利监控操作
    let result = service.arbitrage_monitoring(
        etf,
        &params.arbitrage_type,
        &params.monitoring_method,
        params.monitoring_period,
        params.min_spread,
        params.max_risk,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetArbitrageMonitoring {
        basket_id: etf.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Etf,
        arbitrage_type: params.arbitrage_type,
        monitoring_method: params.monitoring_method,
        opportunities: result.opportunities.clone(),
        monitoring_cost: result.monitoring_cost,
        execution_recommendations: result.execution_recommendations.clone(),
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证套利监控参数
fn validate_arbitrage_monitoring_params(params: &ArbitrageMonitoringParams) -> Result<()> {
    // 验证监控周期
    require!(params.monitoring_period > 0, AssetError::InvalidParams);
    require!(params.monitoring_period <= 24 * 60 * 60, AssetError::InvalidParams); // 最大1天
    
    // 验证最小套利价差
    require!(params.min_spread >= 0.0, AssetError::InvalidParams);
    require!(params.min_spread <= 1.0, AssetError::InvalidParams);
    
    // 验证最大套利风险
    require!(params.max_risk >= 0.0, AssetError::InvalidParams);
    require!(params.max_risk <= 1.0, AssetError::InvalidParams);
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
    Ok(())
}

/// 检查套利监控权限
fn check_arbitrage_monitoring_authority_permission(
    authority: &Signer,
    etf: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == etf.arbitrage_monitoring_authority,
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

/// 验证策略参数
fn validate_strategy_params(strategy_params: &StrategyParams) -> Result<()> {
    require!(strategy_params.max_slippage > 0.0, AssetError::InvalidParams);
    require!(strategy_params.max_slippage <= 1.0, AssetError::InvalidParams);
    require!(strategy_params.execution_timeout > 0, AssetError::InvalidParams);
    
    Ok(())
} 