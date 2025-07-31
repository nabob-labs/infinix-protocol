//! ETF (Exchange Traded Fund) 赎回指令
//!
//! 本模块实现了ETF的赎回功能，包括现金赎回、实物赎回和混合赎回。
//!
//! ## 功能特点
//!
//! - **现金赎回**: 赎回ETF份额获得现金
//! - **实物赎回**: 赎回ETF份额获得成分股
//! - **混合赎回**: 现金和实物混合赎回
//! - **赎回费用**: 自动计算和收取赎回费用
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::etf_service::EtfService;
use crate::events::asset_event::AssetRedeemed;
use crate::errors::AssetError;

/// 赎回类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum RedeemType {
    /// 现金赎回
    Cash,
    /// 实物赎回
    InKind,
    /// 混合赎回
    Mixed,
}

/// 赎回方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum RedeemMethod {
    /// 市价赎回
    Market,
    /// 限价赎回
    Limit,
    /// 条件赎回
    Conditional,
}

/// 赎回结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RedeemResult {
    /// 赎回数量
    pub redeem_amount: u64,
    /// 赎回费用
    pub redeem_fee: u64,
    /// 实际赎回份额
    pub actual_shares: u64,
    /// 赎回时间戳
    pub timestamp: i64,
    /// 赎回类型
    pub redeem_type: RedeemType,
}

/// ETF赎回指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RedeemEtfParams {
    /// 赎回类型
    pub redeem_type: RedeemType,
    /// 赎回方式
    pub redeem_method: RedeemMethod,
    /// 赎回数量
    pub amount: u64,
    /// 赎回价格
    pub price: Option<u64>,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// ETF赎回指令账户上下文
#[derive(Accounts)]
pub struct RedeemEtf<'info> {
    /// ETF资产账户，需可变
    #[account(
        mut,
        constraint = etf.asset_type == AssetType::Etf @ AssetError::InvalidAssetType
    )]
    pub etf: Account<'info, BasketIndexState>,
    
    /// 赎回者签名者
    pub redeemer: Signer<'info>,
    
    /// 赎回者代币账户
    #[account(mut)]
    pub redeemer_token_account: Account<'info, TokenAccount>,
    
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

/// ETF赎回指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 赎回参数，包含赎回类型、方式和数量
///
/// ## 返回值
/// - `Result<RedeemResult>`: 赎回结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InvalidParams`: 无效的参数
/// - `InsufficientBalance`: 余额不足
/// - `RedeemFailed`: 赎回失败
pub fn redeem_etf(
    ctx: Context<RedeemEtf>,
    params: RedeemEtfParams,
) -> Result<RedeemResult> {
    // 参数验证
    validate_redeem_etf_params(&params)?;
    
    // 获取账户引用
    let etf = &mut ctx.accounts.etf;
    let redeemer = &ctx.accounts.redeemer;
    
    // 创建服务实例
    let service = EtfService::new();
    
    // 调用服务层执行赎回操作
    let result = service.redeem_etf(
        etf,
        &params.redeem_type,
        &params.redeem_method,
        params.amount,
        params.price,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetRedeemed {
        basket_id: etf.id,
        redeemer: redeemer.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Etf,
        redeem_type: params.redeem_type,
        redeem_method: params.redeem_method,
        redeem_amount: result.redeem_amount,
        redeem_fee: result.redeem_fee,
        actual_shares: result.actual_shares,
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证赎回参数
fn validate_redeem_etf_params(params: &RedeemEtfParams) -> Result<()> {
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