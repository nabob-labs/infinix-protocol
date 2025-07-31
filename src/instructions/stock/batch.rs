//! 股票 (Stock) 批量操作指令
//!
//! 本模块实现了股票的批量操作功能，包括批量交易、批量处理、批量管理和批量同步。
//!
//! ## 功能特点
//!
//! - **批量交易**: 批量执行股票交易操作
//! - **批量处理**: 批量处理股票相关操作
//! - **批量管理**: 批量管理股票配置和参数
//! - **批量同步**: 批量同步股票数据和状态
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::stock_service::StockService;
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
    /// 批量限价单
    LimitOrder,
    /// 批量止损单
    StopOrder,
}

/// 批量处理类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchProcessType {
    /// 批量分红处理
    DividendProcessing,
    /// 批量投票处理
    VotingProcessing,
    /// 批量公司行为处理
    CorporateActionProcessing,
    /// 批量合规处理
    ComplianceProcessing,
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
pub struct BatchTradeStockParams {
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
pub struct BatchProcessStockParams {
    /// 处理类型
    pub process_type: BatchProcessType,
    /// 处理数量
    pub process_count: u64,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 批量管理参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchManageStockParams {
    /// 管理类型
    pub manage_type: BatchManageType,
    /// 管理数量
    pub manage_count: u64,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 批量同步参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchSyncStockParams {
    /// 同步类型
    pub sync_type: BatchSyncType,
    /// 同步数量
    pub sync_count: u64,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 批量操作账户上下文
#[derive(Accounts)]
pub struct BatchStock<'info> {
    /// 股票资产账户，需可变
    #[account(
        mut,
        constraint = stock.asset_type == AssetType::Stock @ AssetError::InvalidAssetType
    )]
    pub stock: Account<'info, BasketIndexState>,
    
    /// 批量操作权限签名者
    #[account(
        constraint = authority.key() == stock.batch_authority @ AssetError::InsufficientAuthority
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
pub fn batch_trade_stock(
    ctx: Context<BatchStock>,
    params: BatchTradeStockParams,
) -> Result<BatchOperationResult> {
    validate_batch_trade_stock_params(&params)?;
    check_batch_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    let service = StockService::new();
    let result = service.batch_trade_stock(
        stock,
        &params.trade_type,
        params.trade_count,
        &params.exec_params,
        &params.strategy_params,
    )?;
    emit!(AssetBatchTraded {
        basket_id: stock.id,
        trade_type: params.trade_type,
        trade_count: params.trade_count,
        success_count: result.success_count,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Stock,
        exec_params: params.exec_params,
    });
    Ok(result)
}

/// 批量处理指令实现
pub fn batch_process_stock(
    ctx: Context<BatchStock>,
    params: BatchProcessStockParams,
) -> Result<BatchOperationResult> {
    validate_batch_process_stock_params(&params)?;
    check_batch_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    let service = StockService::new();
    let result = service.batch_process_stock(
        stock,
        &params.process_type,
        params.process_count,
        &params.exec_params,
    )?;
    emit!(AssetBatchProcessed {
        basket_id: stock.id,
        process_type: params.process_type,
        process_count: params.process_count,
        success_count: result.success_count,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Stock,
        exec_params: params.exec_params,
    });
    Ok(result)
}

/// 批量管理指令实现
pub fn batch_manage_stock(
    ctx: Context<BatchStock>,
    params: BatchManageStockParams,
) -> Result<BatchOperationResult> {
    validate_batch_manage_stock_params(&params)?;
    check_batch_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    let service = StockService::new();
    let result = service.batch_manage_stock(
        stock,
        &params.manage_type,
        params.manage_count,
        &params.exec_params,
    )?;
    emit!(AssetBatchManaged {
        basket_id: stock.id,
        manage_type: params.manage_type,
        manage_count: params.manage_count,
        success_count: result.success_count,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Stock,
        exec_params: params.exec_params,
    });
    Ok(result)
}

/// 批量同步指令实现
pub fn batch_sync_stock(
    ctx: Context<BatchStock>,
    params: BatchSyncStockParams,
) -> Result<BatchOperationResult> {
    validate_batch_sync_stock_params(&params)?;
    check_batch_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    let service = StockService::new();
    let result = service.batch_sync_stock(
        stock,
        &params.sync_type,
        params.sync_count,
        &params.exec_params,
    )?;
    emit!(AssetBatchSynced {
        basket_id: stock.id,
        sync_type: params.sync_type,
        sync_count: params.sync_count,
        success_count: result.success_count,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Stock,
        exec_params: params.exec_params,
    });
    Ok(result)
}

/// 验证批量交易参数
fn validate_batch_trade_stock_params(params: &BatchTradeStockParams) -> Result<()> {
    require!(params.trade_count > 0, AssetError::InvalidBatchTradeCount);
    require!(params.trade_count <= 1000, AssetError::InvalidBatchTradeCount);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

/// 验证批量处理参数
fn validate_batch_process_stock_params(params: &BatchProcessStockParams) -> Result<()> {
    require!(params.process_count > 0, AssetError::InvalidBatchProcessCount);
    require!(params.process_count <= 1000, AssetError::InvalidBatchProcessCount);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

/// 验证批量管理参数
fn validate_batch_manage_stock_params(params: &BatchManageStockParams) -> Result<()> {
    require!(params.manage_count > 0, AssetError::InvalidBatchManageCount);
    require!(params.manage_count <= 1000, AssetError::InvalidBatchManageCount);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

/// 验证批量同步参数
fn validate_batch_sync_stock_params(params: &BatchSyncStockParams) -> Result<()> {
    require!(params.sync_count > 0, AssetError::InvalidBatchSyncCount);
    require!(params.sync_count <= 1000, AssetError::InvalidBatchSyncCount);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

/// 检查批量操作权限
fn check_batch_authority_permission(
    authority: &Signer,
    stock: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == stock.batch_authority,
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