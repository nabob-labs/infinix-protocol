//! 加密货币 (Crypto) 查询指令
//! 
//! 本模块实现加密货币资产的查询功能，支持余额查询、状态查询、历史记录查询等。
//! 严格遵循最小功能单元原则，确保每个函数职责单一。
//! 
//! ## 功能特性
//! - 余额查询：查询当前余额和可用余额
//! - 状态查询：查询账户状态和权限信息
//! - 历史记录查询：查询交易历史和操作记录
//! - 统计信息查询：查询统计数据和性能指标
//! - 权限验证：确保查询权限合法

use anchor_lang::prelude::*;
use crate::core::types::{AssetType, QueryParams, QueryResult};
use crate::services::crypto_service::CryptoService;
use crate::events::asset_event::AssetQueried;
use crate::validation::business::validate_query_params;
use crate::core::security::check_query_permission;

/// 加密货币查询指令账户上下文
/// 
/// 定义查询操作所需的所有账户，包括：
/// - crypto_asset: 加密货币资产账户（只读）
/// - authority: 查询权限账户（签名者，可选）
/// - system_program: 系统程序（用于账户验证）
#[derive(Accounts)]
#[instruction(params: QueryParams)]
pub struct QueryCrypto<'info> {
    /// 加密货币资产账户，只读权限
    #[account(
        seeds = [b"crypto", crypto_asset.key().as_ref()],
        bump,
        constraint = crypto_asset.asset_type == AssetType::Crypto @ crate::errors::AssetError::InvalidAssetType
    )]
    pub crypto_asset: Account<'info, crate::account_models::asset::Asset>,
    
    /// 查询权限账户（可选），用于权限控制
    #[account(
        constraint = check_query_permission(&authority.key(), &crypto_asset.authority) @ crate::errors::SecurityError::Unauthorized,
        required = false
    )]
    pub authority: Option<Signer<'info>>,
    
    /// 系统程序，用于账户验证
    pub system_program: Program<'info, System>,
    
    /// 时钟账户，用于时间戳记录
    pub clock: Sysvar<'info, Clock>,
}

/// 加密货币查询指令实现
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文，自动校验账户权限与生命周期
/// - `params`: 查询参数，包含查询类型和过滤条件
/// 
/// ## 返回值
/// - `anchor_lang::Result<QueryResult>`: 返回查询结果
/// 
/// ## 安全性
/// - Anchor自动校验账户权限、生命周期、PDA
/// - 业务层校验查询参数合法性
/// - 权限验证机制
/// - 完整的事件记录和审计追踪
pub fn query_crypto(
    ctx: Context<QueryCrypto>, 
    params: QueryParams
) -> anchor_lang::Result<QueryResult> {
    // === 1. 参数校验 ===
    // 校验查询参数合法性
    validate_query_params(&params)?;
    
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
    
    // 执行查询操作
    let result = crypto_service.query(&ctx.accounts.crypto_asset, &params)?;
    
    // === 4. 事件记录 ===
    // 发出查询事件，记录操作详情
    emit!(AssetQueried {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        query_type: params.query_type.clone(),
        authority: ctx.accounts.authority.as_ref().map(|auth| auth.key()),
        timestamp: Clock::get()?.unix_timestamp,
        result_count: result.data.len() as u64,
    });
    
    // === 5. 日志记录 ===
    msg!("Crypto asset queried successfully: asset_id={}, query_type={}, result_count={}", 
         ctx.accounts.crypto_asset.key(), params.query_type, result.data.len());
    
    Ok(result)
}

/// 查询加密货币余额
/// 
/// 返回当前余额、可用余额、冻结余额等详细信息。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// 
/// ## 返回值
/// - `anchor_lang::Result<BalanceInfo>`: 余额信息
pub fn query_crypto_balance(
    ctx: Context<QueryCrypto>
) -> anchor_lang::Result<BalanceInfo> {
    let crypto_service = CryptoService::new();
    let balance_info = crypto_service.get_balance_info(&ctx.accounts.crypto_asset)?;
    
    // 记录查询事件
    emit!(AssetQueried {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        query_type: "balance".to_string(),
        authority: ctx.accounts.authority.as_ref().map(|auth| auth.key()),
        timestamp: Clock::get()?.unix_timestamp,
        result_count: 1,
    });
    
    Ok(balance_info)
}

/// 查询加密货币状态
/// 
/// 返回账户状态、权限信息、配置参数等。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// 
/// ## 返回值
/// - `anchor_lang::Result<StatusInfo>`: 状态信息
pub fn query_crypto_status(
    ctx: Context<QueryCrypto>
) -> anchor_lang::Result<StatusInfo> {
    let crypto_service = CryptoService::new();
    let status_info = crypto_service.get_status_info(&ctx.accounts.crypto_asset)?;
    
    // 记录查询事件
    emit!(AssetQueried {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        query_type: "status".to_string(),
        authority: ctx.accounts.authority.as_ref().map(|auth| auth.key()),
        timestamp: Clock::get()?.unix_timestamp,
        result_count: 1,
    });
    
    Ok(status_info)
}

/// 查询加密货币历史记录
/// 
/// 返回交易历史、操作记录、事件日志等。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `start_time`: 开始时间戳
/// - `end_time`: 结束时间戳
/// - `limit`: 返回记录数量限制
/// 
/// ## 返回值
/// - `anchor_lang::Result<HistoryInfo>`: 历史记录信息
pub fn query_crypto_history(
    ctx: Context<QueryCrypto>,
    start_time: i64,
    end_time: i64,
    limit: u32
) -> anchor_lang::Result<HistoryInfo> {
    // 参数校验
    require!(start_time < end_time, crate::errors::AssetError::InvalidTimeRange);
    require!(limit <= 1000, crate::errors::AssetError::QueryLimitExceeded);
    
    let crypto_service = CryptoService::new();
    let history_info = crypto_service.get_history_info(
        &ctx.accounts.crypto_asset,
        start_time,
        end_time,
        limit
    )?;
    
    // 记录查询事件
    emit!(AssetQueried {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        query_type: "history".to_string(),
        authority: ctx.accounts.authority.as_ref().map(|auth| auth.key()),
        timestamp: Clock::get()?.unix_timestamp,
        result_count: history_info.records.len() as u64,
    });
    
    Ok(history_info)
}

/// 查询加密货币统计信息
/// 
/// 返回统计数据和性能指标。
/// 
/// ## 参数
/// - `ctx`: Anchor账户上下文
/// - `period`: 统计周期
/// 
/// ## 返回值
/// - `anchor_lang::Result<StatsInfo>`: 统计信息
pub fn query_crypto_stats(
    ctx: Context<QueryCrypto>,
    period: String
) -> anchor_lang::Result<StatsInfo> {
    let crypto_service = CryptoService::new();
    let stats_info = crypto_service.get_stats_info(&ctx.accounts.crypto_asset, &period)?;
    
    // 记录查询事件
    emit!(AssetQueried {
        asset_id: ctx.accounts.crypto_asset.key(),
        asset_type: AssetType::Crypto,
        query_type: "stats".to_string(),
        authority: ctx.accounts.authority.as_ref().map(|auth| auth.key()),
        timestamp: Clock::get()?.unix_timestamp,
        result_count: 1,
    });
    
    Ok(stats_info)
}

/// 余额信息结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct BalanceInfo {
    /// 当前总余额
    pub total_balance: u64,
    /// 可用余额
    pub available_balance: u64,
    /// 冻结余额
    pub frozen_balance: u64,
    /// 锁定余额
    pub locked_balance: u64,
    /// 最后更新时间戳
    pub last_updated: i64,
}

/// 状态信息结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct StatusInfo {
    /// 账户状态
    pub status: String,
    /// 是否激活
    pub is_active: bool,
    /// 是否冻结
    pub is_frozen: bool,
    /// 权限账户
    pub authority: Pubkey,
    /// 创建时间戳
    pub created_at: i64,
    /// 最后活动时间戳
    pub last_activity: i64,
}

/// 历史记录信息结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct HistoryInfo {
    /// 记录列表
    pub records: Vec<HistoryRecord>,
    /// 总记录数
    pub total_count: u64,
    /// 是否有更多记录
    pub has_more: bool,
}

/// 历史记录结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct HistoryRecord {
    /// 操作类型
    pub operation_type: String,
    /// 操作数量
    pub amount: u64,
    /// 操作时间戳
    pub timestamp: i64,
    /// 相关账户
    pub related_account: Option<Pubkey>,
    /// 操作描述
    pub description: String,
}

/// 统计信息结构体
#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, InitSpace)]
pub struct StatsInfo {
    /// 总交易次数
    pub total_transactions: u64,
    /// 总交易金额
    pub total_volume: u64,
    /// 平均交易金额
    pub average_amount: u64,
    /// 最大交易金额
    pub max_amount: u64,
    /// 统计周期
    pub period: String,
    /// 统计开始时间
    pub start_time: i64,
    /// 统计结束时间
    pub end_time: i64,
} 