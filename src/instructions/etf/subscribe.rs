//! ETF (Exchange Traded Fund) 申购指令
//!
//! 本模块实现了ETF的申购功能，包括现金申购、实物申购和混合申购。
//!
//! ## 功能特点
//!
//! - **现金申购**: 使用现金申购ETF份额
//! - **实物申购**: 使用成分股申购ETF份额
//! - **混合申购**: 现金和实物混合申购
//! - **申购费用**: 自动计算和收取申购费用
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::etf_service::EtfService;
use crate::events::asset_event::AssetSubscribed;
use crate::errors::AssetError;

/// 申购类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum SubscribeType {
    /// 现金申购
    Cash,
    /// 实物申购
    InKind,
    /// 混合申购
    Mixed,
}

/// 申购方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum SubscribeMethod {
    /// 市价申购
    Market,
    /// 限价申购
    Limit,
    /// 条件申购
    Conditional,
}

/// 申购结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SubscribeResult {
    /// 申购数量
    pub subscribe_amount: u64,
    /// 申购费用
    pub subscribe_fee: u64,
    /// 实际获得份额
    pub actual_shares: u64,
    /// 申购时间戳
    pub timestamp: i64,
    /// 申购类型
    pub subscribe_type: SubscribeType,
}

/// ETF申购指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SubscribeEtfParams {
    /// 申购类型
    pub subscribe_type: SubscribeType,
    /// 申购方式
    pub subscribe_method: SubscribeMethod,
    /// 申购数量
    pub amount: u64,
    /// 申购价格
    pub price: Option<u64>,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// ETF申购指令账户上下文
#[derive(Accounts)]
pub struct SubscribeEtf<'info> {
    /// ETF资产账户，需可变
    #[account(
        mut,
        constraint = etf.asset_type == AssetType::Etf @ AssetError::InvalidAssetType
    )]
    pub etf: Account<'info, BasketIndexState>,
    
    /// 申购者签名者
    pub subscriber: Signer<'info>,
    
    /// 申购者代币账户
    #[account(mut)]
    pub subscriber_token_account: Account<'info, TokenAccount>,
    
    /// ETF代币账户
    #[account(mut)]
    pub etf_token_account: Account<'info, TokenAccount>,
    
    /// 成分股代币账户列表
    #[account(mut)]
    pub constituent_token_accounts: Vec<Account<'info, TokenAccount>>,
    
    /// DEX程序
    pub dex_program: Program<'info, DexProgram>,
    
    /// 价格预言机程序
    pub oracle_program: Program<'info, OracleProgram>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
    
    /// 关联代币程序
    pub associated_token_program: Program<'info, AssociatedToken>,
}

/// ETF申购指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 申购参数，包含申购类型、方式和数量
///
/// ## 返回值
/// - `Result<SubscribeResult>`: 申购结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InvalidParams`: 无效的参数
/// - `InsufficientBalance`: 余额不足
/// - `SubscribeFailed`: 申购失败
pub fn subscribe_etf(
    ctx: Context<SubscribeEtf>,
    params: SubscribeEtfParams,
) -> Result<SubscribeResult> {
    // 参数验证
    validate_subscribe_etf_params(&params)?;
    
    // 获取账户引用
    let etf = &mut ctx.accounts.etf;
    let subscriber = &ctx.accounts.subscriber;
    
    // 创建服务实例
    let service = EtfService::new();
    
    // 调用服务层执行申购操作
    let result = service.subscribe_etf(
        etf,
        &params.subscribe_type,
        &params.subscribe_method,
        params.amount,
        params.price,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetSubscribed {
        basket_id: etf.id,
        subscriber: subscriber.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Etf,
        subscribe_type: params.subscribe_type,
        subscribe_method: params.subscribe_method,
        subscribe_amount: result.subscribe_amount,
        subscribe_fee: result.subscribe_fee,
        actual_shares: result.actual_shares,
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证申购参数
fn validate_subscribe_etf_params(params: &SubscribeEtfParams) -> Result<()> {
    require!(params.amount > 0, AssetError::InvalidAmount);
    require!(params.amount <= u64::MAX, AssetError::InvalidAmount);
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
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