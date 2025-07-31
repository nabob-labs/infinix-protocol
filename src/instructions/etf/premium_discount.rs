//! ETF (Exchange Traded Fund) 溢价折价管理指令
//!
//! 本模块实现了ETF的溢价折价管理功能，包括溢价折价计算、监控和调整。
//!
//! ## 功能特点
//!
//! - **溢价折价计算**: 计算ETF相对于净值的溢价或折价
//! - **溢价折价监控**: 实时监控溢价折价的变化
//! - **自动调整**: 自动调整溢价折价
//! - **风险控制**: 溢价折价风险控制
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::etf_service::EtfService;
use crate::events::asset_event::AssetPremiumDiscountUpdated;
use crate::errors::AssetError;

/// 溢价折价类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum PremiumDiscountType {
    /// 溢价
    Premium,
    /// 折价
    Discount,
    /// 平价
    Par,
    /// 混合
    Mixed,
}

/// 溢价折价管理方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum PremiumDiscountManagementMethod {
    /// 自动管理
    Automatic,
    /// 手动管理
    Manual,
    /// 算法管理
    Algorithmic,
    /// 混合管理
    Hybrid,
}

/// 溢价折价信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PremiumDiscountInfo {
    /// 溢价折价类型
    pub premium_discount_type: PremiumDiscountType,
    /// 溢价折价值
    pub premium_discount_value: f64,
    /// 溢价折价率
    pub premium_discount_rate: f64,
    /// 市场价值
    pub market_value: f64,
    /// 净值
    pub nav_value: f64,
    /// 计算时间戳
    pub timestamp: i64,
}

/// 溢价折价管理结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PremiumDiscountResult {
    /// 溢价折价信息
    pub premium_discount_info: PremiumDiscountInfo,
    /// 管理方式
    pub management_method: PremiumDiscountManagementMethod,
    /// 调整建议
    pub adjustment_recommendations: Vec<String>,
    /// 管理成本
    pub management_cost: u64,
    /// 管理时间戳
    pub timestamp: i64,
}

/// ETF溢价折价管理指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PremiumDiscountParams {
    /// 溢价折价类型
    pub premium_discount_type: PremiumDiscountType,
    /// 管理方式
    pub management_method: PremiumDiscountManagementMethod,
    /// 目标溢价折价值
    pub target_value: f64,
    /// 容忍范围
    pub tolerance_range: f64,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// ETF溢价折价管理指令账户上下文
#[derive(Accounts)]
pub struct PremiumDiscount<'info> {
    /// ETF资产账户，需可变
    #[account(
        mut,
        constraint = etf.asset_type == AssetType::Etf @ AssetError::InvalidAssetType
    )]
    pub etf: Account<'info, BasketIndexState>,
    
    /// 溢价折价管理权限签名者
    #[account(
        constraint = authority.key() == etf.premium_discount_authority @ AssetError::InsufficientAuthority
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

/// ETF溢价折价管理指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 溢价折价管理参数，包含类型、管理方式和目标值
///
/// ## 返回值
/// - `Result<PremiumDiscountResult>`: 溢价折价管理结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidParams`: 无效的参数
/// - `PremiumDiscountManagementFailed`: 溢价折价管理失败
pub fn premium_discount(
    ctx: Context<PremiumDiscount>,
    params: PremiumDiscountParams,
) -> Result<PremiumDiscountResult> {
    // 参数验证
    validate_premium_discount_params(&params)?;
    
    // 权限检查
    check_premium_discount_authority_permission(&ctx.accounts.authority, &ctx.accounts.etf)?;
    
    // 获取账户引用
    let etf = &mut ctx.accounts.etf;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = EtfService::new();
    
    // 调用服务层执行溢价折价管理操作
    let result = service.premium_discount(
        etf,
        &params.premium_discount_type,
        &params.management_method,
        params.target_value,
        params.tolerance_range,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetPremiumDiscountUpdated {
        basket_id: etf.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Etf,
        premium_discount_type: params.premium_discount_type,
        management_method: params.management_method,
        premium_discount_info: result.premium_discount_info.clone(),
        adjustment_recommendations: result.adjustment_recommendations.clone(),
        management_cost: result.management_cost,
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证溢价折价管理参数
fn validate_premium_discount_params(params: &PremiumDiscountParams) -> Result<()> {
    // 验证目标值
    require!(params.target_value >= -1.0, AssetError::InvalidParams);
    require!(params.target_value <= 1.0, AssetError::InvalidParams);
    
    // 验证容忍范围
    require!(params.tolerance_range > 0.0, AssetError::InvalidParams);
    require!(params.tolerance_range <= 1.0, AssetError::InvalidParams);
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
    Ok(())
}

/// 检查溢价折价管理权限
fn check_premium_discount_authority_permission(
    authority: &Signer,
    etf: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == etf.premium_discount_authority,
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