//! ETF (Exchange Traded Fund) 现金申购指令
//!
//! 本模块实现了ETF的现金申购功能，包括现金申购、份额计算和费用管理。
//!
//! ## 功能特点
//!
//! - **现金申购**: 使用现金申购ETF份额
//! - **份额计算**: 基于现金金额计算ETF份额
//! - **费用扣除**: 自动扣除申购费用
//! - **价格验证**: 验证申购价格的合理性
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::etf_service::EtfService;
use crate::events::asset_event::AssetCashCreation;
use crate::errors::AssetError;

/// 现金申购类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CashCreationType {
    /// 标准现金申购
    Standard,
    /// 大额现金申购
    Large,
    /// 小额现金申购
    Small,
    /// 批量现金申购
    Batch,
}

/// 现金申购方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CashCreationMethod {
    /// 市价申购
    Market,
    /// 限价申购
    Limit,
    /// 条件申购
    Conditional,
    /// 算法申购
    Algorithmic,
}

/// 现金申购结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CashCreationResult {
    /// 申购类型
    pub creation_type: CashCreationType,
    /// 申购方式
    pub creation_method: CashCreationMethod,
    /// 现金金额
    pub cash_amount: u64,
    /// 申购费用
    pub creation_fee: u64,
    /// 实际获得份额
    pub actual_shares: u64,
    /// 申购价格
    pub creation_price: f64,
    /// 申购时间戳
    pub timestamp: i64,
}

/// ETF现金申购指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CashCreationParams {
    /// 现金申购类型
    pub creation_type: CashCreationType,
    /// 现金申购方式
    pub creation_method: CashCreationMethod,
    /// 现金金额
    pub cash_amount: u64,
    /// 申购价格
    pub price: Option<f64>,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// ETF现金申购指令账户上下文
#[derive(Accounts)]
pub struct CashCreation<'info> {
    /// ETF资产账户，需可变
    #[account(
        mut,
        constraint = etf.asset_type == AssetType::Etf @ AssetError::InvalidAssetType
    )]
    pub etf: Account<'info, BasketIndexState>,
    
    /// 申购者签名者
    pub creator: Signer<'info>,
    
    /// 申购者现金账户
    #[account(mut)]
    pub creator_cash_account: Account<'info, TokenAccount>,
    
    /// ETF代币账户
    #[account(mut)]
    pub etf_token_account: Account<'info, TokenAccount>,
    
    /// 价格预言机程序
    pub oracle_program: Program<'info, OracleProgram>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
    
    /// 关联代币程序
    pub associated_token_program: Program<'info, AssociatedToken>,
}

/// ETF现金申购指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 现金申购参数，包含申购类型、方式和现金金额
///
/// ## 返回值
/// - `Result<CashCreationResult>`: 现金申购结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InvalidParams`: 无效的参数
/// - `InsufficientBalance`: 余额不足
/// - `CashCreationFailed`: 现金申购失败
pub fn cash_creation(
    ctx: Context<CashCreation>,
    params: CashCreationParams,
) -> Result<CashCreationResult> {
    // 参数验证
    validate_cash_creation_params(&params)?;
    
    // 获取账户引用
    let etf = &mut ctx.accounts.etf;
    let creator = &ctx.accounts.creator;
    
    // 创建服务实例
    let service = EtfService::new();
    
    // 调用服务层执行现金申购操作
    let result = service.cash_creation(
        etf,
        &params.creation_type,
        &params.creation_method,
        params.cash_amount,
        params.price,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetCashCreation {
        basket_id: etf.id,
        creator: creator.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Etf,
        creation_type: params.creation_type,
        creation_method: params.creation_method,
        cash_amount: result.cash_amount,
        creation_fee: result.creation_fee,
        actual_shares: result.actual_shares,
        creation_price: result.creation_price,
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证现金申购参数
fn validate_cash_creation_params(params: &CashCreationParams) -> Result<()> {
    require!(params.cash_amount > 0, AssetError::InvalidAmount);
    require!(params.cash_amount <= u64::MAX, AssetError::InvalidAmount);
    
    // 验证申购价格
    if let Some(price) = params.price {
        require!(price > 0.0, AssetError::InvalidParams);
        require!(price <= f64::MAX, AssetError::InvalidParams);
    }
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
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