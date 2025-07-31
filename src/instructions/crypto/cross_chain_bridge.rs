//! 加密货币 (Crypto) 跨链桥接指令
//! 
//! 本模块实现加密货币资产的跨链桥接功能，支持多链资产转移、桥接验证、跨链交易等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 多链支持：支持主流区块链网络
//! - 桥接验证：安全的跨链验证机制
//! - 跨链交易：无缝的跨链交易体验
//! - 事件记录：完整的审计追踪
//! - 状态管理：原子性操作保证数据一致性

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, ExecutionParams, StrategyParams, CrossChainParams};
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetCrossChainBridged;
use crate::validation::business::validate_cross_chain_params;
use crate::core::security::check_authority_permission;

/// 加密货币跨链桥接指令账户上下文
/// 
/// 定义跨链桥接操作所需的所有账户，包括：
/// - crypto_asset: 加密货币资产账户（可变）
/// - authority: 操作权限账户（签名者）
/// - bridge_program: 桥接程序（用于跨链操作）
/// - oracle_program: 预言机程序（用于价格数据）
/// - system_program: 系统程序（用于账户管理）
#[derive(Accounts)]
#[instruction(params: CrossChainParams)]
pub struct CrossChainBridgeCrypto<'info> {
    /// 加密货币资产账户，需要可变权限以更新状态
    #[account(
        mut,
        seeds = [b"crypto", crypto_asset.key().as_ref()],
        bump,
        constraint = crypto_asset.asset_type == AssetType::Crypto @ crate::errors::AssetError::InvalidAssetType,
        constraint = crypto_asset.balance >= params.bridge_amount @ crate::errors::AssetError::InsufficientBalance
    )]
    pub crypto_asset: Account<'info, crate::account_models::asset::Asset>,
    
    /// 操作权限账户，必须是签名者
    #[account(
        constraint = check_authority_permission(&authority.key(), &crypto_asset.authority) @ crate::errors::SecurityError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    /// 桥接程序，用于跨链操作
    pub bridge_program: Program<'info, crate::core::types::BridgeAdapterTrait>,
    
    /// 预言机程序，用于价格数据
    pub oracle_program: Program<'info, crate::oracles::traits::OracleAdapterTrait>,
    
    /// 系统程序，用于账户管理
    pub system_program: Program<'info, System>,
    
    /// 时钟账户，用于时间戳记录
    pub clock: Sysvar<'info, Clock>,
}

/// 加密货币跨链桥接指令实现
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文，自动校验账户权限与生命周期
/// - `params`: 跨链参数，包含目标链、数量等
/// - `exec_params`: 可选算法执行参数，支持算法热插拔
/// - `strategy_params`: 可选策略参数，支持多策略扩展
/// 
/// ## 返回值
/// - `anchor_lang::Result<()>`: Anchor标准返回类型，表示指令执行成功或失败
/// 
/// ## 安全性
/// - Anchor自动校验账户权限、生命周期、PDA
/// - 业务层校验跨链参数合法性
/// - 桥接验证机制
/// - 完整的事件记录和审计追踪
pub fn cross_chain_bridge_crypto(
    ctx: Context<CrossChainBridgeCrypto>,
    params: CrossChainParams,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    // 校验跨链参数合法性
    validate_cross_chain_params(&params)?;
    
    // 检查资产余额充足性
    require!(
        ctx.accounts.crypto_asset.balance >= params.bridge_amount,
        crate::errors::AssetError::InsufficientBalance
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
    
    // 记录跨链前的状态
    let balance_before = ctx.accounts.crypto_asset.balance;
    
    // 执行跨链桥接操作
    let bridge_result = crypto_service.cross_chain_bridge(
        &mut ctx.accounts.crypto_asset,
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
    // 发出跨链桥接事件，记录操作详情
    emit!(AssetCrossChainBridged {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        source_chain: params.source_chain.clone(),
        target_chain: params.target_chain.clone(),
        bridge_amount: params.bridge_amount,
        bridge_fee: params.bridge_fee,
        balance_before,
        balance_after: ctx.accounts.crypto_asset.balance,
        bridge_id: bridge_result.bridge_id,
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: exec_params.map(|p| p.into()),
        strategy_params: strategy_params.map(|s| s.into()),
    });
    
    // === 7. 日志记录 ===
    msg!("Crypto cross-chain bridge executed successfully: source_chain={}, target_chain={}, bridge_amount={}, bridge_fee={}, authority={}", 
         params.source_chain, params.target_chain, params.bridge_amount, params.bridge_fee, ctx.accounts.authority.key());
    
    Ok(())
}

/// 跨链桥接验证指令
/// 
/// 验证跨链桥接操作的完成状态。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `bridge_id`: 桥接ID
/// - `verification_params`: 验证参数
pub fn verify_cross_chain_bridge_crypto(
    ctx: Context<CrossChainBridgeCrypto>,
    bridge_id: String,
    verification_params: BridgeVerification
) -> anchor_lang::Result<()> {
    // === 1. 参数校验 ===
    require!(!bridge_id.is_empty(), crate::errors::AssetError::InvalidBridgeId);
    
    // === 2. 桥接验证执行 ===
    let crypto_service = CryptoService::new();
    
    // 执行桥接验证
    let verification_result = crypto_service.verify_cross_chain_bridge(
        &mut ctx.accounts.crypto_asset,
        &bridge_id,
        &verification_params
    )?;
    
    // === 3. 事件记录 ===
    emit!(AssetCrossChainBridged {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        source_chain: "verification".to_string(),
        target_chain: "verification".to_string(),
        bridge_amount: 0,
        bridge_fee: 0,
        balance_before: 0,
        balance_after: ctx.accounts.crypto_asset.balance,
        bridge_id: bridge_id.clone(),
        authority: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
        exec_params: None,
        strategy_params: None,
    });
    
    msg!("Crypto cross-chain bridge verification completed: bridge_id={}, verification_status={}, authority={}", 
         bridge_id, verification_result.status, ctx.accounts.authority.key());
    
    Ok(())
}

/// 跨链桥接参数结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct CrossChainParams {
    /// 源链
    pub source_chain: String,
    /// 目标链
    pub target_chain: String,
    /// 桥接数量
    pub bridge_amount: u64,
    /// 桥接费用
    pub bridge_fee: u64,
    /// 目标地址
    pub target_address: String,
    /// 桥接类型
    pub bridge_type: BridgeType,
}

/// 桥接验证结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct BridgeVerification {
    /// 验证类型
    pub verification_type: String,
    /// 验证参数
    pub verification_params: String,
    /// 验证时间
    pub verification_time: i64,
}

/// 桥接类型枚举
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub enum BridgeType {
    /// Wormhole桥接
    Wormhole,
    /// Allbridge桥接
    Allbridge,
    /// 自定义桥接
    Custom,
}

/// 桥接结果结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct BridgeResult {
    /// 桥接ID
    pub bridge_id: String,
    /// 桥接状态
    pub status: String,
    /// 桥接时间
    pub bridge_time: i64,
    /// 桥接费用
    pub bridge_fee: u64,
} 