//! Options Token创建期权指令模块
//! 
//! 本模块提供Options Token资产的创建期权功能，包括：
//! - 参数验证：验证创建期权参数的有效性和边界条件
//! - 权限检查：验证创建期权权限和授权状态
//! - 服务层调用：委托给OptionsTokenService执行核心业务逻辑
//! - 事件发射：发射Options Token创建期权事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于Options Token创建期权功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给OptionsTokenService
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

/// 创建期权参数结构体
/// 
/// 包含创建期权所需的所有参数：
/// - underlying_asset: 底层资产
/// - strike_price: 行权价格
/// - expiration_date: 到期日期
/// - option_type: 期权类型
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct CreateOptionParams {
    /// 底层资产
    pub underlying_asset: Pubkey,
    /// 行权价格
    pub strike_price: f64,
    /// 到期日期
    pub expiration_date: i64,
    /// 期权类型
    pub option_type: OptionType,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 期权类型枚举
/// 
/// 定义期权的类型：
/// - Call: 看涨期权
/// - Put: 看跌期权
/// - American: 美式期权
/// - European: 欧式期权
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum OptionType {
    /// 看涨期权
    Call,
    /// 看跌期权
    Put,
    /// 美式期权
    American,
    /// 欧式期权
    European,
}

/// 创建期权账户上下文
/// 
/// 定义创建期权指令所需的账户结构：
/// - options_token: Options Token账户（可变，Options Token类型约束）
/// - creator: 创建者账户（owner约束）
/// - options_pool: 期权池账户
/// - underlying_asset_pool: 底层资产池账户
/// - oracle: 预言机账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct CreateOption<'info> {
    /// Options Token账户（可变，Options Token类型约束）
    #[account(
        mut,
        constraint = options_token.asset_type == AssetType::OptionsToken @ AssetError::InvalidAssetType
    )]
    pub options_token: Account<'info, Asset>,
    
    /// 创建者账户（owner约束）
    #[account(
        constraint = creator.key() == options_token.owner @ AssetError::InvalidOwner
    )]
    pub creator: Signer<'info>,
    
    /// 期权池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub options_pool: UncheckedAccount<'info>,
    
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

/// 验证创建期权参数
/// 
/// 检查创建期权参数的有效性和边界条件：
/// - 底层资产验证
/// - 行权价格验证
/// - 到期日期验证
/// - 期权类型验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 创建期权参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_create_option_params(params: &CreateOptionParams) -> Result<()> {
    // 验证底层资产
    require!(
        params.underlying_asset != Pubkey::default(),
        AssetError::InvalidUnderlyingAsset
    );
    
    // 验证行权价格
    require!(
        params.strike_price > 0.0,
        AssetError::InvalidStrikePrice
    );
    
    // 验证到期日期
    let current_time = Clock::get()?.unix_timestamp;
    require!(
        params.expiration_date > current_time,
        AssetError::InvalidExpirationDate
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查创建期权权限
/// 
/// 验证创建期权权限和授权状态：
/// - 检查所有权
/// - 验证Options Token状态
/// - 检查创建权限
/// 
/// # 参数
/// - creator: 创建者账户
/// - options_token: Options Token账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_create_option_authority_permission(
    creator: &Signer,
    options_token: &Account<Asset>,
) -> Result<()> {
    // 检查所有权
    require!(
        creator.key() == options_token.owner,
        AssetError::InvalidOwner
    );
    
    // 验证Options Token状态
    require!(
        options_token.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// 创建期权指令
/// 
/// 执行创建期权操作，包括：
/// - 参数验证：验证创建期权参数的有效性
/// - 权限检查：验证创建期权权限
/// - 服务层调用：委托给OptionsTokenService执行创建期权逻辑
/// - 事件发射：发射Options Token创建期权事件
/// 
/// # 参数
/// - ctx: 创建期权账户上下文
/// - params: 创建期权参数
/// 
/// # 返回
/// - Result<()>: 创建期权操作结果
pub fn create_option(
    ctx: Context<CreateOption>,
    params: CreateOptionParams,
) -> Result<()> {
    // 参数验证
    validate_create_option_params(&params)?;
    
    // 权限检查
    check_create_option_authority_permission(
        &ctx.accounts.creator,
        &ctx.accounts.options_token,
    )?;
    
    let options_token = &mut ctx.accounts.options_token;
    let creator = &ctx.accounts.creator;
    
    // 创建Options Token服务实例
    let service = OptionsTokenService::new();
    
    // 执行创建期权
    service.create_option(
        options_token,
        params.underlying_asset,
        params.strike_price,
        params.expiration_date,
        &params.option_type,
        &params.exec_params,
    )?;
    
    // 发射创建期权事件
    emit!(AssetOptionCreated {
        basket_id: options_token.id,
        underlying_asset: params.underlying_asset,
        strike_price: params.strike_price,
        expiration_date: params.expiration_date,
        option_type: format!("{:?}", params.option_type),
        creator: creator.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::OptionsToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 