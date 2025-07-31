//! 稳定币 (Stablecoin) 风险评估指令
//! 
//! 本模块实现稳定币资产的风险评估功能，支持抵押品风险、市场风险、流动性风险等评估。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 抵押品风险评估：评估抵押品的质量和价值
//! - 市场风险评估：评估市场波动和价格风险
//! - 流动性风险评估：评估流动性充足性
//! - 信用风险评估：评估信用违约风险
//! - 事件记录：完整的审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, RiskParams};
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetRiskAssessed;
use crate::validation::business::validate_risk_assessment_params;
use crate::core::security::check_authority_permission;
use crate::dex::traits::DexAdapterTrait;
use crate::oracles::traits::OracleAdapterTrait;

/// 稳定币风险评估参数结构体
/// 
/// 定义风险评估操作所需的所有参数，包括：
/// - risk_type: 风险类型
/// - assessment_period: 评估周期
/// - risk_thresholds: 风险阈值
/// - stress_scenarios: 压力测试场景
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RiskAssessmentStablecoinParams {
    /// 风险类型
    pub risk_type: RiskType,
    /// 评估周期（秒）
    pub assessment_period: u64,
    /// 风险阈值（基点，1/10000）
    pub risk_thresholds: RiskThresholds,
    /// 压力测试场景
    pub stress_scenarios: Option<Vec<StressScenario>>,
    /// 执行参数（可选）
    pub exec_params: Option<ExecutionParams>,
    /// 策略参数（可选）
    pub strategy_params: Option<StrategyParams>,
}

/// 风险类型枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum RiskType {
    /// 抵押品风险
    CollateralRisk,
    /// 市场风险
    MarketRisk,
    /// 流动性风险
    LiquidityRisk,
    /// 信用风险
    CreditRisk,
    /// 操作风险
    OperationalRisk,
    /// 系统性风险
    SystemicRisk,
}

/// 风险阈值结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RiskThresholds {
    /// 最大抵押品比率（基点）
    pub max_collateral_ratio_bps: u16,
    /// 最小抵押品比率（基点）
    pub min_collateral_ratio_bps: u16,
    /// 最大价格偏差（基点）
    pub max_price_deviation_bps: u16,
    /// 最大流动性风险（基点）
    pub max_liquidity_risk_bps: u16,
    /// 最大信用风险（基点）
    pub max_credit_risk_bps: u16,
}

/// 压力测试场景结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StressScenario {
    /// 场景名称
    pub scenario_name: String,
    /// 价格冲击（基点）
    pub price_shock_bps: i32,
    /// 流动性冲击（基点）
    pub liquidity_shock_bps: i32,
    /// 信用冲击（基点）
    pub credit_shock_bps: i32,
    /// 概率权重
    pub probability_weight: u16,
}

/// 风险评估订单结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RiskAssessmentOrder {
    /// 订单ID
    pub order_id: u64,
    /// 风险类型
    pub risk_type: RiskType,
    /// 评估周期
    pub assessment_period: u64,
    /// 时间戳
    pub timestamp: i64,
}

/// 风险评估结果结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RiskAssessmentResult {
    /// 风险评分（0-100）
    pub risk_score: u8,
    /// 风险等级
    pub risk_level: RiskLevel,
    /// 风险指标
    pub risk_metrics: RiskMetrics,
    /// 建议措施
    pub recommendations: Vec<String>,
    /// 评估时间戳
    pub assessment_timestamp: i64,
}

/// 风险等级枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum RiskLevel {
    /// 低风险
    Low,
    /// 中风险
    Medium,
    /// 高风险
    High,
    /// 极高风险
    Critical,
}

/// 风险指标结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RiskMetrics {
    /// 抵押品风险指标
    pub collateral_risk_metric: u16,
    /// 市场风险指标
    pub market_risk_metric: u16,
    /// 流动性风险指标
    pub liquidity_risk_metric: u16,
    /// 信用风险指标
    pub credit_risk_metric: u16,
    /// 综合风险指标
    pub composite_risk_metric: u16,
}

/// 稳定币风险评估指令账户上下文
/// 
/// 定义风险评估操作所需的所有账户，包括：
/// - stablecoin_asset: 稳定币资产账户（只读）
/// - authority: 操作权限账户（签名者）
/// - oracle_program: 预言机程序（用于价格验证）
/// - system_program: 系统程序
/// - clock: 时钟程序
#[derive(Accounts)]
#[instruction(params: RiskAssessmentStablecoinParams)]
pub struct RiskAssessmentStablecoin<'info> {
    /// 稳定币资产账户，只读权限
    #[account(
        seeds = [b"stablecoin", stablecoin_asset.key().as_ref()],
        bump,
        constraint = stablecoin_asset.asset_type == AssetType::Stablecoin @ crate::errors::asset_error::AssetError::InvalidAssetType
    )]
    pub stablecoin_asset: Account<'info, crate::state::baskets::BasketIndexState>,
    
    /// 操作权限账户，必须是签名者
    #[account(
        constraint = authority.key() == stablecoin_asset.authority @ crate::errors::security_error::SecurityError::InvalidAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 预言机程序，用于价格验证
    /// CHECK: 由预言机适配器验证
    pub oracle_program: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 时钟程序
    pub clock: Sysvar<'info, Clock>,
}

/// 稳定币风险评估指令实现
/// 
/// 执行风险评估操作，包括：
/// - 参数验证和权限检查
/// - 多维度风险指标计算
/// - 风险等级评估
/// - 事件记录和结果输出
pub fn risk_assessment_stablecoin(
    ctx: Context<RiskAssessmentStablecoin>,
    params: RiskAssessmentStablecoinParams
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_risk_assessment_params(&params)?;
    require!(params.assessment_period > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "risk_assessment"
    )?;
    
    // 3. 调用服务层执行风险评估
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.risk_assessment(
        stablecoin_asset,
        &params,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射风险评估事件
    emit!(AssetRiskAssessed {
        asset_id: stablecoin_asset.id,
        risk_type: params.risk_type.clone(),
        risk_score: result.risk_score,
        risk_level: result.risk_level.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin risk assessment executed successfully: asset_id={}, risk_type={:?}, risk_score={}, risk_level={:?}", 
         stablecoin_asset.id, params.risk_type, result.risk_score, result.risk_level);
    
    Ok(())
}

/// 抵押品风险评估指令实现
/// 
/// 执行抵押品风险评估操作
pub fn collateral_risk_assessment_stablecoin(
    ctx: Context<RiskAssessmentStablecoin>,
    params: RiskAssessmentStablecoinParams,
    collateral_quality_threshold: u16
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_risk_assessment_params(&params)?;
    require!(params.assessment_period > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(collateral_quality_threshold > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "collateral_risk_assessment"
    )?;
    
    // 3. 调用服务层执行抵押品风险评估
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.collateral_risk_assessment(
        stablecoin_asset,
        &params,
        collateral_quality_threshold,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射抵押品风险评估事件
    emit!(AssetRiskAssessed {
        asset_id: stablecoin_asset.id,
        risk_type: RiskType::CollateralRisk,
        risk_score: result.risk_score,
        risk_level: result.risk_level.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin collateral risk assessment executed successfully: asset_id={}, risk_score={}, risk_level={:?}, quality_threshold={}", 
         stablecoin_asset.id, result.risk_score, result.risk_level, collateral_quality_threshold);
    
    Ok(())
}

/// 市场风险评估指令实现
/// 
/// 执行市场风险评估操作
pub fn market_risk_assessment_stablecoin(
    ctx: Context<RiskAssessmentStablecoin>,
    params: RiskAssessmentStablecoinParams,
    volatility_threshold_bps: u16
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_risk_assessment_params(&params)?;
    require!(params.assessment_period > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(volatility_threshold_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "market_risk_assessment"
    )?;
    
    // 3. 调用服务层执行市场风险评估
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.market_risk_assessment(
        stablecoin_asset,
        &params,
        volatility_threshold_bps,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射市场风险评估事件
    emit!(AssetRiskAssessed {
        asset_id: stablecoin_asset.id,
        risk_type: RiskType::MarketRisk,
        risk_score: result.risk_score,
        risk_level: result.risk_level.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin market risk assessment executed successfully: asset_id={}, risk_score={}, risk_level={:?}, volatility_threshold_bps={}", 
         stablecoin_asset.id, result.risk_score, result.risk_level, volatility_threshold_bps);
    
    Ok(())
}

/// 流动性风险评估指令实现
/// 
/// 执行流动性风险评估操作
pub fn liquidity_risk_assessment_stablecoin(
    ctx: Context<RiskAssessmentStablecoin>,
    params: RiskAssessmentStablecoinParams,
    liquidity_threshold_bps: u64
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_risk_assessment_params(&params)?;
    require!(params.assessment_period > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(liquidity_threshold_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "liquidity_risk_assessment"
    )?;
    
    // 3. 调用服务层执行流动性风险评估
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.liquidity_risk_assessment(
        stablecoin_asset,
        &params,
        liquidity_threshold_bps,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射流动性风险评估事件
    emit!(AssetRiskAssessed {
        asset_id: stablecoin_asset.id,
        risk_type: RiskType::LiquidityRisk,
        risk_score: result.risk_score,
        risk_level: result.risk_level.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin liquidity risk assessment executed successfully: asset_id={}, risk_score={}, risk_level={:?}, liquidity_threshold_bps={}", 
         stablecoin_asset.id, result.risk_score, result.risk_level, liquidity_threshold_bps);
    
    Ok(())
} 