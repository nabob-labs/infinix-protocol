//! 加密货币 (Crypto) 闪电贷指令
//! 
//! 本模块实现加密货币资产的闪电贷功能，支持闪电贷借贷、还款、套利等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 闪电贷借贷：支持无抵押闪电贷
//! - 闪电贷还款：自动还款机制
//! - 套利支持：闪电贷套利功能
//! - 事件记录：完整的审计追踪
//! - 状态管理：原子性操作保证数据一致性

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, FlashLoanParams};
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetFlashLoaned;
use crate::validation::business::validate_flash_loan_params;
use crate::core::security::check_authority_permission;

/// 加密货币闪电贷指令账户上下文
/// 
/// 定义闪电贷操作所需的所有账户，包括：
/// - crypto_asset: 加密货币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - lending_pool: 借贷池账户（可变）
/// - oracle_program: 预言机程序（用于价格数据）
/// - system_program: 系统程序（用于账户管理）
#[derive(Accounts)]
#[instruction(params: FlashLoanParams)]
pub struct FlashLoanCrypto<'info> {
    /// 加密货币资产账户，需要可变权限以更新状态
    #[account(
        mut,
        seeds = [b"crypto", crypto_asset.key().as_ref()],
        bump,
        constraint = crypto_asset.asset_type == AssetType::Crypto @ crate::errors::AssetError::InvalidAssetType
    )]
    pub crypto_asset: Account<'info, crate::account_models::asset::Asset>,
    
    /// 操作权限账户，必须是签名者
    #[account(
        constraint = check_authority_permission(&authority.key(), &crypto_asset.authority) @ crate::errors::SecurityError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    /// 借贷池账户，需要可变权限以管理借贷
    #[account(
        mut,
        seeds = [b"lending_pool", lending_pool.key().as_ref()],
        bump,
        constraint = lending_pool.balance >= params.loan_amount @ crate::errors::AssetError::InsufficientLiquidity
    )]
    pub lending_pool: Account<'info, crate::account_models::asset::Asset>,
    
    /// 预言机程序，用于价格数据
    pub oracle_program: Program<'info, crate::oracles::traits::OracleAdapterTrait>,
    
    /// 系统程序，用于账户管理
    pub system_program: Program<'info, System>,
    
    /// 时钟账户，用于时间戳记录
    pub clock: Sysvar<'info, Clock>,
}

/// 加密货币闪电贷指令实现
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文，自动校验账户权限与生命周期
/// - `params`: 闪电贷参数，包含借贷数量、费用等
/// - `exec_params`: 可选算法执行参数，支持算法热插拔
/// - `strategy_params`: 可选策略参数，支持多策略扩展
/// 
/// ## 返回值
/// - `anchor_lang::Result<()>`: Anchor标准返回类型，表示指令执行成功或失败
/// 
/// ## 安全性
/// - Anchor自动校验账户权限、生命周期、PDA
/// - 业务层校验闪电贷参数合法性
/// - 借贷池流动性检查
/// - 完整的事件记录和审计追踪
pub fn flash_loan_crypto(
    ctx: Context<FlashLoanCrypto>,
    params: FlashLoanParams,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    // 校验闪电贷参数合法性
    validate_flash_loan_params(&params)?;
    
    // 检查借贷池流动性充足性
    require!(
        ctx.accounts.lending_pool.balance >= params.loan_amount,
        crate::errors::AssetError::InsufficientLiquidity
    );
    
    // === 2. 权限校验 ===
    // 检查操作权限
    require!(
        ctx.accounts.authority.key() == ctx.accounts.crypto_asset.authority,
        crate::errors::SecurityError::Unauthorized
    );
    
    // === 3. 业务逻辑执行 ===
    // 创建加密货币服务实例
    let crypto_service = CryptoService::new();
    
    // 记录闪电贷前的状态
    let balance_before = ctx.accounts.crypto_asset.balance;
    let pool_balance_before = ctx.accounts.lending_pool.balance;
    
    // 执行闪电贷操作
    let flash_loan_result = crypto_service.flash_loan(
        &mut ctx.accounts.crypto_asset,
        &mut ctx.accounts.lending_pool,
        &params
    )?;
    
    // === 4. 算法执行（如果提供） ===
    if let Some(algo_params) = exec_params {
        crypto_service.execute_algorithm(&mut ctx.accounts.crypto_asset, algo_params)?;
    }
    
    // === 5. 策略执行（如果提供） ===
    if let Some(strategy) = strategy_params {
        crypto_service.execute_strategy(&mut ctx.accounts.crypto_asset, strategy)?;
    }
    
    // === 6. 事件记录 ===
    // 发出闪电贷事件，记录操作详情
    emit!(AssetFlashLoaned {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        loan_amount: params.loan_amount,
        loan_fee: params.loan_fee,
        balance_before,
        balance_after: ctx.accounts.crypto_asset.balance,
        pool_balance_before,
        pool_balance_after: ctx.accounts.lending_pool.balance,
        loan_id: flash_loan_result.loan_id,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    // === 7. 日志记录 ===
    msg!("Crypto flash loan executed successfully: loan_amount={}, loan_fee={}, authority={}", 
         params.loan_amount, params.loan_fee, ctx.accounts.authority.key());
    
    Ok(())
}

/// 闪电贷套利指令
/// 
/// 使用闪电贷执行套利操作。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `params`: 闪电贷参数
/// - `arbitrage_params`: 套利参数
pub fn flash_loan_arbitrage_crypto(
    ctx: Context<FlashLoanCrypto>,
    params: FlashLoanParams,
    arbitrage_params: FlashLoanArbitrage
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    validate_flash_loan_params(&params)?;
    require!(
        ctx.accounts.lending_pool.balance >= params.loan_amount,
        crate::errors::AssetError::InsufficientLiquidity
    );
    
    // === 2. 闪电贷套利执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行闪电贷套利
    let arbitrage_result = crypto_service.flash_loan_arbitrage(
        &mut ctx.accounts.crypto_asset,
        &mut ctx.accounts.lending_pool,
        &params,
        &arbitrage_params
    )?;
    
    // === 3. 事件记录 ===
    emit!(AssetFlashLoaned {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        loan_amount: params.loan_amount,
        loan_fee: params.loan_fee,
        balance_before: 0,
        balance_after: ctx.accounts.crypto_asset.balance,
        pool_balance_before: 0,
        pool_balance_after: ctx.accounts.lending_pool.balance,
        loan_id: arbitrage_result.loan_id,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: None,
    });
    
    msg!("Crypto flash loan arbitrage executed successfully: loan_amount={}, arbitrage_profit={}, authority={}", 
         params.loan_amount, arbitrage_result.profit, ctx.accounts.authority.key());
    
    Ok(())
}

/// 闪电贷还款指令
/// 
/// 执行闪电贷还款操作。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `loan_id`: 借贷ID
/// - `repayment_params`: 还款参数
pub fn flash_loan_repay_crypto(
    ctx: Context<FlashLoanCrypto>,
    loan_id: String,
    repayment_params: FlashLoanRepayment
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    require!(!loan_id.is_empty(), crate::errors::AssetError::InvalidLoanId);
    
    // === 2. 闪电贷还款执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行闪电贷还款
    let repayment_result = crypto_service.flash_loan_repay(
        &mut ctx.accounts.crypto_asset,
        &mut ctx.accounts.lending_pool,
        &loan_id,
        &repayment_params
    )?;
    
    // === 3. 事件记录 ===
    emit!(AssetFlashLoaned {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        loan_amount: 0,
        loan_fee: repayment_params.repayment_fee,
        balance_before: 0,
        balance_after: ctx.accounts.crypto_asset.balance,
        pool_balance_before: 0,
        pool_balance_after: ctx.accounts.lending_pool.balance,
        loan_id: loan_id.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: None,
    });
    
    msg!("Crypto flash loan repayment completed: loan_id={}, repayment_amount={}, authority={}", 
         loan_id, repayment_params.repayment_amount, ctx.accounts.authority.key());
    
    Ok(())
}

/// 闪电贷参数结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct FlashLoanParams {
    /// 借贷数量
    pub loan_amount: u64,
    /// 借贷费用
    pub loan_fee: u64,
    /// 借贷期限
    pub loan_duration: u64,
    /// 借贷类型
    pub loan_type: FlashLoanType,
}

/// 闪电贷套利结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct FlashLoanArbitrage {
    /// 套利策略
    pub arbitrage_strategy: String,
    /// 套利参数
    pub arbitrage_params: String,
    /// 预期利润
    pub expected_profit: u64,
}

/// 闪电贷还款结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct FlashLoanRepayment {
    /// 还款数量
    pub repayment_amount: u64,
    /// 还款费用
    pub repayment_fee: u64,
    /// 还款时间
    pub repayment_time: i64,
}

/// 闪电贷类型枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub enum FlashLoanType {
    /// 标准闪电贷
    Standard,
    /// 套利闪电贷
    Arbitrage,
    /// 紧急闪电贷
    Emergency,
}

/// 闪电贷结果结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct FlashLoanResult {
    /// 借贷ID
    pub loan_id: String,
    /// 借贷状态
    pub status: String,
    /// 借贷时间
    pub loan_time: i64,
    /// 借贷费用
    pub loan_fee: u64,
    /// 套利利润
    pub profit: u64,
} 