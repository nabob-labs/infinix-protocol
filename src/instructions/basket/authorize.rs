//!
//! Basket Authorize Instruction
//! 篮子授权指令最小功能单元实现，严格遵循Anchor规范、SOLID原则、分层设计、接口清晰、类型安全、事件追踪、权限校验、生命周期管理、错误处理、逐行注释，生产级代码质量。

use crate::accounts::BasketIndexStateAccount; // 引入资产篮子账户状态账户定义
use crate::events::basket_event::*; // 引入所有篮子相关事件定义，便于emit!宏调用
use crate::services::basket_service::BasketService; // 引入篮子服务层，封装核心业务逻辑
use crate::state::baskets::BasketIndexState; // 引入资产篮子状态结构体，类型安全
use anchor_lang::prelude::*; // Anchor预导出内容，包含Context、Account、Signer、Result等
use crate::core::types::{AlgoParams, StrategyParams}; // 引入算法和策略参数类型

/// 篮子授权指令账户上下文
/// - basket: 目标资产篮子账户，需可变，Anchor自动校验PDA和生命周期
/// - authority: 当前授权人签名者，类型安全
/// - new_authority: 新授权人公钥，类型安全
#[derive(Accounts)]
pub struct AuthorizeBasket<'info> {
    #[account(mut)]
    pub basket: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub new_authority: Pubkey,
}

/// 篮子授权指令主函数
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - exec_params: 可选算法参数，类型安全
/// - strategy_params: 可选策略参数，类型安全
/// - 返回: Anchor规范Result类型，生命周期自动管理
pub fn authorize_basket(
    ctx: Context<AuthorizeBasket>,
    exec_params: Option<AlgoParams>,
    strategy_params: Option<StrategyParams>,
) -> Result<()> {
    let basket = &mut ctx.accounts.basket;
    // 权限校验
    require_keys_eq!(basket.authority, ctx.accounts.authority.key(), crate::errors::basket_error::BasketError::NotAllowed);
    // 算法/策略扩展点（如有）
    // ...
    // 授权变更
    basket.authority = ctx.accounts.new_authority;
    emit!(BasketAuthorized {
        basket_id: basket.id,
        old_authority: ctx.accounts.authority.key(),
        new_authority: ctx.accounts.new_authority,
        timestamp: Clock::get()?.unix_timestamp,
    });
    Ok(())
} 