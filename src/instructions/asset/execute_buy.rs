//!
//! Asset Execute Buy Instruction
//! 资产执行买入指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use crate::accounts::BasketIndexStateAccount; // 账户状态结构体定义
use crate::events::asset_event::*; // 资产相关事件定义（Anchor事件）
use crate::services::asset_service::AssetService; // 资产业务逻辑服务层
use crate::state::baskets::BasketIndexState; // 资产篮子状态
use crate::validation::asset_validation::AssetValidatable; // 资产校验trait
use crate::core::types::TradeParams; // 交易参数类型
use anchor_lang::prelude::*; // Anchor预导入，提供Solana合约开发的基础类型和宏

/// 资产执行买入指令账户上下文
/// - asset: 目标资产篮子账户，需可变
/// - authority: 操作人签名者
#[derive(Accounts)] // Anchor宏，自动生成账户校验与生命周期管理代码
pub struct ExecuteBuyAsset<'info> {
    /// 目标资产篮子账户，需可变，Anchor自动校验PDA
    #[account(mut)]
    pub asset: Account<'info, BasketIndexState>,
    /// 操作人签名者，需可变
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 资产执行买入指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - params: 交易参数
/// - price: 买入价格
/// - 返回: Anchor规范Result
pub fn execute_buy_asset(ctx: Context<ExecuteBuyAsset>, params: TradeParams, price: u64) -> Result<()> {
    let asset = &mut ctx.accounts.asset; // 获取可变资产篮子账户
    asset.validate()?; // 校验资产篮子状态
    // 权限校验：必须是当前authority
    require_keys_eq!(asset.authority, ctx.accounts.authority.key(), crate::errors::asset_error::AssetError::AuthorizationFailed);
    // 业务逻辑：执行买入，内部溢出检查
    AssetService::execute_buy(asset, &params, price, ctx.accounts.authority.key())?;
    emit!(AssetBought {
        basket_id: asset.id, // 事件：资产ID
        amount: params.amount_in, // 事件：买入数量
        price, // 事件：买入价格
        buyer: ctx.accounts.authority.key(), // 事件：买方
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，表示指令成功
} 