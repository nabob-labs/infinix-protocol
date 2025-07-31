//! 加密货币 (Crypto) 算法交易指令
//! 
//! 本模块实现加密货币资产的算法交易功能，支持多种算法类型、智能路由、优化执行等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 算法交易：支持多种交易算法的执行
//! - 智能路由：自动选择最优交易路径
//! - 优化执行：TWAP、VWAP等执行算法
//! - 性能监控：算法执行性能统计
//! - 事件记录：完整的审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, AlgoParams, TradeParams, ExecutionParams, OracleParams};
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetAlgoTraded;
use crate::validation::business::validate_algo_params;
use crate::core::security::check_authority_permission;
use crate::dex::traits::DexAdapterTrait;
use crate::oracles::traits::OracleAdapterTrait;
use crate::algorithms::traits::ExecutionStrategy;

/// 加密货币算法交易参数结构体
/// 
/// 定义算法交易所需的所有参数，包括：
/// - algo_params: 算法参数
/// - trade_params: 交易参数
/// - exec_params: 执行参数（可选）
/// - oracle_params: 预言机参数（可选）
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AlgoTradeCryptoParams {
    /// 算法参数
    pub algo_params: AlgoParams,
    /// 交易参数
    pub trade_params: TradeParams,
    /// 执行参数（可选）
    pub exec_params: Option<ExecutionParams>,
    /// 预言机参数（可选）
    pub oracle_params: Option<OracleParams>,
}

/// 加密货币算法交易指令账户上下文
/// 
/// 定义算法交易操作所需的所有账户，包括：
/// - crypto_asset: 加密货币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - dex_program: DEX程序（用于交易执行）
/// - oracle_program: 预言机程序（用于价格验证）
/// - system_program: 系统程序
/// - clock: 时钟程序
#[derive(Accounts)]
#[instruction(params: AlgoTradeCryptoParams)]
pub struct AlgoTradeCrypto<'info> {
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

/// 加密货币算法交易指令实现
/// 
/// 执行算法交易操作，包括：
/// - 参数验证和权限检查
/// - 算法执行和智能路由
/// - 性能监控和优化
/// - 事件记录和状态更新
pub fn algo_trade_crypto(
    ctx: Context<AlgoTradeCrypto>,
    params: AlgoTradeCryptoParams
) -> anchor_lang::Result<()> {
    let crypto_asset = &mut ctx.accounts.crypto_asset;
    
    // 1. 参数验证
    validate_algo_params(&params.algo_params)?;
    require!(!params.trade_params.trade_type.is_empty(), crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &crypto_asset.authority,
        &ctx.accounts.authority.key(),
        "algo_trade"
    )?;
    
    // 3. 调用服务层执行算法交易
    let crypto_service = CryptoService::new();
    crypto_service.algo_trade(
        crypto_asset,
        &params.algo_params,
        &params.trade_params,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射算法交易事件
    emit!(AssetAlgoTraded {
        asset_id: crypto_asset.id,
        algo_name: params.algo_params.algo_name.clone(),
        params: params.algo_params.params.clone(),
        trade_params: params.trade_params.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Crypto algo trade executed successfully: asset_id={}, algo={}, trade_type={}", 
         crypto_asset.id, params.algo_params.algo_name, params.trade_params.trade_type);
    
    Ok(())
}

/// 批量算法交易指令实现
/// 
/// 执行批量算法交易操作，支持多个算法的并行执行
pub fn batch_algo_trade_crypto(
    ctx: Context<AlgoTradeCrypto>,
    params: Vec<AlgoTradeCryptoParams>
) -> anchor_lang::Result<()> {
    let crypto_asset = &mut ctx.accounts.crypto_asset;
    
    // 1. 批量参数验证
    require!(!params.is_empty(), crate::errors::asset_error::AssetError::InvalidParams);
    require!(params.len() <= 10, crate::errors::asset_error::AssetError::BatchTooLarge);
    
    for param in &params {
        validate_algo_params(&param.algo_params)?;
        require!(!param.trade_params.trade_type.is_empty(), crate::errors::asset_error::AssetError::InvalidParams);
    }
    
    // 2. 权限检查
    check_authority_permission(
        &crypto_asset.authority,
        &ctx.accounts.authority.key(),
        "batch_algo_trade"
    )?;
    
    // 3. 批量执行算法交易
    let crypto_service = CryptoService::new();
    for param in params {
        crypto_service.algo_trade(
            crypto_asset,
            &param.algo_params,
            &param.trade_params,
            ctx.accounts.authority.key()
        )?;
        
        // 发射批量算法交易事件
        emit!(AssetAlgoTraded {
            asset_id: crypto_asset.id,
            algo_name: param.algo_params.algo_name.clone(),
            params: param.algo_params.params.clone(),
            trade_params: param.trade_params.clone(),
            authority: ctx.accounts.authority.key(),
            timestamp: ctx.accounts.clock.unix_timestamp,
        });
    }
    
    msg!("[INFO] Crypto batch algo trade executed successfully: asset_id={}, count={}", 
         crypto_asset.id, params.len());
    
    Ok(())
}

/// TWAP算法交易指令实现
/// 
/// 执行时间加权平均价格算法交易
pub fn twap_algo_trade_crypto(
    ctx: Context<AlgoTradeCrypto>,
    params: AlgoTradeCryptoParams,
    twap_params: crate::algorithms::twap::TwapParams
) -> anchor_lang::Result<()> {
    let crypto_asset = &mut ctx.accounts.crypto_asset;
    
    // 1. 参数验证
    validate_algo_params(&params.algo_params)?;
    require!(twap_params.duration > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(twap_params.intervals > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &crypto_asset.authority,
        &ctx.accounts.authority.key(),
        "twap_algo_trade"
    )?;
    
    // 3. 调用服务层执行TWAP算法交易
    let crypto_service = CryptoService::new();
    crypto_service.twap_algo_trade(
        crypto_asset,
        &params.algo_params,
        &params.trade_params,
        &twap_params,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射TWAP算法交易事件
    emit!(AssetAlgoTraded {
        asset_id: crypto_asset.id,
        algo_name: format!("{}_twap", params.algo_params.algo_name),
        params: params.algo_params.params.clone(),
        trade_params: params.trade_params.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Crypto TWAP algo trade executed successfully: asset_id={}, algo={}, duration={}, intervals={}", 
         crypto_asset.id, params.algo_params.algo_name, twap_params.duration, twap_params.intervals);
    
    Ok(())
}

/// VWAP算法交易指令实现
/// 
/// 执行成交量加权平均价格算法交易
pub fn vwap_algo_trade_crypto(
    ctx: Context<AlgoTradeCrypto>,
    params: AlgoTradeCryptoParams,
    vwap_params: crate::algorithms::vwap::VwapParams
) -> anchor_lang::Result<()> {
    let crypto_asset = &mut ctx.accounts.crypto_asset;
    
    // 1. 参数验证
    validate_algo_params(&params.algo_params)?;
    require!(vwap_params.target_volume > 0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(vwap_params.max_slippage_bps > 0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &crypto_asset.authority,
        &ctx.accounts.authority.key(),
        "vwap_algo_trade"
    )?;
    
    // 3. 调用服务层执行VWAP算法交易
    let crypto_service = CryptoService::new();
    crypto_service.vwap_algo_trade(
        crypto_asset,
        &params.algo_params,
        &params.trade_params,
        &vwap_params,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射VWAP算法交易事件
    emit!(AssetAlgoTraded {
        asset_id: crypto_asset.id,
        algo_name: format!("{}_vwap", params.algo_params.algo_name),
        params: params.algo_params.params.clone(),
        trade_params: params.trade_params.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Crypto VWAP algo trade executed successfully: asset_id={}, algo={}, target_volume={}, max_slippage_bps={}", 
         crypto_asset.id, params.algo_params.algo_name, vwap_params.target_volume, vwap_params.max_slippage_bps);
    
    Ok(())
}

/// 智能路由算法交易指令实现
/// 
/// 执行智能路由算法交易
pub fn smart_routing_algo_trade_crypto(
    ctx: Context<AlgoTradeCrypto>,
    params: AlgoTradeCryptoParams,
    routing_params: crate::algorithms::smart_routing::RoutingParams
) -> anchor_lang::Result<()> {
    let crypto_asset = &mut ctx.accounts.crypto_asset;
    
    // 1. 参数验证
    validate_algo_params(&params.algo_params)?;
    require!(!routing_params.dex_list.is_empty(), crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &crypto_asset.authority,
        &ctx.accounts.authority.key(),
        "smart_routing_algo_trade"
    )?;
    
    // 3. 调用服务层执行智能路由算法交易
    let crypto_service = CryptoService::new();
    crypto_service.smart_routing_algo_trade(
        crypto_asset,
        &params.algo_params,
        &params.trade_params,
        &routing_params,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射智能路由算法交易事件
    emit!(AssetAlgoTraded {
        asset_id: crypto_asset.id,
        algo_name: format!("{}_smart_routing", params.algo_params.algo_name),
        params: params.algo_params.params.clone(),
        trade_params: params.trade_params.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Crypto smart routing algo trade executed successfully: asset_id={}, algo={}, dex_count={}", 
         crypto_asset.id, params.algo_params.algo_name, routing_params.dex_list.len());
    
    Ok(())
} 