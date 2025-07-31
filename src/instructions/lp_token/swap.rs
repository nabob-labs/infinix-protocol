//! LP Token兑换指令模块
//! 
//! 本模块提供LP Token资产的兑换功能，包括：
//! - 参数验证：验证兑换参数的有效性和边界条件
//! - 权限检查：验证兑换权限和授权状态
//! - 服务层调用：委托给LpTokenService执行核心业务逻辑
//! - 事件发射：发射LP Token兑换事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于LP Token兑换功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给LpTokenService
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

/// LP Token兑换参数结构体
/// 
/// 包含LP Token兑换所需的所有参数：
/// - input_token: 输入代币
/// - output_token: 输出代币
/// - input_amount: 输入数量
/// - min_output_amount: 最小输出数量
/// - slippage_tolerance: 滑点容忍度
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct SwapLpTokenParams {
    /// 输入代币
    pub input_token: Pubkey,
    /// 输出代币
    pub output_token: Pubkey,
    /// 输入数量
    pub input_amount: u64,
    /// 最小输出数量
    pub min_output_amount: u64,
    /// 滑点容忍度
    pub slippage_tolerance: u64,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// LP Token兑换账户上下文
/// 
/// 定义LP Token兑换指令所需的账户结构：
/// - lp_token: LP Token账户（可变，LP Token类型约束）
/// - authority: 兑换权限账户（owner约束）
/// - pool: 流动性池账户
/// - input_token_account: 输入代币账户
/// - output_token_account: 输出代币账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct SwapLpToken<'info> {
    /// LP Token账户（可变，LP Token类型约束）
    #[account(
        mut,
        constraint = lp_token.asset_type == AssetType::LpToken @ AssetError::InvalidAssetType
    )]
    pub lp_token: Account<'info, Asset>,
    
    /// 兑换权限账户（owner约束）
    #[account(
        constraint = authority.key() == lp_token.owner @ AssetError::InvalidOwner
    )]
    pub authority: Signer<'info>,
    
    /// 流动性池账户
    /// CHECK: 由程序验证
    pub pool: UncheckedAccount<'info>,
    
    /// 输入代币账户
    #[account(
        mut,
        constraint = input_token_account.owner == authority.key() @ AssetError::InvalidTokenAccount
    )]
    pub input_token_account: Account<'info, TokenAccount>,
    
    /// 输出代币账户
    #[account(
        mut,
        constraint = output_token_account.owner == authority.key() @ AssetError::InvalidTokenAccount
    )]
    pub output_token_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证LP Token兑换参数
/// 
/// 检查LP Token兑换参数的有效性和边界条件：
/// - 输入代币验证
/// - 输出代币验证
/// - 输入数量验证
/// - 最小输出数量验证
/// - 滑点容忍度验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: LP Token兑换参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_swap_lp_token_params(params: &SwapLpTokenParams) -> Result<()> {
    // 验证输入代币
    require!(
        params.input_token != Pubkey::default(),
        AssetError::InvalidInputToken
    );
    
    // 验证输出代币
    require!(
        params.output_token != Pubkey::default(),
        AssetError::InvalidOutputToken
    );
    
    // 验证输入代币和输出代币不能相同
    require!(
        params.input_token != params.output_token,
        AssetError::SameInputOutputToken
    );
    
    // 验证输入数量
    require!(
        params.input_amount > 0,
        AssetError::InvalidInputAmount
    );
    
    // 验证最小输出数量
    require!(
        params.min_output_amount > 0,
        AssetError::InvalidMinOutputAmount
    );
    
    // 验证滑点容忍度
    require!(
        params.slippage_tolerance <= MAX_SLIPPAGE_TOLERANCE,
        AssetError::SlippageToleranceTooHigh
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查LP Token兑换权限
/// 
/// 验证LP Token兑换权限和授权状态：
/// - 检查所有权
/// - 验证LP Token状态
/// - 检查输入代币余额
/// 
/// # 参数
/// - authority: 权限账户
/// - lp_token: LP Token账户
/// - input_token_account: 输入代币账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_swap_authority_permission(
    authority: &Signer,
    lp_token: &Account<Asset>,
    input_token_account: &Account<TokenAccount>,
) -> Result<()> {
    // 检查所有权
    require!(
        authority.key() == lp_token.owner,
        AssetError::InvalidOwner
    );
    
    // 验证LP Token状态
    require!(
        lp_token.is_active(),
        AssetError::AssetNotActive
    );
    
    // 检查输入代币余额
    require!(
        input_token_account.amount > 0,
        AssetError::InsufficientInputTokenBalance
    );
    
    Ok(())
}

/// LP Token兑换指令
/// 
/// 执行LP Token兑换操作，包括：
/// - 参数验证：验证兑换参数的有效性
/// - 权限检查：验证兑换权限
/// - 服务层调用：委托给LpTokenService执行兑换逻辑
/// - 事件发射：发射LP Token兑换事件
/// 
/// # 参数
/// - ctx: LP Token兑换账户上下文
/// - params: LP Token兑换参数
/// 
/// # 返回
/// - Result<()>: 兑换操作结果
pub fn swap_lp_token(
    ctx: Context<SwapLpToken>,
    params: SwapLpTokenParams,
) -> Result<()> {
    // 参数验证
    validate_swap_lp_token_params(&params)?;
    
    // 权限检查
    check_swap_authority_permission(
        &ctx.accounts.authority,
        &ctx.accounts.lp_token,
        &ctx.accounts.input_token_account,
    )?;
    
    let lp_token = &mut ctx.accounts.lp_token;
    let authority = &ctx.accounts.authority;
    
    // 创建LP Token服务实例
    let service = LpTokenService::new();
    
    // 执行LP Token兑换
    service.swap(
        lp_token,
        params.input_token,
        params.output_token,
        params.input_amount,
        params.min_output_amount,
        params.slippage_tolerance,
        &params.exec_params,
    )?;
    
    // 发射LP Token兑换事件
    emit!(AssetSwapped {
        basket_id: lp_token.id,
        input_token: params.input_token,
        output_token: params.output_token,
        input_amount: params.input_amount,
        min_output_amount: params.min_output_amount,
        slippage_tolerance: params.slippage_tolerance,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::LpToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 