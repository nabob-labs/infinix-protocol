//! 现实世界资产(RWA)批量操作指令
//!
//! 本模块实现了现实世界资产的批量操作功能，包括批量交易、批量处理、批量管理和批量同步。
//!
//! ## 功能特点
//!
//! - **批量交易**: 批量执行现实世界资产交易操作
//! - **批量处理**: 批量处理现实世界资产相关操作
//! - **批量管理**: 批量管理现实世界资产配置和参数
//! - **批量同步**: 批量同步现实世界资产数据和状态
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::rwa_service::RwaService;
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
    /// 批量代币化
    Tokenization,
    /// 批量退出
    Exit,
}

/// 批量处理类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchProcessType {
    /// 批量合规处理
    ComplianceProcessing,
    /// 批量托管处理
    CustodyProcessing,
    /// 批量估值处理
    ValuationProcessing,
    /// 批量报告处理
    ReportingProcessing,
}

/// 批量管理类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchManageType {
    /// 批量参数更新
    ParameterUpdate,
    /// 批量权限管理
    AuthorityManagement,
    /// 批量配置更新
    ConfigurationUpdate,
    /// 批量策略更新
    StrategyUpdate,
}

/// 批量同步类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchSyncType {
    /// 批量数据同步
    DataSync,
    /// 批量状态同步
    StateSync,
    /// 批量价格同步
    PriceSync,
    /// 批量事件同步
    EventSync,
}

/// 批量操作结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchOperationResult {
    /// 操作类型
    pub operation_type: BatchOperationType,
    /// 操作数量
    pub operation_count: u64,
    /// 成功数量
    pub success_count: u64,
    /// 失败数量
    pub failure_count: u64,
    /// 操作成本
    pub operation_cost: u64,
    /// 操作时间戳
    pub timestamp: i64,
}

/// 批量交易参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchTradeRwaParams {
    /// 交易类型
    pub trade_type: BatchTradeType,
    /// 交易数量
    pub trade_count: u64,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// 批量处理参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchProcessRwaParams {
    /// 处理类型
    pub process_type: BatchProcessType,
    /// 处理数量
    pub process_count: u64,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 批量管理参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchManageRwaParams {
    /// 管理类型
    pub manage_type: BatchManageType,
    /// 管理数量
    pub manage_count: u64,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 批量同步参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchSyncRwaParams {
    /// 同步类型
    pub sync_type: BatchSyncType,
    /// 同步数量
    pub sync_count: u64,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 批量操作账户上下文
#[derive(Accounts)]
pub struct BatchRwa<'info> {
    /// 现实世界资产账户，需可变
    #[account(
        mut,
        constraint = rwa.asset_type == AssetType::RWA @ AssetError::InvalidAssetType
    )]
    pub rwa: Account<'info, BasketIndexState>,
    
    /// 批量操作权限签名者
    #[account(
        constraint = authority.key() == rwa.batch_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 批量操作账户
    #[account(mut)]
    pub batch_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 批量交易指令实现
pub fn batch_trade_rwa(
    ctx: Context<BatchRwa>,
    params: BatchTradeRwaParams,
) -> Result<BatchOperationResult> {
    validate_batch_trade_rwa_params(&params)?;
    check_batch_authority_permission(&ctx.accounts.authority, &ctx.accounts.rwa)?;
    let rwa = &mut ctx.accounts.rwa;
    let authority = &ctx.accounts.authority;
    let service = RwaService::new();
    let result = service.batch_trade_rwa(
        rwa,
        &params.trade_type,
        params.trade_count,
        &params.exec_params,
        &params.strategy_params,
    )?;
    emit!(AssetBatchTraded {
        basket_id: rwa.id,
        trade_type: params.trade_type,
        trade_count: params.trade_count,
        success_count: result.success_count,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::RWA,
        exec_params: params.exec_params,
    });
    Ok(result)
}

/// 批量处理指令实现
pub fn batch_process_rwa(
    ctx: Context<BatchRwa>,
    params: BatchProcessRwaParams,
) -> Result<BatchOperationResult> {
    validate_batch_process_rwa_params(&params)?;
    check_batch_authority_permission(&ctx.accounts.authority, &ctx.accounts.rwa)?;
    let rwa = &mut ctx.accounts.rwa;
    let authority = &ctx.accounts.authority;
    let service = RwaService::new();
    let result = service.batch_process_rwa(
        rwa,
        &params.process_type,
        params.process_count,
        &params.exec_params,
    )?;
    emit!(AssetBatchProcessed {
        basket_id: rwa.id,
        process_type: params.process_type,
        process_count: params.process_count,
        success_count: result.success_count,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::RWA,
        exec_params: params.exec_params,
    });
    Ok(result)
}

/// 批量管理指令实现
pub fn batch_manage_rwa(
    ctx: Context<BatchRwa>,
    params: BatchManageRwaParams,
) -> Result<BatchOperationResult> {
    validate_batch_manage_rwa_params(&params)?;
    check_batch_authority_permission(&ctx.accounts.authority, &ctx.accounts.rwa)?;
    let rwa = &mut ctx.accounts.rwa;
    let authority = &ctx.accounts.authority;
    let service = RwaService::new();
    let result = service.batch_manage_rwa(
        rwa,
        &params.manage_type,
        params.manage_count,
        &params.exec_params,
    )?;
    emit!(AssetBatchManaged {
        basket_id: rwa.id,
        manage_type: params.manage_type,
        manage_count: params.manage_count,
        success_count: result.success_count,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::RWA,
        exec_params: params.exec_params,
    });
    Ok(result)
}

/// 批量同步指令实现
pub fn batch_sync_rwa(
    ctx: Context<BatchRwa>,
    params: BatchSyncRwaParams,
) -> Result<BatchOperationResult> {
    validate_batch_sync_rwa_params(&params)?;
    check_batch_authority_permission(&ctx.accounts.authority, &ctx.accounts.rwa)?;
    let rwa = &mut ctx.accounts.rwa;
    let authority = &ctx.accounts.authority;
    let service = RwaService::new();
    let result = service.batch_sync_rwa(
        rwa,
        &params.sync_type,
        params.sync_count,
        &params.exec_params,
    )?;
    emit!(AssetBatchSynced {
        basket_id: rwa.id,
        sync_type: params.sync_type,
        sync_count: params.sync_count,
        success_count: result.success_count,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::RWA,
        exec_params: params.exec_params,
    });
    Ok(result)
}

/// 验证批量交易参数
fn validate_batch_trade_rwa_params(params: &BatchTradeRwaParams) -> Result<()> {
    require!(params.trade_count > 0, AssetError::InvalidBatchTradeCount);
    require!(params.trade_count <= 1000, AssetError::InvalidBatchTradeCount);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

/// 验证批量处理参数
fn validate_batch_process_rwa_params(params: &BatchProcessRwaParams) -> Result<()> {
    require!(params.process_count > 0, AssetError::InvalidBatchProcessCount);
    require!(params.process_count <= 1000, AssetError::InvalidBatchProcessCount);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

/// 验证批量管理参数
fn validate_batch_manage_rwa_params(params: &BatchManageRwaParams) -> Result<()> {
    require!(params.manage_count > 0, AssetError::InvalidBatchManageCount);
    require!(params.manage_count <= 1000, AssetError::InvalidBatchManageCount);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

/// 验证批量同步参数
fn validate_batch_sync_rwa_params(params: &BatchSyncRwaParams) -> Result<()> {
    require!(params.sync_count > 0, AssetError::InvalidBatchSyncCount);
    require!(params.sync_count <= 1000, AssetError::InvalidBatchSyncCount);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

/// 检查批量操作权限
fn check_batch_authority_permission(
    authority: &Signer,
    rwa: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == rwa.batch_authority,
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