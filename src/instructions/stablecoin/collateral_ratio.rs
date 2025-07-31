//! 稳定币 (Stablecoin) 抵押率管理指令
//! 
//! 本模块实现稳定币资产的抵押率管理功能，支持抵押品管理、比率调整、清算触发等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 抵押品管理：抵押品的添加、移除和评估
//! - 比率调整：动态调整抵押率要求
//! - 清算触发：自动清算机制
//! - 风险监控：抵押率风险监控
//! - 事件记录：完整的审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, CollateralParams};
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetCollateralRatioUpdated;
use crate::validation::business::validate_collateral_ratio_params;
use crate::core::security::check_authority_permission;
use crate::dex::traits::DexAdapterTrait;
use crate::oracles::traits::OracleAdapterTrait;

/// 稳定币抵押率管理参数结构体
/// 
/// 定义抵押率管理操作所需的所有参数，包括：
/// - collateral_ratio: 抵押率要求
/// - collateral_assets: 抵押品资产
/// - risk_thresholds: 风险阈值
/// - liquidation_params: 清算参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CollateralRatioStablecoinParams {
    /// 目标抵押率（基点，1/10000）
    pub target_collateral_ratio_bps: u16,
    /// 最小抵押率（基点，1/10000）
    pub min_collateral_ratio_bps: u16,
    /// 清算阈值（基点，1/10000）
    pub liquidation_threshold_bps: u16,
    /// 抵押品资产列表
    pub collateral_assets: Vec<CollateralAsset>,
    /// 抵押品参数
    pub collateral_params: Option<CollateralParams>,
    /// 执行参数（可选）
    pub exec_params: Option<ExecutionParams>,
    /// 策略参数（可选）
    pub strategy_params: Option<StrategyParams>,
}

/// 抵押品资产结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CollateralAsset {
    /// 资产地址
    pub asset_address: Pubkey,
    /// 资产类型
    pub asset_type: AssetType,
    /// 抵押品价值
    pub collateral_value: u64,
    /// 抵押品权重（基点）
    pub collateral_weight_bps: u16,
    /// 风险等级
    pub risk_level: CollateralRiskLevel,
}

/// 抵押品风险等级枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum CollateralRiskLevel {
    /// 低风险
    Low,
    /// 中风险
    Medium,
    /// 高风险
    High,
    /// 极高风险
    Critical,
}

/// 抵押率管理订单结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CollateralRatioOrder {
    /// 订单ID
    pub order_id: u64,
    /// 目标抵押率
    pub target_collateral_ratio_bps: u16,
    /// 最小抵押率
    pub min_collateral_ratio_bps: u16,
    /// 清算阈值
    pub liquidation_threshold_bps: u16,
    /// 时间戳
    pub timestamp: i64,
}

/// 抵押率管理结果结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CollateralRatioResult {
    /// 当前抵押率（基点）
    pub current_collateral_ratio_bps: u16,
    /// 目标抵押率（基点）
    pub target_collateral_ratio_bps: u16,
    /// 抵押率偏差（基点）
    pub collateral_ratio_deviation_bps: i32,
    /// 抵押品总价值
    pub total_collateral_value: u64,
    /// 债务总额
    pub total_debt: u64,
    /// 风险状态
    pub risk_status: CollateralRiskStatus,
    /// 成功标志
    pub success: bool,
}

/// 抵押品风险状态枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum CollateralRiskStatus {
    /// 安全
    Safe,
    /// 警告
    Warning,
    /// 危险
    Danger,
    /// 清算
    Liquidation,
}

/// 稳定币抵押率管理指令账户上下文
/// 
/// 定义抵押率管理操作所需的所有账户，包括：
/// - stablecoin_asset: 稳定币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - collateral_assets: 抵押品资产账户（可变）
/// - oracle_program: 预言机程序（用于价格验证）
/// - system_program: 系统程序
/// - clock: 时钟程序
#[derive(Accounts)]
#[instruction(params: CollateralRatioStablecoinParams)]
pub struct CollateralRatioStablecoin<'info> {
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
    
    /// 抵押品资产账户（可变），用于抵押品管理
    /// CHECK: 由服务层验证
    #[account(mut)]
    pub collateral_assets: UncheckedAccount<'info>,
    
    /// 预言机程序，用于价格验证
    /// CHECK: 由预言机适配器验证
    pub oracle_program: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 时钟程序
    pub clock: Sysvar<'info, Clock>,
}

/// 稳定币抵押率管理指令实现
/// 
/// 执行抵押率管理操作，包括：
/// - 参数验证和权限检查
/// - 抵押率计算和评估
/// - 抵押品管理
/// - 事件记录和状态更新
pub fn collateral_ratio_stablecoin(
    ctx: Context<CollateralRatioStablecoin>,
    params: CollateralRatioStablecoinParams
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_collateral_ratio_params(&params)?;
    require!(params.target_collateral_ratio_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(params.min_collateral_ratio_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(params.liquidation_threshold_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "collateral_ratio"
    )?;
    
    // 3. 调用服务层执行抵押率管理
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.collateral_ratio_management(
        stablecoin_asset,
        &params,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射抵押率管理事件
    emit!(AssetCollateralRatioUpdated {
        asset_id: stablecoin_asset.id,
        target_collateral_ratio_bps: params.target_collateral_ratio_bps,
        current_collateral_ratio_bps: result.current_collateral_ratio_bps,
        collateral_ratio_deviation_bps: result.collateral_ratio_deviation_bps,
        total_collateral_value: result.total_collateral_value,
        total_debt: result.total_debt,
        risk_status: result.risk_status.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin collateral ratio management executed successfully: asset_id={}, target_ratio_bps={}, current_ratio_bps={}, risk_status={:?}", 
         stablecoin_asset.id, params.target_collateral_ratio_bps, result.current_collateral_ratio_bps, result.risk_status);
    
    Ok(())
}

/// 添加抵押品指令实现
/// 
/// 执行添加抵押品操作
pub fn add_collateral_stablecoin(
    ctx: Context<CollateralRatioStablecoin>,
    params: CollateralRatioStablecoinParams,
    collateral_asset: CollateralAsset
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_collateral_ratio_params(&params)?;
    require!(collateral_asset.collateral_value > 0, crate::errors::asset_error::AssetError::InvalidAmount);
    require!(collateral_asset.collateral_weight_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "add_collateral"
    )?;
    
    // 3. 调用服务层执行添加抵押品
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.add_collateral(
        stablecoin_asset,
        &params,
        collateral_asset,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射添加抵押品事件
    emit!(AssetCollateralRatioUpdated {
        asset_id: stablecoin_asset.id,
        target_collateral_ratio_bps: params.target_collateral_ratio_bps,
        current_collateral_ratio_bps: result.current_collateral_ratio_bps,
        collateral_ratio_deviation_bps: result.collateral_ratio_deviation_bps,
        total_collateral_value: result.total_collateral_value,
        total_debt: result.total_debt,
        risk_status: result.risk_status.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin add collateral executed successfully: asset_id={}, collateral_value={}, collateral_weight_bps={}", 
         stablecoin_asset.id, collateral_asset.collateral_value, collateral_asset.collateral_weight_bps);
    
    Ok(())
}

/// 移除抵押品指令实现
/// 
/// 执行移除抵押品操作
pub fn remove_collateral_stablecoin(
    ctx: Context<CollateralRatioStablecoin>,
    params: CollateralRatioStablecoinParams,
    collateral_asset_address: Pubkey,
    remove_amount: u64
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_collateral_ratio_params(&params)?;
    require!(remove_amount > 0, crate::errors::asset_error::AssetError::InvalidAmount);
    require!(collateral_asset_address != Pubkey::default(), crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "remove_collateral"
    )?;
    
    // 3. 调用服务层执行移除抵押品
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.remove_collateral(
        stablecoin_asset,
        &params,
        collateral_asset_address,
        remove_amount,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射移除抵押品事件
    emit!(AssetCollateralRatioUpdated {
        asset_id: stablecoin_asset.id,
        target_collateral_ratio_bps: params.target_collateral_ratio_bps,
        current_collateral_ratio_bps: result.current_collateral_ratio_bps,
        collateral_ratio_deviation_bps: result.collateral_ratio_deviation_bps,
        total_collateral_value: result.total_collateral_value,
        total_debt: result.total_debt,
        risk_status: result.risk_status.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin remove collateral executed successfully: asset_id={}, collateral_address={}, remove_amount={}", 
         stablecoin_asset.id, collateral_asset_address, remove_amount);
    
    Ok(())
}

/// 清算触发指令实现
/// 
/// 执行清算触发操作
pub fn trigger_liquidation_stablecoin(
    ctx: Context<CollateralRatioStablecoin>,
    params: CollateralRatioStablecoinParams,
    liquidation_threshold_bps: u16
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_collateral_ratio_params(&params)?;
    require!(liquidation_threshold_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "trigger_liquidation"
    )?;
    
    // 3. 调用服务层执行清算触发
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.trigger_liquidation(
        stablecoin_asset,
        &params,
        liquidation_threshold_bps,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射清算触发事件
    emit!(AssetCollateralRatioUpdated {
        asset_id: stablecoin_asset.id,
        target_collateral_ratio_bps: params.target_collateral_ratio_bps,
        current_collateral_ratio_bps: result.current_collateral_ratio_bps,
        collateral_ratio_deviation_bps: result.collateral_ratio_deviation_bps,
        total_collateral_value: result.total_collateral_value,
        total_debt: result.total_debt,
        risk_status: CollateralRiskStatus::Liquidation,
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin liquidation triggered successfully: asset_id={}, liquidation_threshold_bps={}, current_ratio_bps={}", 
         stablecoin_asset.id, liquidation_threshold_bps, result.current_collateral_ratio_bps);
    
    Ok(())
} 