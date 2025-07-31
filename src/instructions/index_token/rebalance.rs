//! 指数代币 (IndexToken) 再平衡指令
//!
//! 本模块实现了指数代币的再平衡功能，包括权重调整、成分股更新和表现追踪。
//!
//! ## 功能特点
//!
//! - **权重调整**: 根据市场表现调整成分股权重
//! - **成分股更新**: 动态更新指数成分股
//! - **表现追踪**: 实时追踪指数表现
//! - **风险控制**: 内置风险控制机制
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::index_token_service::IndexTokenService;
use crate::events::asset_event::AssetRebalanced;
use crate::errors::AssetError;

/// 再平衡调整机制
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum RebalanceAdjustmentMechanism {
    /// 市值加权
    MarketCapWeighted,
    /// 等权重
    EqualWeighted,
    /// 价格加权
    PriceWeighted,
    /// 自定义权重
    CustomWeighted,
}

/// 再平衡触发条件
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum RebalanceTrigger {
    /// 时间触发
    TimeBased,
    /// 价格触发
    PriceBased,
    /// 权重偏差触发
    WeightDeviation,
    /// 手动触发
    Manual,
}

/// 再平衡结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RebalanceResult {
    /// 调整前权重
    pub old_weights: Vec<f64>,
    /// 调整后权重
    pub new_weights: Vec<f64>,
    /// 调整成本
    pub rebalance_cost: u64,
    /// 调整时间戳
    pub timestamp: i64,
    /// 调整机制
    pub mechanism: RebalanceAdjustmentMechanism,
}

/// 指数代币再平衡指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RebalanceIndexParams {
    /// 再平衡触发条件
    pub trigger: RebalanceTrigger,
    /// 调整机制
    pub mechanism: RebalanceAdjustmentMechanism,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// 指数代币再平衡指令账户上下文
#[derive(Accounts)]
pub struct RebalanceIndex<'info> {
    /// 指数代币资产账户，需可变
    #[account(
        mut,
        constraint = index_token.asset_type == AssetType::IndexToken @ AssetError::InvalidAssetType
    )]
    pub index_token: Account<'info, BasketIndexState>,
    
    /// 再平衡权限签名者
    #[account(
        constraint = authority.key() == index_token.rebalance_authority @ AssetError::InsufficientAuthority
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

/// 指数代币再平衡指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 再平衡参数，包含触发条件、调整机制和执行参数
///
/// ## 返回值
/// - `Result<RebalanceResult>`: 再平衡结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidParams`: 无效的参数
/// - `RebalanceFailed`: 再平衡失败
pub fn rebalance_index(
    ctx: Context<RebalanceIndex>,
    params: RebalanceIndexParams,
) -> Result<RebalanceResult> {
    // 参数验证
    validate_rebalance_index_params(&params)?;
    
    // 权限检查
    check_rebalance_authority_permission(&ctx.accounts.authority, &ctx.accounts.index_token)?;
    
    // 获取账户引用
    let index_token = &mut ctx.accounts.index_token;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = IndexTokenService::new();
    
    // 调用服务层执行再平衡操作
    let result = service.rebalance_index(
        index_token,
        &params.trigger,
        &params.mechanism,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetRebalanced {
        basket_id: index_token.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::IndexToken,
        trigger: params.trigger,
        mechanism: params.mechanism,
        rebalance_cost: result.rebalance_cost,
        old_weights: result.old_weights.clone(),
        new_weights: result.new_weights.clone(),
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证再平衡参数
fn validate_rebalance_index_params(params: &RebalanceIndexParams) -> Result<()> {
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
    Ok(())
}

/// 检查再平衡权限
fn check_rebalance_authority_permission(
    authority: &Signer,
    index_token: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == index_token.rebalance_authority,
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