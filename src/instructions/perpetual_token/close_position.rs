//! Perpetual Token平仓指令模块
//! 
//! 本模块提供Perpetual Token资产的平仓功能，包括：
//! - 参数验证：验证平仓参数的有效性和边界条件
//! - 权限检查：验证平仓权限和授权状态
//! - 服务层调用：委托给PerpetualTokenService执行核心业务逻辑
//! - 事件发射：发射Perpetual Token平仓事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于Perpetual Token平仓功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给PerpetualTokenService
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

/// 平仓参数结构体
/// 
/// 包含平仓所需的所有参数：
/// - position_id: 仓位ID
/// - close_amount: 平仓数量
/// - close_price: 平仓价格
/// - close_type: 平仓类型
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct ClosePositionParams {
    /// 仓位ID
    pub position_id: Pubkey,
    /// 平仓数量
    pub close_amount: u64,
    /// 平仓价格
    pub close_price: f64,
    /// 平仓类型
    pub close_type: CloseType,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 平仓类型枚举
/// 
/// 定义平仓的类型：
/// - Full: 全部平仓
/// - Partial: 部分平仓
/// - StopLoss: 止损平仓
/// - TakeProfit: 止盈平仓
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CloseType {
    /// 全部平仓
    Full,
    /// 部分平仓
    Partial,
    /// 止损平仓
    StopLoss,
    /// 止盈平仓
    TakeProfit,
}

/// 平仓账户上下文
/// 
/// 定义平仓指令所需的账户结构：
/// - perpetual_token: Perpetual Token账户（可变，Perpetual Token类型约束）
/// - trader: 交易者账户（owner约束）
/// - perpetual_pool: 永续池账户
/// - underlying_asset_pool: 底层资产池账户
/// - trader_token_account: 交易者代币账户
/// - oracle: 预言机账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct ClosePosition<'info> {
    /// Perpetual Token账户（可变，Perpetual Token类型约束）
    #[account(
        mut,
        constraint = perpetual_token.asset_type == AssetType::PerpetualToken @ AssetError::InvalidAssetType
    )]
    pub perpetual_token: Account<'info, Asset>,
    
    /// 交易者账户（owner约束）
    #[account(
        constraint = trader.key() == perpetual_token.owner @ AssetError::InvalidOwner
    )]
    pub trader: Signer<'info>,
    
    /// 永续池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub perpetual_pool: UncheckedAccount<'info>,
    
    /// 底层资产池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub underlying_asset_pool: UncheckedAccount<'info>,
    
    /// 交易者代币账户
    #[account(
        mut,
        constraint = trader_token_account.owner == trader.key() @ AssetError::InvalidTokenAccount
    )]
    pub trader_token_account: Account<'info, TokenAccount>,
    
    /// 预言机账户
    /// CHECK: 由程序验证
    pub oracle: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证平仓参数
/// 
/// 检查平仓参数的有效性和边界条件：
/// - 仓位ID验证
/// - 平仓数量验证
/// - 平仓价格验证
/// - 平仓类型验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 平仓参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_close_position_params(params: &ClosePositionParams) -> Result<()> {
    // 验证仓位ID
    require!(
        params.position_id != Pubkey::default(),
        AssetError::InvalidPositionId
    );
    
    // 验证平仓数量
    require!(
        params.close_amount > 0,
        AssetError::InvalidCloseAmount
    );
    
    // 验证平仓价格
    require!(
        params.close_price > 0.0,
        AssetError::InvalidClosePrice
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查平仓权限
/// 
/// 验证平仓权限和授权状态：
/// - 检查所有权
/// - 验证Perpetual Token状态
/// - 检查平仓权限
/// 
/// # 参数
/// - trader: 交易者账户
/// - perpetual_token: Perpetual Token账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_close_position_authority_permission(
    trader: &Signer,
    perpetual_token: &Account<Asset>,
) -> Result<()> {
    // 检查所有权
    require!(
        trader.key() == perpetual_token.owner,
        AssetError::InvalidOwner
    );
    
    // 验证Perpetual Token状态
    require!(
        perpetual_token.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// 平仓指令
/// 
/// 执行平仓操作，包括：
/// - 参数验证：验证平仓参数的有效性
/// - 权限检查：验证平仓权限
/// - 服务层调用：委托给PerpetualTokenService执行平仓逻辑
/// - 事件发射：发射Perpetual Token平仓事件
/// 
/// # 参数
/// - ctx: 平仓账户上下文
/// - params: 平仓参数
/// 
/// # 返回
/// - Result<()>: 平仓操作结果
pub fn close_position(
    ctx: Context<ClosePosition>,
    params: ClosePositionParams,
) -> Result<()> {
    // 参数验证
    validate_close_position_params(&params)?;
    
    // 权限检查
    check_close_position_authority_permission(
        &ctx.accounts.trader,
        &ctx.accounts.perpetual_token,
    )?;
    
    let perpetual_token = &mut ctx.accounts.perpetual_token;
    let trader = &ctx.accounts.trader;
    
    // 创建Perpetual Token服务实例
    let service = PerpetualTokenService::new();
    
    // 执行平仓
    service.close_position(
        perpetual_token,
        params.position_id,
        params.close_amount,
        params.close_price,
        &params.close_type,
        &params.exec_params,
    )?;
    
    // 发射平仓事件
    emit!(AssetPositionClosed {
        basket_id: perpetual_token.id,
        position_id: params.position_id,
        close_amount: params.close_amount,
        close_price: params.close_price,
        close_type: format!("{:?}", params.close_type),
        trader: trader.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::PerpetualToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 