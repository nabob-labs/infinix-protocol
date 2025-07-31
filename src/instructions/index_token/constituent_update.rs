//! 指数代币 (IndexToken) 成分股更新指令
//!
//! 本模块实现了指数代币成分股的更新功能，包括成分股添加、删除、替换和权重重新分配。
//!
//! ## 功能特点
//!
//! - **成分股添加**: 动态添加新的成分股
//! - **成分股删除**: 移除不符合条件的成分股
//! - **成分股替换**: 替换成分股并重新分配权重
//! - **权重重新分配**: 根据新成分股重新计算权重
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::index_token_service::IndexTokenService;
use crate::events::asset_event::AssetConstituentUpdated;
use crate::errors::AssetError;

/// 成分股更新类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ConstituentUpdateType {
    /// 添加成分股
    Add,
    /// 删除成分股
    Remove,
    /// 替换成分股
    Replace,
    /// 批量更新
    BatchUpdate,
}

/// 成分股更新策略
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ConstituentUpdateStrategy {
    /// 立即更新
    Immediate,
    /// 渐进式更新
    Gradual,
    /// 条件触发更新
    Conditional,
    /// 定期更新
    Periodic,
}

/// 成分股信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ConstituentInfo {
    /// 代币地址
    pub token_address: Pubkey,
    /// 目标权重
    pub target_weight: f64,
    /// 最小权重
    pub min_weight: f64,
    /// 最大权重
    pub max_weight: f64,
}

/// 成分股更新结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ConstituentUpdateResult {
    /// 更新前成分股
    pub old_constituents: Vec<ConstituentInfo>,
    /// 更新后成分股
    pub new_constituents: Vec<ConstituentInfo>,
    /// 更新成本
    pub update_cost: u64,
    /// 更新时间戳
    pub timestamp: i64,
    /// 更新类型
    pub update_type: ConstituentUpdateType,
}

/// 指数代币成分股更新指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ConstituentUpdateParams {
    /// 成分股更新类型
    pub update_type: ConstituentUpdateType,
    /// 成分股更新策略
    pub strategy: ConstituentUpdateStrategy,
    /// 新成分股列表
    pub new_constituents: Vec<ConstituentInfo>,
    /// 要删除的成分股地址
    pub remove_constituents: Vec<Pubkey>,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// 指数代币成分股更新指令账户上下文
#[derive(Accounts)]
pub struct ConstituentUpdate<'info> {
    /// 指数代币资产账户，需可变
    #[account(
        mut,
        constraint = index_token.asset_type == AssetType::IndexToken @ AssetError::InvalidAssetType
    )]
    pub index_token: Account<'info, BasketIndexState>,
    
    /// 成分股更新权限签名者
    #[account(
        constraint = authority.key() == index_token.constituent_update_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// DEX程序
    pub dex_program: Program<'info, DexProgram>,
    
    /// 价格预言机程序
    pub oracle_program: Program<'info, OracleProgram>,
    
    /// 当前成分股代币账户列表
    #[account(mut)]
    pub current_constituent_tokens: Vec<Account<'info, TokenAccount>>,
    
    /// 新成分股代币账户列表
    #[account(mut)]
    pub new_constituent_tokens: Vec<Account<'info, TokenAccount>>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 指数代币成分股更新指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 成分股更新参数，包含更新类型、策略和新成分股信息
///
/// ## 返回值
/// - `Result<ConstituentUpdateResult>`: 成分股更新结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidParams`: 无效的参数
/// - `ConstituentUpdateFailed`: 成分股更新失败
pub fn constituent_update(
    ctx: Context<ConstituentUpdate>,
    params: ConstituentUpdateParams,
) -> Result<ConstituentUpdateResult> {
    // 参数验证
    validate_constituent_update_params(&params)?;
    
    // 权限检查
    check_constituent_update_authority_permission(&ctx.accounts.authority, &ctx.accounts.index_token)?;
    
    // 获取账户引用
    let index_token = &mut ctx.accounts.index_token;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = IndexTokenService::new();
    
    // 调用服务层执行成分股更新操作
    let result = service.constituent_update(
        index_token,
        &params.update_type,
        &params.strategy,
        &params.new_constituents,
        &params.remove_constituents,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetConstituentUpdated {
        basket_id: index_token.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::IndexToken,
        update_type: params.update_type,
        strategy: params.strategy,
        old_constituents: result.old_constituents.clone(),
        new_constituents: result.new_constituents.clone(),
        update_cost: result.update_cost,
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证成分股更新参数
fn validate_constituent_update_params(params: &ConstituentUpdateParams) -> Result<()> {
    // 验证新成分股
    validate_new_constituents(&params.new_constituents)?;
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
    Ok(())
}

/// 验证新成分股
fn validate_new_constituents(constituents: &[ConstituentInfo]) -> Result<()> {
    require!(!constituents.is_empty(), AssetError::InvalidParams);
    
    let total_weight: f64 = constituents.iter().map(|c| c.target_weight).sum();
    require!((total_weight - 1.0).abs() < 0.001, AssetError::InvalidParams);
    
    for constituent in constituents {
        require!(constituent.target_weight >= 0.0, AssetError::InvalidParams);
        require!(constituent.target_weight <= 1.0, AssetError::InvalidParams);
        require!(constituent.min_weight >= 0.0, AssetError::InvalidParams);
        require!(constituent.max_weight <= 1.0, AssetError::InvalidParams);
        require!(constituent.min_weight <= constituent.max_weight, AssetError::InvalidParams);
        require!(constituent.target_weight >= constituent.min_weight, AssetError::InvalidParams);
        require!(constituent.target_weight <= constituent.max_weight, AssetError::InvalidParams);
    }
    
    Ok(())
}

/// 检查成分股更新权限
fn check_constituent_update_authority_permission(
    authority: &Signer,
    index_token: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == index_token.constituent_update_authority,
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