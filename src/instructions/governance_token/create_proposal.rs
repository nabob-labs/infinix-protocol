//! Governance Token创建提案指令模块
//! 
//! 本模块提供Governance Token资产的创建提案功能，包括：
//! - 参数验证：验证创建提案参数的有效性和边界条件
//! - 权限检查：验证创建提案权限和授权状态
//! - 服务层调用：委托给GovernanceTokenService执行核心业务逻辑
//! - 事件发射：发射Governance Token创建提案事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于Governance Token创建提案功能
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

/// 创建提案参数结构体
/// 
/// 包含创建提案所需的所有参数：
/// - title: 提案标题
/// - description: 提案描述
/// - proposal_type: 提案类型
/// - voting_period: 投票期限
/// - quorum: 法定人数
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct CreateProposalParams {
    /// 提案标题
    pub title: String,
    /// 提案描述
    pub description: String,
    /// 提案类型
    pub proposal_type: ProposalType,
    /// 投票期限
    pub voting_period: i64,
    /// 法定人数
    pub quorum: u64,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 提案类型枚举
/// 
/// 定义提案的类型：
/// - ParameterChange: 参数变更
/// - TreasurySpend: 资金支出
/// - ProtocolUpgrade: 协议升级
/// - EmergencyAction: 紧急行动
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ProposalType {
    /// 参数变更
    ParameterChange,
    /// 资金支出
    TreasurySpend,
    /// 协议升级
    ProtocolUpgrade,
    /// 紧急行动
    EmergencyAction,
}

/// 创建提案账户上下文
/// 
/// 定义创建提案指令所需的账户结构：
/// - governance_token: Governance Token账户（可变，Governance Token类型约束）
/// - authority: 创建提案权限账户（owner约束）
/// - proposal: 提案账户
/// - governance: 治理账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct CreateProposal<'info> {
    /// Governance Token账户（可变，Governance Token类型约束）
    #[account(
        mut,
        constraint = governance_token.asset_type == AssetType::GovernanceToken @ AssetError::InvalidAssetType
    )]
    pub governance_token: Account<'info, Asset>,
    
    /// 创建提案权限账户（owner约束）
    #[account(
        constraint = authority.key() == governance_token.owner @ AssetError::InvalidOwner
    )]
    pub authority: Signer<'info>,
    
    /// 提案账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub proposal: UncheckedAccount<'info>,
    
    /// 治理账户
    /// CHECK: 由程序验证
    pub governance: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证创建提案参数
/// 
/// 检查创建提案参数的有效性和边界条件：
/// - 提案标题验证
/// - 提案描述验证
/// - 投票期限验证
/// - 法定人数验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 创建提案参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_create_proposal_params(params: &CreateProposalParams) -> Result<()> {
    // 验证提案标题
    require!(
        !params.title.is_empty(),
        AssetError::InvalidProposalTitle
    );
    
    require!(
        params.title.len() <= MAX_PROPOSAL_TITLE_LENGTH,
        AssetError::ProposalTitleTooLong
    );
    
    // 验证提案描述
    require!(
        !params.description.is_empty(),
        AssetError::InvalidProposalDescription
    );
    
    require!(
        params.description.len() <= MAX_PROPOSAL_DESCRIPTION_LENGTH,
        AssetError::ProposalDescriptionTooLong
    );
    
    // 验证投票期限
    require!(
        params.voting_period > 0,
        AssetError::InvalidVotingPeriod
    );
    
    require!(
        params.voting_period <= MAX_VOTING_PERIOD,
        AssetError::VotingPeriodTooLong
    );
    
    // 验证法定人数
    require!(
        params.quorum > 0,
        AssetError::InvalidQuorum
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查创建提案权限
/// 
/// 验证创建提案权限和授权状态：
/// - 检查所有权
/// - 验证Governance Token状态
/// - 检查提案创建权限
/// 
/// # 参数
/// - authority: 权限账户
/// - governance_token: Governance Token账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_create_proposal_authority_permission(
    authority: &Signer,
    governance_token: &Account<Asset>,
) -> Result<()> {
    // 检查所有权
    require!(
        authority.key() == governance_token.owner,
        AssetError::InvalidOwner
    );
    
    // 验证Governance Token状态
    require!(
        governance_token.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// 创建提案指令
/// 
/// 执行创建提案操作，包括：
/// - 参数验证：验证创建提案参数的有效性
/// - 权限检查：验证创建提案权限
/// - 服务层调用：委托给GovernanceTokenService执行创建提案逻辑
/// - 事件发射：发射Governance Token创建提案事件
/// 
/// # 参数
/// - ctx: 创建提案账户上下文
/// - params: 创建提案参数
/// 
/// # 返回
/// - Result<()>: 创建提案操作结果
pub fn create_proposal(
    ctx: Context<CreateProposal>,
    params: CreateProposalParams,
) -> Result<()> {
    // 参数验证
    validate_create_proposal_params(&params)?;
    
    // 权限检查
    check_create_proposal_authority_permission(
        &ctx.accounts.authority,
        &ctx.accounts.governance_token,
    )?;
    
    let governance_token = &mut ctx.accounts.governance_token;
    let authority = &ctx.accounts.authority;
    
    // 创建Governance Token服务实例
    let service = GovernanceTokenService::new();
    
    // 执行创建提案
    service.create_proposal(
        governance_token,
        &params.title,
        &params.description,
        &params.proposal_type,
        params.voting_period,
        params.quorum,
        &params.exec_params,
    )?;
    
    // 发射创建提案事件
    emit!(AssetProposalCreated {
        basket_id: governance_token.id,
        title: params.title,
        description: params.description,
        proposal_type: format!("{:?}", params.proposal_type),
        voting_period: params.voting_period,
        quorum: params.quorum,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::GovernanceToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 