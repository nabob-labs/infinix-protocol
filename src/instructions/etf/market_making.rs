//! ETF (Exchange Traded Fund) 做市商机制指令
//!
//! 本模块实现了ETF的做市商机制功能，包括做市商管理、激励和监控。
//!
//! ## 功能特点
//!
//! - **做市商管理**: 管理做市商的注册和权限
//! - **做市商激励**: 提供做市商激励机制
//! - **流动性监控**: 监控做市商的流动性提供
//! - **做市商评估**: 评估做市商的表现
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::etf_service::EtfService;
use crate::events::asset_event::AssetMarketMakingUpdated;
use crate::errors::AssetError;

/// 做市商操作类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum MarketMakingOperationType {
    /// 注册做市商
    Register,
    /// 注销做市商
    Deregister,
    /// 更新做市商
    Update,
    /// 评估做市商
    Evaluate,
}

/// 做市商策略类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum MarketMakingStrategyType {
    /// 连续做市
    Continuous,
    /// 响应式做市
    Responsive,
    /// 算法做市
    Algorithmic,
    /// 混合做市
    Hybrid,
}

/// 做市商信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MarketMakerInfo {
    /// 做市商地址
    pub market_maker: Pubkey,
    /// 做市商策略
    pub strategy_type: MarketMakingStrategyType,
    /// 最小报价价差
    pub min_spread: f64,
    /// 最大报价价差
    pub max_spread: f64,
    /// 最小流动性
    pub min_liquidity: u64,
    /// 注册时间戳
    pub registration_timestamp: i64,
}

/// 做市商机制结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MarketMakingResult {
    /// 做市商信息
    pub market_maker_info: MarketMakerInfo,
    /// 操作类型
    pub operation_type: MarketMakingOperationType,
    /// 激励金额
    pub incentive_amount: u64,
    /// 表现评分
    pub performance_score: f64,
    /// 操作时间戳
    pub timestamp: i64,
}

/// ETF做市商机制指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MarketMakingParams {
    /// 做市商操作类型
    pub operation_type: MarketMakingOperationType,
    /// 做市商策略类型
    pub strategy_type: MarketMakingStrategyType,
    /// 做市商地址
    pub market_maker: Pubkey,
    /// 最小报价价差
    pub min_spread: Option<f64>,
    /// 最大报价价差
    pub max_spread: Option<f64>,
    /// 最小流动性
    pub min_liquidity: Option<u64>,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// ETF做市商机制指令账户上下文
#[derive(Accounts)]
pub struct MarketMaking<'info> {
    /// ETF资产账户，需可变
    #[account(
        mut,
        constraint = etf.asset_type == AssetType::Etf @ AssetError::InvalidAssetType
    )]
    pub etf: Account<'info, BasketIndexState>,
    
    /// 做市商管理权限签名者
    #[account(
        constraint = authority.key() == etf.market_making_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 做市商账户
    #[account(mut)]
    pub market_maker_account: Account<'info, TokenAccount>,
    
    /// 激励账户
    #[account(mut)]
    pub incentive_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// ETF做市商机制指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 做市商机制参数，包含操作类型、策略类型和做市商地址
///
/// ## 返回值
/// - `Result<MarketMakingResult>`: 做市商机制结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidParams`: 无效的参数
/// - `MarketMakingOperationFailed`: 做市商操作失败
pub fn market_making(
    ctx: Context<MarketMaking>,
    params: MarketMakingParams,
) -> Result<MarketMakingResult> {
    // 参数验证
    validate_market_making_params(&params)?;
    
    // 权限检查
    check_market_making_authority_permission(&ctx.accounts.authority, &ctx.accounts.etf)?;
    
    // 获取账户引用
    let etf = &mut ctx.accounts.etf;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = EtfService::new();
    
    // 调用服务层执行做市商机制操作
    let result = service.market_making(
        etf,
        &params.operation_type,
        &params.strategy_type,
        params.market_maker,
        params.min_spread,
        params.max_spread,
        params.min_liquidity,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetMarketMakingUpdated {
        basket_id: etf.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Etf,
        operation_type: params.operation_type,
        strategy_type: params.strategy_type,
        market_maker_info: result.market_maker_info.clone(),
        incentive_amount: result.incentive_amount,
        performance_score: result.performance_score,
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证做市商机制参数
fn validate_market_making_params(params: &MarketMakingParams) -> Result<()> {
    // 验证价差范围
    if let Some(min_spread) = params.min_spread {
        require!(min_spread >= 0.0, AssetError::InvalidParams);
        require!(min_spread <= 1.0, AssetError::InvalidParams);
    }
    
    if let Some(max_spread) = params.max_spread {
        require!(max_spread >= 0.0, AssetError::InvalidParams);
        require!(max_spread <= 1.0, AssetError::InvalidParams);
        if let Some(min_spread) = params.min_spread {
            require!(max_spread >= min_spread, AssetError::InvalidParams);
        }
    }
    
    // 验证最小流动性
    if let Some(min_liquidity) = params.min_liquidity {
        require!(min_liquidity > 0, AssetError::InvalidParams);
    }
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
    Ok(())
}

/// 检查做市商管理权限
fn check_market_making_authority_permission(
    authority: &Signer,
    etf: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == etf.market_making_authority,
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