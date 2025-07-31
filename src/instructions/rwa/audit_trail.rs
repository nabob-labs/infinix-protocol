//! 现实世界资产审计追踪指令
//!
//! 本模块实现了现实世界资产的审计追踪功能，包括审计日志、追踪记录、审计报告等。
//!
//! ## 功能特点
//!
//! - **多种审计类型**: 支持操作审计、交易审计、合规审计等
//! - **灵活追踪方式**: 支持自动、手动、定期审计追踪
//! - **审计信息管理**: 完整的审计记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 操作审计追踪
//! - 交易审计记录
//! - 合规审计验证
//! - 审计报告生成

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetAuditTrailUpdated;
use crate::errors::AssetError;

/// 审计类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum AuditType {
    /// 操作审计
    Operation,
    /// 交易审计
    Transaction,
    /// 合规审计
    Compliance,
    /// 系统审计
    System,
}

/// 审计方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum AuditMethod {
    /// 自动审计
    Automatic,
    /// 手动审计
    Manual,
    /// 定期审计
    Periodic,
    /// 事件触发
    EventTriggered,
}

/// 审计信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AuditInfo {
    /// 审计类型
    pub audit_type: AuditType,
    /// 审计数据
    pub audit_data: Vec<String>,
    /// 审计开始时间
    pub start_time: i64,
    /// 审计结束时间
    pub end_time: i64,
    /// 说明
    pub description: String,
}

/// 审计结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AuditResult {
    /// 审计ID
    pub audit_id: u64,
    /// 审计类型
    pub audit_type: AuditType,
    /// 审计方式
    pub method: AuditMethod,
    /// 状态
    pub status: bool,
    /// 时间戳
    pub timestamp: i64,
}

/// 审计追踪指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AuditTrailParams {
    /// 审计类型
    pub audit_type: AuditType,
    /// 审计方式
    pub method: AuditMethod,
    /// 审计信息
    pub info: AuditInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 审计追踪指令账户上下文
#[derive(Accounts)]
pub struct AuditTrail<'info> {
    /// 现实世界资产账户，需可变
    #[account(
        mut,
        constraint = rwa.asset_type == AssetType::RWA @ AssetError::InvalidAssetType
    )]
    pub rwa: Account<'info, BasketIndexState>,
    
    /// 审计权限签名者
    #[account(
        constraint = authority.key() == rwa.audit_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 审计账户
    #[account(mut)]
    pub audit_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 审计追踪指令实现
pub fn audit_trail(
    ctx: Context<AuditTrail>,
    params: AuditTrailParams,
) -> Result<AuditResult> {
    validate_audit_trail_params(&params)?;
    check_audit_authority_permission(&ctx.accounts.authority, &ctx.accounts.rwa)?;
    let rwa = &mut ctx.accounts.rwa;
    let authority = &ctx.accounts.authority;
    let service = RwaService::new();
    let result = service.audit_trail(
        rwa,
        &params.audit_type,
        &params.method,
        &params.info,
        &params.exec_params,
    )?;
    emit!(AssetAuditTrailUpdated {
        basket_id: rwa.id,
        audit_id: result.audit_id,
        audit_type: params.audit_type,
        status: result.status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::RWA,
        exec_params: params.exec_params,
    });
    Ok(result)
}

fn validate_audit_trail_params(params: &AuditTrailParams) -> Result<()> {
    require!(!params.info.audit_data.is_empty(), AssetError::InvalidAuditData);
    require!(params.info.start_time > 0, AssetError::InvalidAuditTime);
    require!(params.info.end_time > 0, AssetError::InvalidAuditTime);
    require!(params.info.end_time > params.info.start_time, AssetError::InvalidAuditTime);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

fn check_audit_authority_permission(
    authority: &Signer,
    rwa: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == rwa.audit_authority,
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