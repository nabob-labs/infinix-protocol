//! Futures Token创建期货指令模块
//! 
//! 本模块提供Futures Token资产的创建期货功能，包括：
//! - 参数验证：验证创建期货参数的有效性和边界条件
//! - 权限检查：验证创建期货权限和授权状态
//! - 服务层调用：委托给FuturesTokenService执行核心业务逻辑
//! - 事件发射：发射Futures Token创建期货事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于Futures Token创建期货功能
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

/// 创建期货参数结构体
/// 
/// 包含创建期货所需的所有参数：
/// - underlying_asset: 底层资产
/// - contract_size: 合约规模
/// - delivery_date: 交割日期
/// - futures_type: 期货类型
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct CreateFuturesParams {
    /// 底层资产
    pub underlying_asset: Pubkey,
    /// 合约规模
    pub contract_size: u64,
    /// 交割日期
    pub delivery_date: i64,
    /// 期货类型
    pub futures_type: FuturesType,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 期货类型枚举
/// 
/// 定义期货的类型：
/// - Commodity: 商品期货
/// - Financial: 金融期货
/// - Currency: 货币期货
/// - Index: 指数期货
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum FuturesType {
    /// 商品期货
    Commodity,
    /// 金融期货
    Financial,
    /// 货币期货
    Currency,
    /// 指数期货
    Index,
}

/// 创建期货账户上下文
/// 
/// 定义创建期货指令所需的账户结构：
/// - futures_token: Futures Token账户（可变，Futures Token类型约束）
/// - creator: 创建者账户（owner约束）
/// - futures_pool: 期货池账户
/// - underlying_asset_pool: 底层资产池账户
/// - oracle: 预言机账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct CreateFutures<'info> {
    /// Futures Token账户（可变，Futures Token类型约束）
    #[account(
        mut,
        constraint = futures_token.asset_type == AssetType::FuturesToken @ AssetError::InvalidAssetType
    )]
    pub futures_token: Account<'info, Asset>,
    
    /// 创建者账户（owner约束）
    #[account(
        constraint = creator.key() == futures_token.owner @ AssetError::InvalidOwner
    )]
    pub creator: Signer<'info>,
    
    /// 期货池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub futures_pool: UncheckedAccount<'info>,
    
    /// 底层资产池账户
    /// CHECK: 由程序验证
    pub underlying_asset_pool: UncheckedAccount<'info>,
    
    /// 预言机账户
    /// CHECK: 由程序验证
    pub oracle: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证创建期货参数
/// 
/// 检查创建期货参数的有效性和边界条件：
/// - 底层资产验证
/// - 合约规模验证
/// - 交割日期验证
/// - 期货类型验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 创建期货参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_create_futures_params(params: &CreateFuturesParams) -> Result<()> {
    // 验证底层资产
    require!(
        params.underlying_asset != Pubkey::default(),
        AssetError::InvalidUnderlyingAsset
    );
    
    // 验证合约规模
    require!(
        params.contract_size > 0,
        AssetError::InvalidContractSize
    );
    
    // 验证交割日期
    let current_time = Clock::get()?.unix_timestamp;
    require!(
        params.delivery_date > current_time,
        AssetError::InvalidDeliveryDate
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查创建期货权限
/// 
/// 验证创建期货权限和授权状态：
/// - 检查所有权
/// - 验证Futures Token状态
/// - 检查创建权限
/// 
/// # 参数
/// - creator: 创建者账户
/// - futures_token: Futures Token账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_create_futures_authority_permission(
    creator: &Signer,
    futures_token: &Account<Asset>,
) -> Result<()> {
    // 检查所有权
    require!(
        creator.key() == futures_token.owner,
        AssetError::InvalidOwner
    );
    
    // 验证Futures Token状态
    require!(
        futures_token.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// 创建期货指令
/// 
/// 执行创建期货操作，包括：
/// - 参数验证：验证创建期货参数的有效性
/// - 权限检查：验证创建期货权限
/// - 服务层调用：委托给FuturesTokenService执行创建期货逻辑
/// - 事件发射：发射Futures Token创建期货事件
/// 
/// # 参数
/// - ctx: 创建期货账户上下文
/// - params: 创建期货参数
/// 
/// # 返回
/// - Result<()>: 创建期货操作结果
pub fn create_futures(
    ctx: Context<CreateFutures>,
    params: CreateFuturesParams,
) -> Result<()> {
    // 参数验证
    validate_create_futures_params(&params)?;
    
    // 权限检查
    check_create_futures_authority_permission(
        &ctx.accounts.creator,
        &ctx.accounts.futures_token,
    )?;
    
    let futures_token = &mut ctx.accounts.futures_token;
    let creator = &ctx.accounts.creator;
    
    // 创建Futures Token服务实例
    let service = FuturesTokenService::new();
    
    // 执行创建期货
    service.create_futures(
        futures_token,
        params.underlying_asset,
        params.contract_size,
        params.delivery_date,
        &params.futures_type,
        &params.exec_params,
    )?;
    
    // 发射创建期货事件
    emit!(AssetFuturesCreated {
        basket_id: futures_token.id,
        underlying_asset: params.underlying_asset,
        contract_size: params.contract_size,
        delivery_date: params.delivery_date,
        futures_type: format!("{:?}", params.futures_type),
        creator: creator.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::FuturesToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 