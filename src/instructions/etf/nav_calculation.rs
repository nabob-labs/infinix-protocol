//! ETF (Exchange Traded Fund) 净值计算指令
//!
//! 本模块实现了ETF的净值计算功能，包括实时净值计算、历史净值追踪和净值验证。
//!
//! ## 功能特点
//!
//! - **实时净值计算**: 基于成分股价格实时计算ETF净值
//! - **历史净值追踪**: 追踪和记录历史净值数据
//! - **净值验证**: 验证净值的准确性和合理性
//! - **费用扣除**: 自动扣除管理费和托管费
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::etf_service::EtfService;
use crate::events::asset_event::AssetNavCalculated;
use crate::errors::AssetError;

/// 净值计算类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum NavCalculationType {
    /// 实时净值
    RealTime,
    /// 日终净值
    EndOfDay,
    /// 历史净值
    Historical,
    /// 预测净值
    Predictive,
}

/// 净值计算方法
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum NavCalculationMethod {
    /// 市值加权
    MarketCapWeighted,
    /// 等权重
    EqualWeighted,
    /// 价格加权
    PriceWeighted,
    /// 自定义权重
    CustomWeighted,
}

/// 净值信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct NavInfo {
    /// 净值
    pub nav: f64,
    /// 总资产价值
    pub total_assets: f64,
    /// 总负债
    pub total_liabilities: f64,
    /// 总份额
    pub total_shares: u64,
    /// 管理费
    pub management_fee: f64,
    /// 托管费
    pub custody_fee: f64,
    /// 计算时间戳
    pub timestamp: i64,
}

/// 净值计算结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct NavCalculationResult {
    /// 净值信息
    pub nav_info: NavInfo,
    /// 计算类型
    pub calculation_type: NavCalculationType,
    /// 计算方法
    pub calculation_method: NavCalculationMethod,
    /// 计算成本
    pub calculation_cost: u64,
    /// 计算时间戳
    pub timestamp: i64,
}

/// ETF净值计算指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct NavCalculationParams {
    /// 净值计算类型
    pub calculation_type: NavCalculationType,
    /// 净值计算方法
    pub calculation_method: NavCalculationMethod,
    /// 计算时间范围（秒）
    pub time_range: u64,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// ETF净值计算指令账户上下文
#[derive(Accounts)]
pub struct NavCalculation<'info> {
    /// ETF资产账户
    #[account(
        constraint = etf.asset_type == AssetType::Etf @ AssetError::InvalidAssetType
    )]
    pub etf: Account<'info, BasketIndexState>,
    
    /// 净值计算权限签名者
    #[account(
        constraint = authority.key() == etf.nav_calculation_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 价格预言机程序
    pub oracle_program: Program<'info, OracleProgram>,
    
    /// 历史数据程序
    pub historical_data_program: Program<'info, HistoricalDataProgram>,
    
    /// 成分股代币账户列表
    pub constituent_token_accounts: Vec<Account<'info, TokenAccount>>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
}

/// ETF净值计算指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 净值计算参数，包含计算类型、方法和时间范围
///
/// ## 返回值
/// - `Result<NavCalculationResult>`: 净值计算结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidParams`: 无效的参数
/// - `NavCalculationFailed`: 净值计算失败
pub fn nav_calculation(
    ctx: Context<NavCalculation>,
    params: NavCalculationParams,
) -> Result<NavCalculationResult> {
    // 参数验证
    validate_nav_calculation_params(&params)?;
    
    // 权限检查
    check_nav_calculation_authority_permission(&ctx.accounts.authority, &ctx.accounts.etf)?;
    
    // 获取账户引用
    let etf = &ctx.accounts.etf;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = EtfService::new();
    
    // 调用服务层执行净值计算操作
    let result = service.nav_calculation(
        etf,
        &params.calculation_type,
        &params.calculation_method,
        params.time_range,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetNavCalculated {
        basket_id: etf.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Etf,
        calculation_type: params.calculation_type,
        calculation_method: params.calculation_method,
        nav_info: result.nav_info.clone(),
        calculation_cost: result.calculation_cost,
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证净值计算参数
fn validate_nav_calculation_params(params: &NavCalculationParams) -> Result<()> {
    // 验证时间范围
    validate_time_range(params.time_range)?;
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
    Ok(())
}

/// 验证时间范围
fn validate_time_range(time_range: u64) -> Result<()> {
    require!(time_range > 0, AssetError::InvalidParams);
    require!(time_range <= 365 * 24 * 60 * 60, AssetError::InvalidParams); // 最大1年
    
    Ok(())
}

/// 检查净值计算权限
fn check_nav_calculation_authority_permission(
    authority: &Signer,
    etf: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == etf.nav_calculation_authority,
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