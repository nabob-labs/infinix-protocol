//! Staking Token销毁指令模块
//! 
//! 本模块提供Staking Token资产的销毁功能，包括：
//! - 参数验证：验证销毁参数的有效性和边界条件
//! - 权限检查：验证销毁权限和授权状态
//! - 服务层调用：委托给StakingTokenService执行核心业务逻辑
//! - 事件发射：发射Staking Token销毁事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于Staking Token销毁功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给StakingTokenService
//! - 事件驱动：完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

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

/// Staking Token销毁参数结构体
/// 
/// 包含Staking Token销毁所需的所有参数：
/// - amount: 销毁数量
/// - staking_pool_id: 质押池ID
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BurnStakingParams {
    /// 销毁数量
    pub amount: u64,
    /// 质押池ID
    pub staking_pool_id: Pubkey,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// Staking Token销毁账户上下文
/// 
/// 定义Staking Token销毁指令所需的账户结构：
/// - staking_token: Staking Token账户（可变，Staking Token类型约束）
/// - authority: 销毁权限账户（burn_authority约束）
/// - staking_pool: 质押池账户
/// - system_program: 系统程序
/// - token_program: 代币程序
/// - associated_token_account: 关联代币账户
/// - owner_token_account: 所有者代币账户
#[derive(Accounts)]
pub struct BurnStaking<'info> {
    /// Staking Token账户（可变，Staking Token类型约束）
    #[account(
        mut,
        constraint = staking_token.asset_type == AssetType::StakingToken @ AssetError::InvalidAssetType
    )]
    pub staking_token: Account<'info, Asset>,
    
    /// 销毁权限账户（burn_authority约束）
    #[account(
        constraint = authority.key() == staking_token.burn_authority @ AssetError::InvalidBurnAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 质押池账户
    /// CHECK: 由程序验证
    pub staking_pool: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
    
    /// 关联代币账户
    /// CHECK: 由代币程序验证
    pub associated_token_account: UncheckedAccount<'info>,
    
    /// 所有者代币账户
    #[account(mut)]
    pub owner_token_account: Account<'info, TokenAccount>,
}

/// 验证Staking Token销毁参数
/// 
/// 检查Staking Token销毁参数的有效性和边界条件：
/// - 销毁数量验证
/// - 质押池ID验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: Staking Token销毁参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_burn_staking_params(params: &BurnStakingParams) -> Result<()> {
    // 验证销毁数量
    require!(
        params.amount > 0,
        AssetError::InvalidAmount
    );
    
    // 验证质押池ID
    require!(
        params.staking_pool_id != Pubkey::default(),
        AssetError::InvalidStakingPoolId
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查Staking Token销毁权限
/// 
/// 验证Staking Token销毁权限和授权状态：
/// - 检查销毁权限
/// - 验证授权状态
/// 
/// # 参数
/// - authority: 权限账户
/// - staking_token: Staking Token账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_burn_authority_permission(
    authority: &Signer,
    staking_token: &Account<Asset>,
) -> Result<()> {
    // 检查销毁权限
    require!(
        authority.key() == staking_token.burn_authority,
        AssetError::InvalidBurnAuthority
    );
    
    // 验证Staking Token状态
    require!(
        staking_token.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// Staking Token销毁指令
/// 
/// 执行Staking Token销毁操作，包括：
/// - 参数验证：验证销毁参数的有效性
/// - 权限检查：验证销毁权限
/// - 服务层调用：委托给StakingTokenService执行销毁逻辑
/// - 事件发射：发射Staking Token销毁事件
/// 
/// # 参数
/// - ctx: Staking Token销毁账户上下文
/// - params: Staking Token销毁参数
/// 
/// # 返回
/// - Result<()>: 销毁操作结果
pub fn burn_staking_token(
    ctx: Context<BurnStaking>,
    params: BurnStakingParams,
) -> Result<()> {
    // 参数验证
    validate_burn_staking_params(&params)?;
    
    // 权限检查
    check_burn_authority_permission(&ctx.accounts.authority, &ctx.accounts.staking_token)?;
    
    let staking_token = &mut ctx.accounts.staking_token;
    let authority = &ctx.accounts.authority;
    
    // 创建Staking Token服务实例
    let service = StakingTokenService::new();
    
    // 执行Staking Token销毁
    service.burn(
        staking_token,
        params.amount,
        params.staking_pool_id,
        &params.exec_params,
    )?;
    
    // 发射Staking Token销毁事件
    emit!(AssetBurned {
        basket_id: staking_token.id,
        amount: params.amount,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::StakingToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
}
