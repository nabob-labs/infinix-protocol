//!
//! Asset Sell Instruction
//! 资产卖出指令实现，所有业务逻辑下沉到 service 层，指令层只做参数校验、账户校验、事件触发。

use crate::accounts::BasketIndexStateAccount; // 账户状态结构体定义
use crate::events::asset_event::*; // 资产相关事件定义（Anchor事件）
use crate::services::asset_service::AssetService; // 资产业务逻辑服务层
use crate::state::baskets::BasketIndexState; // 资产篮子状态
use crate::validation::asset_validation::AssetValidatable; // 资产校验trait
use crate::core::types::{ExecutionParams, StrategyParams, OracleParams}; // 引入算法、策略、预言机参数类型
use anchor_lang::prelude::*; // Anchor预导入，提供Solana合约开发的基础类型和宏

/// 资产卖出指令账户上下文
/// - basket_index: 目标资产篮子账户，需可变
/// - authority: 操作人签名者
#[derive(Accounts)] // Anchor宏，自动生成账户校验与生命周期管理代码
pub struct SellAsset<'info> {
    /// 目标资产篮子账户，需可变，Anchor自动校验PDA
    #[account(mut)]
    pub basket_index: Account<'info, BasketIndexState>,
    /// 操作人签名者，需可变
    #[account(mut)]
    pub authority: Signer<'info>,
}

/// 资产卖出指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - amount: 卖出数量，最小资产单位
/// - price: 卖出价格
/// - exec_params: 算法执行参数（如最优路由、滑点、聚合DEX等）
/// - strategy_params: 策略参数（如分批卖出、止盈止损等）
/// - oracle_params: 预言机参数（如价格源、校验窗口等）
/// - 返回: Anchor规范Result
pub fn sell_asset(
    ctx: Context<SellAsset>,
    amount: u64,
    price: u64,
    exec_params: Option<ExecutionParams>,
    strategy_params: Option<StrategyParams>,
    oracle_params: Option<OracleParams>,
) -> Result<()> {
    // 获取可变资产篮子账户，生命周期由Anchor自动管理
    let basket_index = &mut ctx.accounts.basket_index;
    // 校验资产篮子状态（如活跃、合法等），防止非法操作
    basket_index.validate()?;
    // 调用服务层卖出逻辑，传递所有参数，内部处理算法/策略/DEX/预言机融合
    AssetService::sell(
        basket_index,
        amount,
        price,
        ctx.accounts.authority.key(),
        exec_params,
        strategy_params,
        oracle_params,
    )?;
    // 触发链上事件，便于审计与追踪
    emit!(AssetSold {
        basket_id: basket_index.id, // 事件：资产篮子ID
        amount, // 事件：卖出数量
        price, // 事件：卖出价格
        seller: ctx.accounts.authority.key(), // 事件：卖方
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
        exec_params, // 事件：算法参数快照
        strategy_params, // 事件：策略参数快照
        oracle_params, // 事件：预言机参数快照
    });
    // Anchor标准返回，表示指令成功
    Ok(())
} 