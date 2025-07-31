//! ETF (Exchange Traded Fund) 流动性提供指令
//!
//! 本模块实现了ETF的流动性提供功能，包括流动性添加、移除和优化。
//!
//! ## 功能特点
//!
//! - **流动性添加**: 向ETF池添加流动性
//! - **流动性移除**: 从ETF池移除流动性
//! - **流动性优化**: 自动优化流动性分布
//! - **做市商激励**: 提供做市商激励机制
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::etf_service::EtfService;
use crate::events::asset_event::AssetLiquidityProvisionUpdated;
use crate::errors::AssetError;

/// 流动性操作类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum LiquidityOperationType {
    /// 添加流动性
    Add,
    /// 移除流动性
    Remove,
    /// 优化流动性
    Optimize,
    /// 再平衡流动性
    Rebalance,
}

/// 流动性提供方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum LiquidityProvisionMethod {
    /// 等值提供
    EqualValue,
    /// 比例提供
    Proportional,
    /// 自定义提供
    Custom,
    /// 算法提供
    Algorithmic,
}

/// 流动性信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LiquidityInfo {
    /// 流动性操作类型
    pub operation_type: LiquidityOperationType,
    /// 流动性数量
    pub liquidity_amount: u64,
    /// 流动性价值
    pub liquidity_value: f64,
    /// 流动性费用
    pub liquidity_fee: u64,
    /// 流动性份额
    pub liquidity_shares: u64,
    /// 操作时间戳
    pub timestamp: i64,
}

/// 流动性提供结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LiquidityProvisionResult {
    /// 流动性信息
    pub liquidity_info: LiquidityInfo,
    /// 提供方式
    pub provision_method: LiquidityProvisionMethod,
    /// 流动性池状态
    pub pool_status: String,
    /// 操作时间戳
    pub timestamp: i64,
}

/// ETF流动性提供指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct LiquidityProvisionParams {
    /// 流动性操作类型
    pub operation_type: LiquidityOperationType,
    /// 提供方式
    pub provision_method: LiquidityProvisionMethod,
    /// 流动性数量
    pub amount: u64,
    /// 流动性价格
    pub price: Option<u64>,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// ETF流动性提供指令账户上下文
#[derive(Accounts)]
pub struct LiquidityProvision<'info> {
    /// ETF资产账户，需可变
    #[account(
        mut,
        constraint = etf.asset_type == AssetType::Etf @ AssetError::InvalidAssetType
    )]
    pub etf: Account<'info, BasketIndexState>,
    
    /// 流动性提供者签名者
    pub liquidity_provider: Signer<'info>,
    
    /// 流动性提供者代币账户
    #[account(mut)]
    pub provider_token_account: Account<'info, TokenAccount>,
    
    /// ETF流动性池账户
    #[account(mut)]
    pub liquidity_pool_account: Account<'info, TokenAccount>,
    
    /// 成分股代币账户列表
    #[account(mut)]
    pub constituent_token_accounts: Vec<Account<'info, TokenAccount>>,
    
    /// DEX程序
    pub dex_program: Program<'info, DexProgram>,
    
    /// 价格预言机程序
    pub oracle_program: Program<'info, OracleProgram>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
    
    /// 关联代币程序
    pub associated_token_program: Program<'info, AssociatedToken>,
}

/// ETF流动性提供指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 流动性提供参数，包含操作类型、方式和数量
///
/// ## 返回值
/// - `Result<LiquidityProvisionResult>`: 流动性提供结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InvalidParams`: 无效的参数
/// - `InsufficientBalance`: 余额不足
/// - `LiquidityProvisionFailed`: 流动性提供失败
pub fn liquidity_provision(
    ctx: Context<LiquidityProvision>,
    params: LiquidityProvisionParams,
) -> Result<LiquidityProvisionResult> {
    // 参数验证
    validate_liquidity_provision_params(&params)?;
    
    // 获取账户引用
    let etf = &mut ctx.accounts.etf;
    let liquidity_provider = &ctx.accounts.liquidity_provider;
    
    // 创建服务实例
    let service = EtfService::new();
    
    // 调用服务层执行流动性提供操作
    let result = service.liquidity_provision(
        etf,
        &params.operation_type,
        &params.provision_method,
        params.amount,
        params.price,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetLiquidityProvisionUpdated {
        basket_id: etf.id,
        liquidity_provider: liquidity_provider.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Etf,
        operation_type: params.operation_type,
        provision_method: params.provision_method,
        liquidity_info: result.liquidity_info.clone(),
        pool_status: result.pool_status.clone(),
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证流动性提供参数
fn validate_liquidity_provision_params(params: &LiquidityProvisionParams) -> Result<()> {
    require!(params.amount > 0, AssetError::InvalidAmount);
    require!(params.amount <= u64::MAX, AssetError::InvalidAmount);
    
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