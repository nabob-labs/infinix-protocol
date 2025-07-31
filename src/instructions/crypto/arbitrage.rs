//! 加密货币 (Crypto) 套利交易指令
//! 
//! 本模块实现加密货币资产的套利交易功能，支持多DEX套利、跨市场套利、统计套利等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 多DEX套利：在不同DEX间寻找价格差异
//! - 跨市场套利：在不同市场间寻找套利机会
//! - 统计套利：基于统计模型的套利策略
//! - 风险控制：套利风险监控和管理
//! - 事件记录：完整的审计追踪

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, TradeParams, ExecutionParams, OracleParams, ArbitrageParams};
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetArbitraged;
use crate::validation::business::validate_arbitrage_params;
use crate::core::security::check_authority_permission;
use crate::dex::traits::DexAdapterTrait;
use crate::oracles::traits::OracleAdapterTrait;

/// 加密货币套利交易参数结构体
/// 
/// 定义套利交易所需的所有参数，包括：
/// - arbitrage_params: 套利参数
/// - trade_params: 交易参数
/// - exec_params: 执行参数（可选）
/// - oracle_params: 预言机参数（可选）
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ArbitrageCryptoParams {
    /// 套利参数
    pub arbitrage_params: ArbitrageParams,
    /// 交易参数
    pub trade_params: TradeParams,
    /// 执行参数（可选）
    pub exec_params: Option<ExecutionParams>,
    /// 预言机参数（可选）
    pub oracle_params: Option<OracleParams>,
}

/// 加密货币套利交易指令账户上下文
/// 
/// 定义套利交易操作所需的所有账户，包括：
/// - crypto_asset: 加密货币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - dex_program: DEX程序（用于交易执行）
/// - oracle_program: 预言机程序（用于价格验证）
/// - system_program: 系统程序
/// - clock: 时钟程序
#[derive(Accounts)]
#[instruction(params: ArbitrageCryptoParams)]
pub struct ArbitrageCrypto<'info> {
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

/// 加密货币套利交易指令实现
/// 
/// 执行套利交易操作，包括：
/// - 参数验证和权限检查
/// - 套利机会检测
/// - 套利执行和风险控制
/// - 事件记录和状态更新
pub fn arbitrage_crypto(
    ctx: Context<ArbitrageCrypto>,
    params: ArbitrageCryptoParams
) -> anchor_lang::Result<()> {
    let crypto_asset = &mut ctx.accounts.crypto_asset;
    
    // 1. 参数验证
    validate_arbitrage_params(&params.arbitrage_params)?;
    require!(!params.trade_params.trade_type.is_empty(), crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &crypto_asset.authority,
        &ctx.accounts.authority.key(),
        "arbitrage"
    )?;
    
    // 3. 调用服务层执行套利交易
    let crypto_service = CryptoService::new();
    crypto_service.arbitrage(
        crypto_asset,
        &params.arbitrage_params,
        &params.trade_params,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射套利交易事件
    emit!(AssetArbitraged {
        asset_id: crypto_asset.id,
        arbitrage_type: params.arbitrage_params.arbitrage_type.clone(),
        profit: params.arbitrage_params.min_profit,
        trade_params: params.trade_params.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Crypto arbitrage executed successfully: asset_id={}, arbitrage_type={}, min_profit={}", 
         crypto_asset.id, params.arbitrage_params.arbitrage_type, params.arbitrage_params.min_profit);
    
    Ok(())
}

/// 批量套利交易指令实现
/// 
/// 执行批量套利交易操作，支持多个套利机会的并行执行
pub fn batch_arbitrage_crypto(
    ctx: Context<ArbitrageCrypto>,
    params: Vec<ArbitrageCryptoParams>
) -> anchor_lang::Result<()> {
    let crypto_asset = &mut ctx.accounts.crypto_asset;
    
    // 1. 批量参数验证
    require!(!params.is_empty(), crate::errors::asset_error::AssetError::InvalidParams);
    require!(params.len() <= 10, crate::errors::asset_error::AssetError::BatchTooLarge);
    
    for param in &params {
        validate_arbitrage_params(&param.arbitrage_params)?;
        require!(!param.trade_params.trade_type.is_empty(), crate::errors::asset_error::AssetError::InvalidParams);
    }
    
    // 2. 权限检查
    check_authority_permission(
        &crypto_asset.authority,
        &ctx.accounts.authority.key(),
        "batch_arbitrage"
    )?;
    
    // 3. 批量执行套利交易
    let crypto_service = CryptoService::new();
    for param in params {
        crypto_service.arbitrage(
            crypto_asset,
            &param.arbitrage_params,
            &param.trade_params,
            ctx.accounts.authority.key()
        )?;
        
        // 发射批量套利交易事件
        emit!(AssetArbitraged {
            asset_id: crypto_asset.id,
            arbitrage_type: param.arbitrage_params.arbitrage_type.clone(),
            profit: param.arbitrage_params.min_profit,
            trade_params: param.trade_params.clone(),
            authority: ctx.accounts.authority.key(),
            timestamp: ctx.accounts.clock.unix_timestamp,
        });
    }
    
    msg!("[INFO] Crypto batch arbitrage executed successfully: asset_id={}, count={}", 
         crypto_asset.id, params.len());
    
    Ok(())
}

/// 多DEX套利交易指令实现
/// 
/// 执行多DEX套利交易，在不同DEX间寻找价格差异
pub fn multi_dex_arbitrage_crypto(
    ctx: Context<ArbitrageCrypto>,
    params: ArbitrageCryptoParams,
    dex_list: Vec<String>
) -> anchor_lang::Result<()> {
    let crypto_asset = &mut ctx.accounts.crypto_asset;
    
    // 1. 参数验证
    validate_arbitrage_params(&params.arbitrage_params)?;
    require!(!dex_list.is_empty(), crate::errors::asset_error::AssetError::InvalidParams);
    require!(dex_list.len() >= 2, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &crypto_asset.authority,
        &ctx.accounts.authority.key(),
        "multi_dex_arbitrage"
    )?;
    
    // 3. 调用服务层执行多DEX套利交易
    let crypto_service = CryptoService::new();
    crypto_service.multi_dex_arbitrage(
        crypto_asset,
        &params.arbitrage_params,
        &params.trade_params,
        &dex_list,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射多DEX套利交易事件
    emit!(AssetArbitraged {
        asset_id: crypto_asset.id,
        arbitrage_type: format!("{}_multi_dex", params.arbitrage_params.arbitrage_type),
        profit: params.arbitrage_params.min_profit,
        trade_params: params.trade_params.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Crypto multi-DEX arbitrage executed successfully: asset_id={}, arbitrage_type={}, dex_count={}", 
         crypto_asset.id, params.arbitrage_params.arbitrage_type, dex_list.len());
    
    Ok(())
}

/// 跨市场套利交易指令实现
/// 
/// 执行跨市场套利交易，在不同市场间寻找套利机会
pub fn cross_market_arbitrage_crypto(
    ctx: Context<ArbitrageCrypto>,
    params: ArbitrageCryptoParams,
    market_list: Vec<String>
) -> anchor_lang::Result<()> {
    let crypto_asset = &mut ctx.accounts.crypto_asset;
    
    // 1. 参数验证
    validate_arbitrage_params(&params.arbitrage_params)?;
    require!(!market_list.is_empty(), crate::errors::asset_error::AssetError::InvalidParams);
    require!(market_list.len() >= 2, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &crypto_asset.authority,
        &ctx.accounts.authority.key(),
        "cross_market_arbitrage"
    )?;
    
    // 3. 调用服务层执行跨市场套利交易
    let crypto_service = CryptoService::new();
    crypto_service.cross_market_arbitrage(
        crypto_asset,
        &params.arbitrage_params,
        &params.trade_params,
        &market_list,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射跨市场套利交易事件
    emit!(AssetArbitraged {
        asset_id: crypto_asset.id,
        arbitrage_type: format!("{}_cross_market", params.arbitrage_params.arbitrage_type),
        profit: params.arbitrage_params.min_profit,
        trade_params: params.trade_params.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Crypto cross-market arbitrage executed successfully: asset_id={}, arbitrage_type={}, market_count={}", 
         crypto_asset.id, params.arbitrage_params.arbitrage_type, market_list.len());
    
    Ok(())
}

/// 统计套利交易指令实现
/// 
/// 执行统计套利交易，基于统计模型的套利策略
pub fn statistical_arbitrage_crypto(
    ctx: Context<ArbitrageCrypto>,
    params: ArbitrageCryptoParams,
    stat_params: crate::core::types::StatisticalArbitrageParams
) -> anchor_lang::Result<()> {
    let crypto_asset = &mut ctx.accounts.crypto_asset;
    
    // 1. 参数验证
    validate_arbitrage_params(&params.arbitrage_params)?;
    require!(stat_params.confidence_level > 0.0, crate::errors::asset_error::AssetError::InvalidParams);
    require!(stat_params.confidence_level <= 1.0, crate::errors::asset_error::AssetError::InvalidParams);
    
    // 2. 权限检查
    check_authority_permission(
        &crypto_asset.authority,
        &ctx.accounts.authority.key(),
        "statistical_arbitrage"
    )?;
    
    // 3. 调用服务层执行统计套利交易
    let crypto_service = CryptoService::new();
    crypto_service.statistical_arbitrage(
        crypto_asset,
        &params.arbitrage_params,
        &params.trade_params,
        &stat_params,
        ctx.accounts.authority.key()
    )?;
    
    // 4. 发射统计套利交易事件
    emit!(AssetArbitraged {
        asset_id: crypto_asset.id,
        arbitrage_type: format!("{}_statistical", params.arbitrage_params.arbitrage_type),
        profit: params.arbitrage_params.min_profit,
        trade_params: params.trade_params.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Crypto statistical arbitrage executed successfully: asset_id={}, arbitrage_type={}, confidence_level={}", 
         crypto_asset.id, params.arbitrage_params.arbitrage_type, stat_params.confidence_level);
    
    Ok(())
}

/// 套利机会检测指令实现
/// 
/// 检测套利机会但不执行交易
pub fn detect_arbitrage_opportunities_crypto(
    ctx: Context<ArbitrageCrypto>,
    params: ArbitrageCryptoParams
) -> anchor_lang::Result<()> {
    let crypto_asset = &mut ctx.accounts.crypto_asset;
    
    // 1. 参数验证
    validate_arbitrage_params(&params.arbitrage_params)?;
    
    // 2. 权限检查
    check_authority_permission(
        &crypto_asset.authority,
        &ctx.accounts.authority.key(),
        "detect_arbitrage_opportunities"
    )?;
    
    // 3. 调用服务层检测套利机会
    let crypto_service = CryptoService::new();
    let opportunities = crypto_service.detect_arbitrage_opportunities(
        crypto_asset,
        &params.arbitrage_params,
        &params.trade_params
    )?;
    
    // 4. 发射套利机会检测事件
    emit!(AssetArbitraged {
        asset_id: crypto_asset.id,
        arbitrage_type: format!("{}_detection", params.arbitrage_params.arbitrage_type),
        profit: opportunities.len() as u64,
        trade_params: params.trade_params.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });
    
    msg!("[INFO] Crypto arbitrage opportunities detected: asset_id={}, arbitrage_type={}, opportunities_count={}", 
         crypto_asset.id, params.arbitrage_params.arbitrage_type, opportunities.len());
    
    Ok(())
} 