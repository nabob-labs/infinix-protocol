//! ETF (Exchange Traded Fund) 批量操作指令
//!
//! 本模块实现了ETF的批量操作功能，包括批量交易、批量处理、批量管理和批量同步。
//!
//! ## 功能特点
//!
//! - **批量交易**: 批量执行ETF交易操作
//! - **批量处理**: 批量处理ETF相关操作
//! - **批量管理**: 批量管理ETF配置和参数
//! - **批量同步**: 批量同步ETF数据和状态
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::etf_service::EtfService;
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
    /// 批量申购
    Subscribe,
    /// 批量赎回
    Redeem,
}

/// 批量处理类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BatchProcessType {
    /// 批量净值计算
    NavCalculation,
    /// 批量费用计算
    FeeCalculation,
    /// 批量再平衡
    Rebalancing,
    /// 批量报告生成
    ReportGeneration,
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

/// 批量交易ETF参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchTradeEtfParams {
    /// 批量交易类型
    pub trade_type: BatchTradeType,
    /// 交易数量
    pub amount: u64,
    /// 交易价格
    pub price: Option<f64>,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// 批量处理ETF参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchProcessEtfParams {
    /// 批量处理类型
    pub process_type: BatchProcessType,
    /// 处理周期
    pub process_period: u64,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// 批量管理ETF参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchManageEtfParams {
    /// 批量管理类型
    pub manage_type: BatchManageType,
    /// 管理参数
    pub management_params: Vec<String>,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// 批量同步ETF参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BatchSyncEtfParams {
    /// 批量同步类型
    pub sync_type: BatchSyncType,
    /// 同步范围
    pub sync_range: u64,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// ETF批量操作指令账户上下文
#[derive(Accounts)]
pub struct BatchEtf<'info> {
    /// ETF资产账户，需可变
    #[account(
        mut,
        constraint = etf.asset_type == AssetType::Etf @ AssetError::InvalidAssetType
    )]
    pub etf: Account<'info, BasketIndexState>,
    
    /// 批量操作权限签名者
    #[account(
        constraint = authority.key() == etf.batch_operation_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 批量操作账户
    #[account(mut)]
    pub batch_operation_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// ETF批量交易指令实现
pub fn batch_trade_etf(
    ctx: Context<BatchEtf>,
    params: BatchTradeEtfParams,
) -> Result<BatchOperationResult> {
    // 参数验证
    validate_batch_trade_etf_params(&params)?;
    
    // 权限检查
    check_batch_operation_authority_permission(&ctx.accounts.authority, &ctx.accounts.etf)?;
    
    // 获取账户引用
    let etf = &mut ctx.accounts.etf;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = EtfService::new();
    
    // 调用服务层执行批量交易操作
    let result = service.batch_trade_etf(
        etf,
        &params.trade_type,
        params.amount,
        params.price,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetBatchTraded {
        basket_id: etf.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Etf,
        trade_type: params.trade_type,
        amount: params.amount,
        price: params.price,
        operation_result: result.clone(),
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// ETF批量处理指令实现
pub fn batch_process_etf(
    ctx: Context<BatchEtf>,
    params: BatchProcessEtfParams,
) -> Result<BatchOperationResult> {
    // 参数验证
    validate_batch_process_etf_params(&params)?;
    
    // 权限检查
    check_batch_operation_authority_permission(&ctx.accounts.authority, &ctx.accounts.etf)?;
    
    // 获取账户引用
    let etf = &mut ctx.accounts.etf;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = EtfService::new();
    
    // 调用服务层执行批量处理操作
    let result = service.batch_process_etf(
        etf,
        &params.process_type,
        params.process_period,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetBatchProcessed {
        basket_id: etf.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Etf,
        process_type: params.process_type,
        process_period: params.process_period,
        operation_result: result.clone(),
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// ETF批量管理指令实现
pub fn batch_manage_etf(
    ctx: Context<BatchEtf>,
    params: BatchManageEtfParams,
) -> Result<BatchOperationResult> {
    // 参数验证
    validate_batch_manage_etf_params(&params)?;
    
    // 权限检查
    check_batch_operation_authority_permission(&ctx.accounts.authority, &ctx.accounts.etf)?;
    
    // 获取账户引用
    let etf = &mut ctx.accounts.etf;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = EtfService::new();
    
    // 调用服务层执行批量管理操作
    let result = service.batch_manage_etf(
        etf,
        &params.manage_type,
        &params.management_params,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetBatchManaged {
        basket_id: etf.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Etf,
        manage_type: params.manage_type,
        management_params: params.management_params.clone(),
        operation_result: result.clone(),
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// ETF批量同步指令实现
pub fn batch_sync_etf(
    ctx: Context<BatchEtf>,
    params: BatchSyncEtfParams,
) -> Result<BatchOperationResult> {
    // 参数验证
    validate_batch_sync_etf_params(&params)?;
    
    // 权限检查
    check_batch_operation_authority_permission(&ctx.accounts.authority, &ctx.accounts.etf)?;
    
    // 获取账户引用
    let etf = &mut ctx.accounts.etf;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = EtfService::new();
    
    // 调用服务层执行批量同步操作
    let result = service.batch_sync_etf(
        etf,
        &params.sync_type,
        params.sync_range,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetBatchSynced {
        basket_id: etf.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Etf,
        sync_type: params.sync_type,
        sync_range: params.sync_range,
        operation_result: result.clone(),
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证批量交易ETF参数
fn validate_batch_trade_etf_params(params: &BatchTradeEtfParams) -> Result<()> {
    require!(params.amount > 0, AssetError::InvalidAmount);
    require!(params.amount <= u64::MAX, AssetError::InvalidAmount);
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
    Ok(())
}

/// 验证批量处理ETF参数
fn validate_batch_process_etf_params(params: &BatchProcessEtfParams) -> Result<()> {
    require!(params.process_period > 0, AssetError::InvalidParams);
    require!(params.process_period <= 365 * 24 * 60 * 60, AssetError::InvalidParams); // 最大1年
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
    Ok(())
}

/// 验证批量管理ETF参数
fn validate_batch_manage_etf_params(params: &BatchManageEtfParams) -> Result<()> {
    require!(!params.management_params.is_empty(), AssetError::InvalidParams);
    require!(params.management_params.len() <= 100, AssetError::InvalidParams);
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
    Ok(())
}

/// 验证批量同步ETF参数
fn validate_batch_sync_etf_params(params: &BatchSyncEtfParams) -> Result<()> {
    require!(params.sync_range > 0, AssetError::InvalidParams);
    require!(params.sync_range <= 365 * 24 * 60 * 60, AssetError::InvalidParams); // 最大1年
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
    Ok(())
}

/// 检查批量操作权限
fn check_batch_operation_authority_permission(
    authority: &Signer,
    etf: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == etf.batch_operation_authority,
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