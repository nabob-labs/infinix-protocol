//! 指数代币 (IndexToken) 权重调整指令
//!
//! 本模块实现了指数代币成分股权重的调整功能，包括动态权重调整、权重验证和权重优化。
//!
//! ## 功能特点
//!
//! - **动态权重调整**: 根据市场条件动态调整成分股权重
//! - **权重验证**: 确保权重调整的合理性和合规性
//! - **权重优化**: 基于算法优化权重分配
//! - **风险控制**: 内置权重调整风险控制机制
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::index_token_service::IndexTokenService;
use crate::events::asset_event::AssetWeightAdjusted;
use crate::errors::AssetError;

/// 权重调整类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum WeightAdjustmentType {
    /// 市值加权调整
    MarketCapWeighted,
    /// 等权重调整
    EqualWeighted,
    /// 价格加权调整
    PriceWeighted,
    /// 自定义权重调整
    CustomWeighted,
    /// 风险平价调整
    RiskParity,
    /// 最小方差调整
    MinimumVariance,
}

/// 权重调整策略
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum WeightAdjustmentStrategy {
    /// 渐进式调整
    Gradual,
    /// 一次性调整
    Immediate,
    /// 条件触发调整
    Conditional,
    /// 算法优化调整
    Algorithmic,
}

/// 权重调整结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct WeightAdjustmentResult {
    /// 调整前权重
    pub old_weights: Vec<f64>,
    /// 调整后权重
    pub new_weights: Vec<f64>,
    /// 权重变化
    pub weight_changes: Vec<f64>,
    /// 调整成本
    pub adjustment_cost: u64,
    /// 调整时间戳
    pub timestamp: i64,
    /// 调整类型
    pub adjustment_type: WeightAdjustmentType,
}

/// 指数代币权重调整指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct WeightAdjustmentParams {
    /// 权重调整类型
    pub adjustment_type: WeightAdjustmentType,
    /// 权重调整策略
    pub strategy: WeightAdjustmentStrategy,
    /// 目标权重
    pub target_weights: Vec<f64>,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// 指数代币权重调整指令账户上下文
#[derive(Accounts)]
pub struct WeightAdjustment<'info> {
    /// 指数代币资产账户，需可变
    #[account(
        mut,
        constraint = index_token.asset_type == AssetType::IndexToken @ AssetError::InvalidAssetType
    )]
    pub index_token: Account<'info, BasketIndexState>,
    
    /// 权重调整权限签名者
    #[account(
        constraint = authority.key() == index_token.weight_adjustment_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// DEX程序
    pub dex_program: Program<'info, DexProgram>,
    
    /// 价格预言机程序
    pub oracle_program: Program<'info, OracleProgram>,
    
    /// 成分股代币账户列表
    #[account(mut)]
    pub constituent_tokens: Vec<Account<'info, TokenAccount>>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 指数代币权重调整指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 权重调整参数，包含调整类型、策略和目标权重
///
/// ## 返回值
/// - `Result<WeightAdjustmentResult>`: 权重调整结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidParams`: 无效的参数
/// - `WeightAdjustmentFailed`: 权重调整失败
pub fn weight_adjustment(
    ctx: Context<WeightAdjustment>,
    params: WeightAdjustmentParams,
) -> Result<WeightAdjustmentResult> {
    // 参数验证
    validate_weight_adjustment_params(&params)?;
    
    // 权限检查
    check_weight_adjustment_authority_permission(&ctx.accounts.authority, &ctx.accounts.index_token)?;
    
    // 获取账户引用
    let index_token = &mut ctx.accounts.index_token;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = IndexTokenService::new();
    
    // 调用服务层执行权重调整操作
    let result = service.weight_adjustment(
        index_token,
        &params.adjustment_type,
        &params.strategy,
        &params.target_weights,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetWeightAdjusted {
        basket_id: index_token.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::IndexToken,
        adjustment_type: params.adjustment_type,
        strategy: params.strategy,
        old_weights: result.old_weights.clone(),
        new_weights: result.new_weights.clone(),
        weight_changes: result.weight_changes.clone(),
        adjustment_cost: result.adjustment_cost,
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证权重调整参数
fn validate_weight_adjustment_params(params: &WeightAdjustmentParams) -> Result<()> {
    // 验证目标权重
    validate_target_weights(&params.target_weights)?;
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
    Ok(())
}

/// 验证目标权重
fn validate_target_weights(target_weights: &[f64]) -> Result<()> {
    require!(!target_weights.is_empty(), AssetError::InvalidParams);
    
    let total_weight: f64 = target_weights.iter().sum();
    require!((total_weight - 1.0).abs() < 0.001, AssetError::InvalidParams);
    
    for weight in target_weights {
        require!(*weight >= 0.0, AssetError::InvalidParams);
        require!(*weight <= 1.0, AssetError::InvalidParams);
    }
    
    Ok(())
}

/// 检查权重调整权限
fn check_weight_adjustment_authority_permission(
    authority: &Signer,
    index_token: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == index_token.weight_adjustment_authority,
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