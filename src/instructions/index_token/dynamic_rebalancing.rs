//! 指数代币 (IndexToken) 动态再平衡指令
//!
//! 本模块实现了指数代币的动态再平衡功能，包括市场条件监控、自动再平衡触发和智能权重调整。
//!
//! ## 功能特点
//!
//! - **市场条件监控**: 实时监控市场条件和价格变化
//! - **自动再平衡触发**: 根据预设条件自动触发再平衡
//! - **智能权重调整**: 基于算法智能调整成分股权重
//! - **风险控制**: 内置动态再平衡风险控制机制
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::index_token_service::IndexTokenService;
use crate::events::asset_event::AssetDynamicRebalanced;
use crate::errors::AssetError;

/// 动态再平衡触发条件
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum DynamicRebalancingTrigger {
    /// 价格偏差触发
    PriceDeviation,
    /// 权重偏差触发
    WeightDeviation,
    /// 波动率触发
    Volatility,
    /// 时间触发
    TimeBased,
    /// 市场冲击触发
    MarketShock,
    /// 组合触发
    Combined,
}

/// 动态再平衡策略
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum DynamicRebalancingStrategy {
    /// 阈值触发策略
    ThresholdBased,
    /// 渐进式策略
    Gradual,
    /// 激进式策略
    Aggressive,
    /// 保守式策略
    Conservative,
    /// 自适应策略
    Adaptive,
}

/// 动态再平衡参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DynamicRebalancingConfig {
    /// 触发阈值
    pub trigger_threshold: f64,
    /// 再平衡频率
    pub rebalance_frequency: u64,
    /// 最大调整幅度
    pub max_adjustment: f64,
    /// 最小调整幅度
    pub min_adjustment: f64,
    /// 冷却期
    pub cooldown_period: u64,
}

/// 动态再平衡结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DynamicRebalancingResult {
    /// 触发条件
    pub trigger: DynamicRebalancingTrigger,
    /// 再平衡策略
    pub strategy: DynamicRebalancingStrategy,
    /// 调整前权重
    pub old_weights: Vec<f64>,
    /// 调整后权重
    pub new_weights: Vec<f64>,
    /// 调整幅度
    pub adjustment_magnitude: f64,
    /// 再平衡成本
    pub rebalance_cost: u64,
    /// 再平衡时间戳
    pub timestamp: i64,
}

/// 指数代币动态再平衡指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DynamicRebalancingParams {
    /// 动态再平衡触发条件
    pub trigger: DynamicRebalancingTrigger,
    /// 动态再平衡策略
    pub strategy: DynamicRebalancingStrategy,
    /// 动态再平衡配置
    pub config: DynamicRebalancingConfig,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// 指数代币动态再平衡指令账户上下文
#[derive(Accounts)]
pub struct DynamicRebalancing<'info> {
    /// 指数代币资产账户，需可变
    #[account(
        mut,
        constraint = index_token.asset_type == AssetType::IndexToken @ AssetError::InvalidAssetType
    )]
    pub index_token: Account<'info, BasketIndexState>,
    
    /// 动态再平衡权限签名者
    #[account(
        constraint = authority.key() == index_token.dynamic_rebalancing_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// DEX程序
    pub dex_program: Program<'info, DexProgram>,
    
    /// 价格预言机程序
    pub oracle_program: Program<'info, OracleProgram>,
    
    /// 市场数据程序
    pub market_data_program: Program<'info, MarketDataProgram>,
    
    /// 成分股代币账户列表
    #[account(mut)]
    pub constituent_tokens: Vec<Account<'info, TokenAccount>>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 指数代币动态再平衡指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 动态再平衡参数，包含触发条件、策略和配置
///
/// ## 返回值
/// - `Result<DynamicRebalancingResult>`: 动态再平衡结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidParams`: 无效的参数
/// - `DynamicRebalancingFailed`: 动态再平衡失败
pub fn dynamic_rebalancing(
    ctx: Context<DynamicRebalancing>,
    params: DynamicRebalancingParams,
) -> Result<DynamicRebalancingResult> {
    // 参数验证
    validate_dynamic_rebalancing_params(&params)?;
    
    // 权限检查
    check_dynamic_rebalancing_authority_permission(&ctx.accounts.authority, &ctx.accounts.index_token)?;
    
    // 获取账户引用
    let index_token = &mut ctx.accounts.index_token;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = IndexTokenService::new();
    
    // 调用服务层执行动态再平衡操作
    let result = service.dynamic_rebalancing(
        index_token,
        &params.trigger,
        &params.strategy,
        &params.config,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetDynamicRebalanced {
        basket_id: index_token.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::IndexToken,
        trigger: params.trigger,
        strategy: params.strategy,
        config: params.config,
        old_weights: result.old_weights.clone(),
        new_weights: result.new_weights.clone(),
        adjustment_magnitude: result.adjustment_magnitude,
        rebalance_cost: result.rebalance_cost,
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证动态再平衡参数
fn validate_dynamic_rebalancing_params(params: &DynamicRebalancingParams) -> Result<()> {
    // 验证动态再平衡配置
    validate_dynamic_rebalancing_config(&params.config)?;
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
    Ok(())
}

/// 验证动态再平衡配置
fn validate_dynamic_rebalancing_config(config: &DynamicRebalancingConfig) -> Result<()> {
    require!(config.trigger_threshold > 0.0, AssetError::InvalidParams);
    require!(config.trigger_threshold <= 1.0, AssetError::InvalidParams);
    require!(config.rebalance_frequency > 0, AssetError::InvalidParams);
    require!(config.max_adjustment > 0.0, AssetError::InvalidParams);
    require!(config.max_adjustment <= 1.0, AssetError::InvalidParams);
    require!(config.min_adjustment >= 0.0, AssetError::InvalidParams);
    require!(config.min_adjustment <= config.max_adjustment, AssetError::InvalidParams);
    require!(config.cooldown_period > 0, AssetError::InvalidParams);
    
    Ok(())
}

/// 检查动态再平衡权限
fn check_dynamic_rebalancing_authority_permission(
    authority: &Signer,
    index_token: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == index_token.dynamic_rebalancing_authority,
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