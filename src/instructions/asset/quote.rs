//!
//! Asset Quote Instruction
//! 资产报价指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use crate::events::asset_event::*; // 资产相关事件定义（Anchor事件）
use crate::services::asset_service::AssetService; // 资产业务逻辑服务层
use crate::core::types::*; // 交易与预言机参数类型
use crate::state::baskets::BasketIndexState; // 篮子状态类型
use anchor_lang::prelude::*; // Anchor预导入，提供Solana合约开发的基础类型和宏

/// 资产报价指令账户上下文
/// - asset: 目标资产篮子账户，需可变
/// - authority: 操作人签名者
#[derive(Accounts)] // Anchor宏，自动生成账户校验与生命周期管理代码
pub struct QuoteAsset<'info> {
    /// 目标资产篮子账户，需可变，Anchor自动校验PDA
    #[account(mut)]
    pub asset: Account<'info, BasketIndexState>,
    /// 操作人签名者，需可变
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 资产报价指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - params: 交易参数
/// - price_params: 预言机参数
/// - 返回: Anchor规范Result，返回报价u64
pub fn quote_asset(ctx: Context<QuoteAsset>, params: TradeParams, price_params: OracleParams) -> anchor_lang::Result<u64> {
    let asset = &ctx.accounts.asset; // 获取资产篮子账户（只读）
    asset.validate()?; // 校验资产篮子状态
    // 权限校验：必须是当前authority
    require_keys_eq!(asset.authority, ctx.accounts.authority.key(), crate::errors::asset_error::AssetError::AuthorizationFailed);
    // 业务逻辑：报价，内部溢出检查
    let price = AssetService::quote(asset, &params, &price_params)?;
    emit!(AssetQuoted {
        asset_id: asset.id, // 事件：资产ID
        price, // 事件：报价
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(price) // Anchor规范返回，返回报价
} 