//! 稳定币 (Stablecoin) 锚定维护指令
//! 
//! 本模块实现稳定币资产的锚定维护功能，支持价格锚定、偏差检测、自动调整等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 价格锚定：保持与目标货币的锚定关系
//! - 偏差检测：监控价格偏差
//! - 自动调整：根据偏差自动调整
//! - 锚定机制：多种锚定策略
//! - 事件记录：完整的审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, PegParams};
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetPegMaintained;
use crate::validation::business::validate_peg_maintenance_params;
use crate::core::security::check_authority_permission;
use crate::dex::traits::DexAdapterTrait;
use crate::oracles::traits::OracleAdapterTrait;

/// 稳定币锚定维护参数结构体
/// 
/// 定义锚定维护操作所需的所有参数，包括：
/// - target_peg: 目标锚定价格
/// - deviation_threshold: 偏差阈值
/// - adjustment_mechanism: 调整机制
/// - peg_type: 锚定类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PegMaintenanceStablecoinParams {
    /// 目标锚定价格（基点，1/10000）
    pub target_peg_bps: u64,
    /// 偏差阈值（基点，1/10000）
    pub deviation_threshold_bps: u16,
    /// 调整机制
    pub adjustment_mechanism: PegAdjustmentMechanism,
    /// 锚定类型
    pub peg_type: PegType,
    /// 锚定参数
    pub peg_params: Option<PegParams>,
    /// 执行参数（可选）
    pub exec_params: Option<ExecutionParams>,
    /// 策略参数（可选）
    pub strategy_params: Option<StrategyParams>,
}

/// 锚定类型枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum PegType {
    /// 软锚定
    SoftPeg,
    /// 硬锚定
    HardPeg,
    /// 爬行锚定
    CrawlingPeg,
    /// 篮子锚定
    BasketPeg,
    /// 算法锚定
    AlgorithmicPeg,
}

/// 锚定调整机制枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum PegAdjustmentMechanism {
    /// 市场操作
    MarketOperation,
    /// 利率调整
    InterestRateAdjustment,
    /// 供应调整
    SupplyAdjustment,
    /// 算法调整
    AlgorithmicAdjustment,
    /// 混合调整
    HybridAdjustment,
}

/// 锚定维护订单结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PegMaintenanceOrder {
    /// 订单ID
    pub order_id: u64,
    /// 锚定类型
    pub peg_type: PegType,
    /// 目标锚定价格
    pub target_peg_bps: u64,
    /// 偏差阈值
    pub deviation_threshold_bps: u16,
    /// 时间戳
    pub timestamp: i64,
}

/// 锚定维护结果结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PegMaintenanceResult {
    /// 当前价格（基点）
    pub current_price_bps: u64,
    /// 目标价格（基点）
    pub target_price_bps: u64,
    /// 价格偏差（基点）
    pub price_deviation_bps: i32,
    /// 调整数量
    pub adjustment_amount: u64,
    /// 调整方向
    pub adjustment_direction: AdjustmentDirection,
    /// 成功标志
    pub success: bool,
}

/// 调整方向枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum AdjustmentDirection {
    /// 向上调整
    Up,
    /// 向下调整
    Down,
    /// 无调整
    None,
}

/// 稳定币锚定维护指令账户上下文
/// 
/// 定义锚定维护操作所需的所有账户，包括：
/// - stablecoin_asset: 稳定币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - oracle_program: 预言机程序（用于价格验证）
/// - dex_program: DEX程序（用于市场操作）
/// - system_program: 系统程序
/// - clock: 时钟程序
#[derive(Accounts)]
#[instruction(params: PegMaintenanceStablecoinParams)]
pub struct PegMaintenanceStablecoin<'info> {
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
    
    /// DEX程序，用于市场操作
    /// CHECK: 由DEX适配器验证
    pub dex_program: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 时钟程序
    pub clock: Sysvar<'info, Clock>,
}

/// 稳定币锚定维护指令实现
/// 
/// 执行锚定维护操作，包括：
/// - 参数验证和权限检查
/// - 价格偏差检测
/// - 锚定调整执行
/// - 事件记录和状态更新
pub fn peg_maintenance_stablecoin(
    ctx: Context<PegMaintenanceStablecoin>,
    params: PegMaintenanceStablecoinParams
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_peg_maintenance_params(&params)?;
    require!(params.target_peg_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(params.deviation_threshold_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "peg_maintenance"
    )?;
    
    // 3. 调用服务层执行锚定维护
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.peg_maintenance(
        stablecoin_asset,
        &params,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射锚定维护事件
    emit!(AssetPegMaintained {
        asset_id: stablecoin_asset.id,
        peg_type: params.peg_type.clone(),
        target_peg_bps: params.target_peg_bps,
        current_price_bps: result.current_price_bps,
        price_deviation_bps: result.price_deviation_bps,
        adjustment_amount: result.adjustment_amount,
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin peg maintenance executed successfully: asset_id={}, peg_type={:?}, target_peg_bps={}, deviation_bps={}", 
         stablecoin_asset.id, params.peg_type, params.target_peg_bps, params.deviation_threshold_bps);
    
    Ok(())
}

/// 软锚定维护指令实现
/// 
/// 执行软锚定维护操作
pub fn soft_peg_maintenance_stablecoin(
    ctx: Context<PegMaintenanceStablecoin>,
    params: PegMaintenanceStablecoinParams,
    flexibility_factor: u16
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_peg_maintenance_params(&params)?;
    require!(params.target_peg_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(flexibility_factor > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "soft_peg_maintenance"
    )?;
    
    // 3. 调用服务层执行软锚定维护
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.soft_peg_maintenance(
        stablecoin_asset,
        &params,
        flexibility_factor,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射软锚定维护事件
    emit!(AssetPegMaintained {
        asset_id: stablecoin_asset.id,
        peg_type: PegType::SoftPeg,
        target_peg_bps: params.target_peg_bps,
        current_price_bps: result.current_price_bps,
        price_deviation_bps: result.price_deviation_bps,
        adjustment_amount: result.adjustment_amount,
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin soft peg maintenance executed successfully: asset_id={}, target_peg_bps={}, flexibility_factor={}", 
         stablecoin_asset.id, params.target_peg_bps, flexibility_factor);
    
    Ok(())
}

/// 硬锚定维护指令实现
/// 
/// 执行硬锚定维护操作
pub fn hard_peg_maintenance_stablecoin(
    ctx: Context<PegMaintenanceStablecoin>,
    params: PegMaintenanceStablecoinParams,
    strict_threshold_bps: u16
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_peg_maintenance_params(&params)?;
    require!(params.target_peg_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(strict_threshold_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "hard_peg_maintenance"
    )?;
    
    // 3. 调用服务层执行硬锚定维护
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.hard_peg_maintenance(
        stablecoin_asset,
        &params,
        strict_threshold_bps,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射硬锚定维护事件
    emit!(AssetPegMaintained {
        asset_id: stablecoin_asset.id,
        peg_type: PegType::HardPeg,
        target_peg_bps: params.target_peg_bps,
        current_price_bps: result.current_price_bps,
        price_deviation_bps: result.price_deviation_bps,
        adjustment_amount: result.adjustment_amount,
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin hard peg maintenance executed successfully: asset_id={}, target_peg_bps={}, strict_threshold_bps={}", 
         stablecoin_asset.id, params.target_peg_bps, strict_threshold_bps);
    
    Ok(())
}

/// 算法锚定维护指令实现
/// 
/// 执行算法锚定维护操作
pub fn algorithmic_peg_maintenance_stablecoin(
    ctx: Context<PegMaintenanceStablecoin>,
    params: PegMaintenanceStablecoinParams,
    algorithm_params: Vec<u8>
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_peg_maintenance_params(&params)?;
    require!(params.target_peg_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(!algorithm_params.is_empty(), crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "algorithmic_peg_maintenance"
    )?;
    
    // 3. 调用服务层执行算法锚定维护
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.algorithmic_peg_maintenance(
        stablecoin_asset,
        &params,
        algorithm_params,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射算法锚定维护事件
    emit!(AssetPegMaintained {
        asset_id: stablecoin_asset.id,
        peg_type: PegType::AlgorithmicPeg,
        target_peg_bps: params.target_peg_bps,
        current_price_bps: result.current_price_bps,
        price_deviation_bps: result.price_deviation_bps,
        adjustment_amount: result.adjustment_amount,
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin algorithmic peg maintenance executed successfully: asset_id={}, target_peg_bps={}, algorithm_params_len={}", 
         stablecoin_asset.id, params.target_peg_bps, algorithm_params.len());
    
    Ok(())
} 