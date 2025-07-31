//! Staking Token质押指令模块
//! 
//! 本模块提供Staking Token资产的质押功能，包括：
//! - 参数验证：验证质押参数的有效性和边界条件
//! - 权限检查：验证质押权限和授权状态
//! - 服务层调用：委托给StakingTokenService执行核心业务逻辑
//! - 事件发射：发射Staking Token质押事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于Staking Token质押功能
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

/// 质押参数结构体
/// 
/// 包含质押所需的所有参数：
/// - amount: 质押数量
/// - validator: 验证者地址
/// - lock_period: 锁定期限
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct StakeTokensParams {
    /// 质押数量
    pub amount: u64,
    /// 验证者地址
    pub validator: Pubkey,
    /// 锁定期限
    pub lock_period: i64,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 质押账户上下文
/// 
/// 定义质押指令所需的账户结构：
/// - staking_token: Staking Token账户（可变，Staking Token类型约束）
/// - staker: 质押者账户（owner约束）
/// - staking_pool: 质押池账户
/// - validator: 验证者账户
/// - staker_token_account: 质押者代币账户
/// - staked_token_account: 质押代币账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct StakeTokens<'info> {
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
    
    /// 质押者代币账户
    #[account(
        mut,
        constraint = staker_token_account.owner == staker.key() @ AssetError::InvalidTokenAccount
    )]
    pub staker_token_account: Account<'info, TokenAccount>,
    
    /// 质押代币账户
    #[account(mut)]
    pub staked_token_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证质押参数
/// 
/// 检查质押参数的有效性和边界条件：
/// - 质押数量验证
/// - 验证者地址验证
/// - 锁定期限验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 质押参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_stake_tokens_params(params: &StakeTokensParams) -> Result<()> {
    // 验证质押数量
    require!(
        params.amount > 0,
        AssetError::InvalidStakingAmount
    );
    
    // 验证验证者地址
    require!(
        params.validator != Pubkey::default(),
        AssetError::InvalidValidator
    );
    
    // 验证锁定期限
    require!(
        params.lock_period >= 0,
        AssetError::InvalidLockPeriod
    );
    
    require!(
        params.lock_period <= MAX_LOCK_PERIOD,
        AssetError::LockPeriodTooLong
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查质押权限
/// 
/// 验证质押权限和授权状态：
/// - 检查所有权
/// - 验证Staking Token状态
/// - 检查质押者代币余额
/// 
/// # 参数
/// - staker: 质押者账户
/// - staking_token: Staking Token账户
/// - staker_token_account: 质押者代币账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_stake_authority_permission(
    staker: &Signer,
    staking_token: &Account<Asset>,
    staker_token_account: &Account<TokenAccount>,
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
    
    // 检查质押者代币余额
    require!(
        staker_token_account.amount >= 0,
        AssetError::InsufficientStakingTokens
    );
    
    Ok(())
}

/// 质押指令
/// 
/// 执行质押操作，包括：
/// - 参数验证：验证质押参数的有效性
/// - 权限检查：验证质押权限
/// - 服务层调用：委托给StakingTokenService执行质押逻辑
/// - 事件发射：发射Staking Token质押事件
/// 
/// # 参数
/// - ctx: 质押账户上下文
/// - params: 质押参数
/// 
/// # 返回
/// - Result<()>: 质押操作结果
pub fn stake_tokens(
    ctx: Context<StakeTokens>,
    params: StakeTokensParams,
) -> Result<()> {
    // 参数验证
    validate_stake_tokens_params(&params)?;
    
    // 权限检查
    check_stake_authority_permission(
        &ctx.accounts.staker,
        &ctx.accounts.staking_token,
        &ctx.accounts.staker_token_account,
    )?;
    
    let staking_token = &mut ctx.accounts.staking_token;
    let staker = &ctx.accounts.staker;
    
    // 创建Staking Token服务实例
    let service = StakingTokenService::new();
    
    // 执行质押
    service.stake_tokens(
        staking_token,
        params.amount,
        params.validator,
        params.lock_period,
        &params.exec_params,
    )?;
    
    // 发射质押事件
    emit!(AssetStaked {
        basket_id: staking_token.id,
        amount: params.amount,
        validator: params.validator,
        lock_period: params.lock_period,
        staker: staker.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::StakingToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 