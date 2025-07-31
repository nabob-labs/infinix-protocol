//! 现实世界资产法律文件指令
//!
//! 本模块实现了现实世界资产的法律文件管理功能，包括文件上传、验证、存储等。
//!
//! ## 功能特点
//!
//! - **多种文件类型**: 支持合同文件、法律文件、证明文件等
//! - **灵活管理方式**: 支持自动、手动、定期文件管理
//! - **文件信息管理**: 完整的文件记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 法律文件上传
//! - 合同文件管理
//! - 证明文件验证
//! - 文件存储管理

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetLegalDocumentationUpdated;
use crate::errors::AssetError;

/// 文件类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum DocumentType {
    /// 合同文件
    Contract,
    /// 法律文件
    Legal,
    /// 证明文件
    Certificate,
    /// 其他文件
    Other,
}

/// 文件管理方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum DocumentMethod {
    /// 自动管理
    Automatic,
    /// 手动管理
    Manual,
    /// 定期管理
    Periodic,
    /// 事件触发
    EventTriggered,
}

/// 文件信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DocumentInfo {
    /// 文件类型
    pub document_type: DocumentType,
    /// 文件数据
    pub document_data: Vec<String>,
    /// 文件开始时间
    pub start_time: i64,
    /// 文件结束时间
    pub end_time: i64,
    /// 说明
    pub description: String,
}

/// 文件结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DocumentResult {
    /// 文件ID
    pub document_id: u64,
    /// 文件类型
    pub document_type: DocumentType,
    /// 文件管理方式
    pub method: DocumentMethod,
    /// 状态
    pub status: bool,
    /// 时间戳
    pub timestamp: i64,
}

/// 法律文件指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LegalDocumentationParams {
    /// 文件类型
    pub document_type: DocumentType,
    /// 文件管理方式
    pub method: DocumentMethod,
    /// 文件信息
    pub info: DocumentInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 法律文件指令账户上下文
#[derive(Accounts)]
pub struct LegalDocumentation<'info> {
    /// 现实世界资产账户，需可变
    #[account(
        mut,
        constraint = rwa.asset_type == AssetType::RWA @ AssetError::InvalidAssetType
    )]
    pub rwa: Account<'info, BasketIndexState>,
    
    /// 文件权限签名者
    #[account(
        constraint = authority.key() == rwa.document_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 文件账户
    #[account(mut)]
    pub document_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 法律文件指令实现
pub fn legal_documentation(
    ctx: Context<LegalDocumentation>,
    params: LegalDocumentationParams,
) -> Result<DocumentResult> {
    validate_legal_documentation_params(&params)?;
    check_document_authority_permission(&ctx.accounts.authority, &ctx.accounts.rwa)?;
    let rwa = &mut ctx.accounts.rwa;
    let authority = &ctx.accounts.authority;
    let service = RwaService::new();
    let result = service.legal_documentation(
        rwa,
        &params.document_type,
        &params.method,
        &params.info,
        &params.exec_params,
    )?;
    emit!(AssetLegalDocumentationUpdated {
        basket_id: rwa.id,
        document_id: result.document_id,
        document_type: params.document_type,
        status: result.status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::RWA,
        exec_params: params.exec_params,
    });
    Ok(result)
}

fn validate_legal_documentation_params(params: &LegalDocumentationParams) -> Result<()> {
    require!(!params.info.document_data.is_empty(), AssetError::InvalidDocumentData);
    require!(params.info.start_time > 0, AssetError::InvalidDocumentTime);
    require!(params.info.end_time > 0, AssetError::InvalidDocumentTime);
    require!(params.info.end_time > params.info.start_time, AssetError::InvalidDocumentTime);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

fn check_document_authority_permission(
    authority: &Signer,
    rwa: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == rwa.document_authority,
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