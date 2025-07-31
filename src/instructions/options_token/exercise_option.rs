//! Options Token期权行权指令模块
//! 
//! 本模块提供Options Token资产的期权行权功能，包括：
//! - 参数验证：验证行权参数的有效性和边界条件
//! - 权限检查：验证行权权限和授权状态
//! - 服务层调用：委托给OptionsTokenService执行核心业务逻辑
//! - 事件发射：发射Options Token期权行权事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于Options Token期权行权功能
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

/// 期权行权参数结构体
/// 
/// 包含期权行权所需的所有参数：
/// - option_id: 期权ID
/// - exercise_amount: 行权数量
/// - exercise_price: 行权价格
/// - exercise_type: 行权类型
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct ExerciseOptionParams {
    /// 期权ID
    pub option_id: Pubkey,
    /// 行权数量
    pub exercise_amount: u64,
    /// 行权价格
    pub exercise_price: f64,
    /// 行权类型
    pub exercise_type: ExerciseType,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// 行权类型枚举
/// 
/// 定义行权的类型：
/// - Physical: 实物行权
/// - Cash: 现金行权
/// - Automatic: 自动行权
/// - Manual: 手动行权
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ExerciseType {
    /// 实物行权
    Physical,
    /// 现金行权
    Cash,
    /// 自动行权
    Automatic,
    /// 手动行权
    Manual,
}

/// 期权行权账户上下文
/// 
/// 定义期权行权指令所需的账户结构：
/// - options_token: Options Token账户（可变，Options Token类型约束）
/// - exerciser: 行权者账户（owner约束）
/// - options_pool: 期权池账户
/// - underlying_asset_pool: 底层资产池账户
/// - exerciser_token_account: 行权者代币账户
/// - oracle: 预言机账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct ExerciseOption<'info> {
    /// Options Token账户（可变，Options Token类型约束）
    #[account(
        mut,
        constraint = options_token.asset_type == AssetType::OptionsToken @ AssetError::InvalidAssetType
    )]
    pub options_token: Account<'info, Asset>,
    
    /// 行权者账户（owner约束）
    #[account(
        constraint = exerciser.key() == options_token.owner @ AssetError::InvalidOwner
    )]
    pub exerciser: Signer<'info>,
    
    /// 期权池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub options_pool: UncheckedAccount<'info>,
    
    /// 底层资产池账户
    /// CHECK: 由程序验证
    #[account(mut)]
    pub underlying_asset_pool: UncheckedAccount<'info>,
    
    /// 行权者代币账户
    #[account(
        mut,
        constraint = exerciser_token_account.owner == exerciser.key() @ AssetError::InvalidTokenAccount
    )]
    pub exerciser_token_account: Account<'info, TokenAccount>,
    
    /// 预言机账户
    /// CHECK: 由程序验证
    pub oracle: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// 验证期权行权参数
/// 
/// 检查期权行权参数的有效性和边界条件：
/// - 期权ID验证
/// - 行权数量验证
/// - 行权价格验证
/// - 行权类型验证
/// - 执行参数验证
/// 
/// # 参数
/// - params: 期权行权参数
/// 
/// # 返回
/// - Result<()>: 验证结果
pub fn validate_exercise_option_params(params: &ExerciseOptionParams) -> Result<()> {
    // 验证期权ID
    require!(
        params.option_id != Pubkey::default(),
        AssetError::InvalidOptionId
    );
    
    // 验证行权数量
    require!(
        params.exercise_amount > 0,
        AssetError::InvalidExerciseAmount
    );
    
    // 验证行权价格
    require!(
        params.exercise_price > 0.0,
        AssetError::InvalidExercisePrice
    );
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    Ok(())
}

/// 检查期权行权权限
/// 
/// 验证期权行权权限和授权状态：
/// - 检查所有权
/// - 验证Options Token状态
/// - 检查行权权限
/// 
/// # 参数
/// - exerciser: 行权者账户
/// - options_token: Options Token账户
/// 
/// # 返回
/// - Result<()>: 权限检查结果
pub fn check_exercise_option_authority_permission(
    exerciser: &Signer,
    options_token: &Account<Asset>,
) -> Result<()> {
    // 检查所有权
    require!(
        exerciser.key() == options_token.owner,
        AssetError::InvalidOwner
    );
    
    // 验证Options Token状态
    require!(
        options_token.is_active(),
        AssetError::AssetNotActive
    );
    
    Ok(())
}

/// 期权行权指令
/// 
/// 执行期权行权操作，包括：
/// - 参数验证：验证行权参数的有效性
/// - 权限检查：验证行权权限
/// - 服务层调用：委托给OptionsTokenService执行行权逻辑
/// - 事件发射：发射Options Token期权行权事件
/// 
/// # 参数
/// - ctx: 期权行权账户上下文
/// - params: 期权行权参数
/// 
/// # 返回
/// - Result<()>: 行权操作结果
pub fn exercise_option(
    ctx: Context<ExerciseOption>,
    params: ExerciseOptionParams,
) -> Result<()> {
    // 参数验证
    validate_exercise_option_params(&params)?;
    
    // 权限检查
    check_exercise_option_authority_permission(
        &ctx.accounts.exerciser,
        &ctx.accounts.options_token,
    )?;
    
    let options_token = &mut ctx.accounts.options_token;
    let exerciser = &ctx.accounts.exerciser;
    
    // 创建Options Token服务实例
    let service = OptionsTokenService::new();
    
    // 执行期权行权
    service.exercise_option(
        options_token,
        params.option_id,
        params.exercise_amount,
        params.exercise_price,
        &params.exercise_type,
        &params.exec_params,
    )?;
    
    // 发射期权行权事件
    emit!(AssetOptionExercised {
        basket_id: options_token.id,
        option_id: params.option_id,
        exercise_amount: params.exercise_amount,
        exercise_price: params.exercise_price,
        exercise_type: format!("{:?}", params.exercise_type),
        exerciser: exerciser.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::OptionsToken,
        exec_params: params.exec_params,
    });
    
    Ok(())
} 