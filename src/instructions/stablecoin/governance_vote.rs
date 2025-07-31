//! 稳定币 (Stablecoin) 治理投票指令
//! 
//! 本模块实现稳定币资产的治理投票功能，支持提案投票、参数更新、治理权管理等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 提案投票：支持多种投票机制
//! - 参数更新：治理参数动态调整
//! - 治理权管理：投票权分配和委托
//! - 投票机制：支持多种投票类型
//! - 事件记录：完整的审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, GovernanceParams};
use crate::services::stablecoin_service::StablecoinService;
use crate::events::asset_event::AssetGovernanceVoted;
use crate::validation::business::validate_governance_vote_params;
use crate::core::security::check_authority_permission;
use crate::dex::traits::DexAdapterTrait;
use crate::oracles::traits::OracleAdapterTrait;

/// 稳定币治理投票参数结构体
/// 
/// 定义治理投票操作所需的所有参数，包括：
/// - proposal_id: 提案ID
/// - vote_type: 投票类型
/// - voting_power: 投票权重
/// - vote_choice: 投票选择
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct GovernanceVoteStablecoinParams {
    /// 提案ID
    pub proposal_id: u64,
    /// 投票类型
    pub vote_type: VoteType,
    /// 投票权重
    pub voting_power: u64,
    /// 投票选择
    pub vote_choice: VoteChoice,
    /// 治理参数
    pub governance_params: Option<GovernanceParams>,
    /// 执行参数（可选）
    pub exec_params: Option<ExecutionParams>,
    /// 策略参数（可选）
    pub strategy_params: Option<StrategyParams>,
}

/// 投票类型枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum VoteType {
    /// 赞成票
    For,
    /// 反对票
    Against,
    /// 弃权票
    Abstain,
    /// 委托投票
    Delegated,
    /// 快照投票
    Snapshot,
    /// 二次投票
    Quadratic,
}

/// 投票选择枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum VoteChoice {
    /// 赞成
    Yes,
    /// 反对
    No,
    /// 弃权
    Abstain,
    /// 委托
    Delegate(Pubkey),
}

/// 治理投票订单结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct GovernanceVoteOrder {
    /// 订单ID
    pub order_id: u64,
    /// 提案ID
    pub proposal_id: u64,
    /// 投票类型
    pub vote_type: VoteType,
    /// 投票权重
    pub voting_power: u64,
    /// 时间戳
    pub timestamp: i64,
}

/// 治理投票结果结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct GovernanceVoteResult {
    /// 提案ID
    pub proposal_id: u64,
    /// 赞成票数
    pub for_votes: u64,
    /// 反对票数
    pub against_votes: u64,
    /// 弃权票数
    pub abstain_votes: u64,
    /// 总投票数
    pub total_votes: u64,
    /// 投票通过阈值
    pub quorum_threshold: u64,
    /// 投票状态
    pub vote_status: VoteStatus,
    /// 投票结束时间
    pub end_time: i64,
}

/// 投票状态枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum VoteStatus {
    /// 活跃
    Active,
    /// 已通过
    Passed,
    /// 已拒绝
    Rejected,
    /// 已过期
    Expired,
    /// 已执行
    Executed,
}

/// 稳定币治理投票指令账户上下文
/// 
/// 定义治理投票操作所需的所有账户，包括：
/// - stablecoin_asset: 稳定币资产账户（只读）
/// - authority: 操作权限账户（签名者）
/// - governance_program: 治理程序（可选）
/// - system_program: 系统程序
/// - clock: 时钟程序
#[derive(Accounts)]
#[instruction(params: GovernanceVoteStablecoinParams)]
pub struct GovernanceVoteStablecoin<'info> {
    /// 稳定币资产账户，只读权限
    #[account(
        seeds = [b"stablecoin", stablecoin_asset.key().as_ref()],
        bump,
        constraint = stablecoin_asset.asset_type == AssetType::Stablecoin @ crate::errors::asset_error::AssetError::InvalidAssetType
    )]
    pub stablecoin_asset: Account<'info, crate::state::baskets::BasketIndexState>,
    
    /// 操作权限账户，必须是签名者
    #[account(
        constraint = authority.key() == stablecoin_asset.authority @ crate::errors::security_error::SecurityError::InvalidAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 治理程序（可选），用于治理功能
    /// CHECK: 由治理适配器验证
    pub governance_program: Option<UncheckedAccount<'info>>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 时钟程序
    pub clock: Sysvar<'info, Clock>,
}

/// 稳定币治理投票指令实现
/// 
/// 执行治理投票操作，包括：
/// - 参数验证和权限检查
/// - 投票权重计算
/// - 投票记录和统计
/// - 事件记录和结果输出
pub fn governance_vote_stablecoin(
    ctx: Context<GovernanceVoteStablecoin>,
    params: GovernanceVoteStablecoinParams
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_governance_vote_params(&params)?;
    require!(params.proposal_id > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(params.voting_power > 0, crate::errors::asset_error::AssetError::InvalidAmount);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "governance_vote"
    )?;
    
    // 3. 调用服务层执行治理投票
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.governance_vote(
        stablecoin_asset,
        &params,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射治理投票事件
    emit!(AssetGovernanceVoted {
        asset_id: stablecoin_asset.id,
        proposal_id: params.proposal_id,
        vote_type: params.vote_type.clone(),
        voting_power: params.voting_power,
        vote_choice: params.vote_choice.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin governance vote executed successfully: asset_id={}, proposal_id={}, vote_type={:?}, voting_power={}", 
         stablecoin_asset.id, params.proposal_id, params.vote_type, params.voting_power);
    
    Ok(())
}

/// 委托投票指令实现
/// 
/// 执行委托投票操作
pub fn delegate_vote_stablecoin(
    ctx: Context<GovernanceVoteStablecoin>,
    params: GovernanceVoteStablecoinParams,
    delegate: Pubkey
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_governance_vote_params(&params)?;
    require!(params.proposal_id > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(params.voting_power > 0, crate::errors::asset_error::AssetError::InvalidAmount);
    require!(delegate != Pubkey::default(), crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "delegate_vote"
    )?;
    
    // 3. 调用服务层执行委托投票
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.delegate_vote(
        stablecoin_asset,
        &params,
        delegate,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射委托投票事件
    emit!(AssetGovernanceVoted {
        asset_id: stablecoin_asset.id,
        proposal_id: params.proposal_id,
        vote_type: VoteType::Delegated,
        voting_power: params.voting_power,
        vote_choice: VoteChoice::Delegate(delegate),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin delegate vote executed successfully: asset_id={}, proposal_id={}, delegate={}, voting_power={}", 
         stablecoin_asset.id, params.proposal_id, delegate, params.voting_power);
    
    Ok(())
}

/// 快照投票指令实现
/// 
/// 执行快照投票操作
pub fn snapshot_vote_stablecoin(
    ctx: Context<GovernanceVoteStablecoin>,
    params: GovernanceVoteStablecoinParams,
    snapshot_block: u64
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_governance_vote_params(&params)?;
    require!(params.proposal_id > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(params.voting_power > 0, crate::errors::asset_error::AssetError::InvalidAmount);
    require!(snapshot_block > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "snapshot_vote"
    )?;
    
    // 3. 调用服务层执行快照投票
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.snapshot_vote(
        stablecoin_asset,
        &params,
        snapshot_block,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射快照投票事件
    emit!(AssetGovernanceVoted {
        asset_id: stablecoin_asset.id,
        proposal_id: params.proposal_id,
        vote_type: VoteType::Snapshot,
        voting_power: params.voting_power,
        vote_choice: params.vote_choice.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin snapshot vote executed successfully: asset_id={}, proposal_id={}, snapshot_block={}, voting_power={}", 
         stablecoin_asset.id, params.proposal_id, snapshot_block, params.voting_power);
    
    Ok(())
}

/// 二次投票指令实现
/// 
/// 执行二次投票操作
pub fn quadratic_vote_stablecoin(
    ctx: Context<GovernanceVoteStablecoin>,
    params: GovernanceVoteStablecoinParams,
    quadratic_factor: u16
) -> anchor_lang::Result<()> {
    let stablecoin_asset = &ctx.accounts.stablecoin_asset;
    
    // 1. 参数验证
    validate_governance_vote_params(&params)?;
    require!(params.proposal_id > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(params.voting_power > 0, crate::errors::asset_error::AssetError::InvalidAmount);
    require!(quadratic_factor > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &stablecoin_asset.authority,
        &ctx.accounts.authority.key(),
        "quadratic_vote"
    )?;
    
    // 3. 调用服务层执行二次投票
    let stablecoin_service = StablecoinService::new();
    let result = stablecoin_service.quadratic_vote(
        stablecoin_asset,
        &params,
        quadratic_factor,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射二次投票事件
    emit!(AssetGovernanceVoted {
        asset_id: stablecoin_asset.id,
        proposal_id: params.proposal_id,
        vote_type: VoteType::Quadratic,
        voting_power: params.voting_power,
        vote_choice: params.vote_choice.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Stablecoin quadratic vote executed successfully: asset_id={}, proposal_id={}, quadratic_factor={}, voting_power={}", 
         stablecoin_asset.id, params.proposal_id, quadratic_factor, params.voting_power);
    
    Ok(())
} 