//! 现实世界资产治理权指令
//!
//! 本模块实现了现实世界资产的治理权功能，包括投票权、提案权、治理决策等。
//!
//! ## 功能特点
//!
//! - **多种治理类型**: 支持投票治理、提案治理、决策治理等
//! - **灵活治理方式**: 支持自动、手动、定期治理管理
//! - **治理信息管理**: 完整的治理记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 投票权管理
//! - 提案权行使
//! - 治理决策执行
//! - 治理权转让

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetGovernanceRightsUpdated;
use crate::errors::AssetError;

/// 治理类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum GovernanceType {
    /// 投票治理
    Voting,
    /// 提案治理
    Proposal,
    /// 决策治理
    Decision,
    /// 其他治理
    Other,
}

/// 治理方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum GovernanceMethod {
    /// 自动治理
    Automatic,
    /// 手动治理
    Manual,
    /// 定期治理
    Periodic,
    /// 事件触发
    EventTriggered,
}

/// 治理信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct GovernanceInfo {
    /// 治理类型
    pub governance_type: GovernanceType,
    /// 治理权重
    pub governance_weight: f64,
    /// 治理数量
    pub governance_amount: u64,
    /// 治理开始时间
    pub start_time: i64,
    /// 治理结束时间
    pub end_time: i64,
    /// 说明
    pub description: String,
}

/// 治理结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct GovernanceResult {
    /// 治理ID
    pub governance_id: u64,
    /// 治理类型
    pub governance_type: GovernanceType,
    /// 治理方式
    pub method: GovernanceMethod,
    /// 状态
    pub status: bool,
    /// 时间戳
    pub timestamp: i64,
}

/// 治理权指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct GovernanceRightsParams {
    /// 治理类型
    pub governance_type: GovernanceType,
    /// 治理方式
    pub method: GovernanceMethod,
    /// 治理信息
    pub info: GovernanceInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 治理权指令账户上下文
#[derive(Accounts)]
pub struct GovernanceRights<'info> {
    /// 现实世界资产账户，需可变
    #[account(
        mut,
        constraint = rwa.asset_type == AssetType::RWA @ AssetError::InvalidAssetType
    )]
    pub rwa: Account<'info, BasketIndexState>,
    
    /// 治理权限签名者
    #[account(
        constraint = authority.key() == rwa.governance_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 治理账户
    #[account(mut)]
    pub governance_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 治理权指令实现
pub fn governance_rights(
    ctx: Context<GovernanceRights>,
    params: GovernanceRightsParams,
) -> Result<GovernanceResult> {
    validate_governance_rights_params(&params)?;
    check_governance_authority_permission(&ctx.accounts.authority, &ctx.accounts.rwa)?;
    let rwa = &mut ctx.accounts.rwa;
    let authority = &ctx.accounts.authority;
    let service = RwaService::new();
    let result = service.governance_rights(
        rwa,
        &params.governance_type,
        &params.method,
        &params.info,
        &params.exec_params,
    )?;
    emit!(AssetGovernanceRightsUpdated {
        basket_id: rwa.id,
        governance_id: result.governance_id,
        governance_type: params.governance_type,
        status: result.status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::RWA,
        exec_params: params.exec_params,
    });
    Ok(result)
}

fn validate_governance_rights_params(params: &GovernanceRightsParams) -> Result<()> {
    require!(params.info.governance_weight > 0.0, AssetError::InvalidGovernanceWeight);
    require!(params.info.governance_weight <= 1.0, AssetError::InvalidGovernanceWeight);
    require!(params.info.governance_amount > 0, AssetError::InvalidGovernanceAmount);
    require!(params.info.start_time > 0, AssetError::InvalidGovernanceTime);
    require!(params.info.end_time > 0, AssetError::InvalidGovernanceTime);
    require!(params.info.end_time > params.info.start_time, AssetError::InvalidGovernanceTime);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

fn check_governance_authority_permission(
    authority: &Signer,
    rwa: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == rwa.governance_authority,
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