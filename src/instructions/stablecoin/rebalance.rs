//! 稳定币 (Stablecoin) 再平衡指令
//! 
//! 本模块实现稳定币资产的再平衡功能，支持锚定维护、价格稳定、流动性管理等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 锚定维护：保持与目标货币的锚定关系
//! - 价格稳定：通过市场操作稳定价格
//! - 流动性管理：优化流动性分布
//! - 风险控制：再平衡过程中的风险监控
//! - 事件记录：完整的审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, RebalanceParams};
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetRebalanced;
use crate::validation::business::validate_rebalance_params;
use crate::core::security::check_authority_permission;
use crate::dex::traits::DexAdapterTrait;
use crate::oracles::traits::OracleAdapterTrait;

/// 稳定币再平衡参数结构体
/// 
/// 定义再平衡操作所需的所有参数，包括：
/// - target_price: 目标价格
/// - rebalance_type: 再平衡类型
/// - liquidity_params: 流动性参数
/// - risk_params: 风险参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RebalanceStablecoinParams {
    /// 目标价格（基点，1/10000）
    pub target_price_bps: u64,
    /// 再平衡类型
    pub rebalance_type: RebalanceType,
    /// 流动性参数
    pub liquidity_params: Option<crate::core::types::LiquidityParams>,
    /// 风险参数
    pub risk_params: Option<crate::core::types::RiskParams>,
    /// 执行参数（可选）
    pub exec_params: Option<ExecutionParams>,
    /// 策略参数（可选）
    pub strategy_params: Option<StrategyParams>,
}

/// 再平衡类型枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum RebalanceType {
    /// 锚定维护
    PegMaintenance,
    /// 价格稳定
    PriceStabilization,
    /// 流动性优化
    LiquidityOptimization,
    /// 风险调整
    RiskAdjustment,
    /// 市场中性
    MarketNeutral,
}

/// 再平衡订单结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RebalanceOrder {
    /// 订单ID
    pub order_id: u64,
    /// 再平衡类型
    pub rebalance_type: RebalanceType,
    /// 目标价格
    pub target_price_bps: u64,
    /// 执行数量
    pub execution_amount: u64,
    /// 时间戳
    pub timestamp: i64,
}

/// 再平衡结果结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RebalanceResult {
    /// 再平衡前价格
    pub pre_price_bps: u64,
    /// 再平衡后价格
    pub post_price_bps: u64,
    /// 执行数量
    pub executed_amount: u64,
    /// 滑点（基点）
    pub slippage_bps: u16,
    /// 费用
    pub fees: u64,
    /// 成功标志
    pub success: bool,
}

/// 稳定币再平衡指令账户上下文
/// 
/// 定义再平衡操作所需的所有账户，包括：
/// - stablecoin_asset: 稳定币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - dex_program: DEX程序（用于交易执行）
/// - oracle_program: 预言机程序（用于价格验证）
/// - system_program: 系统程序
/// - clock: 时钟程序
#[derive(Accounts)]
#[instruction(params: RebalanceStablecoinParams)]
pub struct RebalanceStablecoin<'info> {
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
    
    /// DEX程序，用于交易执行
    /// CHECK: 由DEX适配器验证
    pub dex_program: UncheckedAccount<'info>,
    
    /// 预言机程序，用于价格验证
    /// CHECK: 由预言机适配器验证
    pub oracle_program: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 时钟程序
    pub clock: Sysvar<'info, Clock>,
}

/// 稳定币再平衡指令实现
/// 
/// 执行再平衡操作，包括：
/// - 参数验证和权限检查
/// - 价格监控和偏差检测
/// - 再平衡策略执行
/// - 事件记录和状态更新
pub fn rebalance_stablecoin(
    ctx: Context<RebalanceStablecoin>,
    params: RebalanceStablecoinParams
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_rebalance_params(&params)?;
    require!(params.target_price_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "rebalance"
    )?;
    
    // 3. 调用服务层执行再平衡
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.rebalance(
        stablecoin_asset,
        &params,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射再平衡事件
    emit!(AssetRebalanced {
        asset_id: stablecoin_asset.id,
        rebalance_type: params.rebalance_type.clone(),
        target_price_bps: params.target_price_bps,
        executed_amount: result.executed_amount,
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin rebalance executed successfully: asset_id={}, rebalance_type={:?}, target_price_bps={}", 
         stablecoin_asset.id, params.rebalance_type, params.target_price_bps);
    
    Ok(())
}

/// 锚定维护再平衡指令实现
/// 
/// 执行锚定维护的再平衡操作
pub fn peg_maintenance_rebalance_stablecoin(
    ctx: Context<RebalanceStablecoin>,
    params: RebalanceStablecoinParams,
    peg_deviation_threshold_bps: u16
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_rebalance_params(&params)?;
    require!(params.target_price_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(peg_deviation_threshold_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "peg_maintenance_rebalance"
    )?;
    
    // 3. 调用服务层执行锚定维护再平衡
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.peg_maintenance_rebalance(
        stablecoin_asset,
        &params,
        peg_deviation_threshold_bps,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射锚定维护再平衡事件
    emit!(AssetRebalanced {
        asset_id: stablecoin_asset.id,
        rebalance_type: RebalanceType::PegMaintenance,
        target_price_bps: params.target_price_bps,
        executed_amount: result.executed_amount,
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin peg maintenance rebalance executed successfully: asset_id={}, target_price_bps={}, deviation_threshold_bps={}", 
         stablecoin_asset.id, params.target_price_bps, peg_deviation_threshold_bps);
    
    Ok(())
}

/// 价格稳定再平衡指令实现
/// 
/// 执行价格稳定的再平衡操作
pub fn price_stabilization_rebalance_stablecoin(
    ctx: Context<RebalanceStablecoin>,
    params: RebalanceStablecoinParams,
    volatility_threshold_bps: u16
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_rebalance_params(&params)?;
    require!(params.target_price_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(volatility_threshold_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "price_stabilization_rebalance"
    )?;
    
    // 3. 调用服务层执行价格稳定再平衡
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.price_stabilization_rebalance(
        stablecoin_asset,
        &params,
        volatility_threshold_bps,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射价格稳定再平衡事件
    emit!(AssetRebalanced {
        asset_id: stablecoin_asset.id,
        rebalance_type: RebalanceType::PriceStabilization,
        target_price_bps: params.target_price_bps,
        executed_amount: result.executed_amount,
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin price stabilization rebalance executed successfully: asset_id={}, target_price_bps={}, volatility_threshold_bps={}", 
         stablecoin_asset.id, params.target_price_bps, volatility_threshold_bps);
    
    Ok(())
}

/// 流动性优化再平衡指令实现
/// 
/// 执行流动性优化的再平衡操作
pub fn liquidity_optimization_rebalance_stablecoin(
    ctx: Context<RebalanceStablecoin>,
    params: RebalanceStablecoinParams,
    liquidity_target_bps: u64
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &mut ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_rebalance_params(&params)?;
    require!(params.target_price_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(liquidity_target_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "liquidity_optimization_rebalance"
    )?;
    
    // 3. 调用服务层执行流动性优化再平衡
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.liquidity_optimization_rebalance(
        stablecoin_asset,
        &params,
        liquidity_target_bps,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射流动性优化再平衡事件
    emit!(AssetRebalanced {
        asset_id: stablecoin_asset.id,
        rebalance_type: RebalanceType::LiquidityOptimization,
        target_price_bps: params.target_price_bps,
        executed_amount: result.executed_amount,
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin liquidity optimization rebalance executed successfully: asset_id={}, target_price_bps={}, liquidity_target_bps={}", 
         stablecoin_asset.id, params.target_price_bps, liquidity_target_bps);
    
    Ok(())
} 