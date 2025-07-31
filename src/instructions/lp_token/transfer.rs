//! LP Token转账指令模块
//! 
//! 本模块提供LP Token资产的转账功能，包括：
//! - 参数验证：验证转账参数的有效性和边界条件
//! - 权限检查：验证转账权限和授权状态
//! - 服务层调用：委托给LpTokenService执行核心业务逻辑
//! - 事件发射：发射LP Token转账事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于LP Token转账功能
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

/// LP Token转账参数结构体
/// 
/// 包含LP Token转账所需的所有参数：
/// - amount: 转账数量
/// - recipient: 接收者地址
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct TransferLpTokenParams {
    /// 转账数量
    pub amount: u64,
    /// 接收者地址
    pub recipient: Pubkey,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// LP Token转账账户上下文
/// 
/// 定义LP Token转账指令所需的账户结构：
/// - lp_token: LP Token账户（可变，LP Token类型约束）
/// - sender: 发送者账户（owner约束）
/// - recipient: 接收者账户
/// - sender_token_account: 发送者代币账户
/// - recipient_token_account: 接收者代币账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct TransferLpToken<'info> {
    /// LP Token账户（可变，LP Token类型约束）
    #[account(
        mut,
        constraint = lp_token.asset_type == AssetType::LpToken @ AssetError::InvalidAssetType
    )]
    pub lp_token: Account<'info, Asset>,
    
    /// 发送者账户（owner约束）
    #[account(
        constraint = sender.key() == lp_token.owner @ AssetError::InvalidOwner
    )]
    pub sender: Signer<'info>,
    
    /// 接收者账户
    /// CHECK: 由程序验证
    pub recipient: UncheckedAccount<'info>,
    
    /// 发送者代币账户
    #[account(
        mut,
        constraint = sender_token_account.owner == sender.key() @ AssetError::InvalidTokenAccount
    )]
    pub sender_token_account: Account<'info, TokenAccount>,
    
    /// 接收者代币账户
    #[account(
        mut,
        constraint = recipient_token_account.owner == recipient.key() @ AssetError::InvalidTokenAccount
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证LP Token转账参数
/// 
/// 检查LP Token转账参数的有效性和边界条件：
/// - 转账数量验证
/// - 接收者地址验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: LP Token转账参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_transfer_lp_token_params(params: &TransferLpTokenParams) -> Result<()> {
    // 验证转账数量
    require!(
        params.amount > 0,
        AssetError::InvalidAmount
    );
    
    // 验证接收者地址
    require!(
        params.recipient != Pubkey::default(),
        AssetError::InvalidRecipient
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查LP Token转账权限
/// 
/// 验证LP Token转账权限和授权状态：
/// - 检查所有权
/// - 验证LP Token状态
/// - 检查转账权限
/// 
/// # 参数
/// - sender: 发送者账户
/// - lp_token: LP Token账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_transfer_lp_token_authority_permission(
    sender: &Signer,
    lp_token: &Account<Asset>,
) -> Result<()> {
    // 检查所有权
    require!(
        sender.key() == lp_token.owner,
        AssetError::InvalidOwner
    );
    
    // 验证LP Token状态
    require!(
        lp_token.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// LP Token转账指令
/// 
/// 执行LP Token转账操作，包括：
/// - 参数验证：验证转账参数的有效性
/// - 权限检查：验证转账权限
/// - 服务层调用：委托给LpTokenService执行转账逻辑
/// - 事件发射：发射LP Token转账事件
/// 
/// # 参数
/// - ctx: LP Token转账账户上下文
/// - params: LP Token转账参数
/// 
/// # 返回
/// - Result<()>: 转账操作结果
pub fn transfer_lp_token(
    ctx: Context<TransferLpToken>,
    params: TransferLpTokenParams,
) -> Result<()> {
    // 参数验证
    validate_transfer_lp_token_params(&params)?;
    
    // 权限检查
    check_transfer_lp_token_authority_permission(
        &ctx.accounts.sender,
        &ctx.accounts.lp_token,
    )?;
    
    let lp_token = &mut ctx.accounts.lp_token;
    let sender = &ctx.accounts.sender;
    
    // 创建LP Token服务实例
    let service = LpTokenService::new();
    
    // 执行LP Token转账
    service.transfer(
        lp_token,
        params.amount,
        params.recipient,
        &params.exec_params,
    )?;
    
    // 发射LP Token转账事件
    emit!(AssetTransferred {
        basket_id: lp_token.id,
        amount: params.amount,
        sender: sender.key(),
        recipient: params.recipient,
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::LpToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
}
