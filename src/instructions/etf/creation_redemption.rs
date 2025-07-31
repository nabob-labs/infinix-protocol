//! ETF (Exchange Traded Fund) 创建赎回指令
//!
//! 本模块实现了ETF的创建赎回功能，包括创建单位、赎回单位和批量处理。
//!
//! ## 功能特点
//!
//! - **创建单位**: 授权参与者创建ETF份额
//! - **赎回单位**: 授权参与者赎回ETF份额
//! - **批量处理**: 支持批量创建和赎回
//! - **费用管理**: 自动计算和收取创建赎回费用
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::etf_service::EtfService;
use crate::events::asset_event::AssetCreationRedemption;
use crate::errors::AssetError;

/// 创建赎回类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CreationRedemptionType {
    /// 创建单位
    Creation,
    /// 赎回单位
    Redemption,
    /// 批量创建
    BatchCreation,
    /// 批量赎回
    BatchRedemption,
}

/// 创建赎回方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CreationRedemptionMethod {
    /// 现金方式
    Cash,
    /// 实物方式
    InKind,
    /// 混合方式
    Mixed,
}

/// 创建赎回结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CreationRedemptionResult {
    /// 创建赎回类型
    pub creation_redemption_type: CreationRedemptionType,
    /// 处理数量
    pub amount: u64,
    /// 处理费用
    pub fee: u64,
    /// 实际处理份额
    pub actual_shares: u64,
    /// 处理时间戳
    pub timestamp: i64,
    /// 处理方式
    pub method: CreationRedemptionMethod,
}

/// ETF创建赎回指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CreationRedemptionParams {
    /// 创建赎回类型
    pub creation_redemption_type: CreationRedemptionType,
    /// 创建赎回方式
    pub method: CreationRedemptionMethod,
    /// 处理数量
    pub amount: u64,
    /// 处理价格
    pub price: Option<u64>,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// ETF创建赎回指令账户上下文
#[derive(Accounts)]
pub struct CreationRedemption<'info> {
    /// ETF资产账户，需可变
    #[account(
        mut,
        constraint = etf.asset_type == AssetType::Etf @ AssetError::InvalidAssetType
    )]
    pub etf: Account<'info, BasketIndexState>,
    
    /// 授权参与者签名者
    #[account(
        constraint = authority.key() == etf.authorized_participant @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 授权参与者代币账户
    #[account(mut)]
    pub ap_token_account: Account<'info, TokenAccount>,
    
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

/// ETF创建赎回指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 创建赎回参数，包含类型、方式和数量
///
/// ## 返回值
/// - `Result<CreationRedemptionResult>`: 创建赎回结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidParams`: 无效的参数
/// - `CreationRedemptionFailed`: 创建赎回失败
pub fn creation_redemption(
    ctx: Context<CreationRedemption>,
    params: CreationRedemptionParams,
) -> Result<CreationRedemptionResult> {
    // 参数验证
    validate_creation_redemption_params(&params)?;
    
    // 权限检查
    check_authorized_participant_permission(&ctx.accounts.authority, &ctx.accounts.etf)?;
    
    // 获取账户引用
    let etf = &mut ctx.accounts.etf;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = EtfService::new();
    
    // 调用服务层执行创建赎回操作
    let result = service.creation_redemption(
        etf,
        &params.creation_redemption_type,
        &params.method,
        params.amount,
        params.price,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetCreationRedemption {
        basket_id: etf.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Etf,
        creation_redemption_type: params.creation_redemption_type,
        method: params.method,
        amount: result.amount,
        fee: result.fee,
        actual_shares: result.actual_shares,
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证创建赎回参数
fn validate_creation_redemption_params(params: &CreationRedemptionParams) -> Result<()> {
    require!(params.amount > 0, AssetError::InvalidAmount);
    require!(params.amount <= u64::MAX, AssetError::InvalidAmount);
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
    Ok(())
}

/// 检查授权参与者权限
fn check_authorized_participant_permission(
    authority: &Signer,
    etf: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == etf.authorized_participant,
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