//! ETF (Exchange Traded Fund) 跟踪误差管理指令
//!
//! 本模块实现了ETF的跟踪误差管理功能，包括跟踪误差计算、监控和优化。
//!
//! ## 功能特点
//!
//! - **跟踪误差计算**: 计算ETF相对于基准指数的跟踪误差
//! - **误差监控**: 实时监控跟踪误差的变化
//! - **误差优化**: 自动优化跟踪误差
//! - **基准管理**: 管理基准指数和权重
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::etf_service::EtfService;
use crate::events::asset_event::AssetTrackingErrorUpdated;
use crate::errors::AssetError;

/// 跟踪误差类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum TrackingErrorType {
    /// 绝对跟踪误差
    Absolute,
    /// 相对跟踪误差
    Relative,
    /// 年化跟踪误差
    Annualized,
    /// 滚动跟踪误差
    Rolling,
}

/// 跟踪误差计算方法
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum TrackingErrorMethod {
    /// 标准差方法
    StandardDeviation,
    /// 方差方法
    Variance,
    /// 平均绝对偏差
    MeanAbsoluteDeviation,
    /// 最大回撤
    MaximumDrawdown,
}

/// 跟踪误差信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TrackingErrorInfo {
    /// 跟踪误差类型
    pub tracking_error_type: TrackingErrorType,
    /// 跟踪误差值
    pub tracking_error_value: f64,
    /// 基准收益率
    pub benchmark_return: f64,
    /// ETF收益率
    pub etf_return: f64,
    /// 计算周期
    pub calculation_period: u64,
    /// 计算时间戳
    pub timestamp: i64,
}

/// 跟踪误差更新结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TrackingErrorResult {
    /// 跟踪误差信息
    pub tracking_error_info: TrackingErrorInfo,
    /// 计算方法
    pub calculation_method: TrackingErrorMethod,
    /// 优化建议
    pub optimization_suggestions: Vec<String>,
    /// 更新时间戳
    pub timestamp: i64,
}

/// ETF跟踪误差管理指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct TrackingErrorParams {
    /// 跟踪误差类型
    pub tracking_error_type: TrackingErrorType,
    /// 计算方法
    pub calculation_method: TrackingErrorMethod,
    /// 计算周期（天）
    pub calculation_period: u64,
    /// 基准指数
    pub benchmark_index: String,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// ETF跟踪误差管理指令账户上下文
#[derive(Accounts)]
pub struct TrackingError<'info> {
    /// ETF资产账户
    #[account(
        constraint = etf.asset_type == AssetType::Etf @ AssetError::InvalidAssetType
    )]
    pub etf: Account<'info, BasketIndexState>,
    
    /// 跟踪误差管理权限签名者
    #[account(
        constraint = authority.key() == etf.tracking_error_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 基准指数数据程序
    pub benchmark_data_program: Program<'info, BenchmarkDataProgram>,
    
    /// 历史数据程序
    pub historical_data_program: Program<'info, HistoricalDataProgram>,
    
    /// 价格预言机程序
    pub oracle_program: Program<'info, OracleProgram>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
}

/// ETF跟踪误差管理指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 跟踪误差管理参数，包含类型、方法和周期
///
/// ## 返回值
/// - `Result<TrackingErrorResult>`: 跟踪误差计算结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidParams`: 无效的参数
/// - `TrackingErrorCalculationFailed`: 跟踪误差计算失败
pub fn tracking_error(
    ctx: Context<TrackingError>,
    params: TrackingErrorParams,
) -> Result<TrackingErrorResult> {
    // 参数验证
    validate_tracking_error_params(&params)?;
    
    // 权限检查
    check_tracking_error_authority_permission(&ctx.accounts.authority, &ctx.accounts.etf)?;
    
    // 获取账户引用
    let etf = &ctx.accounts.etf;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = EtfService::new();
    
    // 调用服务层执行跟踪误差管理操作
    let result = service.tracking_error(
        etf,
        &params.tracking_error_type,
        &params.calculation_method,
        params.calculation_period,
        &params.benchmark_index,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetTrackingErrorUpdated {
        basket_id: etf.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Etf,
        tracking_error_type: params.tracking_error_type,
        calculation_method: params.calculation_method,
        tracking_error_info: result.tracking_error_info.clone(),
        optimization_suggestions: result.optimization_suggestions.clone(),
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证跟踪误差管理参数
fn validate_tracking_error_params(params: &TrackingErrorParams) -> Result<()> {
    // 验证计算周期
    require!(params.calculation_period > 0, AssetError::InvalidParams);
    require!(params.calculation_period <= 365, AssetError::InvalidParams); // 最大1年
    
    // 验证基准指数
    require!(!params.benchmark_index.is_empty(), AssetError::InvalidParams);
    require!(params.benchmark_index.len() <= 100, AssetError::InvalidParams);
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
    Ok(())
}

/// 检查跟踪误差管理权限
fn check_tracking_error_authority_permission(
    authority: &Signer,
    etf: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == etf.tracking_error_authority,
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