//!
//! Asset Burn Instruction
//! 资产销毁指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use crate::accounts::BasketIndexStateAccount; // 账户状态结构体定义
use crate::events::asset_event::*; // 资产相关事件定义（Anchor事件）
use crate::services::asset_service::AssetService; // 资产业务逻辑服务层
use crate::state::baskets::BasketIndexState; // 资产篮子状态
use crate::validation::asset_validation::AssetValidatable; // 资产校验trait
use anchor_lang::prelude::*; // Anchor预导入，提供Solana合约开发的基础类型和宏

/// 资产销毁指令账户上下文
/// - basket_index: 目标资产篮子账户，需可变
/// - authority: 操作人签名者
#[derive(Accounts)] // Anchor宏，自动生成账户校验与生命周期管理代码
pub struct BurnAsset<'info> {
    /// 目标资产篮子账户，需可变，Anchor自动校验PDA
    #[account(mut)]
    pub basket_index: Account<'info, BasketIndexState>,
    /// 操作人签名者，需可变
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 资产销毁指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 销毁数量，单位为最小资产单位
/// - 返回: Anchor规范Result
pub fn burn_asset(ctx: Context<BurnAsset>, amount: u64) -> Result<()> {
    let basket_index = &mut ctx.accounts.basket_index; // 获取可变资产篮子账户
    basket_index.validate()?; // 校验资产篮子状态（如活跃、合法等），防止非法操作
    AssetService::burn(basket_index, amount)?; // 调用服务层销毁逻辑，处理实际burn，内部包含溢出检查
    emit!(AssetBurned {
        basket_id: basket_index.id, // 事件：资产篮子ID，便于链上追踪
        amount, // 事件：销毁数量，记录操作明细
        authority: ctx.accounts.authority.key(), // 事件：操作人，便于审计
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳，防篡改
    });
    Ok(()) // Anchor规范返回，表示指令成功
} 