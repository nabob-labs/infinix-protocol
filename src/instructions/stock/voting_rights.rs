//! 股票投票权指令
//!
//! 本模块实现了股票的投票权管理功能，包括投票权分配、投票执行、投票结果统计等。
//!
//! ## 功能特点
//!
//! - **多种投票类型**: 支持普通投票、优先投票、代理投票
//! - **灵活投票方式**: 支持在线投票、离线投票、委托投票
//! - **投票信息管理**: 完整的投票记录和结果追踪
//! - **事件驱动**: 完整的事件发射和审计追踪
//! - **错误处理**: 全面的错误类型和处理机制
//!
//! ## 使用场景
//!
//! - 股东大会投票
//! - 董事会选举
//! - 重要决议投票
//! - 投票权委托

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams};
use crate::services::stock_service::StockService;
use crate::events::asset_event::AssetVotingRightsUpdated;
use crate::errors::AssetError;

/// 投票类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum VotingType {
    /// 普通投票
    Ordinary,
    /// 优先投票
    Preferred,
    /// 代理投票
    Proxy,
    /// 累积投票
    Cumulative,
}

/// 投票方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum VotingMethod {
    /// 在线投票
    Online,
    /// 离线投票
    Offline,
    /// 委托投票
    Delegated,
    /// 邮寄投票
    Mail,
}

/// 投票信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct VotingInfo {
    /// 投票类型
    pub voting_type: VotingType,
    /// 投票议题
    pub voting_issue: String,
    /// 投票选项
    pub voting_options: Vec<String>,
    /// 投票开始时间
    pub voting_start_time: i64,
    /// 投票结束时间
    pub voting_end_time: i64,
    /// 投票说明
    pub description: String,
}

/// 投票权结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct VotingRightsResult {
    /// 投票ID
    pub voting_id: u64,
    /// 投票类型
    pub voting_type: VotingType,
    /// 投票方式
    pub voting_method: VotingMethod,
    /// 投票数量
    pub voting_count: u64,
    /// 投票结果
    pub voting_result: Vec<u64>,
    /// 投票状态
    pub voting_status: bool,
    /// 投票时间戳
    pub voting_timestamp: i64,
}

/// 投票权指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct VotingRightsParams {
    /// 投票类型
    pub voting_type: VotingType,
    /// 投票方式
    pub voting_method: VotingMethod,
    /// 投票信息
    pub voting_info: VotingInfo,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 投票权指令账户上下文
#[derive(Accounts)]
pub struct VotingRights<'info> {
    /// 股票资产账户，需可变
    #[account(
        mut,
        constraint = stock.asset_type == AssetType::Stock @ AssetError::InvalidAssetType
    )]
    pub stock: Account<'info, BasketIndexState>,
    
    /// 投票权限签名者
    #[account(
        constraint = authority.key() == stock.voting_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 投票者账户
    #[account(mut)]
    pub voter_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 投票权指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 投票权参数，包含投票类型、投票方式和投票信息
///
/// ## 返回值
/// - `Result<VotingRightsResult>`: 投票权结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidVotingInfo`: 无效的投票信息
/// - `InvalidVotingTime`: 无效的投票时间
/// - `InvalidParams`: 无效的参数
pub fn voting_rights(
    ctx: Context<VotingRights>,
    params: VotingRightsParams,
) -> Result<VotingRightsResult> {
    // 参数验证
    validate_voting_rights_params(&params)?;
    
    // 权限检查
    check_voting_authority_permission(&ctx.accounts.authority, &ctx.accounts.stock)?;
    
    // 获取账户引用
    let stock = &mut ctx.accounts.stock;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = StockService::new();
    
    // 调用服务层执行投票权操作
    let result = service.voting_rights(
        stock,
        &params.voting_type,
        &params.voting_method,
        &params.voting_info,
        &params.exec_params,
    )?;
    
    // 发射事件
    emit!(AssetVotingRightsUpdated {
        basket_id: stock.id,
        voting_id: result.voting_id,
        voting_type: params.voting_type,
        voting_count: result.voting_count,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Stock,
        exec_params: params.exec_params,
    });
    
    Ok(result)
}

/// 验证投票权参数
fn validate_voting_rights_params(params: &VotingRightsParams) -> Result<()> {
    require!(!params.voting_info.voting_issue.is_empty(), AssetError::InvalidVotingInfo);
    require!(!params.voting_info.voting_options.is_empty(), AssetError::InvalidVotingInfo);
    require!(params.voting_info.voting_start_time > 0, AssetError::InvalidVotingTime);
    require!(params.voting_info.voting_end_time > 0, AssetError::InvalidVotingTime);
    require!(params.voting_info.voting_end_time > params.voting_info.voting_start_time, AssetError::InvalidVotingTime);
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查投票权限
fn check_voting_authority_permission(
    authority: &Signer,
    stock: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == stock.voting_authority,
        AssetError::InsufficientAuthority
    );
    
    Ok(())
}

/// 验证执行参数
fn validate_execution_params(exec_params: &ExecutionParams) -> Result<()> {
    require!(exec_params.slippage_tolerance > 0.0, AssetError::InvalidParams);
    require!(exec_params.slippage_tolerance <= 1.0, AssetError::InvalidParams);
    require!(exec_params.max_retries > 0, AssetError::InvalidParams);
    require!(exec_params.max_retries <= 10, AssetError::InvalidParams);
    
    Ok(())
} 