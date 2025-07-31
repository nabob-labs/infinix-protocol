//! 稳定币 (Stablecoin) 熔断机制指令
//! 
//! 本模块实现稳定币资产的熔断机制功能，支持价格熔断、交易熔断、自动恢复等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 价格熔断：价格异常时的熔断机制
//! - 交易熔断：交易异常时的熔断机制
//! - 自动恢复：熔断后的自动恢复
//! - 熔断级别：多级熔断机制
//! - 事件记录：完整的审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, CircuitBreakerParams};
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetCircuitBreakerTriggered;
use crate::validation::business::validate_circuit_breaker_params;
use crate::core::security::check_authority_permission;
use crate::dex::traits::DexAdapterTrait;
use crate::oracles::traits::OracleAdapterTrait;

/// 稳定币熔断机制参数结构体
/// 
/// 定义熔断机制操作所需的所有参数，包括：
/// - circuit_breaker_type: 熔断类型
/// - trigger_threshold: 触发阈值
/// - recovery_time: 恢复时间
/// - breaker_level: 熔断级别
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CircuitBreakerStablecoinParams {
    /// 熔断类型
    pub circuit_breaker_type: CircuitBreakerType,
    /// 触发阈值（基点，1/10000）
    pub trigger_threshold_bps: u16,
    /// 恢复时间（秒）
    pub recovery_time: u64,
    /// 熔断级别
    pub breaker_level: BreakerLevel,
    /// 熔断参数
    pub circuit_breaker_params: Option<CircuitBreakerParams>,
    /// 执行参数（可选）
    pub exec_params: Option<ExecutionParams>,
    /// 策略参数（可选）
    pub strategy_params: Option<StrategyParams>,
}

/// 熔断类型枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum CircuitBreakerType {
    /// 价格熔断
    PriceBreaker,
    /// 交易熔断
    TradingBreaker,
    /// 流动性熔断
    LiquidityBreaker,
    /// 波动率熔断
    VolatilityBreaker,
    /// 系统性熔断
    SystemicBreaker,
}

/// 熔断级别枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum BreakerLevel {
    /// 一级熔断
    Level1,
    /// 二级熔断
    Level2,
    /// 三级熔断
    Level3,
    /// 完全熔断
    FullBreaker,
}

/// 熔断机制订单结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CircuitBreakerOrder {
    /// 订单ID
    pub order_id: u64,
    /// 熔断类型
    pub circuit_breaker_type: CircuitBreakerType,
    /// 触发阈值
    pub trigger_threshold_bps: u16,
    /// 恢复时间
    pub recovery_time: u64,
    /// 时间戳
    pub timestamp: i64,
}

/// 熔断机制结果结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CircuitBreakerResult {
    /// 熔断状态
    pub breaker_status: BreakerStatus,
    /// 触发时间
    pub trigger_time: i64,
    /// 恢复时间
    pub recovery_time: i64,
    /// 触发原因
    pub trigger_reason: String,
    /// 影响范围
    pub affected_operations: Vec<String>,
    /// 成功标志
    pub success: bool,
}

/// 熔断状态枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum BreakerStatus {
    /// 正常
    Normal,
    /// 已触发
    Triggered,
    /// 恢复中
    Recovering,
    /// 已恢复
    Recovered,
    /// 失败
    Failed,
}

/// 稳定币熔断机制指令账户上下文
/// 
/// 定义熔断机制操作所需的所有账户，包括：
/// - stablecoin_asset: 稳定币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - oracle_program: 预言机程序（用于价格验证）
/// - system_program: 系统程序
/// - clock: 时钟程序
#[derive(Accounts)]
#[instruction(params: CircuitBreakerStablecoinParams)]
pub struct CircuitBreakerStablecoin<'info> {
    /// 稳定币资产账户，需要可变权限以更新状态
    #[account(
        mut,
        seeds = [b"stablecoin", stablecoin_asset.key().as_ref()],
        bump,
        constraint = stablecoin_asset.asset_type == AssetType::Stablecoin @ crate::errors::asset_error::AssetError::InvalidAssetType,
        constraint = !stablecoin_asset.is_frozen @ crate::errors::asset_error::AssetError::AssetFrozen
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

/// 稳定币熔断机制指令实现
/// 
/// 执行熔断机制操作，包括：
/// - 参数验证和权限检查
/// - 熔断条件检查
/// - 熔断执行
/// - 事件记录和状态更新
pub fn circuit_breaker_stablecoin(
    ctx: Context<CircuitBreakerStablecoin>,
    params: CircuitBreakerStablecoinParams
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_circuit_breaker_params(&params)?;
    require!(params.trigger_threshold_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(params.recovery_time > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "circuit_breaker"
    )?;
    
    // 3. 调用服务层执行熔断机制
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.circuit_breaker(
        stablecoin_asset,
        &params,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射熔断机制事件
    emit!(AssetCircuitBreakerTriggered {
        asset_id: stablecoin_asset.id,
        circuit_breaker_type: params.circuit_breaker_type.clone(),
        trigger_threshold_bps: params.trigger_threshold_bps,
        recovery_time: params.recovery_time,
        breaker_level: params.breaker_level.clone(),
        breaker_status: result.breaker_status.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin circuit breaker executed successfully: asset_id={}, breaker_type={:?}, trigger_threshold_bps={}, breaker_level={:?}", 
         stablecoin_asset.id, params.circuit_breaker_type, params.trigger_threshold_bps, params.breaker_level);
    
    Ok(())
}

/// 价格熔断指令实现
/// 
/// 执行价格熔断操作
pub fn price_circuit_breaker_stablecoin(
    ctx: Context<CircuitBreakerStablecoin>,
    params: CircuitBreakerStablecoinParams,
    price_deviation_threshold_bps: u16
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_circuit_breaker_params(&params)?;
    require!(price_deviation_threshold_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "price_circuit_breaker"
    )?;
    
    // 3. 调用服务层执行价格熔断
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.price_circuit_breaker(
        stablecoin_asset,
        &params,
        price_deviation_threshold_bps,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射价格熔断事件
    emit!(AssetCircuitBreakerTriggered {
        asset_id: stablecoin_asset.id,
        circuit_breaker_type: CircuitBreakerType::PriceBreaker,
        trigger_threshold_bps: params.trigger_threshold_bps,
        recovery_time: params.recovery_time,
        breaker_level: params.breaker_level.clone(),
        breaker_status: result.breaker_status.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin price circuit breaker executed successfully: asset_id={}, price_deviation_threshold_bps={}", 
         stablecoin_asset.id, price_deviation_threshold_bps);
    
    Ok(())
}

/// 交易熔断指令实现
/// 
/// 执行交易熔断操作
pub fn trading_circuit_breaker_stablecoin(
    ctx: Context<CircuitBreakerStablecoin>,
    params: CircuitBreakerStablecoinParams,
    trading_volume_threshold: u64
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_circuit_breaker_params(&params)?;
    require!(trading_volume_threshold > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "trading_circuit_breaker"
    )?;
    
    // 3. 调用服务层执行交易熔断
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.trading_circuit_breaker(
        stablecoin_asset,
        &params,
        trading_volume_threshold,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射交易熔断事件
    emit!(AssetCircuitBreakerTriggered {
        asset_id: stablecoin_asset.id,
        circuit_breaker_type: CircuitBreakerType::TradingBreaker,
        trigger_threshold_bps: params.trigger_threshold_bps,
        recovery_time: params.recovery_time,
        breaker_level: params.breaker_level.clone(),
        breaker_status: result.breaker_status.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin trading circuit breaker executed successfully: asset_id={}, trading_volume_threshold={}", 
         stablecoin_asset.id, trading_volume_threshold);
    
    Ok(())
}

/// 波动率熔断指令实现
/// 
/// 执行波动率熔断操作
pub fn volatility_circuit_breaker_stablecoin(
    ctx: Context<CircuitBreakerStablecoin>,
    params: CircuitBreakerStablecoinParams,
    volatility_threshold_bps: u16
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_circuit_breaker_params(&params)?;
    require!(volatility_threshold_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "volatility_circuit_breaker"
    )?;
    
    // 3. 调用服务层执行波动率熔断
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.volatility_circuit_breaker(
        stablecoin_asset,
        &params,
        volatility_threshold_bps,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射波动率熔断事件
    emit!(AssetCircuitBreakerTriggered {
        asset_id: stablecoin_asset.id,
        circuit_breaker_type: CircuitBreakerType::VolatilityBreaker,
        trigger_threshold_bps: params.trigger_threshold_bps,
        recovery_time: params.recovery_time,
        breaker_level: params.breaker_level.clone(),
        breaker_status: result.breaker_status.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin volatility circuit breaker executed successfully: asset_id={}, volatility_threshold_bps={}", 
         stablecoin_asset.id, volatility_threshold_bps);
    
    Ok(())
} 