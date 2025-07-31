//! 现实世界资产托管管理指令
//!
//! 本模块实现了现实世界资产的托管管理功能，包括托管机构管理、资产保管、托管报告等。
//!
//! ## 功能特点
//!
//! - **多种托管类型**: 支持机构托管、个人托管、联合托管等
//! - **灵活托管方式**: 支持自动、手动、定期托管管理
//! - **托管信息管理**: 完整的托管记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 托管机构管理
//! - 资产保管监控
//! - 托管报告生成
//! - 托管费用管理

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetCustodyManaged;
use crate::errors::AssetError;

/// 托管类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CustodyType {
    /// 机构托管
    Institutional,
    /// 个人托管
    Individual,
    /// 联合托管
    Joint,
    /// 数字托管
    Digital,
}

/// 托管方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CustodyMethod {
    /// 自动托管
    Automatic,
    /// 手动托管
    Manual,
    /// 定期托管
    Periodic,
    /// 事件触发
    EventTriggered,
}

/// 托管信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CustodyInfo {
    /// 托管类型
    pub custody_type: CustodyType,
    /// 托管机构
    pub custodian: String,
    /// 托管费用
    pub custody_fee: f64,
    /// 托管开始时间
    pub start_time: i64,
    /// 托管结束时间
    pub end_time: i64,
    /// 说明
    pub description: String,
}

/// 托管结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CustodyResult {
    /// 托管ID
    pub custody_id: u64,
    /// 托管类型
    pub custody_type: CustodyType,
    /// 托管方式
    pub method: CustodyMethod,
    /// 状态
    pub status: bool,
    /// 时间戳
    pub timestamp: i64,
}

/// 托管管理指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CustodyManagementParams {
    /// 托管类型
    pub custody_type: CustodyType,
    /// 托管方式
    pub method: CustodyMethod,
    /// 托管信息
    pub info: CustodyInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 托管管理指令账户上下文
#[derive(Accounts)]
pub struct CustodyManagement<'info> {
    /// 现实世界资产账户，需可变
    #[account(
        mut,
        constraint = rwa.asset_type == AssetType::RWA @ AssetError::InvalidAssetType
    )]
    pub rwa: Account<'info, BasketIndexState>,
    
    /// 托管权限签名者
    #[account(
        constraint = authority.key() == rwa.custody_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 托管账户
    #[account(mut)]
    pub custody_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 托管管理指令实现
pub fn custody_management(
    ctx: Context<CustodyManagement>,
    params: CustodyManagementParams,
) -> Result<CustodyResult> {
    validate_custody_management_params(&params)?;
    check_custody_authority_permission(&ctx.accounts.authority, &ctx.accounts.rwa)?;
    let rwa = &mut ctx.accounts.rwa;
    let authority = &ctx.accounts.authority;
    let service = RwaService::new();
    let result = service.custody_management(
        rwa,
        &params.custody_type,
        &params.method,
        &params.info,
        &params.exec_params,
    )?;
    emit!(AssetCustodyManaged {
        basket_id: rwa.id,
        custody_id: result.custody_id,
        custody_type: params.custody_type,
        status: result.status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::RWA,
        exec_params: params.exec_params,
    });
    Ok(result)
}

fn validate_custody_management_params(params: &CustodyManagementParams) -> Result<()> {
    require!(!params.info.custodian.is_empty(), AssetError::InvalidCustodian);
    require!(params.info.custody_fee >= 0.0, AssetError::InvalidCustodyFee);
    require!(params.info.start_time > 0, AssetError::InvalidCustodyTime);
    require!(params.info.end_time > 0, AssetError::InvalidCustodyTime);
    require!(params.info.end_time > params.info.start_time, AssetError::InvalidCustodyTime);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

fn check_custody_authority_permission(
    authority: &Signer,
    rwa: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == rwa.custody_authority,
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