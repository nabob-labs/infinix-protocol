//!
//! Asset Batch Swap Instruction
//! 资产批量swap指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use crate::accounts::BasketIndexStateAccount; // 账户状态结构体定义
use crate::events::asset_event::*; // 资产相关事件定义（Anchor事件）
use crate::services::asset_service::AssetService; // 资产业务逻辑服务层
use crate::state::baskets::BasketIndexState; // 资产篮子状态
use crate::validation::asset_validation::AssetValidatable; // 资产校验trait
use crate::core::types::BatchTradeParams; // 批量交易参数类型
use anchor_lang::prelude::*; // Anchor预导入，提供Solana合约开发的基础类型和宏

/// 资产批量swap指令账户上下文
/// - from: 源资产篮子账户，需可变
/// - to_asset: 目标资产篮子账户，需可变
/// - authority: 操作人签名者
#[derive(Accounts)] // Anchor宏，自动生成账户校验与生命周期管理代码
pub struct BatchSwapAsset<'info> {
    /// 源资产篮子账户，需可变，Anchor自动校验PDA
    #[account(mut)]
    pub from: Account<'info, BasketIndexState>,
    /// 目标资产篮子账户，需可变，Anchor自动校验PDA
    #[account(mut)]
    pub to_asset: Account<'info, BasketIndexState>,
    /// 操作人签名者，需可变
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 资产批量swap指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - params: 批量swap参数
/// - 返回: Anchor规范Result
pub fn batch_swap_asset(ctx: Context<BatchSwapAsset>, params: BatchTradeParams) -> Result<()> {
    let from = &mut ctx.accounts.from; // 获取可变源资产篮子账户
    let to_assets = &mut ctx.accounts.to_asset; // 获取可变目标资产篮子账户数组
    from.validate()?; // 校验源资产篮子状态
    for to in to_assets.iter_mut() {
        to.validate()?; // 校验每个目标资产篮子状态
    }
    // 权限校验：必须是from的authority
    require_keys_eq!(from.authority, ctx.accounts.authority.key(), crate::errors::asset_error::AssetError::AuthorizationFailed);
    // 业务逻辑：批量swap，内部溢出检查
    let mut to_refs: Vec<&mut BasketIndexState> = to_assets.iter_mut().map(|a| a.as_mut()).collect();
    AssetService::batch_swap(from, &mut to_refs, &params, ctx.accounts.authority.key())?;
    emit!(AssetBatchSwapped {
        from_asset_id: from.id, // 事件：源资产ID
        to_asset_ids: to_assets.iter().map(|a| a.id).collect(), // 事件：目标资产ID数组
        from_amounts: params.trades.iter().map(|s| s.from_amount).collect(), // 事件：转出数量数组
        to_amounts: params.trades.iter().map(|s| s.to_amount).collect(), // 事件：转入数量数组
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，表示指令成功
} 