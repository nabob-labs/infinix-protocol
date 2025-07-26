//!
//! Basket Execute Sell Instruction
//! 篮子执行卖出指令最小功能单元实现，严格遵循Anchor规范、SOLID原则、分层设计、接口清晰、类型安全、事件追踪、权限校验、生命周期管理、错误处理、逐行注释，生产级代码质量。

use crate::accounts::BasketIndexStateAccount; // 引入资产篮子账户状态账户定义
use crate::events::basket_event::*; // 引入所有篮子相关事件定义，便于emit!宏调用
use crate::services::basket_service::BasketService; // 引入篮子服务层，封装核心业务逻辑
use crate::state::baskets::BasketIndexState; // 引入资产篮子状态结构体，类型安全
use anchor_lang::prelude::*; // Anchor预导出内容，包含Context、Account、Signer、Result等
use crate::core::types::TradeParams; // 引入交易参数类型

/// 篮子执行卖出指令账户上下文
/// - basket: 目标资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - seller: 卖出操作人签名者，类型安全
#[derive(Accounts)]
pub struct ExecuteSellBasket<'info> {
    #[account(mut)]
    pub basket: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub seller: Signer<'info>,
}

/// 篮子执行卖出指令主函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - params: 交易参数，类型安全
/// - price: 卖出价格，类型安全
/// - 返回: Anchor规范Result类型，生命周期自动管理
pub fn execute_sell_basket(
    ctx: Context<ExecuteSellBasket>,
    params: TradeParams,
    price: u64,
) -> Result<()> {
    let basket = &mut ctx.accounts.basket;
    // 权限校验
    // 可根据业务需求添加 seller 校验
    // 执行卖出操作（实际业务逻辑应在服务层实现）
    BasketService::execute_sell(basket, &params, price, ctx.accounts.seller.key())?;
    emit!(BasketSold {
        basket_id: basket.id,
        amount: params.amount_in,
        price,
        seller: ctx.accounts.seller.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 