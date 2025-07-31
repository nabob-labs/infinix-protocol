//! Perpetual Token开仓指令模块
//! 
//! 本模块提供Perpetual Token资产的开仓功能，包括：
//! - 参数验证：验证开仓参数的有效性和边界条件
//! - 权限检查：验证开仓权限和授权状态
//! - 服务层调用：委托给PerpetualTokenService执行核心业务逻辑
//! - 事件发射：发射Perpetual Token开仓事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于Perpetual Token开仓功能
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

/// 开仓参数结构体
/// 
/// 包含开仓所需的所有参数：
/// - underlying_asset: 底层资产
/// - position_size: 仓位大小
/// - leverage: 杠杆倍数
/// - position_type: 仓位类型
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct OpenPositionParams {
    /// 底层资产
    pub underlying_asset: Pubkey,
    /// 仓位大小
    pub position_size: u64,
    /// 杠杆倍数
    pub leverage: f64,
    /// 仓位类型
    pub position_type: PositionType,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 仓位类型枚举
/// 
/// 定义仓位的类型：
/// - Long: 多头
/// - Short: 空头
/// - Cross: 全仓
/// - Isolated: 逐仓
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum PositionType {
    /// 多头
    Long,
    /// 空头
    Short,
    /// 全仓
    Cross,
    /// 逐仓
    Isolated,
}

/// 开仓账户上下文
/// 
/// 定义开仓指令所需的账户结构：
/// - perpetual_token: Perpetual Token账户（可变，Perpetual Token类型约束）
/// - trader: 交易者账户（owner约束）
/// - perpetual_pool: 永续池账户
/// - underlying_asset_pool: 底层资产池账户
/// - trader_token_account: 交易者代币账户
/// - oracle: 预言机账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct OpenPosition<'info> {
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

/// 验证开仓参数
/// 
/// 检查开仓参数的有效性和边界条件：
/// - 底层资产验证
/// - 仓位大小验证
/// - 杠杆倍数验证
/// - 仓位类型验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 开仓参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_open_position_params(params: &OpenPositionParams) -> Result<()> {
    // 验证底层资产
    require!(
        params.underlying_asset != Pubkey::default(),
        AssetError::InvalidUnderlyingAsset
    );
    
    // 验证仓位大小
    require!(
        params.position_size > 0,
        AssetError::InvalidPositionSize
    );
    
    // 验证杠杆倍数
    require!(
        params.leverage > 0.0 && params.leverage <= MAX_LEVERAGE,
        AssetError::InvalidLeverage
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查开仓权限
/// 
/// 验证开仓权限和授权状态：
/// - 检查所有权
/// - 验证Perpetual Token状态
/// - 检查开仓权限
/// 
/// # 参数
/// - trader: 交易者账户
/// - perpetual_token: Perpetual Token账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_open_position_authority_permission(
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

/// 开仓指令
/// 
/// 执行开仓操作，包括：
/// - 参数验证：验证开仓参数的有效性
/// - 权限检查：验证开仓权限
/// - 服务层调用：委托给PerpetualTokenService执行开仓逻辑
/// - 事件发射：发射Perpetual Token开仓事件
/// 
/// # 参数
/// - ctx: 开仓账户上下文
/// - params: 开仓参数
/// 
/// # 返回
/// - Result<()>: 开仓操作结果
pub fn open_position(
    ctx: Context<OpenPosition>,
    params: OpenPositionParams,
) -> Result<()> {
    // 参数验证
    validate_open_position_params(&params)?;
    
    // 权限检查
    check_open_position_authority_permission(
        &ctx.accounts.trader,
        &ctx.accounts.perpetual_token,
    )?;
    
    let perpetual_token = &mut ctx.accounts.perpetual_token;
    let trader = &ctx.accounts.trader;
    
    // 创建Perpetual Token服务实例
    let service = PerpetualTokenService::new();
    
    // 执行开仓
    service.open_position(
        perpetual_token,
        params.underlying_asset,
        params.position_size,
        params.leverage,
        &params.position_type,
        &params.exec_params,
    )?;
    
    // 发射开仓事件
    emit!(AssetPositionOpened {
        basket_id: perpetual_token.id,
        underlying_asset: params.underlying_asset,
        position_size: params.position_size,
        leverage: params.leverage,
        position_type: format!("{:?}", params.position_type),
        trader: trader.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::PerpetualToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 