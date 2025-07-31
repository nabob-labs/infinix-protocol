//! 稳定币 (Stablecoin) 紧急暂停指令
//! 
//! 本模块实现稳定币资产的紧急暂停功能，支持紧急暂停、恢复、保护机制等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 紧急暂停：紧急情况下的暂停机制
//! - 暂停恢复：暂停后的恢复机制
//! - 保护机制：暂停期间的资产保护
//! - 权限控制：紧急权限管理
//! - 事件记录：完整的审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, EmergencyParams};
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetEmergencyPaused;
use crate::validation::business::validate_emergency_pause_params;
use crate::core::security::check_authority_permission;
use crate::dex::traits::DexAdapterTrait;
use crate::oracles::traits::OracleAdapterTrait;

/// 稳定币紧急暂停参数结构体
/// 
/// 定义紧急暂停操作所需的所有参数，包括：
/// - emergency_type: 紧急类型
/// - pause_reason: 暂停原因
/// - pause_duration: 暂停持续时间
/// - protection_level: 保护级别
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EmergencyPauseStablecoinParams {
    /// 紧急类型
    pub emergency_type: EmergencyType,
    /// 暂停原因
    pub pause_reason: String,
    /// 暂停持续时间（秒）
    pub pause_duration: u64,
    /// 保护级别
    pub protection_level: ProtectionLevel,
    /// 紧急参数
    pub emergency_params: Option<EmergencyParams>,
    /// 执行参数（可选）
    pub exec_params: Option<ExecutionParams>,
    /// 策略参数（可选）
    pub strategy_params: Option<StrategyParams>,
}

/// 紧急类型枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum EmergencyType {
    /// 市场异常
    MarketAnomaly,
    /// 流动性危机
    LiquidityCrisis,
    /// 安全漏洞
    SecurityBreach,
    /// 监管要求
    RegulatoryRequirement,
    /// 技术故障
    TechnicalFailure,
    /// 系统性风险
    SystemicRisk,
}

/// 保护级别枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum ProtectionLevel {
    /// 轻度保护
    Light,
    /// 中度保护
    Medium,
    /// 重度保护
    Heavy,
    /// 完全保护
    Full,
}

/// 紧急暂停订单结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EmergencyPauseOrder {
    /// 订单ID
    pub order_id: u64,
    /// 紧急类型
    pub emergency_type: EmergencyType,
    /// 暂停原因
    pub pause_reason: String,
    /// 暂停持续时间
    pub pause_duration: u64,
    /// 时间戳
    pub timestamp: i64,
}

/// 紧急暂停结果结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EmergencyPauseResult {
    /// 暂停状态
    pub pause_status: PauseStatus,
    /// 暂停开始时间
    pub pause_start_time: i64,
    /// 暂停结束时间
    pub pause_end_time: i64,
    /// 保护措施
    pub protection_measures: Vec<String>,
    /// 影响范围
    pub affected_operations: Vec<String>,
    /// 成功标志
    pub success: bool,
}

/// 暂停状态枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum PauseStatus {
    /// 活跃
    Active,
    /// 暂停中
    Paused,
    /// 恢复中
    Resuming,
    /// 已恢复
    Resumed,
    /// 失败
    Failed,
}

/// 稳定币紧急暂停指令账户上下文
/// 
/// 定义紧急暂停操作所需的所有账户，包括：
/// - stablecoin_asset: 稳定币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - emergency_authority: 紧急权限账户（签名者）
/// - system_program: 系统程序
/// - clock: 时钟程序
#[derive(Accounts)]
#[instruction(params: EmergencyPauseStablecoinParams)]
pub struct EmergencyPauseStablecoin<'info> {
    /// 稳定币资产账户，需要可变权限以更新状态
    #[account(
        mut,
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
    
    /// 紧急权限账户，必须是签名者
    pub emergency_authority: Signer<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 时钟程序
    pub clock: Sysvar<'info, Clock>,
}

/// 稳定币紧急暂停指令实现
/// 
/// 执行紧急暂停操作，包括：
/// - 参数验证和权限检查
/// - 暂停条件检查
/// - 暂停执行
/// - 事件记录和状态更新
pub fn emergency_pause_stablecoin(
    ctx: Context<EmergencyPauseStablecoin>,
    params: EmergencyPauseStablecoinParams
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_emergency_pause_params(&params)?;
    require!(!params.pause_reason.is_empty(), crate::errors::asset_error::AssetError::InvalidParams);
    require!(params.pause_duration > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "emergency_pause"
    )?;
    
    // 3. 调用服务层执行紧急暂停
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.emergency_pause(
        stablecoin_asset,
        &params,
        ctx.accounts.emergency_authority.key(),
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射紧急暂停事件
    emit!(AssetEmergencyPaused {
        asset_id: stablecoin_asset.id,
        emergency_type: params.emergency_type.clone(),
        pause_reason: params.pause_reason.clone(),
        pause_duration: params.pause_duration,
        protection_level: params.protection_level.clone(),
        emergency_authority: ctx.accounts.emergency_authority.key(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin emergency pause executed successfully: asset_id={}, emergency_type={:?}, pause_reason={}, protection_level={:?}", 
         stablecoin_asset.id, params.emergency_type, params.pause_reason, params.protection_level);
    
    Ok(())
}

/// 紧急恢复指令实现
/// 
/// 执行紧急恢复操作
pub fn emergency_resume_stablecoin(
    ctx: Context<EmergencyPauseStablecoin>,
    params: EmergencyPauseStablecoinParams,
    resume_conditions: Vec<String>
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_emergency_pause_params(&params)?;
    require!(!resume_conditions.is_empty(), crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "emergency_resume"
    )?;
    
    // 3. 调用服务层执行紧急恢复
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.emergency_resume(
        stablecoin_asset,
        &params,
        resume_conditions,
        ctx.accounts.emergency_authority.key(),
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射紧急恢复事件
    emit!(AssetEmergencyPaused {
        asset_id: stablecoin_asset.id,
        emergency_type: params.emergency_type.clone(),
        pause_reason: "Emergency Resume".to_string(),
        pause_duration: 0,
        protection_level: params.protection_level.clone(),
        emergency_authority: ctx.accounts.emergency_authority.key(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin emergency resume executed successfully: asset_id={}, resume_conditions_count={}", 
         stablecoin_asset.id, resume_conditions.len());
    
    Ok(())
}

/// 市场异常紧急暂停指令实现
/// 
/// 执行市场异常紧急暂停操作
pub fn market_anomaly_emergency_pause_stablecoin(
    ctx: Context<EmergencyPauseStablecoin>,
    params: EmergencyPauseStablecoinParams,
    anomaly_threshold_bps: u16
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_emergency_pause_params(&params)?;
    require!(anomaly_threshold_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "market_anomaly_emergency_pause"
    )?;
    
    // 3. 调用服务层执行市场异常紧急暂停
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.market_anomaly_emergency_pause(
        stablecoin_asset,
        &params,
        anomaly_threshold_bps,
        ctx.accounts.emergency_authority.key(),
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射市场异常紧急暂停事件
    emit!(AssetEmergencyPaused {
        asset_id: stablecoin_asset.id,
        emergency_type: EmergencyType::MarketAnomaly,
        pause_reason: params.pause_reason.clone(),
        pause_duration: params.pause_duration,
        protection_level: params.protection_level.clone(),
        emergency_authority: ctx.accounts.emergency_authority.key(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin market anomaly emergency pause executed successfully: asset_id={}, anomaly_threshold_bps={}", 
         stablecoin_asset.id, anomaly_threshold_bps);
    
    Ok(())
}

/// 安全漏洞紧急暂停指令实现
/// 
/// 执行安全漏洞紧急暂停操作
pub fn security_breach_emergency_pause_stablecoin(
    ctx: Context<EmergencyPauseStablecoin>,
    params: EmergencyPauseStablecoinParams,
    security_breach_level: u8
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_emergency_pause_params(&params)?;
    require!(security_breach_level > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "security_breach_emergency_pause"
    )?;
    
    // 3. 调用服务层执行安全漏洞紧急暂停
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.security_breach_emergency_pause(
        stablecoin_asset,
        &params,
        security_breach_level,
        ctx.accounts.emergency_authority.key(),
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射安全漏洞紧急暂停事件
    emit!(AssetEmergencyPaused {
        asset_id: stablecoin_asset.id,
        emergency_type: EmergencyType::SecurityBreach,
        pause_reason: params.pause_reason.clone(),
        pause_duration: params.pause_duration,
        protection_level: params.protection_level.clone(),
        emergency_authority: ctx.accounts.emergency_authority.key(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin security breach emergency pause executed successfully: asset_id={}, security_breach_level={}", 
         stablecoin_asset.id, security_breach_level);
    
    Ok(())
} 