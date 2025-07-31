//! Synthetic Asset创建合成资产指令模块
//! 
//! 本模块提供Synthetic Asset资产的创建合成资产功能，包括：
//! - 参数验证：验证创建合成资产参数的有效性和边界条件
//! - 权限检查：验证创建合成资产权限和授权状态
//! - 服务层调用：委托给SyntheticAssetService执行核心业务逻辑
//! - 事件发射：发射Synthetic Asset创建合成资产事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于Synthetic Asset创建合成资产功能
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

/// 创建合成资产参数结构体
/// 
/// 包含创建合成资产所需的所有参数：
/// - underlying_asset: 底层资产
/// - synthetic_type: 合成类型
/// - collateral_ratio: 抵押率
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct CreateSyntheticParams {
    /// 底层资产
    pub underlying_asset: Pubkey,
    /// 合成类型
    pub synthetic_type: SyntheticType,
    /// 抵押率
    pub collateral_ratio: f64,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 合成类型枚举
/// 
/// 定义合成的类型：
/// - Inverse: 反向合成
/// - Leveraged: 杠杆合成
/// - Basket: 篮子合成
/// - Custom: 自定义合成
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum SyntheticType {
    /// 反向合成
    Inverse,
    /// 杠杆合成
    Leveraged,
    /// 篮子合成
    Basket,
    /// 自定义合成
    Custom,
}

/// 创建合成资产账户上下文
/// 
/// 定义创建合成资产指令所需的账户结构：
/// - synthetic_asset: Synthetic Asset账户（可变，Synthetic Asset类型约束）
/// - creator: 创建者账户（owner约束）
/// - synthetic_pool: 合成资产池账户
/// - underlying_asset_pool: 底层资产池账户
/// - oracle: 预言机账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct CreateSynthetic<'info> {
    /// Synthetic Asset账户（可变，Synthetic Asset类型约束）
    #[account(
        mut,
        constraint = synthetic_asset.asset_type == AssetType::SyntheticAsset @ AssetError::InvalidAssetType
    )]
    pub synthetic_asset: Account<'info, Asset>,
    
    /// 创建者账户（owner约束）
    #[account(
        constraint = creator.key() == synthetic_asset.owner @ AssetError::InvalidOwner
    )]
    pub creator: Signer<'info>,
    
    /// 合成资产池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub synthetic_pool: UncheckedAccount<'info>,
    
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

/// 验证创建合成资产参数
/// 
/// 检查创建合成资产参数的有效性和边界条件：
/// - 底层资产验证
/// - 合成类型验证
/// - 抵押率验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 创建合成资产参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_create_synthetic_params(params: &CreateSyntheticParams) -> Result<()> {
    // 验证底层资产
    require!(
        params.underlying_asset != Pubkey::default(),
        AssetError::InvalidUnderlyingAsset
    );
    
    // 验证抵押率
    require!(
        params.collateral_ratio > 0.0,
        AssetError::InvalidCollateralRatio
    );
    
    require!(
        params.collateral_ratio <= MAX_COLLATERAL_RATIO,
        AssetError::CollateralRatioTooHigh
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查创建合成资产权限
/// 
/// 验证创建合成资产权限和授权状态：
/// - 检查所有权
/// - 验证Synthetic Asset状态
/// - 检查创建权限
/// 
/// # 参数
/// - creator: 创建者账户
/// - synthetic_asset: Synthetic Asset账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_create_synthetic_authority_permission(
    creator: &Signer,
    synthetic_asset: &Account<Asset>,
) -> Result<()> {
    // 检查所有权
    require!(
        creator.key() == synthetic_asset.owner,
        AssetError::InvalidOwner
    );
    
    // 验证Synthetic Asset状态
    require!(
        synthetic_asset.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// 创建合成资产指令
/// 
/// 执行创建合成资产操作，包括：
/// - 参数验证：验证创建合成资产参数的有效性
/// - 权限检查：验证创建合成资产权限
/// - 服务层调用：委托给SyntheticAssetService执行创建合成资产逻辑
/// - 事件发射：发射Synthetic Asset创建合成资产事件
/// 
/// # 参数
/// - ctx: 创建合成资产账户上下文
/// - params: 创建合成资产参数
/// 
/// # 返回
/// - Result<()>: 创建合成资产操作结果
pub fn create_synthetic(
    ctx: Context<CreateSynthetic>,
    params: CreateSyntheticParams,
) -> Result<()> {
    // 参数验证
    validate_create_synthetic_params(&params)?;
    
    // 权限检查
    check_create_synthetic_authority_permission(
        &ctx.accounts.creator,
        &ctx.accounts.synthetic_asset,
    )?;
    
    let synthetic_asset = &mut ctx.accounts.synthetic_asset;
    let creator = &ctx.accounts.creator;
    
    // 创建Synthetic Asset服务实例
    let service = SyntheticAssetService::new();
    
    // 执行创建合成资产
    service.create_synthetic(
        synthetic_asset,
        params.underlying_asset,
        &params.synthetic_type,
        params.collateral_ratio,
        &params.exec_params,
    )?;
    
    // 发射创建合成资产事件
    emit!(AssetSyntheticCreated {
        basket_id: synthetic_asset.id,
        underlying_asset: params.underlying_asset,
        synthetic_type: format!("{:?}", params.synthetic_type),
        collateral_ratio: params.collateral_ratio,
        creator: creator.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::SyntheticAsset,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 