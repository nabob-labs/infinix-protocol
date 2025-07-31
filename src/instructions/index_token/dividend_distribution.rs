//! 指数代币 (IndexToken) 分红分配指令
//!
//! 本模块实现了指数代币的分红分配功能，包括分红计算、分配执行和分红记录。
//!
//! ## 功能特点
//!
//! - **分红计算**: 根据成分股分红计算指数分红
//! - **分红分配**: 按比例分配给指数代币持有者
//! - **分红记录**: 记录分红历史和分配明细
//! - **自动分配**: 支持自动分红分配机制
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::index_token_service::IndexTokenService;
use crate::events::asset_event::AssetDividendDistributed;
use crate::errors::AssetError;

/// 分红分配类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum DividendDistributionType {
    /// 现金分红
    Cash,
    /// 代币分红
    Token,
    /// 混合分红
    Mixed,
    /// 自动再投资
    Reinvest,
}

/// 分红分配策略
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum DividendDistributionStrategy {
    /// 按比例分配
    Proportional,
    /// 等额分配
    Equal,
    /// 阶梯分配
    Tiered,
    /// 自定义分配
    Custom,
}

/// 分红信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DividendInfo {
    /// 分红代币地址
    pub token_address: Pubkey,
    /// 分红数量
    pub amount: u64,
    /// 分红比例
    pub ratio: f64,
    /// 分红时间戳
    pub timestamp: i64,
}

/// 分红分配结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DividendDistributionResult {
    /// 分红分配类型
    pub distribution_type: DividendDistributionType,
    /// 分红信息列表
    pub dividends: Vec<DividendInfo>,
    /// 分配成本
    pub distribution_cost: u64,
    /// 分配时间戳
    pub timestamp: i64,
    /// 总分红金额
    pub total_dividend: u64,
}

/// 指数代币分红分配指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DividendDistributionParams {
    /// 分红分配类型
    pub distribution_type: DividendDistributionType,
    /// 分红分配策略
    pub strategy: DividendDistributionStrategy,
    /// 分红代币列表
    pub dividend_tokens: Vec<Pubkey>,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// 指数代币分红分配指令账户上下文
#[derive(Accounts)]
pub struct DividendDistribution<'info> {
    /// 指数代币资产账户，需可变
    #[account(
        mut,
        constraint = index_token.asset_type == AssetType::IndexToken @ AssetError::InvalidAssetType
    )]
    pub index_token: Account<'info, BasketIndexState>,
    
    /// 分红分配权限签名者
    #[account(
        constraint = authority.key() == index_token.dividend_distribution_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 分红代币账户列表
    #[account(mut)]
    pub dividend_token_accounts: Vec<Account<'info, TokenAccount>>,
    
    /// 持有者代币账户列表
    #[account(mut)]
    pub holder_token_accounts: Vec<Account<'info, TokenAccount>>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
    
    /// 关联代币程序
    pub associated_token_program: Program<'info, AssociatedToken>,
}

/// 指数代币分红分配指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 分红分配参数，包含分配类型、策略和分红代币
///
/// ## 返回值
/// - `Result<DividendDistributionResult>`: 分红分配结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidParams`: 无效的参数
/// - `DividendDistributionFailed`: 分红分配失败
pub fn dividend_distribution(
    ctx: Context<DividendDistribution>,
    params: DividendDistributionParams,
) -> Result<DividendDistributionResult> {
    // 参数验证
    validate_dividend_distribution_params(&params)?;
    
    // 权限检查
    check_dividend_distribution_authority_permission(&ctx.accounts.authority, &ctx.accounts.index_token)?;
    
    // 获取账户引用
    let index_token = &mut ctx.accounts.index_token;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = IndexTokenService::new();
    
    // 调用服务层执行分红分配操作
    let result = service.dividend_distribution(
        index_token,
        &params.distribution_type,
        &params.strategy,
        &params.dividend_tokens,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetDividendDistributed {
        basket_id: index_token.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::IndexToken,
        distribution_type: params.distribution_type,
        strategy: params.strategy,
        dividends: result.dividends.clone(),
        total_dividend: result.total_dividend,
        distribution_cost: result.distribution_cost,
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证分红分配参数
fn validate_dividend_distribution_params(params: &DividendDistributionParams) -> Result<()> {
    // 验证分红代币列表
    validate_dividend_tokens(&params.dividend_tokens)?;
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
    Ok(())
}

/// 验证分红代币列表
fn validate_dividend_tokens(dividend_tokens: &[Pubkey]) -> Result<()> {
    require!(!dividend_tokens.is_empty(), AssetError::InvalidParams);
    require!(dividend_tokens.len() <= 20, AssetError::InvalidParams);
    
    // 检查代币地址唯一性
    let mut unique_tokens = std::collections::HashSet::new();
    for token in dividend_tokens {
        require!(unique_tokens.insert(*token), AssetError::InvalidParams);
    }
    
    Ok(())
}

/// 检查分红分配权限
fn check_dividend_distribution_authority_permission(
    authority: &Signer,
    index_token: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == index_token.dividend_distribution_authority,
        AssetError::InsufficientAuthority
    );
    
    Ok(())
}

/// 验证执行参数
fn validate_execution_params(exec_params: &ExecutionParams) -> Result<()> {
    require!(exec_params.slippage_tolerance > 0.0, AssetError::InvalidParams);
    require!(exec_params.slippage_tolerance <= 1.0, AssetError::InvalidParams);
    require!(exec_params.max_retries > 0, AssetError::InvalidParams);
    require!(exec_params.max_retries <= 10, AssetError::InvalidParams);
    
    Ok(())
}

/// 验证策略参数
fn validate_strategy_params(strategy_params: &StrategyParams) -> Result<()> {
    require!(strategy_params.max_slippage > 0.0, AssetError::InvalidParams);
    require!(strategy_params.max_slippage <= 1.0, AssetError::InvalidParams);
    require!(strategy_params.execution_timeout > 0, AssetError::InvalidParams);
    
    Ok(())
} 