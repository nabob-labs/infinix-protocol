//! 现实世界资产退出策略指令
//!
//! 本模块实现了现实世界资产的退出策略功能，包括退出计划、清算策略、退出管理等。
//!
//! ## 功能特点
//!
//! - **多种退出类型**: 支持自愿退出、强制退出、清算退出等
//! - **灵活退出方式**: 支持自动、手动、定期退出管理
//! - **退出信息管理**: 完整的退出记录和追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 自愿退出管理
//! - 强制退出执行
//! - 清算退出处理
//! - 退出策略调整

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::rwa_service::RwaService;
use crate::events::asset_event::AssetExitStrategyUpdated;
use crate::errors::AssetError;

/// 退出类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ExitType {
    /// 自愿退出
    Voluntary,
    /// 强制退出
    Forced,
    /// 清算退出
    Liquidation,
    /// 其他退出
    Other,
}

/// 退出方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ExitMethod {
    /// 自动退出
    Automatic,
    /// 手动退出
    Manual,
    /// 定期退出
    Periodic,
    /// 事件触发
    EventTriggered,
}

/// 退出信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ExitInfo {
    /// 退出类型
    pub exit_type: ExitType,
    /// 退出比例
    pub exit_ratio: f64,
    /// 退出金额
    pub exit_amount: f64,
    /// 退出开始时间
    pub start_time: i64,
    /// 退出结束时间
    pub end_time: i64,
    /// 说明
    pub description: String,
}

/// 退出结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ExitResult {
    /// 退出ID
    pub exit_id: u64,
    /// 退出类型
    pub exit_type: ExitType,
    /// 退出方式
    pub method: ExitMethod,
    /// 状态
    pub status: bool,
    /// 时间戳
    pub timestamp: i64,
}

/// 退出策略指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ExitStrategyParams {
    /// 退出类型
    pub exit_type: ExitType,
    /// 退出方式
    pub method: ExitMethod,
    /// 退出信息
    pub info: ExitInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 退出策略指令账户上下文
#[derive(Accounts)]
pub struct ExitStrategy<'info> {
    /// 现实世界资产账户，需可变
    #[account(
        mut,
        constraint = rwa.asset_type == AssetType::RWA @ AssetError::InvalidAssetType
    )]
    pub rwa: Account<'info, BasketIndexState>,
    
    /// 退出权限签名者
    #[account(
        constraint = authority.key() == rwa.exit_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 退出账户
    #[account(mut)]
    pub exit_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 退出策略指令实现
pub fn exit_strategy(
    ctx: Context<ExitStrategy>,
    params: ExitStrategyParams,
) -> Result<ExitResult> {
    validate_exit_strategy_params(&params)?;
    check_exit_authority_permission(&ctx.accounts.authority, &ctx.accounts.rwa)?;
    let rwa = &mut ctx.accounts.rwa;
    let authority = &ctx.accounts.authority;
    let service = RwaService::new();
    let result = service.exit_strategy(
        rwa,
        &params.exit_type,
        &params.method,
        &params.info,
        &params.exec_params,
    )?;
    emit!(AssetExitStrategyUpdated {
        basket_id: rwa.id,
        exit_id: result.exit_id,
        exit_type: params.exit_type,
        status: result.status,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::RWA,
        exec_params: params.exec_params,
    });
    Ok(result)
}

fn validate_exit_strategy_params(params: &ExitStrategyParams) -> Result<()> {
    require!(params.info.exit_ratio > 0.0, AssetError::InvalidExitRatio);
    require!(params.info.exit_ratio <= 1.0, AssetError::InvalidExitRatio);
    require!(params.info.exit_amount > 0.0, AssetError::InvalidExitAmount);
    require!(params.info.start_time > 0, AssetError::InvalidExitTime);
    require!(params.info.end_time > 0, AssetError::InvalidExitTime);
    require!(params.info.end_time > params.info.start_time, AssetError::InvalidExitTime);
    validate_execution_params(&params.exec_params)?;
    Ok(())
}

fn check_exit_authority_permission(
    authority: &Signer,
    rwa: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == rwa.exit_authority,
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