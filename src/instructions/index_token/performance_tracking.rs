//! 指数代币 (IndexToken) 表现追踪指令
//!
//! 本模块实现了指数代币表现的追踪功能，包括收益率计算、风险指标监控和表现报告生成。
//!
//! ## 功能特点
//!
//! - **收益率计算**: 实时计算指数收益率
//! - **风险指标监控**: 监控波动率、夏普比率等风险指标
//! - **表现报告生成**: 生成详细的表现报告
//! - **历史数据追踪**: 追踪历史表现数据
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::index_token_service::IndexTokenService;
use crate::events::asset_event::AssetPerformanceTracked;
use crate::errors::AssetError;

/// 表现追踪类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum PerformanceTrackingType {
    /// 实时追踪
    RealTime,
    /// 日度追踪
    Daily,
    /// 周度追踪
    Weekly,
    /// 月度追踪
    Monthly,
    /// 年度追踪
    Yearly,
}

/// 表现指标类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum PerformanceMetricType {
    /// 总收益率
    TotalReturn,
    /// 年化收益率
    AnnualizedReturn,
    /// 波动率
    Volatility,
    /// 夏普比率
    SharpeRatio,
    /// 最大回撤
    MaxDrawdown,
    /// 贝塔系数
    Beta,
    /// 阿尔法系数
    Alpha,
}

/// 表现指标
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PerformanceMetric {
    /// 指标类型
    pub metric_type: PerformanceMetricType,
    /// 指标值
    pub value: f64,
    /// 基准值
    pub benchmark: f64,
    /// 时间戳
    pub timestamp: i64,
}

/// 表现追踪结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PerformanceTrackingResult {
    /// 追踪类型
    pub tracking_type: PerformanceTrackingType,
    /// 表现指标列表
    pub metrics: Vec<PerformanceMetric>,
    /// 追踪时间戳
    pub timestamp: i64,
    /// 追踪成本
    pub tracking_cost: u64,
}

/// 指数代币表现追踪指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PerformanceTrackingParams {
    /// 表现追踪类型
    pub tracking_type: PerformanceTrackingType,
    /// 要追踪的指标类型列表
    pub metric_types: Vec<PerformanceMetricType>,
    /// 追踪时间范围（秒）
    pub time_range: u64,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// 指数代币表现追踪指令账户上下文
#[derive(Accounts)]
pub struct PerformanceTracking<'info> {
    /// 指数代币资产账户
    #[account(
        constraint = index_token.asset_type == AssetType::IndexToken @ AssetError::InvalidAssetType
    )]
    pub index_token: Account<'info, BasketIndexState>,
    
    /// 表现追踪权限签名者
    #[account(
        constraint = authority.key() == index_token.performance_tracking_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 价格预言机程序
    pub oracle_program: Program<'info, OracleProgram>,
    
    /// 历史数据程序
    pub historical_data_program: Program<'info, HistoricalDataProgram>,
    
    /// 成分股代币账户列表
    pub constituent_tokens: Vec<Account<'info, TokenAccount>>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
}

/// 指数代币表现追踪指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 表现追踪参数，包含追踪类型、指标类型和时间范围
///
/// ## 返回值
/// - `Result<PerformanceTrackingResult>`: 表现追踪结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidParams`: 无效的参数
/// - `PerformanceTrackingFailed`: 表现追踪失败
pub fn performance_tracking(
    ctx: Context<PerformanceTracking>,
    params: PerformanceTrackingParams,
) -> Result<PerformanceTrackingResult> {
    // 参数验证
    validate_performance_tracking_params(&params)?;
    
    // 权限检查
    check_performance_tracking_authority_permission(&ctx.accounts.authority, &ctx.accounts.index_token)?;
    
    // 获取账户引用
    let index_token = &ctx.accounts.index_token;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = IndexTokenService::new();
    
    // 调用服务层执行表现追踪操作
    let result = service.performance_tracking(
        index_token,
        &params.tracking_type,
        &params.metric_types,
        params.time_range,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetPerformanceTracked {
        basket_id: index_token.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::IndexToken,
        tracking_type: params.tracking_type,
        metric_types: params.metric_types,
        metrics: result.metrics.clone(),
        tracking_cost: result.tracking_cost,
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证表现追踪参数
fn validate_performance_tracking_params(params: &PerformanceTrackingParams) -> Result<()> {
    // 验证指标类型列表
    validate_metric_types(&params.metric_types)?;
    
    // 验证时间范围
    validate_time_range(params.time_range)?;
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
    Ok(())
}

/// 验证指标类型列表
fn validate_metric_types(metric_types: &[PerformanceMetricType]) -> Result<()> {
    require!(!metric_types.is_empty(), AssetError::InvalidParams);
    require!(metric_types.len() <= 10, AssetError::InvalidParams);
    
    Ok(())
}

/// 验证时间范围
fn validate_time_range(time_range: u64) -> Result<()> {
    require!(time_range > 0, AssetError::InvalidParams);
    require!(time_range <= 365 * 24 * 60 * 60, AssetError::InvalidParams); // 最大1年
    
    Ok(())
}

/// 检查表现追踪权限
fn check_performance_tracking_authority_permission(
    authority: &Signer,
    index_token: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == index_token.performance_tracking_authority,
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