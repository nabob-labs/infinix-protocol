//! 稳定币 (Stablecoin) 清算机制指令
//! 
//! 本模块实现稳定币资产的清算机制功能，支持自动清算、手动清算、清算奖励等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 自动清算：抵押率不足时的自动清算
//! - 手动清算：手动触发清算操作
//! - 清算奖励：清算人的奖励机制
//! - 清算保护：清算保护机制
//! - 事件记录：完整的审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, LiquidationParams};
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetLiquidated;
use crate::validation::business::validate_liquidation_params;
use crate::core::security::check_authority_permission;
use crate::dex::traits::DexAdapterTrait;
use crate::oracles::traits::OracleAdapterTrait;

/// 稳定币清算机制参数结构体
/// 
/// 定义清算机制操作所需的所有参数，包括：
/// - liquidation_type: 清算类型
/// - liquidation_amount: 清算数量
/// - reward_rate: 奖励比率
/// - protection_params: 保护参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LiquidationStablecoinParams {
    /// 清算类型
    pub liquidation_type: LiquidationType,
    /// 清算数量
    pub liquidation_amount: u64,
    /// 奖励比率（基点，1/10000）
    pub reward_rate_bps: u16,
    /// 清算参数
    pub liquidation_params: Option<LiquidationParams>,
    /// 执行参数（可选）
    pub exec_params: Option<ExecutionParams>,
    /// 策略参数（可选）
    pub strategy_params: Option<StrategyParams>,
}

/// 清算类型枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum LiquidationType {
    /// 自动清算
    Automatic,
    /// 手动清算
    Manual,
    /// 部分清算
    Partial,
    /// 完全清算
    Full,
    /// 紧急清算
    Emergency,
}

/// 清算订单结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LiquidationOrder {
    /// 订单ID
    pub order_id: u64,
    /// 清算类型
    pub liquidation_type: LiquidationType,
    /// 清算数量
    pub liquidation_amount: u64,
    /// 奖励比率
    pub reward_rate_bps: u16,
    /// 时间戳
    pub timestamp: i64,
}

/// 清算结果结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LiquidationResult {
    /// 清算数量
    pub liquidated_amount: u64,
    /// 清算价格
    pub liquidation_price_bps: u64,
    /// 清算费用
    pub liquidation_fees: u64,
    /// 清算奖励
    pub liquidation_reward: u64,
    /// 清算状态
    pub liquidation_status: LiquidationStatus,
    /// 成功标志
    pub success: bool,
}

/// 清算状态枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum LiquidationStatus {
    /// 待清算
    Pending,
    /// 清算中
    InProgress,
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 稳定币清算机制指令账户上下文
/// 
/// 定义清算机制操作所需的所有账户，包括：
/// - stablecoin_asset: 稳定币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - liquidator: 清算人账户（签名者）
/// - collateral_assets: 抵押品资产账户（可变）
/// - oracle_program: 预言机程序（用于价格验证）
/// - system_program: 系统程序
/// - clock: 时钟程序
#[derive(Accounts)]
#[instruction(params: LiquidationStablecoinParams)]
pub struct LiquidationStablecoin<'info> {
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
    
    /// 清算人账户，必须是签名者
    pub liquidator: Signer<'info>,
    
    /// 抵押品资产账户（可变），用于清算操作
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

/// 稳定币清算机制指令实现
/// 
/// 执行清算机制操作，包括：
/// - 参数验证和权限检查
/// - 清算条件检查
/// - 清算执行
/// - 事件记录和状态更新
pub fn liquidation_stablecoin(
    ctx: Context<LiquidationStablecoin>,
    params: LiquidationStablecoinParams
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_liquidation_params(&params)?;
    require!(params.liquidation_amount > 0, crate::errors::asset_error::AssetError::InvalidAmount);
    require!(params.reward_rate_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "liquidation"
    )?;
    
    // 3. 调用服务层执行清算
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.liquidation(
        stablecoin_asset,
        &params,
        ctx.accounts.liquidator.key(),
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射清算事件
    emit!(AssetLiquidated {
        asset_id: stablecoin_asset.id,
        liquidation_type: params.liquidation_type.clone(),
        liquidated_amount: result.liquidated_amount,
        liquidation_price_bps: result.liquidation_price_bps,
        liquidation_fees: result.liquidation_fees,
        liquidation_reward: result.liquidation_reward,
        liquidator: ctx.accounts.liquidator.key(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin liquidation executed successfully: asset_id={}, liquidation_type={:?}, liquidated_amount={}, reward={}", 
         stablecoin_asset.id, params.liquidation_type, result.liquidated_amount, result.liquidation_reward);
    
    Ok(())
}

/// 自动清算指令实现
/// 
/// 执行自动清算操作
pub fn automatic_liquidation_stablecoin(
    ctx: Context<LiquidationStablecoin>,
    params: LiquidationStablecoinParams,
    collateral_ratio_threshold_bps: u16
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_liquidation_params(&params)?;
    require!(params.liquidation_amount > 0, crate::errors::asset_error::AssetError::InvalidAmount);
    require!(collateral_ratio_threshold_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "automatic_liquidation"
    )?;
    
    // 3. 调用服务层执行自动清算
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.automatic_liquidation(
        stablecoin_asset,
        &params,
        collateral_ratio_threshold_bps,
        ctx.accounts.liquidator.key(),
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射自动清算事件
    emit!(AssetLiquidated {
        asset_id: stablecoin_asset.id,
        liquidation_type: LiquidationType::Automatic,
        liquidated_amount: result.liquidated_amount,
        liquidation_price_bps: result.liquidation_price_bps,
        liquidation_fees: result.liquidation_fees,
        liquidation_reward: result.liquidation_reward,
        liquidator: ctx.accounts.liquidator.key(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin automatic liquidation executed successfully: asset_id={}, liquidated_amount={}, collateral_ratio_threshold_bps={}", 
         stablecoin_asset.id, result.liquidated_amount, collateral_ratio_threshold_bps);
    
    Ok(())
}

/// 手动清算指令实现
/// 
/// 执行手动清算操作
pub fn manual_liquidation_stablecoin(
    ctx: Context<LiquidationStablecoin>,
    params: LiquidationStablecoinParams,
    target_collateral_asset: Pubkey
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_liquidation_params(&params)?;
    require!(params.liquidation_amount > 0, crate::errors::asset_error::AssetError::InvalidAmount);
    require!(target_collateral_asset != Pubkey::default(), crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "manual_liquidation"
    )?;
    
    // 3. 调用服务层执行手动清算
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.manual_liquidation(
        stablecoin_asset,
        &params,
        target_collateral_asset,
        ctx.accounts.liquidator.key(),
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射手动清算事件
    emit!(AssetLiquidated {
        asset_id: stablecoin_asset.id,
        liquidation_type: LiquidationType::Manual,
        liquidated_amount: result.liquidated_amount,
        liquidation_price_bps: result.liquidation_price_bps,
        liquidation_fees: result.liquidation_fees,
        liquidation_reward: result.liquidation_reward,
        liquidator: ctx.accounts.liquidator.key(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin manual liquidation executed successfully: asset_id={}, liquidated_amount={}, target_collateral_asset={}", 
         stablecoin_asset.id, result.liquidated_amount, target_collateral_asset);
    
    Ok(())
}

/// 紧急清算指令实现
/// 
/// 执行紧急清算操作
pub fn emergency_liquidation_stablecoin(
    ctx: Context<LiquidationStablecoin>,
    params: LiquidationStablecoinParams,
    emergency_threshold_bps: u16
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_liquidation_params(&params)?;
    require!(params.liquidation_amount > 0, crate::errors::asset_error::AssetError::InvalidAmount);
    require!(emergency_threshold_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "emergency_liquidation"
    )?;
    
    // 3. 调用服务层执行紧急清算
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.emergency_liquidation(
        stablecoin_asset,
        &params,
        emergency_threshold_bps,
        ctx.accounts.liquidator.key(),
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射紧急清算事件
    emit!(AssetLiquidated {
        asset_id: stablecoin_asset.id,
        liquidation_type: LiquidationType::Emergency,
        liquidated_amount: result.liquidated_amount,
        liquidation_price_bps: result.liquidation_price_bps,
        liquidation_fees: result.liquidation_fees,
        liquidation_reward: result.liquidation_reward,
        liquidator: ctx.accounts.liquidator.key(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin emergency liquidation executed successfully: asset_id={}, liquidated_amount={}, emergency_threshold_bps={}", 
         stablecoin_asset.id, result.liquidated_amount, emergency_threshold_bps);
    
    Ok(())
} 