//! 加密货币 (Crypto) 策略交易指令
//! 
//! 本模块实现加密货币资产的策略交易功能，支持多种交易策略、算法集成、风险管理等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 策略交易：支持多种交易策略的执行
//! - 算法集成：集成高级交易算法
//! - 风险管理：内置风险控制和监控
//! - 性能优化：批量处理和并行执行
//! - 事件记录：完整的审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, StrategyParams, TradeParams, OracleParams, ExecutionParams};
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetStrategyTraded;
use crate::validation::business::validate_strategy_params;
use crate::core::security::check_authority_permission;
use crate::dex::traits::DexAdapterTrait;
use crate::oracles::traits::OracleAdapterTrait;

/// 加密货币策略交易参数结构体
/// 
/// 定义策略交易所需的所有参数，包括：
/// - strategy: 策略参数
/// - swap_params: 交换参数（可选）
/// - price_params: 价格参数（可选）
/// - exec_params: 执行参数（可选）
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StrategyTradeCryptoParams {
    /// 策略参数
    pub strategy: StrategyParams,
    /// 交换参数（可选）
    pub swap_params: Option<TradeParams>,
    /// 价格参数（可选）
    pub price_params: Option<OracleParams>,
    /// 执行参数（可选）
    pub exec_params: Option<ExecutionParams>,
}

/// 加密货币策略交易指令账户上下文
/// 
/// 定义策略交易操作所需的所有账户，包括：
/// - crypto_asset: 加密货币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - dex_program: DEX程序（用于交易执行）
/// - oracle_program: 预言机程序（用于价格验证）
/// - system_program: 系统程序
/// - clock: 时钟程序
#[derive(Accounts)]
#[instruction(params: StrategyTradeCryptoParams)]
pub struct StrategyTradeCrypto<'info> {
    /// 加密货币资产账户，需要可变权限以更新状态
    #[account(
        mut,
        seeds = [b"crypto", crypto_asset.key().as_ref()],
        bump,
        constraint = crypto_asset.asset_type == AssetType::Crypto @ crate::errors::asset_error::AssetError::InvalidAssetType,
        constraint = !crypto_asset.is_frozen @ crate::errors::asset_error::AssetError::AssetFrozen
    )]
    pub crypto_asset: Account<'info, crate::state::baskets::BasketIndexState>,
    
    /// 操作权限账户，必须是签名者
    #[account(
        constraint = authority.key() == crypto_asset.authority @ crate::errors::security_error::SecurityError::InvalidAuthority
    )]
    pub authority: Signer<'info>,
    
    /// DEX程序，用于交易执行
    /// CHECK: 由DEX适配器验证
    pub dex_program: UncheckedAccount<'info>,
    
    /// 预言机程序，用于价格验证
    /// CHECK: 由预言机适配器验证
    pub oracle_program: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 时钟程序
    pub clock: Sysvar<'info, Clock>,
}

/// 加密货币策略交易指令实现
/// 
/// 执行策略交易操作，包括：
/// - 参数验证和权限检查
/// - 策略执行和算法集成
/// - 风险控制和监控
/// - 事件记录和状态更新
pub fn strategy_trade_crypto(
    ctx: Context<StrategyTradeCrypto>,
    params: StrategyTradeCryptoParams
) -> anchor_lang::Result<()> {
    let crypto_asset = &mut ctx.accounts.crypto_asset;
    
    // 1. 参数验证
    validate_strategy_params(&params.strategy)?;
    
    // 2. 权限检查
    check_authority_permission(
        &crypto_asset.authority,
        &ctx.accounts.authority.key(),
        "strategy_trade"
    )?;
    
    // 3. 调用服务层执行策略交易
    let crypto_service = CryptoService::new();
    crypto_service.strategy_trade(
        crypto_asset,
        &params,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射策略交易事件
    emit!(AssetStrategyTraded {
        asset_id: crypto_asset.id,
        strategy: params.strategy.strategy_name.clone(),
        params: params.strategy.params.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Crypto strategy trade executed successfully: asset_id={}, strategy={}", 
         crypto_asset.id, params.strategy.strategy_name);
    
    Ok(())
}

/// 批量策略交易指令实现
/// 
/// 执行批量策略交易操作，支持多个策略的并行执行
pub fn batch_strategy_trade_crypto(
    ctx: Context<StrategyTradeCrypto>,
    params: Vec<StrategyTradeCryptoParams>
) -> anchor_lang::Result<()> {
    let crypto_asset = &mut ctx.accounts.crypto_asset;
    
    // 1. 批量参数验证
    require!(!params.is_empty(), crate::errors::asset_error::AssetError::InvalidParams);
    require!(params.len() <= 10, crate::errors::asset_error::AssetError::BatchTooLarge);
    
    for param in &params {
        validate_strategy_params(&param.strategy)?;
    }
    
    // 2. 权限检查
    check_authority_permission(
        &crypto_asset.authority,
        &ctx.accounts.authority.key(),
        "batch_strategy_trade"
    )?;
    
    // 3. 批量执行策略交易
    let crypto_service = CryptoService::new();
    for param in params {
        crypto_service.strategy_trade(
            crypto_asset,
            &param,
            ctx.accounts.authority.key()
        )?;
        
        // 发射批量策略交易事件
        emit!(AssetStrategyTraded {
            asset_id: crypto_asset.id,
            strategy: param.strategy.strategy_name.clone(),
            params: param.strategy.params.clone(),
            authority: ctx.accounts.authority.key(),
            timestamp: ctx.accounts.clock.unix_timestamp,
        });
    }
    
    msg!("[INFO] Crypto batch strategy trade executed successfully: asset_id={}, count={}", 
         crypto_asset.id, params.len());
    
    Ok(())
}

/// 算法策略交易指令实现
/// 
/// 执行算法驱动的策略交易操作
pub fn algo_strategy_trade_crypto(
    ctx: Context<StrategyTradeCrypto>,
    params: StrategyTradeCryptoParams,
    algo_params: crate::core::types::AlgoParams
) -> anchor_lang::Result<()> {
    let crypto_asset = &mut ctx.accounts.crypto_asset;
    
    // 1. 参数验证
    validate_strategy_params(&params.strategy)?;
    require!(!algo_params.algo_name.is_empty(), crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &crypto_asset.authority,
        &ctx.accounts.authority.key(),
        "algo_strategy_trade"
    )?;
    
    // 3. 调用服务层执行算法策略交易
    let crypto_service = CryptoService::new();
    crypto_service.algo_strategy_trade(
        crypto_asset,
        &params,
        &algo_params,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射算法策略交易事件
    emit!(AssetStrategyTraded {
        asset_id: crypto_asset.id,
        strategy: format!("{}_algo", params.strategy.strategy_name),
        params: params.strategy.params.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Crypto algo strategy trade executed successfully: asset_id={}, strategy={}, algo={}", 
         crypto_asset.id, params.strategy.strategy_name, algo_params.algo_name);
    
    Ok(())
}

/// 风险控制策略交易指令实现
/// 
/// 执行带风险控制的策略交易操作
pub fn risk_controlled_strategy_trade_crypto(
    ctx: Context<StrategyTradeCrypto>,
    params: StrategyTradeCryptoParams,
    risk_params: crate::core::types::RiskParams
) -> anchor_lang::Result<()> {
    let crypto_asset = &mut ctx.accounts.crypto_asset;
    
    // 1. 参数验证
    validate_strategy_params(&params.strategy)?;
    require!(risk_params.max_loss_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &crypto_asset.authority,
        &ctx.accounts.authority.key(),
        "risk_controlled_strategy_trade"
    )?;
    
    // 3. 风险检查
    let crypto_service = CryptoService::new();
    crypto_service.check_risk_limits(crypto_asset, &risk_params)?;
    
    // 4. 调用服务层执行风险控制策略交易
    crypto_service.risk_controlled_strategy_trade(
        crypto_asset,
        &params,
        &risk_params,
        ctx.accounts.authority.key()
    )?;
    
    // 5. 发射风险控制策略交易事件
    emit!(AssetStrategyTraded {
        asset_id: crypto_asset.id,
        strategy: format!("{}_risk_controlled", params.strategy.strategy_name),
        params: params.strategy.params.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Crypto risk controlled strategy trade executed successfully: asset_id={}, strategy={}, max_loss_bps={}", 
         crypto_asset.id, params.strategy.strategy_name, risk_params.max_loss_bps);
    
    Ok(())
} 