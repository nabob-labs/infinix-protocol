//! 现实世界资产估值验证指令
//!
//! 本模块实现了现实世界资产的估值验证功能，包括估值模型、验证机制、估值报告等。
//!
//! ## 功能特点
//!
//! - **多种估值类型**: 支持市场估值、收益估值、成本估值等
//! - **灵活验证方式**: 支持自动、手动、定期估值验证
//! - **估值信息管理**: 完整的估值记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 资产估值验证
//! - 估值模型管理
//! - 估值报告生成
//! - 估值风险控制

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetValuationVerified;
use crate::errors::AssetError;

/// 估值类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ValuationType {
    /// 市场估值
    MarketValue,
    /// 收益估值
    IncomeValue,
    /// 成本估值
    CostValue,
    /// 重置成本
    ReplacementCost,
}

/// 估值方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ValuationMethod {
    /// 自动估值
    Automatic,
    /// 手动估值
    Manual,
    /// 定期估值
    Periodic,
    /// 事件触发
    EventTriggered,
}

/// 估值信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ValuationInfo {
    /// 估值类型
    pub valuation_type: ValuationType,
    /// 估值金额
    pub valuation_amount: f64,
    /// 估值模型
    pub valuation_model: String,
    /// 估值开始时间
    pub start_time: i64,
    /// 估值结束时间
    pub end_time: i64,
    /// 说明
    pub description: String,
}

/// 估值结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ValuationResult {
    /// 估值ID
    pub valuation_id: u64,
    /// 估值类型
    pub valuation_type: ValuationType,
    /// 估值方式
    pub method: ValuationMethod,
    /// 状态
    pub status: bool,
    /// 时间戳
    pub timestamp: i64,
}

/// 估值验证指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ValuationVerificationParams {
    /// 估值类型
    pub valuation_type: ValuationType,
    /// 估值方式
    pub method: ValuationMethod,
    /// 估值信息
    pub info: ValuationInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 估值验证指令账户上下文
#[derive(Accounts)]
pub struct ValuationVerification<'info> {
    /// 现实世界资产账户，需可变
    #[account(
        mut,
        constraint = rwa.asset_type == AssetType::RWA @ AssetError::InvalidAssetType
    )]
    pub rwa: Account<'info, BasketIndexState>,
    
    /// 估值权限签名者
    #[account(
        constraint = authority.key() == rwa.valuation_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 估值账户
    #[account(mut)]
    pub valuation_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 估值验证指令实现
pub fn valuation_verification(
    ctx: Context<ValuationVerification>,
    params: ValuationVerificationParams,
) -> Result<ValuationResult> {
    validate_valuation_verification_params(&params)?;
    check_valuation_authority_permission(&ctx.accounts.authority, &ctx.accounts.rwa)?;
    let rwa = &mut ctx.accounts.rwa;
    let authority = &ctx.accounts.authority;
    let service = RwaService::new();
    let result = service.valuation_verification(
        rwa,
        &params.valuation_type,
        &params.method,
        &params.info,
        &params.exec_params,
    )?;
    emit!(AssetValuationVerified {
        basket_id: rwa.id,
        valuation_id: result.valuation_id,
        valuation_type: params.valuation_type,
        status: result.status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::RWA,
        exec_params: params.exec_params,
    });
    Ok(result)
}

fn validate_valuation_verification_params(params: &ValuationVerificationParams) -> Result<()> {
    require!(params.info.valuation_amount > 0.0, AssetError::InvalidValuationAmount);
    require!(!params.info.valuation_model.is_empty(), AssetError::InvalidValuationModel);
    require!(params.info.start_time > 0, AssetError::InvalidValuationTime);
    require!(params.info.end_time > 0, AssetError::InvalidValuationTime);
    require!(params.info.end_time > params.info.start_time, AssetError::InvalidValuationTime);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

fn check_valuation_authority_permission(
    authority: &Signer,
    rwa: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == rwa.valuation_authority,
        AssetError::InsufficientAuthority
    );
    Ok(())
}

fn validate_execution_params(exec_params: &ExecutionParams) -> Result<()> {
    require!(exec_params.slippage_tolerance > 0.0, AssetError::InvalidParams);
    require!(exec_params.slippage_tolerance <= 1.0, AssetError::InvalidParams);
    require!(exec_params.max_retries > 0, AssetError::InvalidParams);
    require!(exec_params.max_retries <= 10, AssetError::InvalidParams);
    Ok(())
} 