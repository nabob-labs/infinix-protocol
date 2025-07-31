//! LP Token销毁指令模块
//! 
//! 本模块提供LP Token资产的销毁功能，包括：
//! - 参数验证：验证销毁参数的有效性和边界条件
//! - 权限检查：验证销毁权限和授权状态
//! - 服务层调用：委托给LpTokenService执行核心业务逻辑
//! - 事件发射：发射LP Token销毁事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于LP Token销毁功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给LpTokenService
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

/// LP Token销毁参数结构体
/// 
/// 包含LP Token销毁所需的所有参数：
/// - amount: 销毁数量
/// - pool_id: 流动性池ID
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BurnLpTokenParams {
    /// 销毁数量
    pub amount: u64,
    /// 流动性池ID
    pub pool_id: Pubkey,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// LP Token销毁账户上下文
/// 
/// 定义LP Token销毁指令所需的账户结构：
/// - lp_token: LP Token账户（可变，LP Token类型约束）
/// - authority: 销毁权限账户（burn_authority约束）
/// - pool: 流动性池账户
/// - system_program: 系统程序
/// - token_program: 代币程序
/// - associated_token_account: 关联代币账户
/// - owner_token_account: 所有者代币账户
#[derive(Accounts)]
pub struct BurnLpToken<'info> {
    /// LP Token账户（可变，LP Token类型约束）
    #[account(
        mut,
        constraint = lp_token.asset_type == AssetType::LpToken @ AssetError::InvalidAssetType
    )]
    pub lp_token: Account<'info, Asset>,
    
    /// 销毁权限账户（burn_authority约束）
    #[account(
        constraint = authority.key() == lp_token.burn_authority @ AssetError::InvalidBurnAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 流动性池账户
    /// CHECK: 由程序验证
    pub pool: UncheckedAccount<'info>,
    
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

/// 验证LP Token销毁参数
/// 
/// 检查LP Token销毁参数的有效性和边界条件：
/// - 销毁数量验证
/// - 流动性池ID验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: LP Token销毁参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_burn_lp_token_params(params: &BurnLpTokenParams) -> Result<()> {
    // 验证销毁数量
    require!(
        params.amount > 0,
        AssetError::InvalidAmount
    );
    
    // 验证流动性池ID
    require!(
        params.pool_id != Pubkey::default(),
        AssetError::InvalidPoolId
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查LP Token销毁权限
/// 
/// 验证LP Token销毁权限和授权状态：
/// - 检查销毁权限
/// - 验证授权状态
/// 
/// # 参数
/// - authority: 权限账户
/// - lp_token: LP Token账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_burn_authority_permission(
    authority: &Signer,
    lp_token: &Account<Asset>,
) -> Result<()> {
    // 检查销毁权限
    require!(
        authority.key() == lp_token.burn_authority,
        AssetError::InvalidBurnAuthority
    );
    
    // 验证LP Token状态
    require!(
        lp_token.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// LP Token销毁指令
/// 
/// 执行LP Token销毁操作，包括：
/// - 参数验证：验证销毁参数的有效性
/// - 权限检查：验证销毁权限
/// - 服务层调用：委托给LpTokenService执行销毁逻辑
/// - 事件发射：发射LP Token销毁事件
/// 
/// # 参数
/// - ctx: LP Token销毁账户上下文
/// - params: LP Token销毁参数
/// 
/// # 返回
/// - Result<()>: 销毁操作结果
pub fn burn_lp_token(
    ctx: Context<BurnLpToken>,
    params: BurnLpTokenParams,
) -> Result<()> {
    // 参数验证
    validate_burn_lp_token_params(&params)?;
    
    // 权限检查
    check_burn_authority_permission(&ctx.accounts.authority, &ctx.accounts.lp_token)?;
    
    let lp_token = &mut ctx.accounts.lp_token;
    let authority = &ctx.accounts.authority;
    
    // 创建LP Token服务实例
    let service = LpTokenService::new();
    
    // 执行LP Token销毁
    service.burn(
        lp_token,
        params.amount,
        params.pool_id,
        &params.exec_params,
    )?;
    
    // 发射LP Token销毁事件
    emit!(AssetBurned {
        basket_id: lp_token.id,
        amount: params.amount,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::LpToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
}
