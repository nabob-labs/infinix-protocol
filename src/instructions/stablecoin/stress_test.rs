//! 稳定币 (Stablecoin) 压力测试指令
//! 
//! 本模块实现稳定币资产的压力测试功能，支持场景测试、风险模拟、压力评估等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 场景测试：多种压力场景模拟
//! - 风险模拟：风险事件模拟
//! - 压力评估：压力下的表现评估
//! - 压力报告：详细的压力测试报告
//! - 事件记录：完整的审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, StressTestParams};
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetStressTested;
use crate::validation::business::validate_stress_test_params;
use crate::core::security::check_authority_permission;
use crate::dex::traits::DexAdapterTrait;
use crate::oracles::traits::OracleAdapterTrait;

/// 稳定币压力测试参数结构体
/// 
/// 定义压力测试操作所需的所有参数，包括：
/// - stress_scenario: 压力场景
/// - test_duration: 测试持续时间
/// - stress_levels: 压力水平
/// - risk_factors: 风险因子
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StressTestStablecoinParams {
    /// 压力场景
    pub stress_scenario: StressScenario,
    /// 测试持续时间（秒）
    pub test_duration: u64,
    /// 压力水平
    pub stress_levels: StressLevels,
    /// 风险因子
    pub risk_factors: Vec<RiskFactor>,
    /// 压力测试参数
    pub stress_test_params: Option<StressTestParams>,
    /// 执行参数（可选）
    pub exec_params: Option<ExecutionParams>,
    /// 策略参数（可选）
    pub strategy_params: Option<StrategyParams>,
}

/// 压力场景枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum StressScenario {
    /// 市场崩盘
    MarketCrash,
    /// 流动性枯竭
    LiquidityCrisis,
    /// 利率冲击
    InterestRateShock,
    /// 信用违约
    CreditDefault,
    /// 系统性风险
    SystemicRisk,
    /// 黑天鹅事件
    BlackSwan,
}

/// 压力水平结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StressLevels {
    /// 轻度压力
    pub mild_stress: u16,
    /// 中度压力
    pub moderate_stress: u16,
    /// 重度压力
    pub severe_stress: u16,
    /// 极端压力
    pub extreme_stress: u16,
}

/// 风险因子结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RiskFactor {
    /// 风险因子名称
    pub factor_name: String,
    /// 风险因子权重
    pub factor_weight: u16,
    /// 风险因子冲击
    pub factor_shock: i32,
    /// 风险因子类型
    pub factor_type: RiskFactorType,
}

/// 风险因子类型枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum RiskFactorType {
    /// 价格风险
    PriceRisk,
    /// 流动性风险
    LiquidityRisk,
    /// 信用风险
    CreditRisk,
    /// 操作风险
    OperationalRisk,
    /// 市场风险
    MarketRisk,
}

/// 压力测试订单结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StressTestOrder {
    /// 订单ID
    pub order_id: u64,
    /// 压力场景
    pub stress_scenario: StressScenario,
    /// 测试持续时间
    pub test_duration: u64,
    /// 时间戳
    pub timestamp: i64,
}

/// 压力测试结果结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StressTestResult {
    /// 测试场景
    pub test_scenario: StressScenario,
    /// 压力评分（0-100）
    pub stress_score: u8,
    /// 风险等级
    pub risk_level: StressRiskLevel,
    /// 压力指标
    pub stress_metrics: StressMetrics,
    /// 建议措施
    pub recommendations: Vec<String>,
    /// 测试时间戳
    pub test_timestamp: i64,
}

/// 压力风险等级枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum StressRiskLevel {
    /// 低风险
    Low,
    /// 中风险
    Medium,
    /// 高风险
    High,
    /// 极高风险
    Critical,
}

/// 压力指标结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StressMetrics {
    /// 价格压力指标
    pub price_stress_metric: u16,
    /// 流动性压力指标
    pub liquidity_stress_metric: u16,
    /// 信用压力指标
    pub credit_stress_metric: u16,
    /// 系统性压力指标
    pub systemic_stress_metric: u16,
    /// 综合压力指标
    pub composite_stress_metric: u16,
}

/// 稳定币压力测试指令账户上下文
/// 
/// 定义压力测试操作所需的所有账户，包括：
/// - stablecoin_asset: 稳定币资产账户（只读）
/// - authority: 操作权限账户（签名者）
/// - oracle_program: 预言机程序（用于价格验证）
/// - system_program: 系统程序
/// - clock: 时钟程序
#[derive(Accounts)]
#[instruction(params: StressTestStablecoinParams)]
pub struct StressTestStablecoin<'info> {
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

/// 稳定币压力测试指令实现
/// 
/// 执行压力测试操作，包括：
/// - 参数验证和权限检查
/// - 压力场景模拟
/// - 风险因子分析
/// - 事件记录和结果输出
pub fn stress_test_stablecoin(
    ctx: Context<StressTestStablecoin>,
    params: StressTestStablecoinParams
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_stress_test_params(&params)?;
    require!(params.test_duration > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(!params.risk_factors.is_empty(), crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "stress_test"
    )?;
    
    // 3. 调用服务层执行压力测试
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.stress_test(
        stablecoin_asset,
        &params,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射压力测试事件
    emit!(AssetStressTested {
        asset_id: stablecoin_asset.id,
        stress_scenario: params.stress_scenario.clone(),
        stress_score: result.stress_score,
        risk_level: result.risk_level.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin stress test executed successfully: asset_id={}, stress_scenario={:?}, stress_score={}, risk_level={:?}", 
         stablecoin_asset.id, params.stress_scenario, result.stress_score, result.risk_level);
    
    Ok(())
}

/// 市场崩盘压力测试指令实现
/// 
/// 执行市场崩盘压力测试操作
pub fn market_crash_stress_test_stablecoin(
    ctx: Context<StressTestStablecoin>,
    params: StressTestStablecoinParams,
    market_crash_severity_bps: u16
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_stress_test_params(&params)?;
    require!(params.test_duration > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(market_crash_severity_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "market_crash_stress_test"
    )?;
    
    // 3. 调用服务层执行市场崩盘压力测试
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.market_crash_stress_test(
        stablecoin_asset,
        &params,
        market_crash_severity_bps,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射市场崩盘压力测试事件
    emit!(AssetStressTested {
        asset_id: stablecoin_asset.id,
        stress_scenario: StressScenario::MarketCrash,
        stress_score: result.stress_score,
        risk_level: result.risk_level.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin market crash stress test executed successfully: asset_id={}, stress_score={}, risk_level={:?}, crash_severity_bps={}", 
         stablecoin_asset.id, result.stress_score, result.risk_level, market_crash_severity_bps);
    
    Ok(())
}

/// 流动性危机压力测试指令实现
/// 
/// 执行流动性危机压力测试操作
pub fn liquidity_crisis_stress_test_stablecoin(
    ctx: Context<StressTestStablecoin>,
    params: StressTestStablecoinParams,
    liquidity_drain_rate_bps: u16
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_stress_test_params(&params)?;
    require!(params.test_duration > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(liquidity_drain_rate_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "liquidity_crisis_stress_test"
    )?;
    
    // 3. 调用服务层执行流动性危机压力测试
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.liquidity_crisis_stress_test(
        stablecoin_asset,
        &params,
        liquidity_drain_rate_bps,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射流动性危机压力测试事件
    emit!(AssetStressTested {
        asset_id: stablecoin_asset.id,
        stress_scenario: StressScenario::LiquidityCrisis,
        stress_score: result.stress_score,
        risk_level: result.risk_level.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin liquidity crisis stress test executed successfully: asset_id={}, stress_score={}, risk_level={:?}, drain_rate_bps={}", 
         stablecoin_asset.id, result.stress_score, result.risk_level, liquidity_drain_rate_bps);
    
    Ok(())
}

/// 系统性风险压力测试指令实现
/// 
/// 执行系统性风险压力测试操作
pub fn systemic_risk_stress_test_stablecoin(
    ctx: Context<StressTestStablecoin>,
    params: StressTestStablecoinParams,
    systemic_risk_factors: Vec<RiskFactor>
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_stress_test_params(&params)?;
    require!(params.test_duration > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(!systemic_risk_factors.is_empty(), crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "systemic_risk_stress_test"
    )?;
    
    // 3. 调用服务层执行系统性风险压力测试
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.systemic_risk_stress_test(
        stablecoin_asset,
        &params,
        systemic_risk_factors,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射系统性风险压力测试事件
    emit!(AssetStressTested {
        asset_id: stablecoin_asset.id,
        stress_scenario: StressScenario::SystemicRisk,
        stress_score: result.stress_score,
        risk_level: result.risk_level.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin systemic risk stress test executed successfully: asset_id={}, stress_score={}, risk_level={:?}, risk_factors_count={}", 
         stablecoin_asset.id, result.stress_score, result.risk_level, systemic_risk_factors.len());
    
    Ok(())
} 