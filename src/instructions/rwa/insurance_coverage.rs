//! 现实世界资产保险覆盖指令
//!
//! 本模块实现了现实世界资产的保险覆盖功能，包括保险管理、理赔处理、保险报告等。
//!
//! ## 功能特点
//!
//! - **多种保险类型**: 支持财产保险、责任保险、信用保险等
//! - **灵活保险方式**: 支持自动、手动、定期保险管理
//! - **保险信息管理**: 完整的保险记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 保险覆盖管理
//! - 理赔处理
//! - 保险报告生成
//! - 风险管理

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetInsuranceCoverageUpdated;
use crate::errors::AssetError;

/// 保险类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum InsuranceType {
    /// 财产保险
    Property,
    /// 责任保险
    Liability,
    /// 信用保险
    Credit,
    /// 其他保险
    Other,
}

/// 保险方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum InsuranceMethod {
    /// 自动保险
    Automatic,
    /// 手动保险
    Manual,
    /// 定期保险
    Periodic,
    /// 事件触发
    EventTriggered,
}

/// 保险信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InsuranceInfo {
    /// 保险类型
    pub insurance_type: InsuranceType,
    /// 保险金额
    pub insurance_amount: f64,
    /// 保险公司
    pub insurance_company: String,
    /// 保险开始时间
    pub start_time: i64,
    /// 保险结束时间
    pub end_time: i64,
    /// 说明
    pub description: String,
}

/// 保险结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InsuranceResult {
    /// 保险ID
    pub insurance_id: u64,
    /// 保险类型
    pub insurance_type: InsuranceType,
    /// 保险方式
    pub method: InsuranceMethod,
    /// 状态
    pub status: bool,
    /// 时间戳
    pub timestamp: i64,
}

/// 保险覆盖指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InsuranceCoverageParams {
    /// 保险类型
    pub insurance_type: InsuranceType,
    /// 保险方式
    pub method: InsuranceMethod,
    /// 保险信息
    pub info: InsuranceInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 保险覆盖指令账户上下文
#[derive(Accounts)]
pub struct InsuranceCoverage<'info> {
    /// 现实世界资产账户，需可变
    #[account(
        mut,
        constraint = rwa.asset_type == AssetType::RWA @ AssetError::InvalidAssetType
    )]
    pub rwa: Account<'info, BasketIndexState>,
    
    /// 保险权限签名者
    #[account(
        constraint = authority.key() == rwa.insurance_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 保险账户
    #[account(mut)]
    pub insurance_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 保险覆盖指令实现
pub fn insurance_coverage(
    ctx: Context<InsuranceCoverage>,
    params: InsuranceCoverageParams,
) -> Result<InsuranceResult> {
    validate_insurance_coverage_params(&params)?;
    check_insurance_authority_permission(&ctx.accounts.authority, &ctx.accounts.rwa)?;
    let rwa = &mut ctx.accounts.rwa;
    let authority = &ctx.accounts.authority;
    let service = RwaService::new();
    let result = service.insurance_coverage(
        rwa,
        &params.insurance_type,
        &params.method,
        &params.info,
        &params.exec_params,
    )?;
    emit!(AssetInsuranceCoverageUpdated {
        basket_id: rwa.id,
        insurance_id: result.insurance_id,
        insurance_type: params.insurance_type,
        status: result.status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::RWA,
        exec_params: params.exec_params,
    });
    Ok(result)
}

fn validate_insurance_coverage_params(params: &InsuranceCoverageParams) -> Result<()> {
    require!(params.info.insurance_amount > 0.0, AssetError::InvalidInsuranceAmount);
    require!(!params.info.insurance_company.is_empty(), AssetError::InvalidInsuranceCompany);
    require!(params.info.start_time > 0, AssetError::InvalidInsuranceTime);
    require!(params.info.end_time > 0, AssetError::InvalidInsuranceTime);
    require!(params.info.end_time > params.info.start_time, AssetError::InvalidInsuranceTime);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

fn check_insurance_authority_permission(
    authority: &Signer,
    rwa: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == rwa.insurance_authority,
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