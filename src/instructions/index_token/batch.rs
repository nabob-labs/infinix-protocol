//! 指数代币 (IndexToken) 批量操作指令
//!
//! 本模块实现了指数代币的批量操作功能，包括批量交易、批量处理、批量管理和批量同步。
//!
//! ## 功能特点
//!
//! - **批量交易**: 支持批量买入、卖出、兑换操作
//! - **批量处理**: 支持批量再平衡、权重调整、成分股更新
//! - **批量管理**: 支持批量分红分配、费用管理、治理操作
//! - **批量同步**: 支持批量数据同步和状态更新
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, BatchParams};
use crate::services::index_token_service::IndexTokenService;
use crate::events::asset_event::{AssetBatchTraded, AssetBatchProcessed, AssetBatchManaged, AssetBatchSynced};
use crate::errors::AssetError;

/// 批量操作类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchOperationType {
    /// 批量交易
    Trade,
    /// 批量处理
    Process,
    /// 批量管理
    Manage,
    /// 批量同步
    Sync,
}

/// 批量交易类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchTradeType {
    /// 批量买入
    Buy,
    /// 批量卖出
    Sell,
    /// 批量兑换
    Swap,
    /// 混合交易
    Mixed,
}

/// 批量处理类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchProcessType {
    /// 批量再平衡
    Rebalance,
    /// 批量权重调整
    WeightAdjustment,
    /// 批量成分股更新
    ConstituentUpdate,
    /// 批量表现追踪
    PerformanceTracking,
}

/// 批量管理类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchManageType {
    /// 批量分红分配
    DividendDistribution,
    /// 批量费用管理
    FeeManagement,
    /// 批量治理操作
    Governance,
    /// 批量投票权管理
    VotingRights,
}

/// 批量同步类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchSyncType {
    /// 批量数据同步
    DataSync,
    /// 批量状态更新
    StatusUpdate,
    /// 批量价格同步
    PriceSync,
    /// 批量权重同步
    WeightSync,
}

/// 批量操作结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchOperationResult {
    /// 批量操作类型
    pub operation_type: BatchOperationType,
    /// 操作成功数量
    pub success_count: u32,
    /// 操作失败数量
    pub failure_count: u32,
    /// 操作成本
    pub operation_cost: u64,
    /// 操作时间戳
    pub timestamp: i64,
    /// 错误信息列表
    pub errors: Vec<String>,
}

/// 指数代币批量交易指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchTradeIndexTokenParams {
    /// 批量交易类型
    pub trade_type: BatchTradeType,
    /// 交易参数列表
    pub trade_params: Vec<ExecutionParams>,
    /// 批量参数
    pub batch_params: BatchParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// 指数代币批量处理指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchProcessIndexTokenParams {
    /// 批量处理类型
    pub process_type: BatchProcessType,
    /// 处理参数列表
    pub process_params: Vec<ExecutionParams>,
    /// 批量参数
    pub batch_params: BatchParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// 指数代币批量管理指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchManageIndexTokenParams {
    /// 批量管理类型
    pub manage_type: BatchManageType,
    /// 管理参数列表
    pub manage_params: Vec<ExecutionParams>,
    /// 批量参数
    pub batch_params: BatchParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// 指数代币批量同步指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchSyncIndexTokenParams {
    /// 批量同步类型
    pub sync_type: BatchSyncType,
    /// 同步参数列表
    pub sync_params: Vec<ExecutionParams>,
    /// 批量参数
    pub batch_params: BatchParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// 指数代币批量操作指令账户上下文
#[derive(Accounts)]
pub struct BatchIndexToken<'info> {
    /// 指数代币资产账户，需可变
    #[account(
        mut,
        constraint = index_token.asset_type == AssetType::IndexToken @ AssetError::InvalidAssetType
    )]
    pub index_token: Account<'info, BasketIndexState>,
    
    /// 批量操作权限签名者
    #[account(
        constraint = authority.key() == index_token.batch_operation_authority @ AssetError::InsufficientAuthority
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

/// 指数代币批量交易指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 批量交易参数，包含交易类型、交易参数和批量参数
///
/// ## 返回值
/// - `Result<BatchOperationResult>`: 批量操作结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidParams`: 无效的参数
/// - `BatchOperationFailed`: 批量操作失败
pub fn batch_trade_index_token(
    ctx: Context<BatchIndexToken>,
    params: BatchTradeIndexTokenParams,
) -> Result<BatchOperationResult> {
    // 参数验证
    validate_batch_trade_index_token_params(&params)?;
    
    // 权限检查
    check_batch_operation_authority_permission(&ctx.accounts.authority, &ctx.accounts.index_token)?;
    
    // 获取账户引用
    let index_token = &mut ctx.accounts.index_token;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = IndexTokenService::new();
    
    // 调用服务层执行批量交易操作
    let result = service.batch_trade_index_token(
        index_token,
        &params.trade_type,
        &params.trade_params,
        &params.batch_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetBatchTraded {
        basket_id: index_token.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::IndexToken,
        trade_type: params.trade_type,
        success_count: result.success_count,
        failure_count: result.failure_count,
        operation_cost: result.operation_cost,
        errors: result.errors.clone(),
        batch_params: params.batch_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 指数代币批量处理指令实现
pub fn batch_process_index_token(
    ctx: Context<BatchIndexToken>,
    params: BatchProcessIndexTokenParams,
) -> Result<BatchOperationResult> {
    // 参数验证
    validate_batch_process_index_token_params(&params)?;
    
    // 权限检查
    check_batch_operation_authority_permission(&ctx.accounts.authority, &ctx.accounts.index_token)?;
    
    // 获取账户引用
    let index_token = &mut ctx.accounts.index_token;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = IndexTokenService::new();
    
    // 调用服务层执行批量处理操作
    let result = service.batch_process_index_token(
        index_token,
        &params.process_type,
        &params.process_params,
        &params.batch_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetBatchProcessed {
        basket_id: index_token.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::IndexToken,
        process_type: params.process_type,
        success_count: result.success_count,
        failure_count: result.failure_count,
        operation_cost: result.operation_cost,
        errors: result.errors.clone(),
        batch_params: params.batch_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 指数代币批量管理指令实现
pub fn batch_manage_index_token(
    ctx: Context<BatchIndexToken>,
    params: BatchManageIndexTokenParams,
) -> Result<BatchOperationResult> {
    // 参数验证
    validate_batch_manage_index_token_params(&params)?;
    
    // 权限检查
    check_batch_operation_authority_permission(&ctx.accounts.authority, &ctx.accounts.index_token)?;
    
    // 获取账户引用
    let index_token = &mut ctx.accounts.index_token;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = IndexTokenService::new();
    
    // 调用服务层执行批量管理操作
    let result = service.batch_manage_index_token(
        index_token,
        &params.manage_type,
        &params.manage_params,
        &params.batch_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetBatchManaged {
        basket_id: index_token.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::IndexToken,
        manage_type: params.manage_type,
        success_count: result.success_count,
        failure_count: result.failure_count,
        operation_cost: result.operation_cost,
        errors: result.errors.clone(),
        batch_params: params.batch_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 指数代币批量同步指令实现
pub fn batch_sync_index_token(
    ctx: Context<BatchIndexToken>,
    params: BatchSyncIndexTokenParams,
) -> Result<BatchOperationResult> {
    // 参数验证
    validate_batch_sync_index_token_params(&params)?;
    
    // 权限检查
    check_batch_operation_authority_permission(&ctx.accounts.authority, &ctx.accounts.index_token)?;
    
    // 获取账户引用
    let index_token = &mut ctx.accounts.index_token;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = IndexTokenService::new();
    
    // 调用服务层执行批量同步操作
    let result = service.batch_sync_index_token(
        index_token,
        &params.sync_type,
        &params.sync_params,
        &params.batch_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetBatchSynced {
        basket_id: index_token.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::IndexToken,
        sync_type: params.sync_type,
        success_count: result.success_count,
        failure_count: result.failure_count,
        operation_cost: result.operation_cost,
        errors: result.errors.clone(),
        batch_params: params.batch_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证批量交易参数
fn validate_batch_trade_index_token_params(params: &BatchTradeIndexTokenParams) -> Result<()> {
    validate_batch_params(&params.batch_params)?;
    validate_strategy_params(&params.strategy_params)?;
    
    for trade_param in &params.trade_params {
        validate_execution_params(trade_param)?;
    }
    
    Ok(())
}

/// 验证批量处理参数
fn validate_batch_process_index_token_params(params: &BatchProcessIndexTokenParams) -> Result<()> {
    validate_batch_params(&params.batch_params)?;
    validate_strategy_params(&params.strategy_params)?;
    
    for process_param in &params.process_params {
        validate_execution_params(process_param)?;
    }
    
    Ok(())
}

/// 验证批量管理参数
fn validate_batch_manage_index_token_params(params: &BatchManageIndexTokenParams) -> Result<()> {
    validate_batch_params(&params.batch_params)?;
    validate_strategy_params(&params.strategy_params)?;
    
    for manage_param in &params.manage_params {
        validate_execution_params(manage_param)?;
    }
    
    Ok(())
}

/// 验证批量同步参数
fn validate_batch_sync_index_token_params(params: &BatchSyncIndexTokenParams) -> Result<()> {
    validate_batch_params(&params.batch_params)?;
    validate_strategy_params(&params.strategy_params)?;
    
    for sync_param in &params.sync_params {
        validate_execution_params(sync_param)?;
    }
    
    Ok(())
}

/// 验证批量参数
fn validate_batch_params(batch_params: &BatchParams) -> Result<()> {
    require!(batch_params.batch_size > 0, AssetError::InvalidParams);
    require!(batch_params.batch_size <= 100, AssetError::InvalidParams);
    require!(batch_params.max_retries > 0, AssetError::InvalidParams);
    require!(batch_params.max_retries <= 10, AssetError::InvalidParams);
    
    Ok(())
}

/// 检查批量操作权限
fn check_batch_operation_authority_permission(
    authority: &Signer,
    index_token: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == index_token.batch_operation_authority,
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