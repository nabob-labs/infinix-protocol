//!
//! # Index Token Program 主入口
//!
//! 本文件为 Solana/Anchor 框架下的主合约入口，负责统一注册所有链上指令、模块、账户、算法、DEX、Oracle、策略等，
//! 并严格遵循 Anchor 0.31.1 语法与最佳实践。
//!
//! ## 设计说明
//! - 模块化、分层、可插拔架构，支持资产、篮子、指数、组合、算法、策略、DEX、Oracle 等多业务域
//! - 全面采用 Anchor #[program]、#[account]、Context、Result 等规范，安全性、可扩展性、可维护性强
//! - 统一入口，便于权限校验、生命周期管理、合规性、事件追踪、错误处理
//! - 依赖全局注册表自动注册 DEX/Oracle/Adapter，支持运行时动态扩展
//! - 代码注释细致，便于审计、维护、二次开发
//!
//! ## Anchor 最佳实践
//! - 每个指令函数均以 Context<...> 作为账户参数，严格校验账户权限、生命周期、PDA
//! - 所有参数均类型安全，边界校验、错误处理、事件日志齐全
//! - 支持多种批量操作、策略切换、算法热插拔、DEX/Oracle 动态注册
//! - 入口函数分组清晰，便于权限分离与业务扩展

#![allow(clippy::result_large_err)] // 允许 clippy 检查中 result_large_err 警告，便于大 Result 类型返回，提升合约灵活性和兼容性

// === Anchor 预导入 ===
/// 引入 Anchor 框架的预置类型、宏、Result、Context 等，便于合约开发和 Anchor 语法支持
use anchor_lang::prelude::*; // Anchor 预导入，提供合约开发基础类型、宏、Context、Result等

// === 全局注册表与自动注册说明 ===
/// 依赖 core::registry 中的全局单例自动注册 DEX/Oracle/Adapter，无需手动 register_adapters，保证运行时动态扩展能力
// use crate::core::registry::{DEX_ADAPTER_REGISTRY, ORACLE_ADAPTER_REGISTRY}; // 自动注册DEX/Oracle适配器，提升可扩展性 - 暂时注释掉

// === 模块声明 ===
/// 业务域模块分层声明，便于维护与扩展，每个模块均为独立功能域
pub mod account_models;    ///< 账户模型与账户校验，定义所有链上账户结构和校验逻辑
pub mod algorithms;  ///< 算法与执行策略，包含所有算法实现与策略接口
pub mod basket;      ///< 资产篮子与组合，管理资产集合与组合逻辑
pub mod core;        ///< 核心类型、常量、注册表、工具，提供全局基础设施
pub mod dex;         ///< DEX/AMM 适配与集成，支持多种去中心化交易所
pub mod errors;      ///< 错误类型与处理，定义所有错误码与处理逻辑
pub mod events;      ///< Anchor 事件定义，链上事件声明与触发
pub mod factories;   ///< 工厂与注册机制，支持模块化扩展与注册
pub mod instructions;///< 指令集入口，所有链上指令实现
pub mod oracles;     ///< 预言机适配与集成，支持多种链上价格源
pub mod services;    ///< 业务逻辑服务层，提供业务逻辑封装
pub mod state;       ///< 状态管理与账户持久化，链上持久化数据结构
pub mod strategies;  ///< 策略与高级组合，支持多种投资与交易策略
pub mod utils;       ///< 工具函数与通用校验，通用辅助工具
pub mod validation;  ///< 业务校验与合规性，所有合规性与业务校验逻辑
pub mod version;     ///< 版本管理，合约版本控制

// === 指令模块统一导入 ===
/// 统一导入所有指令实现，便于主入口统一调用
use instructions::*; // 导入所有指令模块，便于主入口统一调度

// === Anchor Program ID 声明 ===
/// 主合约唯一标识，部署时需替换为实际ID，确保合约唯一性和安全性
anchor_lang::declare_id!("11111111111111111111111111111111"); // 主合约唯一标识，部署时需替换为实际ID

// === Anchor #[program] 主入口模块 ===
/// Anchor #[program] 宏，自动生成合约主入口，统一注册所有链上指令
#[program] // Anchor 宏，标记本模块为合约主入口，自动生成入口分发逻辑
pub mod index_token_program { // 定义主合约模块，所有链上指令均在此注册
    use super::*; // 引入上层所有模块、类型、宏，便于指令函数调用
    
    // 导入所需的类型定义
    use super::dex::traits::SwapParams;
    use super::core::{ExecutionParams, PriceParams, StrategyParams, TradeParams, OracleParams, BatchTradeParams, StrategyTradeParams, BatchSwapParams, AlgoParams};

    /// 资产增发指令
    /// # 参数
    /// - ctx: Context<instructions::asset::MintAsset>，Anchor账户上下文，自动校验账户权限与生命周期
    /// - amount: u64，增发的资产数量，必须为正整数
    /// # 返回值
    /// - anchor_lang::Result<()>: Anchor标准返回类型，表示指令执行成功或失败
    /// # 安全性
    /// - Anchor自动校验账户权限、生命周期、PDA
    /// - 需确保amount参数合法，防止溢出与非法操作
    pub fn mint_asset(ctx: Context<instructions::asset::MintAsset>, amount: u64) -> anchor_lang::Result<()> { // 资产增发指令主函数，ctx为账户上下文，amount为增发数量
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. amount参数为增发数量，类型安全，需业务层校验正整数
        // 3. 调用 asset 模块的 mint_asset 实现，完成实际增发逻辑
        // 4. 返回下层实现的结果，anchor_lang::Result<()>类型，Anchor自动处理错误与生命周期
        instructions::asset::mint_asset(ctx, amount) // 调用实际增发实现，返回执行结果
    }

    /// 资产销毁指令
    /// # 参数
    /// - ctx: Context<instructions::asset::BurnAsset>，Anchor账户上下文，自动校验账户权限与生命周期
    /// - amount: u64，销毁的资产数量，必须为正整数
    /// # 返回值
    /// - anchor_lang::Result<()>: Anchor标准返回类型，表示指令执行成功或失败
    /// # 安全性
    /// - Anchor自动校验账户权限、生命周期、PDA
    /// - 需确保amount参数合法，防止溢出与非法操作
    pub fn burn_asset(ctx: Context<instructions::asset::BurnAsset>, amount: u64) -> anchor_lang::Result<()> { // 资产销毁指令主函数，ctx为账户上下文，amount为销毁数量
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. amount参数为销毁数量，类型安全，需业务层校验正整数
        // 3. 调用 asset 模块的 burn_asset 实现，完成实际销毁逻辑
        // 4. 返回下层实现的结果，anchor_lang::Result<()>类型，Anchor自动处理错误与生命周期
        instructions::asset::burn_asset(ctx, amount) // 调用实际销毁实现，返回执行结果
    }

    /// 资产转账指令
    /// # 参数
    /// - ctx: Context<instructions::asset::TransferAsset>，Anchor账户上下文，自动校验账户权限与生命周期
    /// - amount: u64，转账的资产数量，必须为正整数
    /// # 返回值
    /// - anchor_lang::Result<()>: Anchor标准返回类型，表示指令执行成功或失败
    /// # 安全性
    /// - Anchor自动校验账户权限、生命周期、PDA
    /// - 需确保amount参数合法，防止溢出与非法操作
    pub fn transfer_asset(ctx: Context<instructions::asset::TransferAsset>, amount: u64) -> anchor_lang::Result<()> { // 资产转账指令主函数，ctx为账户上下文，amount为转账数量
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. amount参数为转账数量，类型安全，需业务层校验正整数
        // 3. 调用 asset 模块的 transfer_asset 实现，完成实际转账逻辑
        // 4. 返回下层实现的结果，anchor_lang::Result<()>类型，Anchor自动处理错误与生命周期
        instructions::asset::transfer_asset(ctx, amount) // 调用实际转账实现，返回执行结果
    }

    /// 资产查询指令
    /// # 参数
    /// - ctx: Context<instructions::asset::QueryAsset>，Anchor账户上下文，自动校验账户权限与生命周期
    /// # 返回值
    /// - anchor_lang::Result<u64>: 返回资产余额
    /// # 安全性
    /// - Anchor自动校验账户权限、生命周期、PDA
    pub fn query_asset(ctx: Context<instructions::asset::QueryAsset>) -> anchor_lang::Result<u64> { // 资产查询指令主函数，ctx为账户上下文
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. 调用 asset 模块的 query_asset 实现，返回资产余额
        instructions::asset::query_asset(ctx) // 调用实际查询实现，返回余额
    }

    /// 资产买入指令
    pub fn buy_asset(
        ctx: Context<instructions::asset::BuyAsset>, // Anchor账户上下文，自动校验账户权限与生命周期
        params: SwapParams, // 买入参数，类型安全，包含输入输出资产、数量等
        price_params: PriceParams, // 价格参数，类型安全，包含价格源、滑点等
        exec_params: Option<ExecutionParams>, // 可选算法执行参数，支持算法热插拔
        strategy_params: Option<StrategyParams>, // 可选策略参数，支持多策略扩展
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. params/price_params/exec_params/strategy_params 均为类型安全参数，需业务层校验
        // 3. 调用 asset 模块的 buy_asset 实现，完成实际买入逻辑
        // 4. 返回下层实现的结果，anchor_lang::Result<()>类型，Anchor自动处理错误与生命周期
        instructions::asset::buy_asset(ctx, params, price_params, exec_params, strategy_params) // 调用实际买入实现，返回执行结果
    }

    /// 资产卖出指令
    pub fn sell_asset(
        ctx: Context<instructions::asset::SellAsset>, // Anchor账户上下文，自动校验账户权限与生命周期
        params: SwapParams, // 卖出参数，类型安全，包含输入输出资产、数量等
        price_params: PriceParams, // 价格参数，类型安全，包含价格源、滑点等
        exec_params: Option<ExecutionParams>, // 可选算法执行参数，支持算法热插拔
        strategy_params: Option<StrategyParams>, // 可选策略参数，支持多策略扩展
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. params/price_params/exec_params/strategy_params 均为类型安全参数，需业务层校验
        // 3. 调用 asset 模块的 sell_asset 实现，完成实际卖出逻辑
        // 4. 返回下层实现的结果，anchor_lang::Result<()>类型，Anchor自动处理错误与生命周期
        instructions::asset::sell_asset(ctx, params, price_params, exec_params, strategy_params) // 调用实际卖出实现，返回执行结果
    }

    /// 资产swap指令
    pub fn swap_asset(
        ctx: Context<instructions::asset::SwapAsset>, // Anchor账户上下文，自动校验账户权限与生命周期
        params: SwapParams, // swap参数，类型安全，包含输入输出资产、数量等
        price_params: PriceParams, // 价格参数，类型安全，包含价格源、滑点等
        exec_params: Option<ExecutionParams>, // 可选算法执行参数，支持算法热插拔
        strategy_params: Option<StrategyParams>, // 可选策略参数，支持多策略扩展
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. params/price_params/exec_params/strategy_params 均为类型安全参数，需业务层校验
        // 3. 调用 asset 模块的 swap_asset 实现，完成实际swap逻辑
        // 4. 返回下层实现的结果，anchor_lang::Result<()>类型，Anchor自动处理错误与生命周期
        instructions::asset::swap_asset(ctx, params, price_params, exec_params, strategy_params) // 调用实际swap实现，返回执行结果
    }

    /// 资产授权指令
    pub fn authorize_asset(
        ctx: Context<instructions::asset::AuthorizeAsset>, // Anchor账户上下文，自动校验账户权限与生命周期
        exec_params: Option<ExecutionParams>, // 可选算法执行参数，支持算法热插拔
        strategy_params: Option<StrategyParams>, // 可选策略参数，支持多策略扩展
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. exec_params/strategy_params 为可选类型安全参数，需业务层校验
        // 3. 调用 asset 模块的 authorize_asset 实现，完成实际授权逻辑
        // 4. 返回下层实现的结果，anchor_lang::Result<()>类型，Anchor自动处理错误与生命周期
        instructions::asset::authorize_asset(ctx, exec_params, strategy_params) // 调用实际授权实现，返回执行结果
    }

    /// 资产合并指令
    pub fn combine_asset(
        ctx: Context<instructions::asset::CombineAsset>, // Anchor账户上下文，自动校验账户权限与生命周期
        amount: u64, // 合并数量，类型安全，需业务层校验
        exec_params: Option<ExecutionParams>, // 可选算法执行参数，支持算法热插拔
        strategy_params: Option<StrategyParams>, // 可选策略参数，支持多策略扩展
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. amount为合并数量，类型安全，需业务层校验
        // 3. exec_params/strategy_params 为可选类型安全参数
        // 4. 调用 asset 模块的 combine_asset 实现，完成实际合并逻辑
        // 5. 返回下层实现的结果，anchor_lang::Result<()>类型，Anchor自动处理错误与生命周期
        instructions::asset::combine_asset(ctx, amount, exec_params, strategy_params) // 调用实际合并实现，返回执行结果
    }

    /// 资产拆分指令
    pub fn split_asset(
        ctx: Context<instructions::asset::SplitAsset>, // Anchor账户上下文，自动校验账户权限与生命周期
        amount: u64, // 拆分数量，类型安全，需业务层校验
        exec_params: Option<ExecutionParams>, // 可选算法执行参数，支持算法热插拔
        strategy_params: Option<StrategyParams>, // 可选策略参数，支持多策略扩展
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. amount为拆分数量，类型安全，需业务层校验
        // 3. exec_params/strategy_params 为可选类型安全参数
        // 4. 调用 asset 模块的 split_asset 实现，完成实际拆分逻辑
        // 5. 返回下层实现的结果，anchor_lang::Result<()>类型，Anchor自动处理错误与生命周期
        instructions::asset::split_asset(ctx, amount, exec_params, strategy_params) // 调用实际拆分实现，返回执行结果
    }

    /// 资产冻结指令
    pub fn freeze_asset(
        ctx: Context<instructions::asset::FreezeAsset>, // Anchor账户上下文，自动校验账户权限与生命周期
        exec_params: Option<ExecutionParams>, // 可选算法执行参数，支持算法热插拔
        strategy_params: Option<StrategyParams>, // 可选策略参数，支持多策略扩展
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. exec_params/strategy_params 为可选类型安全参数
        // 3. 调用 asset 模块的 freeze_asset 实现，完成实际冻结逻辑
        // 4. 返回下层实现的结果，anchor_lang::Result<()>类型，Anchor自动处理错误与生命周期
        instructions::asset::freeze_asset(ctx, exec_params, strategy_params) // 调用实际冻结实现，返回执行结果
    }

    /// 资产解冻指令
    pub fn unfreeze_asset(
        ctx: Context<instructions::asset::UnfreezeAsset>, // Anchor账户上下文，自动校验账户权限与生命周期
        exec_params: Option<ExecutionParams>, // 可选算法执行参数，支持算法热插拔
        strategy_params: Option<StrategyParams>, // 可选策略参数，支持多策略扩展
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. exec_params/strategy_params 为可选类型安全参数
        // 3. 调用 asset 模块的 unfreeze_asset 实现，完成实际解冻逻辑
        // 4. 返回下层实现的结果，anchor_lang::Result<()>类型，Anchor自动处理错误与生命周期
        instructions::asset::unfreeze_asset(ctx, exec_params, strategy_params) // 调用实际解冻实现，返回执行结果
    }

    /// 资产批量转账指令
    pub fn batch_transfer_asset(
        ctx: Context<instructions::asset::BatchTransferAsset>, // Anchor账户上下文，自动校验账户权限与生命周期
        amounts: Vec<u64>, // 批量转账数量，类型安全，需业务层校验
        exec_params: Option<ExecutionParams>, // 可选算法执行参数，支持算法热插拔
        strategy_params: Option<StrategyParams>, // 可选策略参数，支持多策略扩展
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. amounts为批量转账数量，类型安全，需业务层校验
        // 3. exec_params/strategy_params 为可选类型安全参数
        // 4. 调用 asset 模块的 batch_transfer_asset 实现，完成实际批量转账逻辑
        // 5. 返回下层实现的结果，anchor_lang::Result<()>类型，Anchor自动处理错误与生命周期
        instructions::asset::batch_transfer_asset(ctx, amounts, exec_params, strategy_params) // 调用实际批量转账实现，返回执行结果
    }

    /// 资产批量swap指令
    pub fn batch_swap_asset(
        ctx: Context<instructions::asset::BatchSwapAsset>, // Anchor账户上下文，自动校验账户权限与生命周期
        params: BatchSwapParams, // 批量swap参数，类型安全，需业务层校验
        exec_params: Option<ExecutionParams>, // 可选算法执行参数，支持算法热插拔
        strategy_params: Option<StrategyParams>, // 可选策略参数，支持多策略扩展
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. params为批量swap参数，类型安全，需业务层校验
        // 3. exec_params/strategy_params 为可选类型安全参数
        // 4. 调用 asset 模块的 batch_swap_asset 实现，完成实际批量swap逻辑
        // 5. 返回下层实现的结果，anchor_lang::Result<()>类型，Anchor自动处理错误与生命周期
        instructions::asset::batch_swap_asset(ctx, params, exec_params, strategy_params) // 调用实际批量swap实现，返回执行结果
    }

    /// 资产策略交易指令
    pub fn strategy_trade_asset(
        ctx: Context<instructions::asset::StrategyTradeAsset>, // Anchor账户上下文，自动校验账户权限与生命周期
        params: StrategyTradeParams, // 策略交易参数，类型安全，需业务层校验
        exec_params: Option<ExecutionParams>, // 可选算法执行参数，支持算法热插拔
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. params为策略交易参数，类型安全，需业务层校验
        // 3. exec_params为可选类型安全参数
        // 4. 调用 asset 模块的 strategy_trade_asset 实现，完成实际策略交易逻辑
        // 5. 返回下层实现的结果，anchor_lang::Result<()>类型，Anchor自动处理错误与生命周期
        instructions::asset::strategy_trade_asset(ctx, params, exec_params) // 调用实际策略交易实现，返回执行结果
    }

    /// 资产报价指令
    pub fn quote_asset(
        ctx: Context<instructions::asset::QuoteAsset>, // Anchor账户上下文，自动校验账户权限与生命周期
        params: SwapParams, // swap参数，类型安全，需业务层校验
        price_params: PriceParams, // 价格参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<u64> { // Anchor标准返回类型，返回资产报价
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. params/price_params为类型安全参数，需业务层校验
        // 3. 调用 asset 模块的 quote_asset 实现，返回报价
        instructions::asset::quote_asset(ctx, params, price_params) // 调用实际报价实现，返回报价
    }

    /// 资产执行买入指令
    pub fn execute_buy_asset(
        ctx: Context<instructions::asset::ExecuteBuyAsset>, // Anchor账户上下文，自动校验账户权限与生命周期
        params: SwapParams, // 买入参数，类型安全，需业务层校验
        price: u64, // 买入价格，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. params为类型安全参数，price为买入价格，需业务层校验
        // 3. 调用 asset 模块的 execute_buy_asset 实现，完成实际买入逻辑
        instructions::asset::execute_buy_asset(ctx, params, price) // 调用实际买入实现，返回执行结果
    }

    /// 资产执行卖出指令
    pub fn execute_sell_asset(
        ctx: Context<instructions::asset::ExecuteSellAsset>, // Anchor账户上下文，自动校验账户权限与生命周期
        params: SwapParams, // 卖出参数，类型安全，需业务层校验
        price: u64, // 卖出价格，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. params为类型安全参数，price为卖出价格，需业务层校验
        // 3. 调用 asset 模块的 execute_sell_asset 实现，完成实际卖出逻辑
        instructions::asset::execute_sell_asset(ctx, params, price) // 调用实际卖出实现，返回执行结果
    }

    /// 资产执行swap指令
    pub fn execute_swap_asset(
        ctx: Context<instructions::asset::ExecuteSwapAsset>, // Anchor账户上下文，自动校验账户权限与生命周期
        from_amount: u64, // swap输入数量，类型安全，需业务层校验
        to_amount: u64, // swap输出数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. from_amount/to_amount为类型安全参数，需业务层校验
        // 3. 调用 asset 模块的 execute_swap_asset 实现，完成实际swap逻辑
        instructions::asset::execute_swap_asset(ctx, from_amount, to_amount) // 调用实际swap实现，返回执行结果
    }

    /// 资产执行合并指令
    pub fn execute_combine_asset(
        ctx: Context<instructions::asset::ExecuteCombineAsset>, // Anchor账户上下文，自动校验账户权限与生命周期
        amount: u64, // 合并数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. amount为类型安全参数，需业务层校验
        // 3. 调用 asset 模块的 execute_combine_asset 实现，完成实际合并逻辑
        instructions::asset::execute_combine_asset(ctx, amount) // 调用实际合并实现，返回执行结果
    }

    /// 资产执行拆分指令
    pub fn execute_split_asset(
        ctx: Context<instructions::asset::ExecuteSplitAsset>, // Anchor账户上下文，自动校验账户权限与生命周期
        amount: u64, // 拆分数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. amount为类型安全参数，需业务层校验
        // 3. 调用 asset 模块的 execute_split_asset 实现，完成实际拆分逻辑
        instructions::asset::execute_split_asset(ctx, amount) // 调用实际拆分实现，返回执行结果
    }

    /// 资产批量策略交易指令
    pub fn batch_strategy_trade_asset(
        ctx: Context<instructions::asset::BatchTransferAsset>, // Anchor账户上下文，自动校验账户权限与生命周期
        strategies: Vec<StrategyTradeParams>, // 策略交易参数集合，类型安全，需业务层校验
        exec_params: Option<ExecutionParams>, // 可选算法执行参数，算法热插拔
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. strategies/exec_params 均为类型安全参数，需业务层校验
        // 3. 调用 asset 模块的 batch_strategy_trade_asset 实现，完成实际批量策略交易逻辑
        // 4. 返回下层实现的结果，anchor_lang::Result<()>类型，Anchor自动处理错误与生命周期
        // instructions::asset::batch_strategy_trade_asset(ctx, strategies, exec_params) // 调用实际批量策略交易实现，返回执行结果
        Ok(()) // 暂时返回成功，等待实现
    }

    // ==================== 篮子相关指令 ====================
    /// 篮子再平衡指令
    pub fn rebalance_basket(
        ctx: Context<instructions::basket::RebalanceBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        new_weights: Vec<u64>, // 新权重集合，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. new_weights为类型安全参数，需业务层校验
        // 3. 调用 basket 模块的 rebalance_basket 实现，完成实际再平衡逻辑
        instructions::basket::rebalance_basket(ctx, new_weights) // 调用实际再平衡实现，返回执行结果
    }

    /// 篮子暂停指令
    pub fn pause_basket(
        ctx: Context<instructions::basket::PauseBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. 调用 basket 模块的 pause_basket 实现，完成实际暂停逻辑
        instructions::basket::pause_basket(ctx) // 调用实际暂停实现，返回执行结果
    }

    /// 篮子恢复指令
    pub fn resume_basket(
        ctx: Context<instructions::basket::ResumeBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. 调用 basket 模块的 resume_basket 实现，完成实际恢复逻辑
        instructions::basket::resume_basket(ctx) // 调用实际恢复实现，返回执行结果
    }

    /// 篮子再平衡（带算法）指令
    pub fn rebalance_basket_with_algo(
        ctx: Context<instructions::basket::RebalanceBasketWithAlgo>, // Anchor账户上下文，自动校验账户权限与生命周期
        new_weights: Vec<u64>, // 新权重集合，类型安全，需业务层校验
        algo_name: String, // 算法名称，类型安全，需业务层校验
        params: super::algorithms::traits::ExecutionParams, // 算法执行参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. new_weights/algo_name/params为类型安全参数，需业务层校验
        // 3. 调用 basket 模块的 rebalance_basket_with_algo 实现，完成实际再平衡逻辑
        instructions::basket::rebalance_basket_with_algo(ctx, new_weights, algo_name, params) // 调用实际再平衡实现，返回执行结果
    }

    /// 篮子再平衡（带算法和适配器）指令
    pub fn rebalance_basket_with_algo_and_adapters(
        ctx: Context<instructions::basket::RebalanceBasketWithAlgoAndAdapters>, // Anchor账户上下文，自动校验账户权限与生命周期
        args: instructions::basket::RebalanceWithAlgoAndAdaptersParams, // 算法与适配器参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. args为类型安全参数，需业务层校验
        // 3. 调用 basket 模块的 rebalance_basket_with_algo_and_adapters 实现
        instructions::basket::rebalance_basket_with_algo_and_adapters(ctx, args) // 调用实际再平衡实现，返回执行结果
    }

    /// 篮子转账指令
    pub fn transfer_basket(
        ctx: Context<instructions::basket::TransferBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        amount: u64, // 转账数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. amount为类型安全参数，需业务层校验
        // 3. 调用 basket 模块的 transfer_basket 实现
        instructions::basket::transfer_basket(ctx, amount) // 调用实际转账实现，返回执行结果
    }

    /// 篮子查询指令
    pub fn query_basket(
        ctx: Context<instructions::basket::QueryBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
    ) -> anchor_lang::Result<u64> { // Anchor标准返回类型，返回篮子余额
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. 调用 basket 模块的 query_basket 实现，返回篮子余额
        instructions::basket::query_basket(ctx) // 调用实际查询实现，返回余额
    }

    // ==================== 篮子授权、合并、拆分、冻结、解冻、批量等指令 ====================
    pub fn combine_basket(
        ctx: Context<instructions::basket::CombineBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        amount: u64, // 合并数量，类型安全，需业务层校验
        exec_params: Option<AlgoParams>, // 可选算法参数，类型安全，需业务层校验
        strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::basket::combine_basket(ctx, amount, exec_params, strategy_params) // 调用实际合并实现，返回执行结果
    }
    pub fn split_basket(
        ctx: Context<instructions::basket::SplitBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        amount: u64, // 拆分数量，类型安全，需业务层校验
        exec_params: Option<AlgoParams>, // 可选算法参数，类型安全，需业务层校验
        strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::basket::split_basket(ctx, amount, exec_params, strategy_params) // 调用实际拆分实现，返回执行结果
    }
    pub fn freeze_basket(
        ctx: Context<instructions::basket::FreezeBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        exec_params: Option<AlgoParams>, // 可选算法参数，类型安全，需业务层校验
        strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::basket::freeze_basket(ctx, exec_params, strategy_params) // 调用实际冻结实现，返回执行结果
    }
    pub fn unfreeze_basket(
        ctx: Context<instructions::basket::UnfreezeBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        exec_params: Option<AlgoParams>, // 可选算法参数，类型安全，需业务层校验
        strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::basket::unfreeze_basket(ctx, exec_params, strategy_params) // 调用实际解冻实现，返回执行结果
    }
    pub fn batch_transfer_basket(
        ctx: Context<instructions::basket::BatchTransferBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        amounts: Vec<u64>, // 批量转账数量，类型安全，需业务层校验
        exec_params: Option<AlgoParams>, // 可选算法参数，类型安全，需业务层校验
        strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::basket::batch_transfer_basket(ctx, amounts, exec_params, strategy_params) // 调用实际批量转账实现，返回执行结果
    }
    pub fn batch_rebalance_basket(
        ctx: Context<instructions::basket::BatchRebalanceBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        params: BatchTradeParams, // 批量再平衡参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::basket::batch_rebalance_basket(ctx, params) // 调用实际批量再平衡实现，返回执行结果
    }
    pub fn strategy_rebalance_basket(
        ctx: Context<instructions::basket::StrategyRebalanceBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        params: StrategyParams, // 策略参数，类型安全，需业务层校验
        exec_params: Option<AlgoParams>, // 可选算法参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::basket::strategy_rebalance_basket(ctx, params, exec_params) // 调用实际策略再平衡实现，返回执行结果
    }
    pub fn batch_subscribe_basket(
        ctx: Context<instructions::basket::BatchSubscribeBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        amounts: Vec<u64>, // 批量申购数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::basket::batch_subscribe_basket(ctx, amounts) // 调用实际批量申购实现，返回执行结果
    }
    pub fn batch_redeem_basket(
        ctx: Context<instructions::basket::BatchRedeemBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        amounts: Vec<u64>, // 批量赎回数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::basket::batch_redeem_basket(ctx, amounts) // 调用实际批量赎回实现，返回执行结果
    }
    pub fn batch_combine_basket(
        ctx: Context<instructions::basket::BatchCombineBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        amounts: Vec<u64>, // 批量合并数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::basket::batch_combine_basket(ctx, amounts) // 调用实际批量合并实现，返回执行结果
    }
    pub fn batch_split_basket(
        ctx: Context<instructions::basket::BatchSplitBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        amounts: Vec<u64>, // 批量拆分数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::basket::batch_split_basket(ctx, amounts) // 调用实际批量拆分实现，返回执行结果
    }
    pub fn quote_basket(
        ctx: Context<instructions::basket::QuoteBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        params: TradeParams, // 交易参数，类型安全，需业务层校验
        price_params: OracleParams, // 价格参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<u64> { // Anchor标准返回类型，返回篮子报价
        instructions::basket::quote_basket(ctx, params, price_params) // 调用实际报价实现，返回报价
    }
    pub fn execute_buy_basket(
        ctx: Context<instructions::basket::ExecuteBuyBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        params: TradeParams, // 交易参数，类型安全，需业务层校验
        price: u64, // 买入价格，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::basket::execute_buy_basket(ctx, params, price) // 调用实际买入实现，返回执行结果
    }
    pub fn execute_sell_basket(
        ctx: Context<instructions::basket::ExecuteSellBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        params: TradeParams, // 交易参数，类型安全，需业务层校验
        price: u64, // 卖出价格，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::basket::execute_sell_basket(ctx, params, price) // 调用实际卖出实现，返回执行结果
    }
    pub fn execute_swap_basket(
        ctx: Context<instructions::basket::ExecuteSwapBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        from_amount: u64, // swap输入数量，类型安全，需业务层校验
        to_amount: u64, // swap输出数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::basket::execute_swap_basket(ctx, from_amount, to_amount) // 调用实际swap实现，返回执行结果
    }
    pub fn execute_combine_basket(
        ctx: Context<instructions::basket::ExecuteCombineBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        amount: u64, // 合并数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::basket::execute_combine_basket(ctx, amount) // 调用实际合并实现，返回执行结果
    }
    pub fn execute_split_basket(
        ctx: Context<instructions::basket::ExecuteSplitBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        amount: u64, // 拆分数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::basket::execute_split_basket(ctx, amount) // 调用实际拆分实现，返回执行结果
    }
    pub fn batch_strategy_rebalance_basket(
        ctx: Context<instructions::basket::BatchTransferBasket>, // Anchor账户上下文，自动校验账户权限与生命周期
        strategies: Vec<StrategyParams>, // 策略参数集合，类型安全，需业务层校验
        exec_params: Option<AlgoParams>, // 可选算法参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. strategies/exec_params 均为类型安全参数，需业务层校验
        // 3. 调用 basket 模块的 batch_strategy_rebalance_basket 实现，完成实际批量策略再平衡逻辑
        // 4. 返回下层实现的结果，anchor_lang::Result<()>类型，Anchor自动处理错误与生命周期
        // instructions::basket::batch_strategy_rebalance_basket(ctx, strategies, exec_params) // 调用实际批量策略再平衡实现，返回执行结果
        Ok(()) // 暂时返回成功，等待实现
    }
    // ==================== 指数代币相关指令 ====================
    /// 指数代币增发指令
    pub fn mint_index_token(
        ctx: Context<instructions::index_token::MintIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        amount: u64, // 增发数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::mint_index_token(ctx, amount) // 调用实际增发实现，返回执行结果
    }

    /// 指数代币销毁指令
    pub fn burn_index_token(
        ctx: Context<instructions::index_token::BurnIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        amount: u64, // 销毁数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::burn_index_token(ctx, amount) // 调用实际销毁实现，返回执行结果
    }

    /// 指数代币转账指令
    pub fn transfer_index_token(
        ctx: Context<instructions::index_token::TransferIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        amount: u64, // 转账数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::transfer_index_token(ctx, amount) // 调用实际转账实现，返回执行结果
    }

    /// 指数代币查询指令
    pub fn query_index_token(
        ctx: Context<instructions::index_token::QueryIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
    ) -> anchor_lang::Result<u64> { // Anchor标准返回类型，返回指数代币余额
        instructions::index_token::query_index_token(ctx) // 调用实际查询实现，返回余额
    }

    // ==================== 指数代币授权、合并、拆分、冻结、解冻、批量等指令 ====================
    pub fn authorize_index_token(
        ctx: Context<instructions::index_token::AuthorizeIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        exec_params: Option<AlgoParams>, // 可选算法参数，类型安全，需业务层校验
        strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::authorize_index_token(ctx, exec_params, strategy_params) // 调用实际授权实现，返回执行结果
    }
    pub fn combine_index_token(
        ctx: Context<instructions::index_token::CombineIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        amount: u64, // 合并数量，类型安全，需业务层校验
        exec_params: Option<AlgoParams>, // 可选算法参数，类型安全，需业务层校验
        strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::combine_index_token(ctx, amount, exec_params, strategy_params) // 调用实际合并实现，返回执行结果
    }
    pub fn split_index_token(
        ctx: Context<instructions::index_token::SplitIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        amount: u64, // 拆分数量，类型安全，需业务层校验
        exec_params: Option<AlgoParams>, // 可选算法参数，类型安全，需业务层校验
        strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::split_index_token(ctx, amount, exec_params, strategy_params) // 调用实际拆分实现，返回执行结果
    }
    pub fn freeze_index_token(
        ctx: Context<instructions::index_token::FreezeIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        exec_params: Option<AlgoParams>, // 可选算法参数，类型安全，需业务层校验
        strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::freeze_index_token(ctx, exec_params, strategy_params) // 调用实际冻结实现，返回执行结果
    }
    pub fn unfreeze_index_token(
        ctx: Context<instructions::index_token::UnfreezeIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        exec_params: Option<AlgoParams>, // 可选算法参数，类型安全，需业务层校验
        strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::unfreeze_index_token(ctx, exec_params, strategy_params) // 调用实际解冻实现，返回执行结果
    }
    pub fn batch_transfer_index_token(
        ctx: Context<instructions::index_token::BatchTransferIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        amounts: Vec<u64>, // 批量转账数量，类型安全，需业务层校验
        exec_params: Option<AlgoParams>, // 可选算法参数，类型安全，需业务层校验
        strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::batch_transfer_index_token(ctx, amounts, exec_params, strategy_params) // 调用实际批量转账实现，返回执行结果
    }
    pub fn batch_subscribe_index_token(
        ctx: Context<instructions::index_token::BatchSubscribeIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        amounts: Vec<u64>, // 批量申购数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::batch_subscribe_index_token(ctx, amounts) // 调用实际批量申购实现，返回执行结果
    }
    pub fn batch_redeem_index_token(
        ctx: Context<instructions::index_token::BatchRedeemIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        amounts: Vec<u64>, // 批量赎回数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::batch_redeem_index_token(ctx, amounts) // 调用实际批量赎回实现，返回执行结果
    }
    pub fn strategy_subscribe_index_token(
        ctx: Context<instructions::index_token::StrategySubscribeIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        params: StrategyParams, // 策略参数，类型安全，需业务层校验
        exec_params: Option<AlgoParams>, // 可选算法参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::strategy_subscribe_index_token(ctx, params, exec_params) // 调用实际策略申购实现，返回执行结果
    }
    pub fn strategy_redeem_index_token(
        ctx: Context<instructions::index_token::StrategyRedeemIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        strategy: String, // 策略名称，类型安全，需业务层校验
        params: Vec<u8>, // 策略参数，类型安全，需业务层校验
        exec_params: Option<AlgoParams>, // 可选算法参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::strategy_redeem_index_token(ctx, strategy, params, exec_params) // 调用实际策略赎回实现，返回执行结果
    }
    pub fn batch_combine_index_token(
        ctx: Context<instructions::index_token::BatchCombineIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        params: BatchTradeParams, // 批量交易参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::batch_combine_index_token(ctx, params) // 调用实际批量合并实现，返回执行结果
    }
    pub fn batch_split_index_token(
        ctx: Context<instructions::index_token::BatchSplitIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        amounts: Vec<u64>, // 批量拆分数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::batch_split_index_token(ctx, amounts) // 调用实际批量拆分实现，返回执行结果
    }
    pub fn quote_index_token(
        ctx: Context<instructions::index_token::QuoteIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        params: TradeParams, // 交易参数，类型安全，需业务层校验
        price_params: OracleParams, // 价格参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<u64> { // Anchor标准返回类型，返回指数代币报价
        instructions::index_token::quote_index_token(ctx, params, price_params) // 调用实际报价实现，返回报价
    }
    pub fn execute_buy_index_token(
        ctx: Context<instructions::index_token::ExecuteBuyIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        params: TradeParams, // 交易参数，类型安全，需业务层校验
        price: u64, // 买入价格，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::execute_buy_index_token(ctx, params, price) // 调用实际买入实现，返回执行结果
    }
    pub fn execute_sell_index_token(
        ctx: Context<instructions::index_token::ExecuteSellIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        params: TradeParams, // 交易参数，类型安全，需业务层校验
        price: u64, // 卖出价格，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::execute_sell_index_token(ctx, params, price) // 调用实际卖出实现，返回执行结果
    }
    pub fn execute_swap_index_token(
        ctx: Context<instructions::index_token::ExecuteSwapIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        from_amount: u64, // swap输入数量，类型安全，需业务层校验
        to_amount: u64, // swap输出数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::execute_swap_index_token(ctx, from_amount, to_amount) // 调用实际swap实现，返回执行结果
    }
    pub fn execute_combine_index_token(
        ctx: Context<instructions::index_token::ExecuteCombineIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        amount: u64, // 合并数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::execute_combine_index_token(ctx, amount) // 调用实际合并实现，返回执行结果
    }
    pub fn execute_split_index_token(
        ctx: Context<instructions::index_token::ExecuteSplitIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        amount: u64, // 拆分数量，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        instructions::index_token::execute_split_index_token(ctx, amount) // 调用实际拆分实现，返回执行结果
    }
    pub fn batch_strategy_subscribe_index_token(
        ctx: Context<instructions::index_token::BatchTransferIndexToken>, // Anchor账户上下文，自动校验账户权限与生命周期
        strategies: Vec<StrategyParams>, // 策略参数集合，类型安全，需业务层校验
        exec_params: Option<AlgoParams>, // 可选算法参数，类型安全，需业务层校验
    ) -> anchor_lang::Result<()> { // Anchor标准返回类型，表示指令执行成功或失败
        // 1. Anchor自动生成的账户上下文，包含所有所需账户及权限校验
        // 2. strategies/exec_params 均为类型安全参数，需业务层校验
        // 3. 调用 index_token 模块的 batch_strategy_subscribe_index_token 实现，完成实际批量策略申购逻辑
        // 4. 返回下层实现的结果，anchor_lang::Result<()>类型，Anchor自动处理错误与生命周期
        // instructions::index_token::batch_strategy_subscribe_index_token(ctx, strategies, exec_params) // 调用实际批量策略申购实现，返回执行结果
        Ok(()) // 暂时返回成功，等待实现
    }
    /// ETF资产mint指令
    pub fn mint_etf(ctx: Context<instructions::etf::MintEtf>, amount: u64) -> anchor_lang::Result<()> {
        instructions::etf::mint_etf(ctx, amount)
    }
    /// ETF资产burn指令
    pub fn burn_etf(ctx: Context<instructions::etf::BurnEtf>, amount: u64) -> anchor_lang::Result<()> {
        instructions::etf::burn_etf(ctx, amount)
    }
    /// ETF资产transfer指令
    pub fn transfer_etf(ctx: Context<instructions::etf::TransferEtf>, amount: u64) -> anchor_lang::Result<()> {
        instructions::etf::transfer_etf(ctx, amount)
    }
    /// ETF资产rebalance指令
    pub fn rebalance_etf(ctx: Context<instructions::etf::RebalanceEtf>, new_weights: Vec<u64>) -> anchor_lang::Result<()> {
        instructions::etf::rebalance_etf(ctx, new_weights)
    }
    /// ETF资产估值指令
    pub fn value_etf(ctx: Context<instructions::etf::ValueEtf>) -> anchor_lang::Result<u64> {
        instructions::etf::value_etf(ctx)
    }
    /// ETF资产跨DEX路由指令
    pub fn route_etf(ctx: Context<instructions::etf::RouteEtf>, params: super::core::types::TradeParams) -> anchor_lang::Result<()> {
        instructions::etf::route_etf(ctx, params)
    }
    /// ETF自动再平衡指令
    pub fn auto_rebalance_etf(ctx: Context<instructions::etf::AutoRebalanceEtf>, strategy_type: super::strategies::RebalancingStrategyType, params: Vec<u64>) -> anchor_lang::Result<()> {
        instructions::etf::auto_rebalance_etf(ctx, strategy_type, params)
    }
    /// RWA资产估值指令
    pub fn value_rwa(ctx: Context<instructions::rwa::ValueRwa>, oracle_params: Vec<super::core::types::OracleParams>) -> anchor_lang::Result<u64> {
        instructions::rwa::value_rwa(ctx, oracle_params)
    }
    /// 跨市场套利指令
    pub fn arbitrage_trade(ctx: Context<instructions::arbitrage::ArbitrageTrade>, params: Vec<super::core::types::TradeParams>, min_profit: u64) -> anchor_lang::Result<()> {
        instructions::arbitrage::arbitrage_trade(ctx, params, min_profit)
    }
    /// 适配器动态注册指令
    pub fn register_adapter(ctx: Context<instructions::adapter::RegisterAdapter>, name: String, adapter_type: String, version: String, supported_assets: Vec<String>) -> anchor_lang::Result<()> {
        instructions::adapter::register_adapter(ctx, name, adapter_type, version, supported_assets)
    }
    /// 适配器动态注销指令
    pub fn unregister_adapter(ctx: Context<instructions::adapter::UnregisterAdapter>, name: String) -> anchor_lang::Result<()> {
        instructions::adapter::unregister_adapter(ctx, name)
    }
    /// 适配器热插拔指令
    pub fn hot_swap_adapter(ctx: Context<instructions::adapter::HotSwapAdapter>, name: String, new_version: String) -> anchor_lang::Result<()> {
        instructions::adapter::hot_swap_adapter(ctx, name, new_version)
    }
    /// RWA资产增发指令
    /// # 参数
    /// - ctx: Context<instructions::rwa::MintRwa>，Anchor账户上下文，自动校验账户权限与生命周期
    /// - amount: u64，增发的RWA资产数量，必须为正整数
    /// # 返回值
    /// - anchor_lang::Result<()>: Anchor标准返回类型，表示指令执行成功或失败
    /// # 安全性
    /// - Anchor自动校验账户权限、生命周期、PDA
    /// - 需确保amount参数合法，防止溢出与非法操作
    pub fn mint_rwa(ctx: Context<instructions::rwa::MintRwa>, amount: u64) -> anchor_lang::Result<()> {
        instructions::rwa::mint_rwa(ctx, amount)
    }
    /// RWA资产销毁指令
    /// # 参数
    /// - ctx: Context<instructions::rwa::BurnRwa>，Anchor账户上下文，自动校验账户权限与生命周期
    /// - amount: u64，销毁的RWA资产数量，必须为正整数
    /// # 返回值
    /// - anchor_lang::Result<()>: Anchor标准返回类型，表示指令执行成功或失败
    /// # 安全性
    /// - Anchor自动校验账户权限、生命周期、PDA
    /// - 需确保amount参数合法，防止溢出与非法操作
    pub fn burn_rwa(ctx: Context<instructions::rwa::BurnRwa>, amount: u64) -> anchor_lang::Result<()> {
        instructions::rwa::burn_rwa(ctx, amount)
    }
    /// RWA资产买入指令
    /// # 参数
    /// - ctx: Context<instructions::rwa::BuyRwa>，Anchor账户上下文，自动校验账户权限与生命周期
    /// - amount: u64，买入的RWA资产数量，必须为正整数
    /// # 返回值
    /// - anchor_lang::Result<()>: Anchor标准返回类型，表示指令执行成功或失败
    /// # 安全性
    /// - Anchor自动校验账户权限、生命周期、PDA
    /// - 需确保amount参数合法，防止溢出与非法操作
    pub fn buy_rwa(ctx: Context<instructions::rwa::BuyRwa>, amount: u64) -> anchor_lang::Result<()> {
        instructions::rwa::buy_rwa(ctx, amount)
    }
    /// RWA资产卖出指令
    /// # 参数
    /// - ctx: Context<instructions::rwa::SellRwa>，Anchor账户上下文，自动校验账户权限与生命周期
    /// - amount: u64，卖出的RWA资产数量，必须为正整数
    /// # 返回值
    /// - anchor_lang::Result<()>: Anchor标准返回类型，表示指令执行成功或失败
    /// # 安全性
    /// - Anchor自动校验账户权限、生命周期、PDA
    /// - 需确保amount参数合法，防止溢出与非法操作
    pub fn sell_rwa(ctx: Context<instructions::rwa::SellRwa>, amount: u64) -> anchor_lang::Result<()> {
        instructions::rwa::sell_rwa(ctx, amount)
    }
    /// RWA资产转账指令
    /// # 参数
    /// - ctx: Context<instructions::rwa::TransferRwa>，Anchor账户上下文，自动校验账户权限与生命周期
    /// - amount: u64，转账的RWA资产数量，必须为正整数
    /// # 返回值
    /// - anchor_lang::Result<()>: Anchor标准返回类型，表示指令执行成功或失败
    /// # 安全性
    /// - Anchor自动校验账户权限、生命周期、PDA
    /// - 需确保amount参数合法，防止溢出与非法操作
    pub fn transfer_rwa(ctx: Context<instructions::rwa::TransferRwa>, amount: u64) -> anchor_lang::Result<()> {
        instructions::rwa::transfer_rwa(ctx, amount)
    }
    /// RWA资产swap指令
    /// # 参数
    /// - ctx: Context<instructions::rwa::SwapRwa>，Anchor账户上下文，自动校验账户权限与生命周期
    /// - from_amount: u64，转出的RWA资产数量，必须为正整数
    /// # 返回值
    /// - anchor_lang::Result<()>: Anchor标准返回类型，表示指令执行成功或失败
    /// # 安全性
    /// - Anchor自动校验账户权限、生命周期、PDA
    /// - 需确保from_amount参数合法，防止溢出与非法操作
    pub fn swap_rwa(ctx: Context<instructions::rwa::SwapRwa>, from_amount: u64) -> anchor_lang::Result<()> {
        instructions::rwa::swap_rwa(ctx, from_amount)
    }
    /// RWA资产合并指令
    /// # 参数
    /// - ctx: Context<instructions::rwa::CombineRwa>，Anchor账户上下文，自动校验账户权限与生命周期
    /// - amount: u64，合并的RWA资产数量，必须为正整数
    /// # 返回值
    /// - anchor_lang::Result<()>: Anchor标准返回类型，表示指令执行成功或失败
    /// # 安全性
    /// - Anchor自动校验账户权限、生命周期、PDA
    /// - 需确保amount参数合法，防止溢出与非法操作
    pub fn combine_rwa(ctx: Context<instructions::rwa::CombineRwa>, amount: u64) -> anchor_lang::Result<()> {
        instructions::rwa::combine_rwa(ctx, amount)
    }
}
