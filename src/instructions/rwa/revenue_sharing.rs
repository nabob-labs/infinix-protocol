//! 现实世界资产收益分享指令
//!
//! 本模块实现了现实世界资产的收益分享功能，包括收益分配、分享比例、收益管理等。
//!
//! ## 功能特点
//!
//! - **多种收益类型**: 支持租金收益、分红收益、利息收益等
//! - **灵活分享方式**: 支持自动、手动、定期收益分享
//! - **收益信息管理**: 完整的收益记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 租金收益分享
//! - 分红收益分配
//! - 利息收益管理
//! - 收益比例调整

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetRevenueSharingUpdated;
use crate::errors::AssetError;

/// 收益类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum RevenueType {
    /// 租金收益
    Rental,
    /// 分红收益
    Dividend,
    /// 利息收益
    Interest,
    /// 其他收益
    Other,
}

/// 收益分享方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum RevenueMethod {
    /// 自动分享
    Automatic,
    /// 手动分享
    Manual,
    /// 定期分享
    Periodic,
    /// 事件触发
    EventTriggered,
}

/// 收益信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RevenueInfo {
    /// 收益类型
    pub revenue_type: RevenueType,
    /// 收益金额
    pub revenue_amount: f64,
    /// 分享比例
    pub sharing_ratio: f64,
    /// 收益开始时间
    pub start_time: i64,
    /// 收益结束时间
    pub end_time: i64,
    /// 说明
    pub description: String,
}

/// 收益结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RevenueResult {
    /// 收益ID
    pub revenue_id: u64,
    /// 收益类型
    pub revenue_type: RevenueType,
    /// 收益分享方式
    pub method: RevenueMethod,
    /// 状态
    pub status: bool,
    /// 时间戳
    pub timestamp: i64,
}

/// 收益分享指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RevenueSharingParams {
    /// 收益类型
    pub revenue_type: RevenueType,
    /// 收益分享方式
    pub method: RevenueMethod,
    /// 收益信息
    pub info: RevenueInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 收益分享指令账户上下文
#[derive(Accounts)]
pub struct RevenueSharing<'info> {
    /// 现实世界资产账户，需可变
    #[account(
        mut,
        constraint = rwa.asset_type == AssetType::RWA @ AssetError::InvalidAssetType
    )]
    pub rwa: Account<'info, BasketIndexState>,
    
    /// 收益权限签名者
    #[account(
        constraint = authority.key() == rwa.revenue_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 收益账户
    #[account(mut)]
    pub revenue_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 收益分享指令实现
pub fn revenue_sharing(
    ctx: Context<RevenueSharing>,
    params: RevenueSharingParams,
) -> Result<RevenueResult> {
    validate_revenue_sharing_params(&params)?;
    check_revenue_authority_permission(&ctx.accounts.authority, &ctx.accounts.rwa)?;
    let rwa = &mut ctx.accounts.rwa;
    let authority = &ctx.accounts.authority;
    let service = RwaService::new();
    let result = service.revenue_sharing(
        rwa,
        &params.revenue_type,
        &params.method,
        &params.info,
        &params.exec_params,
    )?;
    emit!(AssetRevenueSharingUpdated {
        basket_id: rwa.id,
        revenue_id: result.revenue_id,
        revenue_type: params.revenue_type,
        status: result.status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::RWA,
        exec_params: params.exec_params,
    });
    Ok(result)
}

fn validate_revenue_sharing_params(params: &RevenueSharingParams) -> Result<()> {
    require!(params.info.revenue_amount > 0.0, AssetError::InvalidRevenueAmount);
    require!(params.info.sharing_ratio > 0.0, AssetError::InvalidRevenueRatio);
    require!(params.info.sharing_ratio <= 1.0, AssetError::InvalidRevenueRatio);
    require!(params.info.start_time > 0, AssetError::InvalidRevenueTime);
    require!(params.info.end_time > 0, AssetError::InvalidRevenueTime);
    require!(params.info.end_time > params.info.start_time, AssetError::InvalidRevenueTime);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

fn check_revenue_authority_permission(
    authority: &Signer,
    rwa: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == rwa.revenue_authority,
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