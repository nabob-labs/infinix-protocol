//! Yield Token生成收益指令模块
//! 
//! 本模块提供Yield Token资产的生成收益功能，包括：
//! - 参数验证：验证生成收益参数的有效性和边界条件
//! - 权限检查：验证生成收益权限和授权状态
//! - 服务层调用：委托给YieldTokenService执行核心业务逻辑
//! - 事件发射：发射Yield Token生成收益事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于Yield Token生成收益功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给YieldTokenService
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

/// 生成收益参数结构体
/// 
/// 包含生成收益所需的所有参数：
/// - yield_rate: 收益率
/// - yield_amount: 收益数量
/// - yield_strategy: 收益策略
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct GenerateYieldParams {
    /// 收益率
    pub yield_rate: f64,
    /// 收益数量
    pub yield_amount: u64,
    /// 收益策略
    pub yield_strategy: YieldStrategy,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 收益策略枚举
/// 
/// 定义收益策略的类型：
/// - Lending: 借贷收益
/// - LiquidityProvision: 流动性提供收益
/// - Staking: 质押收益
/// - Trading: 交易收益
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum YieldStrategy {
    /// 借贷收益
    Lending,
    /// 流动性提供收益
    LiquidityProvision,
    /// 质押收益
    Staking,
    /// 交易收益
    Trading,
}

/// 生成收益账户上下文
/// 
/// 定义生成收益指令所需的账户结构：
/// - yield_token: Yield Token账户（可变，Yield Token类型约束）
/// - authority: 生成收益权限账户（owner约束）
/// - yield_pool: 收益池账户
/// - yield_strategy_pool: 收益策略池账户
/// - yield_distribution_pool: 收益分配池账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct GenerateYield<'info> {
    /// Yield Token账户（可变，Yield Token类型约束）
    #[account(
        mut,
        constraint = yield_token.asset_type == AssetType::YieldToken @ AssetError::InvalidAssetType
    )]
    pub yield_token: Account<'info, Asset>,
    
    /// 生成收益权限账户（owner约束）
    #[account(
        constraint = authority.key() == yield_token.owner @ AssetError::InvalidOwner
    )]
    pub authority: Signer<'info>,
    
    /// 收益池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub yield_pool: UncheckedAccount<'info>,
    
    /// 收益策略池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub yield_strategy_pool: UncheckedAccount<'info>,
    
    /// 收益分配池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub yield_distribution_pool: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证生成收益参数
/// 
/// 检查生成收益参数的有效性和边界条件：
/// - 收益率验证
/// - 收益数量验证
/// - 收益策略验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 生成收益参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_generate_yield_params(params: &GenerateYieldParams) -> Result<()> {
    // 验证收益率
    require!(
        params.yield_rate > 0.0,
        AssetError::InvalidYieldRate
    );
    
    require!(
        params.yield_rate <= MAX_YIELD_RATE,
        AssetError::YieldRateTooHigh
    );
    
    // 验证收益数量
    require!(
        params.yield_amount > 0,
        AssetError::InvalidYieldAmount
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查生成收益权限
/// 
/// 验证生成收益权限和授权状态：
/// - 检查所有权
/// - 验证Yield Token状态
/// - 检查收益生成权限
/// 
/// # 参数
/// - authority: 权限账户
/// - yield_token: Yield Token账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_generate_yield_authority_permission(
    authority: &Signer,
    yield_token: &Account<Asset>,
) -> Result<()> {
    // 检查所有权
    require!(
        authority.key() == yield_token.owner,
        AssetError::InvalidOwner
    );
    
    // 验证Yield Token状态
    require!(
        yield_token.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// 生成收益指令
/// 
/// 执行生成收益操作，包括：
/// - 参数验证：验证生成收益参数的有效性
/// - 权限检查：验证生成收益权限
/// - 服务层调用：委托给YieldTokenService执行生成收益逻辑
/// - 事件发射：发射Yield Token生成收益事件
/// 
/// # 参数
/// - ctx: 生成收益账户上下文
/// - params: 生成收益参数
/// 
/// # 返回
/// - Result<()>: 生成收益操作结果
pub fn generate_yield(
    ctx: Context<GenerateYield>,
    params: GenerateYieldParams,
) -> Result<()> {
    // 参数验证
    validate_generate_yield_params(&params)?;
    
    // 权限检查
    check_generate_yield_authority_permission(
        &ctx.accounts.authority,
        &ctx.accounts.yield_token,
    )?;
    
    let yield_token = &mut ctx.accounts.yield_token;
    let authority = &ctx.accounts.authority;
    
    // 创建Yield Token服务实例
    let service = YieldTokenService::new();
    
    // 执行生成收益
    service.generate_yield(
        yield_token,
        params.yield_rate,
        params.yield_amount,
        &params.yield_strategy,
        &params.exec_params,
    )?;
    
    // 发射生成收益事件
    emit!(AssetYieldGenerated {
        basket_id: yield_token.id,
        yield_rate: params.yield_rate,
        yield_amount: params.yield_amount,
        yield_strategy: format!("{:?}", params.yield_strategy),
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::YieldToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 