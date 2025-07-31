//! 现实世界资产监管报告指令
//!
//! 本模块实现了现实世界资产的监管报告功能，包括报告生成、提交、审核等。
//!
//! ## 功能特点
//!
//! - **多种报告类型**: 支持定期报告、事件报告、合规报告等
//! - **灵活报告方式**: 支持自动、手动、定期报告生成
//! - **报告信息管理**: 完整的报告记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 监管报告生成
//! - 合规报告提交
//! - 审计报告管理
//! - 风险报告生成

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetRegulatoryReported;
use crate::errors::AssetError;

/// 报告类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ReportType {
    /// 定期报告
    Periodic,
    /// 事件报告
    Event,
    /// 合规报告
    Compliance,
    /// 风险报告
    Risk,
}

/// 报告方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ReportMethod {
    /// 自动报告
    Automatic,
    /// 手动报告
    Manual,
    /// 定期报告
    Periodic,
    /// 事件触发
    EventTriggered,
}

/// 报告信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ReportInfo {
    /// 报告类型
    pub report_type: ReportType,
    /// 报告数据
    pub report_data: Vec<String>,
    /// 报告开始时间
    pub start_time: i64,
    /// 报告结束时间
    pub end_time: i64,
    /// 说明
    pub description: String,
}

/// 报告结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ReportResult {
    /// 报告ID
    pub report_id: u64,
    /// 报告类型
    pub report_type: ReportType,
    /// 报告方式
    pub method: ReportMethod,
    /// 状态
    pub status: bool,
    /// 时间戳
    pub timestamp: i64,
}

/// 监管报告指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RegulatoryReportingParams {
    /// 报告类型
    pub report_type: ReportType,
    /// 报告方式
    pub method: ReportMethod,
    /// 报告信息
    pub info: ReportInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 监管报告指令账户上下文
#[derive(Accounts)]
pub struct RegulatoryReporting<'info> {
    /// 现实世界资产账户，需可变
    #[account(
        mut,
        constraint = rwa.asset_type == AssetType::RWA @ AssetError::InvalidAssetType
    )]
    pub rwa: Account<'info, BasketIndexState>,
    
    /// 报告权限签名者
    #[account(
        constraint = authority.key() == rwa.reporting_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 报告账户
    #[account(mut)]
    pub reporting_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 监管报告指令实现
pub fn regulatory_reporting(
    ctx: Context<RegulatoryReporting>,
    params: RegulatoryReportingParams,
) -> Result<ReportResult> {
    validate_regulatory_reporting_params(&params)?;
    check_reporting_authority_permission(&ctx.accounts.authority, &ctx.accounts.rwa)?;
    let rwa = &mut ctx.accounts.rwa;
    let authority = &ctx.accounts.authority;
    let service = RwaService::new();
    let result = service.regulatory_reporting(
        rwa,
        &params.report_type,
        &params.method,
        &params.info,
        &params.exec_params,
    )?;
    emit!(AssetRegulatoryReported {
        basket_id: rwa.id,
        report_id: result.report_id,
        report_type: params.report_type,
        status: result.status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::RWA,
        exec_params: params.exec_params,
    });
    Ok(result)
}

fn validate_regulatory_reporting_params(params: &RegulatoryReportingParams) -> Result<()> {
    require!(!params.info.report_data.is_empty(), AssetError::InvalidReportData);
    require!(params.info.start_time > 0, AssetError::InvalidReportTime);
    require!(params.info.end_time > 0, AssetError::InvalidReportTime);
    require!(params.info.end_time > params.info.start_time, AssetError::InvalidReportTime);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

fn check_reporting_authority_permission(
    authority: &Signer,
    rwa: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == rwa.reporting_authority,
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