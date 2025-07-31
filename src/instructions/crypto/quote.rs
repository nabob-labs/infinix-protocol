//! 加密货币 (Crypto) 报价指令
//! 
//! 本模块实现加密货币资产的报价功能，支持多DEX报价聚合、价格验证、滑点计算等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 多DEX报价聚合：从多个DEX获取最优报价
//! - 价格验证：实时价格验证和准确性检查
//! - 滑点计算：精确的滑点计算和预估
//! - 流动性检查：检查DEX流动性充足性
//! - 事件记录：完整的审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, SwapParams, PriceParams, QuoteResult};
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetQuoted;
use crate::validation::business::validate_quote_params;
use crate::core::security::check_authority_permission;
use crate::dex::traits::DexAdapterTrait;

/// 加密货币报价指令账户上下文
/// 
/// 定义报价操作所需的所有账户，包括：
/// - crypto_asset: 加密货币资产账户（只读）
/// - authority: 操作权限账户（签名者，可选）
/// - dex_program: DEX程序（用于获取报价）
/// - oracle_program: 预言机程序（用于价格验证）
#[derive(Accounts)]
#[instruction(params: SwapParams, price_params: PriceParams)]
pub struct QuoteCrypto<'info> {
    /// 加密货币资产账户，只读权限
    #[account(
        seeds = [b"crypto", crypto_asset.key().as_ref()],
        bump,
        constraint = crypto_asset.asset_type == AssetType::Crypto @ crate::errors::AssetError::InvalidAssetType
    )]
    pub crypto_asset: Account<'info, crate::account_models::asset::Asset>,
    
    /// 操作权限账户（可选），用于权限控制
    #[account(
        constraint = check_authority_permission(&authority.key(), &crypto_asset.authority) @ crate::errors::SecurityError::Unauthorized,
        required = false
    )]
    pub authority: Option<Signer<'info>>,
    
    /// DEX程序，用于获取报价
    pub dex_program: Program<'info, crate::dex::traits::DexAdapterTrait>,
    
    /// 预言机程序，用于价格验证
    pub oracle_program: Program<'info, crate::oracles::traits::OracleAdapterTrait>,
    
    /// 系统程序，用于账户验证
    pub system_program: Program<'info, System>,
    
    /// 时钟账户，用于时间戳记录
    pub clock: Sysvar<'info, Clock>,
}

/// 加密货币报价指令实现
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文，自动校验账户权限与生命周期
/// - `params`: 报价参数，包含输入输出资产、数量等
/// - `price_params`: 价格参数，包含价格源、滑点等
/// 
/// ## 返回值
/// - `anchor_lang::Result<QuoteResult>`: 返回报价结果
/// 
/// ## 安全性
/// - Anchor自动校验账户权限、生命周期、PDA
/// - 业务层校验报价参数合法性
/// - 价格验证和准确性检查
/// - 完整的事件记录和审计追踪
pub fn quote_crypto(
    ctx: Context<QuoteCrypto>,
    params: SwapParams,
    price_params: PriceParams
) -> anchor_lang::Result<QuoteResult> {
    // === 1. 参数校验 ===
    // 校验报价参数合法性
    validate_quote_params(&params, &price_params)?;
    
    // === 2. 权限校验 ===
    // 检查查询权限（如果提供了权限账户）
    if let Some(auth) = &ctx.accounts.authority {
        require!(
            auth.key() == ctx.accounts.crypto_asset.authority || 
            auth.key() == ctx.accounts.crypto_asset.query_authority,
            crate::errors::SecurityError::Unauthorized
        );
    }
    
    // === 3. 业务逻辑执行 ===
    // 创建加密货币服务实例
    let crypto_service = CryptoService::new();
    
    // 获取多DEX报价
    let quote_result = crypto_service.get_quote(
        &ctx.accounts.crypto_asset,
        &params,
        &price_params
    )?;
    
    // === 4. 价格验证 ===
    // 验证报价的准确性
    validate_quote_accuracy(&quote_result, &price_params)?;
    
    // === 5. 事件记录 ===
    // 发出报价事件，记录操作详情
    emit!(AssetQuoted {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        input_asset: params.input_asset,
        output_asset: params.output_asset,
        input_amount: params.input_amount,
        expected_output_amount: params.output_amount,
        quoted_output_amount: quote_result.output_amount,
        price: quote_result.price,
        slippage: quote_result.slippage,
        best_dex: quote_result.best_dex.clone(),
        alternative_dexes: quote_result.alternative_dexes.clone(),
        authority: ctx.accounts.authority.as_ref().map(|auth| auth.key()),
        dex_program: ctx.accounts.dex_program.key(),
        oracle_program: ctx.accounts.oracle_program.key(),
        timestamp: Clock::get()?.unix_timestamp,
        quote_validity_period: quote_result.validity_period,
    });
    
    // === 6. 日志记录 ===
    msg!("Crypto asset quoted successfully: input_amount={}, quoted_output_amount={}, price={}, best_dex={}", 
         params.input_amount, quote_result.output_amount, quote_result.price, quote_result.best_dex);
    
    Ok(quote_result)
}

/// 批量报价加密货币指令
/// 
/// 支持一次性获取多个加密货币的报价，提高操作效率。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `quote_requests`: 报价请求集合
/// 
/// ## 返回值
/// - `anchor_lang::Result<Vec<QuoteResult>>`: 返回报价结果集合
pub fn batch_quote_crypto(
    ctx: Context<QuoteCrypto>,
    quote_requests: Vec<QuoteRequest>
) -> anchor_lang::Result<Vec<QuoteResult>> {
    // === 1. 批量参数校验 ===
    require!(!quote_requests.is_empty(), crate::errors::AssetError::EmptyBatch);
    require!(quote_requests.len() <= 50, crate::errors::AssetError::BatchTooLarge);
    
    // 校验每个报价请求
    for request in &quote_requests {
        validate_quote_params(&request.params, &request.price_params)?;
    }
    
    // === 2. 批量执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行批量报价
    let results = crypto_service.batch_get_quote(
        &ctx.accounts.crypto_asset,
        quote_requests
    )?;
    
    // === 3. 事件记录 ===
    for (i, result) in results.iter().enumerate() {
        let request = &quote_requests[i];
        emit!(AssetQuoted {
            asset_id: ctx.accounts.crypto_asset.key(),
            asset_type: AssetType::Crypto,
            input_asset: request.params.input_asset,
            output_asset: request.params.output_asset,
            input_amount: request.params.input_amount,
            expected_output_amount: request.params.output_amount,
            quoted_output_amount: result.output_amount,
            price: result.price,
            slippage: result.slippage,
            best_dex: result.best_dex.clone(),
            alternative_dexes: result.alternative_dexes.clone(),
            authority: ctx.accounts.authority.as_ref().map(|auth| auth.key()),
            dex_program: ctx.accounts.dex_program.key(),
            oracle_program: ctx.accounts.oracle_program.key(),
            timestamp: Clock::get()?.unix_timestamp,
            quote_validity_period: result.validity_period,
        });
    }
    
    msg!("Batch crypto assets quoted successfully: batch_size={}", results.len());
    
    Ok(results)
}

/// 实时报价加密货币指令
/// 
/// 获取实时报价，包含最新的价格和流动性信息。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `params`: 报价参数
/// - `price_params`: 价格参数
/// 
/// ## 返回值
/// - `anchor_lang::Result<RealTimeQuoteResult>`: 返回实时报价结果
pub fn real_time_quote_crypto(
    ctx: Context<QuoteCrypto>,
    params: SwapParams,
    price_params: PriceParams
) -> anchor_lang::Result<RealTimeQuoteResult> {
    // === 1. 参数校验 ===
    validate_quote_params(&params, &price_params)?;
    
    // === 2. 获取实时报价 ===
    let crypto_service = CryptoService::new();
    
    // 获取实时报价
    let real_time_result = crypto_service.get_real_time_quote(
        &ctx.accounts.crypto_asset,
        &params,
        &price_params
    )?;
    
    // === 3. 事件记录 ===
    emit!(AssetQuoted {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        input_asset: params.input_asset,
        output_asset: params.output_asset,
        input_amount: params.input_amount,
        expected_output_amount: params.output_amount,
        quoted_output_amount: real_time_result.output_amount,
        price: real_time_result.price,
        slippage: real_time_result.slippage,
        best_dex: real_time_result.best_dex.clone(),
        alternative_dexes: real_time_result.alternative_dexes.clone(),
        authority: ctx.accounts.authority.as_ref().map(|auth| auth.key()),
        dex_program: ctx.accounts.dex_program.key(),
        oracle_program: ctx.accounts.oracle_program.key(),
        timestamp: Clock::get()?.unix_timestamp,
        quote_validity_period: real_time_result.validity_period,
    });
    
    msg!("Real-time crypto asset quoted successfully: input_amount={}, quoted_output_amount={}, price={}, best_dex={}", 
         params.input_amount, real_time_result.output_amount, real_time_result.price, real_time_result.best_dex);
    
    Ok(real_time_result)
}

/// 历史报价查询指令
/// 
/// 查询历史报价数据，用于分析和回测。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `params`: 查询参数
/// - `start_time`: 开始时间戳
/// - `end_time`: 结束时间戳
/// 
/// ## 返回值
/// - `anchor_lang::Result<HistoricalQuoteResult>`: 返回历史报价结果
pub fn historical_quote_crypto(
    ctx: Context<QuoteCrypto>,
    params: SwapParams,
    start_time: i64,
    end_time: i64
) -> anchor_lang::Result<HistoricalQuoteResult> {
    // === 1. 参数校验 ===
    require!(start_time < end_time, crate::errors::AssetError::InvalidTimeRange);
    require!(end_time - start_time <= 86400 * 30, crate::errors::AssetError::TimeRangeTooLarge); // 最多30天
    
    // === 2. 获取历史报价 ===
    let crypto_service = CryptoService::new();
    
    // 获取历史报价
    let historical_result = crypto_service.get_historical_quote(
        &ctx.accounts.crypto_asset,
        &params,
        start_time,
        end_time
    )?;
    
    // === 3. 事件记录 ===
    emit!(AssetQuoted {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        input_asset: params.input_asset,
        output_asset: params.output_asset,
        input_amount: params.input_amount,
        expected_output_amount: params.output_amount,
        quoted_output_amount: historical_result.average_output_amount,
        price: historical_result.average_price,
        slippage: historical_result.average_slippage,
        best_dex: "historical".to_string(),
        alternative_dexes: vec![],
        authority: ctx.accounts.authority.as_ref().map(|auth| auth.key()),
        dex_program: ctx.accounts.dex_program.key(),
        oracle_program: ctx.accounts.oracle_program.key(),
        timestamp: Clock::get()?.unix_timestamp,
        quote_validity_period: 0,
    });
    
    msg!("Historical crypto asset quoted successfully: start_time={}, end_time={}, data_points={}", 
         start_time, end_time, historical_result.data_points.len());
    
    Ok(historical_result)
}

/// 报价请求结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct QuoteRequest {
    /// 报价参数
    pub params: SwapParams,
    /// 价格参数
    pub price_params: PriceParams,
    /// 请求优先级
    pub priority: u8,
}

/// 实时报价结果结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct RealTimeQuoteResult {
    /// 输出数量
    pub output_amount: u64,
    /// 价格
    pub price: u64,
    /// 滑点
    pub slippage: u64,
    /// 最佳DEX
    pub best_dex: String,
    /// 替代DEX列表
    pub alternative_dexes: Vec<String>,
    /// 报价有效期
    pub validity_period: i64,
    /// 流动性信息
    pub liquidity_info: LiquidityInfo,
    /// 价格影响
    pub price_impact: u64,
    /// 更新时间戳
    pub updated_at: i64,
}

/// 历史报价结果结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct HistoricalQuoteResult {
    /// 平均输出数量
    pub average_output_amount: u64,
    /// 平均价格
    pub average_price: u64,
    /// 平均滑点
    pub average_slippage: u64,
    /// 最高价格
    pub max_price: u64,
    /// 最低价格
    pub min_price: u64,
    /// 价格波动率
    pub price_volatility: u64,
    /// 数据点列表
    pub data_points: Vec<HistoricalDataPoint>,
    /// 开始时间
    pub start_time: i64,
    /// 结束时间
    pub end_time: i64,
}

/// 历史数据点结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct HistoricalDataPoint {
    /// 时间戳
    pub timestamp: i64,
    /// 价格
    pub price: u64,
    /// 输出数量
    pub output_amount: u64,
    /// 滑点
    pub slippage: u64,
    /// DEX名称
    pub dex_name: String,
}

/// 流动性信息结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct LiquidityInfo {
    /// 总流动性
    pub total_liquidity: u64,
    /// 可用流动性
    pub available_liquidity: u64,
    /// 流动性深度
    pub liquidity_depth: u64,
    /// 流动性提供者数量
    pub lp_count: u64,
}

/// 验证报价准确性
fn validate_quote_accuracy(
    quote_result: &QuoteResult,
    price_params: &PriceParams
) -> anchor_lang::Result<()> {
    // 检查报价是否在合理范围内
    if quote_result.price == 0 {
        return Err(crate::errors::AssetError::InvalidQuote.into());
    }
    
    // 检查滑点是否超过最大限制
    if quote_result.slippage > price_params.max_slippage {
        return Err(crate::errors::AssetError::SlippageTooHigh.into());
    }
    
    Ok(())
} 