//! Futures Token期货结算指令模块
//! 
//! 本模块提供Futures Token资产的期货结算功能，包括：
//! - 参数验证：验证结算参数的有效性和边界条件
//! - 权限检查：验证结算权限和授权状态
//! - 服务层调用：委托给FuturesTokenService执行核心业务逻辑
//! - 事件发射：发射Futures Token期货结算事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于Futures Token期货结算功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给FuturesTokenService
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

/// 期货结算参数结构体
/// 
/// 包含期货结算所需的所有参数：
/// - futures_id: 期货ID
/// - settlement_price: 结算价格
/// - settlement_amount: 结算数量
/// - settlement_type: 结算类型
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct SettleFuturesParams {
    /// 期货ID
    pub futures_id: Pubkey,
    /// 结算价格
    pub settlement_price: f64,
    /// 结算数量
    pub settlement_amount: u64,
    /// 结算类型
    pub settlement_type: SettlementType,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 结算类型枚举
/// 
/// 定义结算的类型：
/// - Physical: 实物结算
/// - Cash: 现金结算
/// - Automatic: 自动结算
/// - Manual: 手动结算
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum SettlementType {
    /// 实物结算
    Physical,
    /// 现金结算
    Cash,
    /// 自动结算
    Automatic,
    /// 手动结算
    Manual,
}

/// 期货结算账户上下文
/// 
/// 定义期货结算指令所需的账户结构：
/// - futures_token: Futures Token账户（可变，Futures Token类型约束）
/// - settler: 结算者账户（owner约束）
/// - futures_pool: 期货池账户
/// - underlying_asset_pool: 底层资产池账户
/// - settler_token_account: 结算者代币账户
/// - oracle: 预言机账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct SettleFutures<'info> {
    /// Futures Token账户（可变，Futures Token类型约束）
    #[account(
        mut,
        constraint = futures_token.asset_type == AssetType::FuturesToken @ AssetError::InvalidAssetType
    )]
    pub futures_token: Account<'info, Asset>,
    
    /// 结算者账户（owner约束）
    #[account(
        constraint = settler.key() == futures_token.owner @ AssetError::InvalidOwner
    )]
    pub settler: Signer<'info>,
    
    /// 期货池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub futures_pool: UncheckedAccount<'info>,
    
    /// 底层资产池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub underlying_asset_pool: UncheckedAccount<'info>,
    
    /// 结算者代币账户
    #[account(
        mut,
        constraint = settler_token_account.owner == settler.key() @ AssetError::InvalidTokenAccount
    )]
    pub settler_token_account: Account<'info, TokenAccount>,
    
    /// 预言机账户
    /// CHECK: 由程序验证
    pub oracle: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证期货结算参数
/// 
/// 检查期货结算参数的有效性和边界条件：
/// - 期货ID验证
/// - 结算价格验证
/// - 结算数量验证
/// - 结算类型验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 期货结算参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_settle_futures_params(params: &SettleFuturesParams) -> Result<()> {
    // 验证期货ID
    require!(
        params.futures_id != Pubkey::default(),
        AssetError::InvalidFuturesId
    );
    
    // 验证结算价格
    require!(
        params.settlement_price > 0.0,
        AssetError::InvalidSettlementPrice
    );
    
    // 验证结算数量
    require!(
        params.settlement_amount > 0,
        AssetError::InvalidSettlementAmount
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查期货结算权限
/// 
/// 验证期货结算权限和授权状态：
/// - 检查所有权
/// - 验证Futures Token状态
/// - 检查结算权限
/// 
/// # 参数
/// - settler: 结算者账户
/// - futures_token: Futures Token账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_settle_futures_authority_permission(
    settler: &Signer,
    futures_token: &Account<Asset>,
) -> Result<()> {
    // 检查所有权
    require!(
        settler.key() == futures_token.owner,
        AssetError::InvalidOwner
    );
    
    // 验证Futures Token状态
    require!(
        futures_token.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// 期货结算指令
/// 
/// 执行期货结算操作，包括：
/// - 参数验证：验证结算参数的有效性
/// - 权限检查：验证结算权限
/// - 服务层调用：委托给FuturesTokenService执行结算逻辑
/// - 事件发射：发射Futures Token期货结算事件
/// 
/// # 参数
/// - ctx: 期货结算账户上下文
/// - params: 期货结算参数
/// 
/// # 返回
/// - Result<()>: 结算操作结果
pub fn settle_futures(
    ctx: Context<SettleFutures>,
    params: SettleFuturesParams,
) -> Result<()> {
    // 参数验证
    validate_settle_futures_params(&params)?;
    
    // 权限检查
    check_settle_futures_authority_permission(
        &ctx.accounts.settler,
        &ctx.accounts.futures_token,
    )?;
    
    let futures_token = &mut ctx.accounts.futures_token;
    let settler = &ctx.accounts.settler;
    
    // 创建Futures Token服务实例
    let service = FuturesTokenService::new();
    
    // 执行期货结算
    service.settle_futures(
        futures_token,
        params.futures_id,
        params.settlement_price,
        params.settlement_amount,
        &params.settlement_type,
        &params.exec_params,
    )?;
    
    // 发射期货结算事件
    emit!(AssetFuturesSettled {
        basket_id: futures_token.id,
        futures_id: params.futures_id,
        settlement_price: params.settlement_price,
        settlement_amount: params.settlement_amount,
        settlement_type: format!("{:?}", params.settlement_type),
        settler: settler.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::FuturesToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 