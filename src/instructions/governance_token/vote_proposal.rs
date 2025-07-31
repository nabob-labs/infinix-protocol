//! Governance Token投票指令模块
//! 
//! 本模块提供Governance Token资产的投票功能，包括：
//! - 参数验证：验证投票参数的有效性和边界条件
//! - 权限检查：验证投票权限和授权状态
//! - 服务层调用：委托给GovernanceTokenService执行核心业务逻辑
//! - 事件发射：发射Governance Token投票事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于Governance Token投票功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给GovernanceTokenService
//! - 事件驱动：完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::{
    core::{
        constants::*,
        events::*,
        types::*,
        validation::*,
    },
    errors::*,
    services::*,
    utils::*,
};

/// 投票参数结构体
/// 
/// 包含投票所需的所有参数：
/// - proposal_id: 提案ID
/// - vote: 投票选择
/// - voting_power: 投票权重
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct VoteProposalParams {
    /// 提案ID
    pub proposal_id: Pubkey,
    /// 投票选择
    pub vote: VoteChoice,
    /// 投票权重
    pub voting_power: u64,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 投票选择枚举
/// 
/// 定义投票的选择：
/// - Yes: 赞成
/// - No: 反对
/// - Abstain: 弃权
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum VoteChoice {
    /// 赞成
    Yes,
    /// 反对
    No,
    /// 弃权
    Abstain,
}

/// 投票账户上下文
/// 
/// 定义投票指令所需的账户结构：
/// - governance_token: Governance Token账户（可变，Governance Token类型约束）
/// - voter: 投票者账户（owner约束）
/// - proposal: 提案账户
/// - governance: 治理账户
/// - voter_token_account: 投票者代币账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct VoteProposal<'info> {
    /// Governance Token账户（可变，Governance Token类型约束）
    #[account(
        mut,
        constraint = governance_token.asset_type == AssetType::GovernanceToken @ AssetError::InvalidAssetType
    )]
    pub governance_token: Account<'info, Asset>,
    
    /// 投票者账户（owner约束）
    #[account(
        constraint = voter.key() == governance_token.owner @ AssetError::InvalidOwner
    )]
    pub voter: Signer<'info>,
    
    /// 提案账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub proposal: UncheckedAccount<'info>,
    
    /// 治理账户
    /// CHECK: 由程序验证
    pub governance: UncheckedAccount<'info>,
    
    /// 投票者代币账户
    #[account(
        mut,
        constraint = voter_token_account.owner == voter.key() @ AssetError::InvalidTokenAccount
    )]
    pub voter_token_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证投票参数
/// 
/// 检查投票参数的有效性和边界条件：
/// - 提案ID验证
/// - 投票权重验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 投票参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_vote_proposal_params(params: &VoteProposalParams) -> Result<()> {
    // 验证提案ID
    require!(
        params.proposal_id != Pubkey::default(),
        AssetError::InvalidProposalId
    );
    
    // 验证投票权重
    require!(
        params.voting_power > 0,
        AssetError::InvalidVotingPower
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查投票权限
/// 
/// 验证投票权限和授权状态：
/// - 检查所有权
/// - 验证Governance Token状态
/// - 检查投票者代币余额
/// 
/// # 参数
/// - voter: 投票者账户
/// - governance_token: Governance Token账户
/// - voter_token_account: 投票者代币账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_vote_authority_permission(
    voter: &Signer,
    governance_token: &Account<Asset>,
    voter_token_account: &Account<TokenAccount>,
) -> Result<()> {
    // 检查所有权
    require!(
        voter.key() == governance_token.owner,
        AssetError::InvalidOwner
    );
    
    // 验证Governance Token状态
    require!(
        governance_token.is_active(),
        AssetError::AssetNotActive
    );
    
    // 检查投票者代币余额
    require!(
        voter_token_account.amount > 0,
        AssetError::InsufficientVotingTokens
    );
    
    Ok(())
}

/// 投票指令
/// 
/// 执行投票操作，包括：
/// - 参数验证：验证投票参数的有效性
/// - 权限检查：验证投票权限
/// - 服务层调用：委托给GovernanceTokenService执行投票逻辑
/// - 事件发射：发射Governance Token投票事件
/// 
/// # 参数
/// - ctx: 投票账户上下文
/// - params: 投票参数
/// 
/// # 返回
/// - Result<()>: 投票操作结果
pub fn vote_proposal(
    ctx: Context<VoteProposal>,
    params: VoteProposalParams,
) -> Result<()> {
    // 参数验证
    validate_vote_proposal_params(&params)?;
    
    // 权限检查
    check_vote_authority_permission(
        &ctx.accounts.voter,
        &ctx.accounts.governance_token,
        &ctx.accounts.voter_token_account,
    )?;
    
    let governance_token = &mut ctx.accounts.governance_token;
    let voter = &ctx.accounts.voter;
    
    // 创建Governance Token服务实例
    let service = GovernanceTokenService::new();
    
    // 执行投票
    service.vote_proposal(
        governance_token,
        params.proposal_id,
        &params.vote,
        params.voting_power,
        &params.exec_params,
    )?;
    
    // 发射投票事件
    emit!(AssetVoted {
        basket_id: governance_token.id,
        proposal_id: params.proposal_id,
        vote: format!("{:?}", params.vote),
        voting_power: params.voting_power,
        voter: voter.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::GovernanceToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 