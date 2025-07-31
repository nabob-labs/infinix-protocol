//! 现实世界资产法律合规指令
//!
//! 本模块实现了现实世界资产的法律合规功能，包括法律文件管理、合规检查、监管报告等。
//!
//! ## 功能特点
//!
//! - **多种合规类型**: 支持法律文件、监管合规、审计合规等
//! - **灵活合规方式**: 支持自动、手动、定期合规检查
//! - **合规信息管理**: 完整的合规记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 法律文件管理
//! - 监管合规检查
//! - 审计合规验证
//! - 合规报告生成

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetLegalComplianceUpdated;
use crate::errors::AssetError;

/// 合规类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ComplianceType {
    /// 法律文件
    LegalDocument,
    /// 监管合规
    RegulatoryCompliance,
    /// 审计合规
    AuditCompliance,
    /// 税务合规
    TaxCompliance,
}

/// 合规方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ComplianceMethod {
    /// 自动合规
    Automatic,
    /// 手动合规
    Manual,
    /// 定期合规
    Periodic,
    /// 事件触发
    EventTriggered,
}

/// 合规信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ComplianceInfo {
    /// 合规类型
    pub compliance_type: ComplianceType,
    /// 合规数据
    pub compliance_data: Vec<String>,
    /// 合规开始时间
    pub start_time: i64,
    /// 合规结束时间
    pub end_time: i64,
    /// 说明
    pub description: String,
}

/// 合规结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ComplianceResult {
    /// 合规ID
    pub compliance_id: u64,
    /// 合规类型
    pub compliance_type: ComplianceType,
    /// 合规方式
    pub method: ComplianceMethod,
    /// 状态
    pub status: bool,
    /// 时间戳
    pub timestamp: i64,
}

/// 法律合规指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LegalComplianceParams {
    /// 合规类型
    pub compliance_type: ComplianceType,
    /// 合规方式
    pub method: ComplianceMethod,
    /// 合规信息
    pub info: ComplianceInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 法律合规指令账户上下文
#[derive(Accounts)]
pub struct LegalCompliance<'info> {
    /// 现实世界资产账户，需可变
    #[account(
        mut,
        constraint = rwa.asset_type == AssetType::RWA @ AssetError::InvalidAssetType
    )]
    pub rwa: Account<'info, BasketIndexState>,
    
    /// 合规权限签名者
    #[account(
        constraint = authority.key() == rwa.compliance_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 合规账户
    #[account(mut)]
    pub compliance_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 法律合规指令实现
pub fn legal_compliance(
    ctx: Context<LegalCompliance>,
    params: LegalComplianceParams,
) -> Result<ComplianceResult> {
    validate_legal_compliance_params(&params)?;
    check_compliance_authority_permission(&ctx.accounts.authority, &ctx.accounts.rwa)?;
    let rwa = &mut ctx.accounts.rwa;
    let authority = &ctx.accounts.authority;
    let service = RwaService::new();
    let result = service.legal_compliance(
        rwa,
        &params.compliance_type,
        &params.method,
        &params.info,
        &params.exec_params,
    )?;
    emit!(AssetLegalComplianceUpdated {
        basket_id: rwa.id,
        compliance_id: result.compliance_id,
        compliance_type: params.compliance_type,
        status: result.status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::RWA,
        exec_params: params.exec_params,
    });
    Ok(result)
}

fn validate_legal_compliance_params(params: &LegalComplianceParams) -> Result<()> {
    require!(!params.info.compliance_data.is_empty(), AssetError::InvalidComplianceData);
    require!(params.info.start_time > 0, AssetError::InvalidComplianceTime);
    require!(params.info.end_time > 0, AssetError::InvalidComplianceTime);
    require!(params.info.end_time > params.info.start_time, AssetError::InvalidComplianceTime);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

fn check_compliance_authority_permission(
    authority: &Signer,
    rwa: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == rwa.compliance_authority,
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