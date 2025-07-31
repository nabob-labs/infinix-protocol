//! 现实世界资产部分所有权指令
//!
//! 本模块实现了现实世界资产的部分所有权功能，包括所有权分割、转让、管理等。
//!
//! ## 功能特点
//!
//! - **多种所有权类型**: 支持股权分割、房产分割、艺术品分割等
//! - **灵活管理方式**: 支持自动、手动、定期所有权管理
//! - **所有权信息管理**: 完整的所有权记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 股权分割管理
//! - 房产分割转让
//! - 艺术品分割投资
//! - 所有权变更管理

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetFractionalOwnershipUpdated;
use crate::errors::AssetError;

/// 所有权类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum OwnershipType {
    /// 股权分割
    Equity,
    /// 房产分割
    RealEstate,
    /// 艺术品分割
    Artwork,
    /// 其他分割
    Other,
}

/// 所有权管理方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum OwnershipMethod {
    /// 自动管理
    Automatic,
    /// 手动管理
    Manual,
    /// 定期管理
    Periodic,
    /// 事件触发
    EventTriggered,
}

/// 所有权信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct OwnershipInfo {
    /// 所有权类型
    pub ownership_type: OwnershipType,
    /// 分割比例
    pub fractional_ratio: f64,
    /// 所有权数量
    pub ownership_amount: u64,
    /// 所有权开始时间
    pub start_time: i64,
    /// 所有权结束时间
    pub end_time: i64,
    /// 说明
    pub description: String,
}

/// 所有权结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct OwnershipResult {
    /// 所有权ID
    pub ownership_id: u64,
    /// 所有权类型
    pub ownership_type: OwnershipType,
    /// 所有权管理方式
    pub method: OwnershipMethod,
    /// 状态
    pub status: bool,
    /// 时间戳
    pub timestamp: i64,
}

/// 部分所有权指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FractionalOwnershipParams {
    /// 所有权类型
    pub ownership_type: OwnershipType,
    /// 所有权管理方式
    pub method: OwnershipMethod,
    /// 所有权信息
    pub info: OwnershipInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 部分所有权指令账户上下文
#[derive(Accounts)]
pub struct FractionalOwnership<'info> {
    /// 现实世界资产账户，需可变
    #[account(
        mut,
        constraint = rwa.asset_type == AssetType::RWA @ AssetError::InvalidAssetType
    )]
    pub rwa: Account<'info, BasketIndexState>,
    
    /// 所有权权限签名者
    #[account(
        constraint = authority.key() == rwa.ownership_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 所有权账户
    #[account(mut)]
    pub ownership_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 部分所有权指令实现
pub fn fractional_ownership(
    ctx: Context<FractionalOwnership>,
    params: FractionalOwnershipParams,
) -> Result<OwnershipResult> {
    validate_fractional_ownership_params(&params)?;
    check_ownership_authority_permission(&ctx.accounts.authority, &ctx.accounts.rwa)?;
    let rwa = &mut ctx.accounts.rwa;
    let authority = &ctx.accounts.authority;
    let service = RwaService::new();
    let result = service.fractional_ownership(
        rwa,
        &params.ownership_type,
        &params.method,
        &params.info,
        &params.exec_params,
    )?;
    emit!(AssetFractionalOwnershipUpdated {
        basket_id: rwa.id,
        ownership_id: result.ownership_id,
        ownership_type: params.ownership_type,
        status: result.status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::RWA,
        exec_params: params.exec_params,
    });
    Ok(result)
}

fn validate_fractional_ownership_params(params: &FractionalOwnershipParams) -> Result<()> {
    require!(params.info.fractional_ratio > 0.0, AssetError::InvalidOwnershipRatio);
    require!(params.info.fractional_ratio <= 1.0, AssetError::InvalidOwnershipRatio);
    require!(params.info.ownership_amount > 0, AssetError::InvalidOwnershipAmount);
    require!(params.info.start_time > 0, AssetError::InvalidOwnershipTime);
    require!(params.info.end_time > 0, AssetError::InvalidOwnershipTime);
    require!(params.info.end_time > params.info.start_time, AssetError::InvalidOwnershipTime);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

fn check_ownership_authority_permission(
    authority: &Signer,
    rwa: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == rwa.ownership_authority,
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