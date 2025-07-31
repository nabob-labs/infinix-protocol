//! Margin Token借入指令模块
//! 
//! 本模块提供Margin Token资产的借入功能，包括：
//! - 参数验证：验证借入参数的有效性和边界条件
//! - 权限检查：验证借入权限和授权状态
//! - 服务层调用：委托给MarginTokenService执行核心业务逻辑
//! - 事件发射：发射Margin Token借入事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于Margin Token借入功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给MarginTokenService
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

/// 借入参数结构体
/// 
/// 包含借入所需的所有参数：
/// - borrow_amount: 借入数量
/// - collateral_asset: 抵押品资产
/// - collateral_amount: 抵押品数量
/// - interest_rate: 利率
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct BorrowMarginParams {
    /// 借入数量
    pub borrow_amount: u64,
    /// 抵押品资产
    pub collateral_asset: Pubkey,
    /// 抵押品数量
    pub collateral_amount: u64,
    /// 利率
    pub interest_rate: f64,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 借入账户上下文
/// 
/// 定义借入指令所需的账户结构：
/// - margin_token: Margin Token账户（可变，Margin Token类型约束）
/// - borrower: 借入者账户（owner约束）
/// - margin_pool: 保证金池账户
/// - collateral_pool: 抵押品池账户
/// - borrower_token_account: 借入者代币账户
/// - collateral_token_account: 抵押品代币账户
/// - oracle: 预言机账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct BorrowMargin<'info> {
    /// Margin Token账户（可变，Margin Token类型约束）
    #[account(
        mut,
        constraint = margin_token.asset_type == AssetType::MarginToken @ AssetError::InvalidAssetType
    )]
    pub margin_token: Account<'info, Asset>,
    
    /// 借入者账户（owner约束）
    #[account(
        constraint = borrower.key() == margin_token.owner @ AssetError::InvalidOwner
    )]
    pub borrower: Signer<'info>,
    
    /// 保证金池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub margin_pool: UncheckedAccount<'info>,
    
    /// 抵押品池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub collateral_pool: UncheckedAccount<'info>,
    
    /// 借入者代币账户
    #[account(
        mut,
        constraint = borrower_token_account.owner == borrower.key() @ AssetError::InvalidTokenAccount
    )]
    pub borrower_token_account: Account<'info, TokenAccount>,
    
    /// 抵押品代币账户
    #[account(
        mut,
        constraint = collateral_token_account.owner == borrower.key() @ AssetError::InvalidTokenAccount
    )]
    pub collateral_token_account: Account<'info, TokenAccount>,
    
    /// 预言机账户
    /// CHECK: 由程序验证
    pub oracle: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证借入参数
/// 
/// 检查借入参数的有效性和边界条件：
/// - 借入数量验证
/// - 抵押品资产验证
/// - 抵押品数量验证
/// - 利率验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 借入参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_borrow_margin_params(params: &BorrowMarginParams) -> Result<()> {
    // 验证借入数量
    require!(
        params.borrow_amount > 0,
        AssetError::InvalidBorrowAmount
    );
    
    // 验证抵押品资产
    require!(
        params.collateral_asset != Pubkey::default(),
        AssetError::InvalidCollateralAsset
    );
    
    // 验证抵押品数量
    require!(
        params.collateral_amount > 0,
        AssetError::InvalidCollateralAmount
    );
    
    // 验证利率
    require!(
        params.interest_rate >= 0.0 && params.interest_rate <= MAX_INTEREST_RATE,
        AssetError::InvalidInterestRate
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查借入权限
/// 
/// 验证借入权限和授权状态：
/// - 检查所有权
/// - 验证Margin Token状态
/// - 检查借入权限
/// 
/// # 参数
/// - borrower: 借入者账户
/// - margin_token: Margin Token账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_borrow_margin_authority_permission(
    borrower: &Signer,
    margin_token: &Account<Asset>,
) -> Result<()> {
    // 检查所有权
    require!(
        borrower.key() == margin_token.owner,
        AssetError::InvalidOwner
    );
    
    // 验证Margin Token状态
    require!(
        margin_token.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// 借入指令
/// 
/// 执行借入操作，包括：
/// - 参数验证：验证借入参数的有效性
/// - 权限检查：验证借入权限
/// - 服务层调用：委托给MarginTokenService执行借入逻辑
/// - 事件发射：发射Margin Token借入事件
/// 
/// # 参数
/// - ctx: 借入账户上下文
/// - params: 借入参数
/// 
/// # 返回
/// - Result<()>: 借入操作结果
pub fn borrow_margin(
    ctx: Context<BorrowMargin>,
    params: BorrowMarginParams,
) -> Result<()> {
    // 参数验证
    validate_borrow_margin_params(&params)?;
    
    // 权限检查
    check_borrow_margin_authority_permission(
        &ctx.accounts.borrower,
        &ctx.accounts.margin_token,
    )?;
    
    let margin_token = &mut ctx.accounts.margin_token;
    let borrower = &ctx.accounts.borrower;
    
    // 创建Margin Token服务实例
    let service = MarginTokenService::new();
    
    // 执行借入
    service.borrow_margin(
        margin_token,
        params.borrow_amount,
        params.collateral_asset,
        params.collateral_amount,
        params.interest_rate,
        &params.exec_params,
    )?;
    
    // 发射借入事件
    emit!(AssetMarginBorrowed {
        basket_id: margin_token.id,
        borrow_amount: params.borrow_amount,
        collateral_asset: params.collateral_asset,
        collateral_amount: params.collateral_amount,
        interest_rate: params.interest_rate,
        borrower: borrower.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::MarginToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 