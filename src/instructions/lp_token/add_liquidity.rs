//! LP Token添加流动性指令模块
//! 
//! 本模块提供LP Token资产的添加流动性功能，包括：
//! - 参数验证：验证添加流动性参数的有效性和边界条件
//! - 权限检查：验证添加流动性权限和授权状态
//! - 服务层调用：委托给LpTokenService执行核心业务逻辑
//! - 事件发射：发射LP Token添加流动性事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于LP Token添加流动性功能
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

/// 添加流动性参数结构体
/// 
/// 包含添加流动性所需的所有参数：
/// - token_a_amount: 代币A数量
/// - token_b_amount: 代币B数量
/// - min_lp_tokens: 最小LP代币数量
/// - slippage_tolerance: 滑点容忍度
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct AddLiquidityParams {
    /// 代币A数量
    pub token_a_amount: u64,
    /// 代币B数量
    pub token_b_amount: u64,
    /// 最小LP代币数量
    pub min_lp_tokens: u64,
    /// 滑点容忍度
    pub slippage_tolerance: u64,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 添加流动性账户上下文
/// 
/// 定义添加流动性指令所需的账户结构：
/// - lp_token: LP Token账户（可变，LP Token类型约束）
/// - authority: 添加流动性权限账户（owner约束）
/// - pool: 流动性池账户
/// - token_a_account: 代币A账户
/// - token_b_account: 代币B账户
/// - lp_token_account: LP代币账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    /// LP Token账户（可变，LP Token类型约束）
    #[account(
        mut,
        constraint = lp_token.asset_type == AssetType::LpToken @ AssetError::InvalidAssetType
    )]
    pub lp_token: Account<'info, Asset>,
    
    /// 添加流动性权限账户（owner约束）
    #[account(
        constraint = authority.key() == lp_token.owner @ AssetError::InvalidOwner
    )]
    pub authority: Signer<'info>,
    
    /// 流动性池账户
    /// CHECK: 由程序验证
    pub pool: UncheckedAccount<'info>,
    
    /// 代币A账户
    #[account(
        mut,
        constraint = token_a_account.owner == authority.key() @ AssetError::InvalidTokenAccount
    )]
    pub token_a_account: Account<'info, TokenAccount>,
    
    /// 代币B账户
    #[account(
        mut,
        constraint = token_b_account.owner == authority.key() @ AssetError::InvalidTokenAccount
    )]
    pub token_b_account: Account<'info, TokenAccount>,
    
    /// LP代币账户
    #[account(mut)]
    pub lp_token_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证添加流动性参数
/// 
/// 检查添加流动性参数的有效性和边界条件：
/// - 代币数量验证
/// - 最小LP代币数量验证
/// - 滑点容忍度验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 添加流动性参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_add_liquidity_params(params: &AddLiquidityParams) -> Result<()> {
    // 验证代币A数量
    require!(
        params.token_a_amount > 0,
        AssetError::InvalidTokenAmount
    );
    
    // 验证代币B数量
    require!(
        params.token_b_amount > 0,
        AssetError::InvalidTokenAmount
    );
    
    // 验证最小LP代币数量
    require!(
        params.min_lp_tokens > 0,
        AssetError::InvalidMinLpTokens
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

/// 检查添加流动性权限
/// 
/// 验证添加流动性权限和授权状态：
/// - 检查所有权
/// - 验证LP Token状态
/// - 检查代币余额
/// 
/// # 参数
/// - authority: 权限账户
/// - lp_token: LP Token账户
/// - token_a_account: 代币A账户
/// - token_b_account: 代币B账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_add_liquidity_authority_permission(
    authority: &Signer,
    lp_token: &Account<Asset>,
    token_a_account: &Account<TokenAccount>,
    token_b_account: &Account<TokenAccount>,
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
    
    // 检查代币A余额
    require!(
        token_a_account.amount > 0,
        AssetError::InsufficientTokenABalance
    );
    
    // 检查代币B余额
    require!(
        token_b_account.amount > 0,
        AssetError::InsufficientTokenBBalance
    );
    
    Ok(())
}

/// 添加流动性指令
/// 
/// 执行添加流动性操作，包括：
/// - 参数验证：验证添加流动性参数的有效性
/// - 权限检查：验证添加流动性权限
/// - 服务层调用：委托给LpTokenService执行添加流动性逻辑
/// - 事件发射：发射LP Token添加流动性事件
/// 
/// # 参数
/// - ctx: 添加流动性账户上下文
/// - params: 添加流动性参数
/// 
/// # 返回
/// - Result<()>: 添加流动性操作结果
pub fn add_liquidity(
    ctx: Context<AddLiquidity>,
    params: AddLiquidityParams,
) -> Result<()> {
    // 参数验证
    validate_add_liquidity_params(&params)?;
    
    // 权限检查
    check_add_liquidity_authority_permission(
        &ctx.accounts.authority,
        &ctx.accounts.lp_token,
        &ctx.accounts.token_a_account,
        &ctx.accounts.token_b_account,
    )?;
    
    let lp_token = &mut ctx.accounts.lp_token;
    let authority = &ctx.accounts.authority;
    
    // 创建LP Token服务实例
    let service = LpTokenService::new();
    
    // 执行添加流动性
    service.add_liquidity(
        lp_token,
        params.token_a_amount,
        params.token_b_amount,
        params.min_lp_tokens,
        params.slippage_tolerance,
        &params.exec_params,
    )?;
    
    // 发射添加流动性事件
    emit!(AssetLiquidityAdded {
        basket_id: lp_token.id,
        token_a_amount: params.token_a_amount,
        token_b_amount: params.token_b_amount,
        min_lp_tokens: params.min_lp_tokens,
        slippage_tolerance: params.slippage_tolerance,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::LpToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 