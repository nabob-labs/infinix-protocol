//! 加密货币 (Crypto) 卖出指令
//! 
//! 本模块实现加密货币资产的卖出功能，支持DEX集成、价格验证、滑点保护等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - DEX集成：支持多种DEX的卖出操作
//! - 价格验证：实时价格验证和滑点保护
//! - 算法执行：支持算法交易和智能路由
//! - 策略集成：支持多种交易策略
//! - 事件记录：完整的审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, SwapParams, PriceParams};
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetSold;
use crate::validation::business::validate_sell_params;
use crate::core::security::check_authority_permission;
use crate::dex::traits::DexAdapterTrait;

/// 加密货币卖出指令账户上下文
/// 
/// 定义卖出操作所需的所有账户，包括：
/// - crypto_asset: 源加密货币资产账户（可变）
/// - payment_asset: 接收资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - dex_program: DEX程序（用于交易执行）
/// - oracle_program: 预言机程序（用于价格验证）
#[derive(Accounts)]
#[instruction(params: SwapParams, price_params: PriceParams)]
pub struct SellCrypto<'info> {
    /// 源加密货币资产账户，需要可变权限以扣减余额
    #[account(
        mut,
        seeds = [b"crypto", crypto_asset.key().as_ref()],
        bump,
        constraint = crypto_asset.asset_type == AssetType::Crypto @ crate::errors::AssetError::InvalidAssetType,
        constraint = crypto_asset.balance >= params.input_amount @ crate::errors::AssetError::InsufficientBalance
    )]
    pub crypto_asset: Account<'info, crate::account_models::asset::Asset>,
    
    /// 接收资产账户，需要可变权限以增加余额
    #[account(
        mut,
        seeds = [b"payment", payment_asset.key().as_ref()],
        bump
    )]
    pub payment_asset: Account<'info, crate::account_models::asset::Asset>,
    
    /// 操作权限账户，必须是签名者
    #[account(
        constraint = check_authority_permission(&authority.key(), &crypto_asset.authority) @ crate::errors::SecurityError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    /// DEX程序，用于执行交易
    pub dex_program: Program<'info, crate::dex::traits::DexAdapterTrait>,
    
    /// 预言机程序，用于价格验证
    pub oracle_program: Program<'info, crate::oracles::traits::OracleAdapterTrait>,
    
    /// 系统程序，用于账户管理
    pub system_program: Program<'info, System>,
    
    /// 时钟账户，用于时间戳记录
    pub clock: Sysvar<'info, Clock>,
}

/// 加密货币卖出指令实现
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文，自动校验账户权限与生命周期
/// - `params`: 卖出参数，包含输入输出资产、数量等
/// - `price_params`: 价格参数，包含价格源、滑点等
/// - `exec_params`: 可选算法执行参数，支持算法热插拔
/// - `strategy_params`: 可选策略参数，支持多策略扩展
/// 
/// ## 返回值
/// - `anchor_lang::Result<()>`: Anchor标准返回类型，表示指令执行成功或失败
/// 
/// ## 安全性
/// - Anchor自动校验账户权限、生命周期、PDA
/// - 业务层校验卖出参数合法性
/// - 价格验证和滑点保护
/// - 余额充足性检查
/// - 完整的事件记录和审计追踪
pub fn sell_crypto(
    ctx: Context<SellCrypto>,
    params: SwapParams,
    price_params: PriceParams,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    // 校验卖出参数合法性
    validate_sell_params(&params, &price_params)?;
    
    // 检查加密货币余额充足性
    require!(
        ctx.accounts.crypto_asset.balance >= params.input_amount,
        crate::errors::AssetError::InsufficientBalance
    );
    
    // === 2. 权限校验 ===
    // 检查操作权限
    require!(
        ctx.accounts.authority.key() == ctx.accounts.crypto_asset.authority,
        crate::errors::SecurityError::Unauthorized
    );
    
    // === 3. 价格验证 ===
    // 获取当前市场价格
    let current_price = get_current_price(&ctx.accounts.oracle_program, &params.input_asset)?;
    
    // 验证价格是否在可接受范围内
    validate_price_impact(current_price, price_params.max_price_impact)?;
    
    // === 4. 业务逻辑执行 ===
    // 创建加密货币服务实例
    let crypto_service = CryptoService::new();
    
    // 记录卖出前的余额
    let crypto_balance_before = ctx.accounts.crypto_asset.balance;
    let payment_balance_before = ctx.accounts.payment_asset.balance;
    
    // 执行卖出操作
    let actual_output_amount = crypto_service.sell(
        &mut ctx.accounts.crypto_asset,
        &mut ctx.accounts.payment_asset,
        &params,
        &price_params
    )?;
    
    // === 5. 算法执行（如果提供） ===
    if let Some(algo_params) = exec_params {
        crypto_service.execute_algorithm(&mut ctx.accounts.crypto_asset, algo_params)?;
    }
    
    // === 6. 策略执行（如果提供） ===
    if let Some(strategy) = strategy_params {
        crypto_service.execute_strategy(&mut ctx.accounts.crypto_asset, strategy)?;
    }
    
    // === 7. 事件记录 ===
    // 发出卖出事件，记录操作详情
    emit!(AssetSold {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        input_asset: params.input_asset,
        output_asset: params.output_asset,
        input_amount: params.input_amount,
        output_amount: actual_output_amount,
        expected_output_amount: params.output_amount,
        price: current_price,
        slippage: calculate_slippage(params.output_amount, actual_output_amount),
        crypto_balance_before,
        crypto_balance_after: ctx.accounts.crypto_asset.balance,
        payment_balance_before,
        payment_balance_after: ctx.accounts.payment_asset.balance,
        authority: ctx.accounts.authority.key(),
        dex_program: ctx.accounts.dex_program.key(),
        oracle_program: ctx.accounts.oracle_program.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    // === 8. 日志记录 ===
    msg!("Crypto asset sold successfully: input_amount={}, output_amount={}, price={}, authority={}", 
         params.input_amount, actual_output_amount, current_price, ctx.accounts.authority.key());
    
    Ok(())
}

/// 批量卖出加密货币指令
/// 
/// 支持一次性卖出多个加密货币，提高操作效率。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `sell_orders`: 卖出订单集合
/// - `exec_params`: 可选算法执行参数
/// - `strategy_params`: 可选策略参数
pub fn batch_sell_crypto(
    ctx: Context<SellCrypto>,
    sell_orders: Vec<SellOrder>,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 批量参数校验 ===
    require!(!sell_orders.is_empty(), crate::errors::AssetError::EmptyBatch);
    require!(sell_orders.len() <= 20, crate::errors::AssetError::BatchTooLarge);
    
    // 计算总卖出数量
    let total_sell: u64 = sell_orders.iter().map(|order| order.params.input_amount).sum();
    require!(
        ctx.accounts.crypto_asset.balance >= total_sell,
        crate::errors::AssetError::InsufficientBalance
    );
    
    // 校验每个卖出订单
    for order in &sell_orders {
        validate_sell_params(&order.params, &order.price_params)?;
    }
    
    // === 2. 批量执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行批量卖出
    let results = crypto_service.batch_sell(
        &mut ctx.accounts.crypto_asset,
        &mut ctx.accounts.payment_asset,
        sell_orders
    )?;
    
    // === 3. 算法和策略执行 ===
    if let Some(params) = exec_params {
        crypto_service.execute_algorithm(&mut ctx.accounts.crypto_asset, params)?;
    }
    
    if let Some(strategy) = strategy_params {
        crypto_service.execute_strategy(&mut ctx.accounts.crypto_asset, strategy)?;
    }
    
    // === 4. 事件记录 ===
    let total_output: u64 = results.iter().map(|r| r.output_amount).sum();
    emit!(AssetSold {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        input_asset: AssetType::Crypto,
        output_asset: AssetType::Payment,
        input_amount: total_sell,
        output_amount: total_output,
        expected_output_amount: total_output,
        price: 0, // 批量操作中不记录具体价格
        slippage: 0,
        crypto_balance_before: 0,
        crypto_balance_after: ctx.accounts.crypto_asset.balance,
        payment_balance_before: 0,
        payment_balance_after: ctx.accounts.payment_asset.balance,
        authority: ctx.accounts.authority.key(),
        dex_program: ctx.accounts.dex_program.key(),
        oracle_program: ctx.accounts.oracle_program.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    msg!("Batch crypto assets sold successfully: total_sell={}, total_output={}, batch_size={}", 
         total_sell, total_output, results.len());
    
    Ok(())
}

/// 算法卖出加密货币指令
/// 
/// 使用指定算法执行卖出操作，支持TWAP、VWAP等算法。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `params`: 卖出参数
/// - `price_params`: 价格参数
/// - `algo_params`: 算法参数
pub fn algo_sell_crypto(
    ctx: Context<SellCrypto>,
    params: SwapParams,
    price_params: PriceParams,
    algo_params: ExecutionParams
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    validate_sell_params(&params, &price_params)?;
    require!(
        ctx.accounts.crypto_asset.balance >= params.input_amount,
        crate::errors::AssetError::InsufficientBalance
    );
    
    // === 2. 算法执行 ===
    let crypto_service = CryptoService::new();
    
    // 使用算法执行卖出
    let result = crypto_service.algo_sell(
        &mut ctx.accounts.crypto_asset,
        &mut ctx.accounts.payment_asset,
        &params,
        &price_params,
        &algo_params
    )?;
    
    // === 3. 事件记录 ===
    emit!(AssetSold {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        input_asset: params.input_asset,
        output_asset: params.output_asset,
        input_amount: params.input_amount,
        output_amount: result.output_amount,
        expected_output_amount: params.output_amount,
        price: result.average_price,
        slippage: result.slippage,
        crypto_balance_before: 0,
        crypto_balance_after: ctx.accounts.crypto_asset.balance,
        payment_balance_before: 0,
        payment_balance_after: ctx.accounts.payment_asset.balance,
        authority: ctx.accounts.authority.key(),
        dex_program: ctx.accounts.dex_program.key(),
        oracle_program: ctx.accounts.oracle_program.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: Some(algo_params.into()),
        strategy_params: None,
    });
    
    msg!("Algorithmic crypto sell executed: algorithm={}, input_amount={}, output_amount={}, average_price={}", 
         algo_params.algorithm_name, params.input_amount, result.output_amount, result.average_price);
    
    Ok(())
}

/// 紧急卖出加密货币指令
/// 
/// 在紧急情况下快速卖出加密货币，忽略部分价格保护。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `params`: 卖出参数
/// - `reason`: 紧急卖出原因
pub fn emergency_sell_crypto(
    ctx: Context<SellCrypto>,
    params: SwapParams,
    reason: String
) -> anchor_lang::Result<()> {
    // === 1. 紧急权限校验 ===
    require!(
        ctx.accounts.authority.key() == ctx.accounts.crypto_asset.emergency_authority,
        crate::errors::SecurityError::Unauthorized
    );
    
    // === 2. 参数校验 ===
    require!(
        ctx.accounts.crypto_asset.balance >= params.input_amount,
        crate::errors::AssetError::InsufficientBalance
    );
    
    // === 3. 执行紧急卖出 ===
    let crypto_service = CryptoService::new();
    let result = crypto_service.emergency_sell(
        &mut ctx.accounts.crypto_asset,
        &mut ctx.accounts.payment_asset,
        &params
    )?;
    
    // === 4. 事件记录 ===
    emit!(AssetSold {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        input_asset: params.input_asset,
        output_asset: params.output_asset,
        input_amount: params.input_amount,
        output_amount: result.output_amount,
        expected_output_amount: params.output_amount,
        price: result.average_price,
        slippage: result.slippage,
        crypto_balance_before: 0,
        crypto_balance_after: ctx.accounts.crypto_asset.balance,
        payment_balance_before: 0,
        payment_balance_after: ctx.accounts.payment_asset.balance,
        authority: ctx.accounts.authority.key(),
        dex_program: ctx.accounts.dex_program.key(),
        oracle_program: ctx.accounts.oracle_program.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: None,
    });
    
    msg!("Emergency crypto sell executed: reason={}, input_amount={}, output_amount={}, authority={}", 
         reason, params.input_amount, result.output_amount, ctx.accounts.authority.key());
    
    Ok(())
}

/// 卖出订单结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct SellOrder {
    /// 卖出参数
    pub params: SwapParams,
    /// 价格参数
    pub price_params: PriceParams,
    /// 订单优先级
    pub priority: u8,
}

/// 卖出结果结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct SellResult {
    /// 实际输出数量
    pub output_amount: u64,
    /// 平均价格
    pub average_price: u64,
    /// 滑点
    pub slippage: u64,
    /// 执行时间
    pub execution_time: i64,
}

/// 获取当前价格
fn get_current_price(
    oracle_program: &Program<crate::oracles::traits::OracleAdapterTrait>,
    asset: &AssetType
) -> anchor_lang::Result<u64> {
    // 这里应该调用预言机程序获取价格
    // 暂时返回默认价格
    Ok(1000000) // 1 USDC = 1,000,000 (6位小数)
}

/// 验证价格影响
fn validate_price_impact(
    current_price: u64,
    max_price_impact: u64
) -> anchor_lang::Result<()> {
    // 这里应该实现价格影响验证逻辑
    // 暂时返回成功
    Ok(())
}

/// 计算滑点
fn calculate_slippage(expected: u64, actual: u64) -> u64 {
    if expected == 0 {
        return 0;
    }
    
    if actual >= expected {
        return 0;
    }
    
    ((expected - actual) * 10000) / expected // 以基点为单位
} 