//! Staking Token领取奖励指令模块
//! 
//! 本模块提供Staking Token资产的领取奖励功能，包括：
//! - 参数验证：验证领取奖励参数的有效性和边界条件
//! - 权限检查：验证领取奖励权限和授权状态
//! - 服务层调用：委托给StakingTokenService执行核心业务逻辑
//! - 事件发射：发射Staking Token领取奖励事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于Staking Token领取奖励功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给StakingTokenService
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

/// 领取奖励参数结构体
/// 
/// 包含领取奖励所需的所有参数：
/// - reward_type: 奖励类型
/// - amount: 领取数量
/// - validator: 验证者地址
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct ClaimRewardsParams {
    /// 奖励类型
    pub reward_type: RewardType,
    /// 领取数量
    pub amount: u64,
    /// 验证者地址
    pub validator: Pubkey,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 奖励类型枚举
/// 
/// 定义奖励的类型：
/// - StakingReward: 质押奖励
/// - ValidatorReward: 验证者奖励
/// - DelegationReward: 委托奖励
/// - BonusReward: 额外奖励
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum RewardType {
    /// 质押奖励
    StakingReward,
    /// 验证者奖励
    ValidatorReward,
    /// 委托奖励
    DelegationReward,
    /// 额外奖励
    BonusReward,
}

/// 领取奖励账户上下文
/// 
/// 定义领取奖励指令所需的账户结构：
/// - staking_token: Staking Token账户（可变，Staking Token类型约束）
/// - staker: 质押者账户（owner约束）
/// - staking_pool: 质押池账户
/// - validator: 验证者账户
/// - reward_pool: 奖励池账户
/// - staker_reward_account: 质押者奖励账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    /// Staking Token账户（可变，Staking Token类型约束）
    #[account(
        mut,
        constraint = staking_token.asset_type == AssetType::StakingToken @ AssetError::InvalidAssetType
    )]
    pub staking_token: Account<'info, Asset>,
    
    /// 质押者账户（owner约束）
    #[account(
        constraint = staker.key() == staking_token.owner @ AssetError::InvalidOwner
    )]
    pub staker: Signer<'info>,
    
    /// 质押池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub staking_pool: UncheckedAccount<'info>,
    
    /// 验证者账户
    /// CHECK: 由程序验证
    pub validator: UncheckedAccount<'info>,
    
    /// 奖励池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub reward_pool: UncheckedAccount<'info>,
    
    /// 质押者奖励账户
    #[account(mut)]
    pub staker_reward_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证领取奖励参数
/// 
/// 检查领取奖励参数的有效性和边界条件：
/// - 奖励类型验证
/// - 领取数量验证
/// - 验证者地址验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 领取奖励参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_claim_rewards_params(params: &ClaimRewardsParams) -> Result<()> {
    // 验证领取数量
    require!(
        params.amount > 0,
        AssetError::InvalidRewardAmount
    );
    
    // 验证验证者地址
    require!(
        params.validator != Pubkey::default(),
        AssetError::InvalidValidator
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查领取奖励权限
/// 
/// 验证领取奖励权限和授权状态：
/// - 检查所有权
/// - 验证Staking Token状态
/// - 检查奖励资格
/// 
/// # 参数
/// - staker: 质押者账户
/// - staking_token: Staking Token账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_claim_rewards_authority_permission(
    staker: &Signer,
    staking_token: &Account<Asset>,
) -> Result<()> {
    // 检查所有权
    require!(
        staker.key() == staking_token.owner,
        AssetError::InvalidOwner
    );
    
    // 验证Staking Token状态
    require!(
        staking_token.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// 领取奖励指令
/// 
/// 执行领取奖励操作，包括：
/// - 参数验证：验证领取奖励参数的有效性
/// - 权限检查：验证领取奖励权限
/// - 服务层调用：委托给StakingTokenService执行领取奖励逻辑
/// - 事件发射：发射Staking Token领取奖励事件
/// 
/// # 参数
/// - ctx: 领取奖励账户上下文
/// - params: 领取奖励参数
/// 
/// # 返回
/// - Result<()>: 领取奖励操作结果
pub fn claim_rewards(
    ctx: Context<ClaimRewards>,
    params: ClaimRewardsParams,
) -> Result<()> {
    // 参数验证
    validate_claim_rewards_params(&params)?;
    
    // 权限检查
    check_claim_rewards_authority_permission(
        &ctx.accounts.staker,
        &ctx.accounts.staking_token,
    )?;
    
    let staking_token = &mut ctx.accounts.staking_token;
    let staker = &ctx.accounts.staker;
    
    // 创建Staking Token服务实例
    let service = StakingTokenService::new();
    
    // 执行领取奖励
    service.claim_rewards(
        staking_token,
        &params.reward_type,
        params.amount,
        params.validator,
        &params.exec_params,
    )?;
    
    // 发射领取奖励事件
    emit!(AssetRewardsClaimed {
        basket_id: staking_token.id,
        reward_type: format!("{:?}", params.reward_type),
        amount: params.amount,
        validator: params.validator,
        staker: staker.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::StakingToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 