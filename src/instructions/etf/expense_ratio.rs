//! ETF (Exchange Traded Fund) 费用率管理指令
//!
//! 本模块实现了ETF的费用率管理功能，包括管理费、托管费、申购赎回费用等。
//!
//! ## 功能特点
//!
//! - **管理费管理**: 设置和调整管理费率
//! - **托管费管理**: 设置和调整托管费率
//! - **申购赎回费用**: 管理申购赎回相关费用
//! - **费用计算**: 自动计算和收取各项费用
//! - **事件驱动**: 完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams};
use crate::services::etf_service::EtfService;
use crate::events::asset_event::AssetExpenseRatioUpdated;
use crate::errors::AssetError;

/// 费用类型
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ExpenseType {
    /// 管理费
    ManagementFee,
    /// 托管费
    CustodyFee,
    /// 申购费
    SubscriptionFee,
    /// 赎回费
    RedemptionFee,
    /// 交易费
    TradingFee,
    /// 其他费用
    OtherFee,
}

/// 费用调整方式
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ExpenseAdjustmentType {
    /// 绝对调整
    Absolute,
    /// 相对调整
    Relative,
    /// 百分比调整
    Percentage,
}

/// 费用信息
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ExpenseInfo {
    /// 费用类型
    pub expense_type: ExpenseType,
    /// 费用率
    pub rate: f64,
    /// 最小费用
    pub min_fee: u64,
    /// 最大费用
    pub max_fee: u64,
    /// 费用上限
    pub fee_cap: u64,
    /// 生效时间
    pub effective_time: i64,
}

/// 费用率更新结果
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ExpenseRatioResult {
    /// 费用信息
    pub expense_info: ExpenseInfo,
    /// 调整方式
    pub adjustment_type: ExpenseAdjustmentType,
    /// 调整前费率
    pub old_rate: f64,
    /// 调整后费率
    pub new_rate: f64,
    /// 更新时间戳
    pub timestamp: i64,
}

/// ETF费用率管理指令参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ExpenseRatioParams {
    /// 费用类型
    pub expense_type: ExpenseType,
    /// 调整方式
    pub adjustment_type: ExpenseAdjustmentType,
    /// 新费率
    pub new_rate: f64,
    /// 最小费用
    pub min_fee: Option<u64>,
    /// 最大费用
    pub max_fee: Option<u64>,
    /// 费用上限
    pub fee_cap: Option<u64>,
    /// 执行参数
    pub exec_params: ExecutionParams,
    /// 策略参数
    pub strategy_params: StrategyParams,
}

/// ETF费用率管理指令账户上下文
#[derive(Accounts)]
pub struct ExpenseRatio<'info> {
    /// ETF资产账户，需可变
    #[account(
        mut,
        constraint = etf.asset_type == AssetType::Etf @ AssetError::InvalidAssetType
    )]
    pub etf: Account<'info, BasketIndexState>,
    
    /// 费用管理权限签名者
    #[account(
        constraint = authority.key() == etf.expense_management_authority @ AssetError::InsufficientAuthority
    )]
    pub authority: Signer<'info>,
    
    /// 费用账户
    #[account(mut)]
    pub fee_account: Account<'info, TokenAccount>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// ETF费用率管理指令实现
///
/// ## 参数说明
/// - `ctx`: Anchor账户上下文，自动校验权限与生命周期
/// - `params`: 费用率管理参数，包含费用类型、调整方式和新费率
///
/// ## 返回值
/// - `Result<ExpenseRatioResult>`: 费用率更新结果
///
/// ## 错误处理
/// - `InvalidAssetType`: 资产类型不匹配
/// - `InsufficientAuthority`: 权限不足
/// - `InvalidParams`: 无效的参数
/// - `ExpenseRatioUpdateFailed`: 费用率更新失败
pub fn expense_ratio(
    ctx: Context<ExpenseRatio>,
    params: ExpenseRatioParams,
) -> Result<ExpenseRatioResult> {
    // 参数验证
    validate_expense_ratio_params(&params)?;
    
    // 权限检查
    check_expense_management_authority_permission(&ctx.accounts.authority, &ctx.accounts.etf)?;
    
    // 获取账户引用
    let etf = &mut ctx.accounts.etf;
    let authority = &ctx.accounts.authority;
    
    // 创建服务实例
    let service = EtfService::new();
    
    // 调用服务层执行费用率管理操作
    let result = service.expense_ratio(
        etf,
        &params.expense_type,
        &params.adjustment_type,
        params.new_rate,
        params.min_fee,
        params.max_fee,
        params.fee_cap,
        &params.exec_params,
        &params.strategy_params,
    )?;
    
    // 发射事件
    emit!(AssetExpenseRatioUpdated {
        basket_id: etf.id,
        authority: authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        asset_type: AssetType::Etf,
        expense_type: params.expense_type,
        adjustment_type: params.adjustment_type,
        old_rate: result.old_rate,
        new_rate: result.new_rate,
        expense_info: result.expense_info.clone(),
        exec_params: params.exec_params,
        strategy_params: params.strategy_params,
    });
    
    Ok(result)
}

/// 验证费用率管理参数
fn validate_expense_ratio_params(params: &ExpenseRatioParams) -> Result<()> {
    // 验证费率范围
    require!(params.new_rate >= 0.0, AssetError::InvalidParams);
    require!(params.new_rate <= 1.0, AssetError::InvalidParams); // 最大100%
    
    // 验证费用范围
    if let Some(min_fee) = params.min_fee {
        require!(min_fee > 0, AssetError::InvalidParams);
    }
    
    if let Some(max_fee) = params.max_fee {
        require!(max_fee > 0, AssetError::InvalidParams);
        if let Some(min_fee) = params.min_fee {
            require!(max_fee >= min_fee, AssetError::InvalidParams);
        }
    }
    
    if let Some(fee_cap) = params.fee_cap {
        require!(fee_cap > 0, AssetError::InvalidParams);
    }
    
    // 验证执行参数
    validate_execution_params(&params.exec_params)?;
    
    // 验证策略参数
    validate_strategy_params(&params.strategy_params)?;
    
    Ok(())
}

/// 检查费用管理权限
fn check_expense_management_authority_permission(
    authority: &Signer,
    etf: &Account<BasketIndexState>,
) -> Result<()> {
    require!(
        authority.key() == etf.expense_management_authority,
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