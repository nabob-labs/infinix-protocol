//! ETF (Exchange Traded Fund) 实物申购指令
//!
//! 本模块实现了ETF的实物申购功能，包括实物申购、成分股验证和份额计算。
//!
//! ## 功能特点
//!
//! - **实物申购**: 使用成分股实物申购ETF份额
//! - **成分股验证**: 验证成分股的数量和质量
//! - **份额计算**: 基于成分股价值计算ETF份额
//! - **费用扣除**: 自动扣除申购费用
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::etf_service::EtfService;
use crate::events::asset_event::AssetInKindCreation;
use crate::errors::AssetError;

/// 实物申购类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum InKindCreationType {
    /// 标准实物申购
    Standard,
    /// 自定义实物申购
    Custom,
    /// 部分实物申购
    Partial,
    /// 完全实物申购
    Full,
}

/// 成分股信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ConstituentInfo {
    /// 成分股地址
    pub token_address: Pubkey,
    /// 成分股数量
    pub amount: u64,
    /// 成分股权重
    pub weight: f64,
    /// 成分股价格
    pub price: f64,
}

/// 实物申购结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InKindCreationResult {
    /// 申购类型
    pub creation_type: InKindCreationType,
    /// 申购数量
    pub creation_amount: u64,
    /// 申购费用
    pub creation_fee: u64,
    /// 实际获得份额
    pub actual_shares: u64,
    /// 成分股信息
    pub constituents: Vec<ConstituentInfo>,
    /// 申购时间戳
    pub timestamp: i64,
}

/// ETF实物申购指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InKindCreationParams {
    /// 实物申购类型
    pub creation_type: InKindCreationType,
    /// 申购数量
    pub amount: u64,
    /// 成分股信息
    pub constituents: Vec<ConstituentInfo>,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// ETF实物申购指令账户上下文
#[derive(Accounts)]
pub struct InKindCreation<'info> {
    /// ETF资产账户，需可变
    #[account(
        mut,
        constraint = etf.asset_type == AssetType::Etf @ AssetError::InvalidAssetType
    )]
    pub etf: Account<'info, BasketIndexState>,
    
    /// 申购者签名者
    pub creator: Signer<'info>,
    
    /// 申购者代币账户
    #[account(mut)]
    pub creator_token_account: Account<'info, TokenAccount>,
    
    /// ETF代币账户
    #[account(mut)]
    pub etf_token_account: Account<'info, TokenAccount>,
    
    /// 成分股代币账户列表
    #[account(mut)]
    pub constituent_token_accounts: Vec<Account<'info, TokenAccount>>,
    
    /// 价格预言机程序
    pub oracle_program: Program<'info, OracleProgram>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
    
    /// 关联代币程序
    pub associated_token_program: Program<'info, AssociatedToken>,
}

/// ETF实物申购指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 实物申购参数，包含申购类型、数量和成分股信息
///
/// ## 返回值
/// - `Result<InKindCreationResult>`: 实物申购结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InvalidParams`: 无效的参数
/// - `InsufficientBalance`: 余额不足
/// - `InKindCreationFailed`: 实物申购失败
pub fn in_kind_creation(
    ctx: Context<InKindCreation>,
    params: InKindCreationParams,
) -> Result<InKindCreationResult> {
    // 参数验证
    validate_in_kind_creation_params(&params)?;
    
    // 获取账户引用
    let etf = &mut ctx.accounts.etf;
    let creator = &ctx.accounts.creator;
    
    // 创建服务实例
    let service = EtfService::new();
    
    // 调用服务层执行实物申购操作
    let result = service.in_kind_creation(
        etf,
        &params.creation_type,
        params.amount,
        &params.constituents,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetInKindCreation {
        basket_id: etf.id,
        creator: creator.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Etf,
        creation_type: params.creation_type,
        creation_amount: result.creation_amount,
        creation_fee: result.creation_fee,
        actual_shares: result.actual_shares,
        constituents: result.constituents.clone(),
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证实物申购参数
fn validate_in_kind_creation_params(params: &InKindCreationParams) -> Result<()> {
    require!(params.amount > 0, AssetError::InvalidAmount);
    require!(params.amount <= u64::MAX, AssetError::InvalidAmount);
    
    // 验证成分股信息
    validate_constituents(&params.constituents)?;
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
    Ok(())
}

/// 验证成分股信息
fn validate_constituents(constituents: &[ConstituentInfo]) -> Result<()> {
    require!(!constituents.is_empty(), AssetError::InvalidParams);
    require!(constituents.len() <= 100, AssetError::InvalidParams); // 最大100个成分股
    
    for constituent in constituents {
        require!(constituent.amount > 0, AssetError::InvalidAmount);
        require!(constituent.weight >= 0.0, AssetError::InvalidParams);
        require!(constituent.weight <= 1.0, AssetError::InvalidParams);
        require!(constituent.price > 0.0, AssetError::InvalidParams);
    }
    
    // 验证权重总和
    let total_weight: f64 = constituents.iter().map(|c| c.weight).sum();
    require!((total_weight - 1.0).abs() < 0.001, AssetError::InvalidParams); // 权重总和应为1
    
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