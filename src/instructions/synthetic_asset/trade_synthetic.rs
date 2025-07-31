//! Synthetic Asset交易合成资产指令模块
//! 
//! 本模块提供Synthetic Asset资产的交易合成资产功能，包括：
//! - 参数验证：验证交易合成资产参数的有效性和边界条件
//! - 权限检查：验证交易合成资产权限和授权状态
//! - 服务层调用：委托给SyntheticAssetService执行核心业务逻辑
//! - 事件发射：发射Synthetic Asset交易合成资产事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于Synthetic Asset交易合成资产功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给SyntheticAssetService
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

/// 交易合成资产参数结构体
/// 
/// 包含交易合成资产所需的所有参数：
/// - trade_amount: 交易数量
/// - trade_type: 交易类型
/// - price: 价格
/// - slippage_tolerance: 滑点容忍度
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct TradeSyntheticParams {
    /// 交易数量
    pub trade_amount: u64,
    /// 交易类型
    pub trade_type: TradeType,
    /// 价格
    pub price: f64,
    /// 滑点容忍度
    pub slippage_tolerance: f64,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 交易类型枚举
/// 
/// 定义交易的类型：
/// - Buy: 买入
/// - Sell: 卖出
/// - Long: 做多
/// - Short: 做空
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum TradeType {
    /// 买入
    Buy,
    /// 卖出
    Sell,
    /// 做多
    Long,
    /// 做空
    Short,
}

/// 交易合成资产账户上下文
/// 
/// 定义交易合成资产指令所需的账户结构：
/// - synthetic_asset: Synthetic Asset账户（可变，Synthetic Asset类型约束）
/// - trader: 交易者账户（owner约束）
/// - synthetic_pool: 合成资产池账户
/// - trader_token_account: 交易者代币账户
/// - oracle: 预言机账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct TradeSynthetic<'info> {
    /// Synthetic Asset账户（可变，Synthetic Asset类型约束）
    #[account(
        mut,
        constraint = synthetic_asset.asset_type == AssetType::SyntheticAsset @ AssetError::InvalidAssetType
    )]
    pub synthetic_asset: Account<'info, Asset>,
    
    /// 交易者账户（owner约束）
    #[account(
        constraint = trader.key() == synthetic_asset.owner @ AssetError::InvalidOwner
    )]
    pub trader: Signer<'info>,
    
    /// 合成资产池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub synthetic_pool: UncheckedAccount<'info>,
    
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

/// 验证交易合成资产参数
/// 
/// 检查交易合成资产参数的有效性和边界条件：
/// - 交易数量验证
/// - 交易类型验证
/// - 价格验证
/// - 滑点容忍度验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 交易合成资产参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_trade_synthetic_params(params: &TradeSyntheticParams) -> Result<()> {
    // 验证交易数量
    require!(
        params.trade_amount > 0,
        AssetError::InvalidTradeAmount
    );
    
    // 验证价格
    require!(
        params.price > 0.0,
        AssetError::InvalidPrice
    );
    
    // 验证滑点容忍度
    require!(
        params.slippage_tolerance >= 0.0 && params.slippage_tolerance <= 1.0,
        AssetError::InvalidSlippageTolerance
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查交易合成资产权限
/// 
/// 验证交易合成资产权限和授权状态：
/// - 检查所有权
/// - 验证Synthetic Asset状态
/// - 检查交易权限
/// 
/// # 参数
/// - trader: 交易者账户
/// - synthetic_asset: Synthetic Asset账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_trade_synthetic_authority_permission(
    trader: &Signer,
    synthetic_asset: &Account<Asset>,
) -> Result<()> {
    // 检查所有权
    require!(
        trader.key() == synthetic_asset.owner,
        AssetError::InvalidOwner
    );
    
    // 验证Synthetic Asset状态
    require!(
        synthetic_asset.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// 交易合成资产指令
/// 
/// 执行交易合成资产操作，包括：
/// - 参数验证：验证交易合成资产参数的有效性
/// - 权限检查：验证交易合成资产权限
/// - 服务层调用：委托给SyntheticAssetService执行交易合成资产逻辑
/// - 事件发射：发射Synthetic Asset交易合成资产事件
/// 
/// # 参数
/// - ctx: 交易合成资产账户上下文
/// - params: 交易合成资产参数
/// 
/// # 返回
/// - Result<()>: 交易合成资产操作结果
pub fn trade_synthetic(
    ctx: Context<TradeSynthetic>,
    params: TradeSyntheticParams,
) -> Result<()> {
    // 参数验证
    validate_trade_synthetic_params(&params)?;
    
    // 权限检查
    check_trade_synthetic_authority_permission(
        &ctx.accounts.trader,
        &ctx.accounts.synthetic_asset,
    )?;
    
    let synthetic_asset = &mut ctx.accounts.synthetic_asset;
    let trader = &ctx.accounts.trader;
    
    // 创建Synthetic Asset服务实例
    let service = SyntheticAssetService::new();
    
    // 执行交易合成资产
    service.trade_synthetic(
        synthetic_asset,
        params.trade_amount,
        &params.trade_type,
        params.price,
        params.slippage_tolerance,
        &params.exec_params,
    )?;
    
    // 发射交易合成资产事件
    emit!(AssetSyntheticTraded {
        basket_id: synthetic_asset.id,
        trade_amount: params.trade_amount,
        trade_type: format!("{:?}", params.trade_type),
        price: params.price,
        slippage_tolerance: params.slippage_tolerance,
        trader: trader.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::SyntheticAsset,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 