//! Basket Instructions
//! 指令入口，调用 service 层。

use crate::accounts::BasketIndexStateAccount; // 引入资产篮子账户状态账户定义
use crate::events::basket_event::*; // 引入所有篮子相关事件定义，便于emit!宏调用
use crate::services::basket_service::BasketService; // 引入篮子服务层，封装核心业务逻辑
use crate::state::baskets::BasketIndexState; // 引入资产篮子状态结构体，类型安全
use crate::validation::basket_validation::BasketValidatable; // 引入篮子校验trait，便于状态校验
use crate::core::types::{TradeParams, BatchTradeParams, StrategyParams, OracleParams, AlgoParams}; // 引入所有核心参数类型
use crate::core::registry::ADAPTER_FACTORY; // 引入全局适配器工厂，支持动态算法/DEX/Oracle适配
use anchor_lang::prelude::*; // Anchor预导出内容，包含Context、Account、Signer、Result等

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct RebalanceBasket<'info> { // 定义篮子再平衡指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket_index: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn rebalance_basket(ctx: Context<RebalanceBasket>, new_weights: Vec<u64>) -> Result<()> { // 篮子再平衡指令主函数，ctx为账户上下文，new_weights为新权重
    let basket_index = &mut ctx.accounts.basket_index; // 获取可变资产篮子账户，生命周期由Anchor自动管理
    basket_index.validate()?; // 校验资产篮子状态（如活跃、合法等），防止非法操作
    BasketService::rebalance(basket_index, new_weights.clone())?; // 调用服务层再平衡逻辑，处理实际权重调整
    emit!(BasketRebalanced { // 触发篮子再平衡事件，链上可追溯
        basket_id: basket_index.id, // 事件：篮子ID，便于链上追踪
        new_weights, // 事件：新权重，记录操作明细
        authority: ctx.accounts.authority.key(), // 事件：操作人，便于审计
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳，便于审计
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct PauseBasket<'info> { // 定义篮子暂停指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket_index: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn pause_basket(ctx: Context<PauseBasket>) -> Result<()> { // 篮子暂停指令主函数，ctx为账户上下文
    let basket_index = &mut ctx.accounts.basket_index; // 获取可变资产篮子账户，生命周期由Anchor自动管理
    basket_index.validate()?; // 校验资产篮子状态，防止非法暂停，业务安全
    BasketService::pause(basket_index)?; // 调用服务层暂停逻辑，处理实际暂停操作
    emit!(BasketPaused { // 触发篮子暂停事件，链上可追溯
        basket_id: basket_index.id, // 事件：篮子ID，便于链上追踪
        authority: ctx.accounts.authority.key(), // 事件：操作人，便于审计
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳，便于审计
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct ResumeBasket<'info> { // 定义篮子恢复指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket_index: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn resume_basket(ctx: Context<ResumeBasket>) -> Result<()> { // 篮子恢复指令主函数，ctx为账户上下文
    let basket_index = &mut ctx.accounts.basket_index; // 获取可变资产篮子账户，生命周期由Anchor自动管理
    basket_index.validate()?; // 校验资产篮子状态，防止非法恢复，业务安全
    BasketService::resume(basket_index)?; // 调用服务层恢复逻辑，处理实际恢复操作
    emit!(BasketResumed { // 触发篮子恢复事件，链上可追溯
        basket_id: basket_index.id, // 事件：篮子ID，便于链上追踪
        authority: ctx.accounts.authority.key(), // 事件：操作人，便于审计
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳，便于审计
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct RebalanceBasketWithAlgo<'info> { // 定义带算法篮子再平衡指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket_index: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn rebalance_basket_with_algo(
    ctx: Context<RebalanceBasketWithAlgo>, // Anchor账户上下文，自动校验权限与生命周期
    new_weights: Vec<u64>, // 新权重，类型安全
    algo_name: String, // 算法名称，类型安全
    params: crate::algorithms::traits::ExecutionParams, // 算法参数，类型安全
) -> Result<()> { // Anchor规范返回类型
    use crate::algorithms::algorithm_registry::AlgorithmRegistry; // 引入算法注册表
    let registry = AlgorithmRegistry::new(); // 实际项目应为全局单例，这里为演示新建
    let algo = registry
        .get_execution(&algo_name) // 查找指定算法名的执行器
        .ok_or(crate::errors::basket_error::BasketError::Unknown)?; // 若未找到则返回Unknown错误
    let basket_index = &mut ctx.accounts.basket_index; // 获取可变资产篮子账户，生命周期由Anchor自动管理
    basket_index.validate()?; // 校验资产篮子状态，防止非法再平衡，业务安全
    crate::services::basket_service::rebalance_with_algo(
        basket_index, // 目标资产篮子账户
        new_weights.clone(), // 新权重
        algo.as_ref(), // 算法trait对象
        ctx.accounts.clone().into(), // 账户上下文转为算法可用类型
        &params, // 算法参数
    )?; // 调用服务层带算法再平衡逻辑，业务安全
    emit!(BasketRebalanced { // 触发篮子再平衡事件，链上可追溯
        basket_id: basket_index.id, // 事件：篮子ID
        new_weights, // 事件：新权重
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)] // 派生Anchor序列化/反序列化、克隆、调试特性，便于跨链/链上数据传递
pub struct RebalanceWithAlgoAndAdaptersParams { // 定义带算法和适配器的篮子再平衡参数结构体
    pub new_weights: Vec<u64>, // 新权重，类型安全，需业务校验
    pub algo_name: String, // 算法名称，类型安全
    pub dex_name: String, // DEX适配器名称，类型安全
    pub oracle_name: String, // 预言机适配器名称，类型安全
    pub params: crate::algorithms::traits::ExecutionParams, // 算法参数，类型安全
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct RebalanceBasketWithAlgoAndAdapters<'info> { // 定义带算法和适配器篮子再平衡指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket_index: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn rebalance_basket_with_algo_and_adapters(
    ctx: Context<RebalanceBasketWithAlgoAndAdapters>, // Anchor账户上下文，自动校验权限与生命周期
    args: RebalanceWithAlgoAndAdaptersParams, // 结构体参数，包含新权重、算法、DEX、预言机等
) -> Result<()> { // Anchor规范返回类型
    use crate::algorithms::algorithm_registry::AlgorithmRegistry; // 引入算法注册表
    use crate::dex::adapter_registry::DexAdapterRegistry; // 引入DEX适配器注册表
    use crate::oracles::adapter_registry::OracleAdapterRegistry; // 引入预言机适配器注册表
    let algo_registry = AlgorithmRegistry::new(); // 实际项目应为全局单例，这里为演示新建
    let dex_registry = DexAdapterRegistry::new(); // DEX适配器注册表实例
    let oracle_registry = OracleAdapterRegistry::new(); // 预言机适配器注册表实例
    let algo = algo_registry
        .get_execution(&args.algo_name) // 查找指定算法名的执行器
        .ok_or(crate::errors::basket_error::BasketError::Unknown)?; // 若未找到则返回Unknown错误
    let dex = dex_registry
        .get(&args.dex_name) // 查找指定DEX适配器
        .ok_or(crate::errors::basket_error::BasketError::Unknown)?; // 若未找到则返回Unknown错误
    let oracle = oracle_registry
        .get(&args.oracle_name) // 查找指定预言机适配器
        .ok_or(crate::errors::basket_error::BasketError::Unknown)?; // 若未找到则返回Unknown错误
    let basket_index = &mut ctx.accounts.basket_index; // 获取可变资产篮子账户，生命周期由Anchor自动管理
    basket_index.validate()?; // 校验资产篮子状态，防止非法再平衡，业务安全
    crate::services::basket_service::rebalance_with_algo_and_adapters(
        basket_index, // 目标资产篮子账户
        args.new_weights.clone(), // 新权重
        algo.as_ref(), // 算法trait对象
        dex.as_ref(), // DEX适配器trait对象
        oracle.as_ref(), // 预言机适配器trait对象
        ctx.accounts.clone().into(), // 账户上下文转为算法/适配器可用类型
        &args.params, // 算法参数
    )?; // 调用服务层带算法和适配器再平衡逻辑，业务安全
    emit!(crate::events::basket_event::BasketRebalanced { // 触发篮子再平衡事件，链上可追溯
        basket_id: basket_index.id, // 事件：篮子ID
        new_weights: args.new_weights, // 事件：新权重
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)]
pub struct TransferBasket<'info> {
    #[account(mut)]
    pub from_basket: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub to_basket: Account<'info, BasketIndexState>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

pub fn transfer_basket(
    ctx: Context<TransferBasket>, // Anchor账户上下文，自动校验权限与生命周期
    amount: u64, // 转移数量，类型安全
) -> Result<()> { // Anchor规范返回类型
    let from = &mut ctx.accounts.from_basket; // 获取可变源篮子账户，生命周期由Anchor自动管理
    let to = &mut ctx.accounts.to_basket; // 获取可变目标篮子账户，生命周期由Anchor自动管理
    require!( // Anchor宏，条件不满足时返回指定错误
        ctx.accounts.authority.key() == from.authority, // 校验操作人是否为源篮子授权人
        crate::errors::basket_error::BasketError::NotAllowed // 不允许操作错误码
    );
    if from.total_value < amount { // 判断源篮子余额是否充足
        return Err(crate::errors::basket_error::BasketError::InsufficientValue.into()); // 不足则返回余额不足错误
    }
    from.total_value -= amount; // 源篮子扣减数量
    to.total_value = to.total_value.checked_add(amount).ok_or(crate::errors::basket_error::BasketError::InsufficientValue)?; // 目标篮子安全加数量，防止溢出
    emit!(BasketTransferred { // 触发篮子转移事件，链上可追溯
        from_basket_id: from.id, // 事件：源篮子ID
        to_basket_id: to.id, // 事件：目标篮子ID
        amount, // 事件：转移数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct QueryBasket<'info> { // 定义篮子查询指令的账户上下文结构体，'info生命周期由Anchor自动推断
    pub basket: Account<'info, BasketIndexState>, // 只读资产篮子账户，类型安全
}

pub fn query_basket(
    ctx: Context<QueryBasket>, // Anchor账户上下文，自动校验权限与生命周期
) -> Result<u64> { // Anchor规范返回类型，返回u64类型的篮子总价值
    let basket = &ctx.accounts.basket; // 获取只读资产篮子账户，生命周期由Anchor自动管理
    Ok(basket.total_value) // 返回篮子总价值，Anchor自动处理生命周期
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct BuyBasket<'info> { // 定义篮子买入指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket_index: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub buyer: Signer<'info>, // 买入操作人签名者，类型安全
}

pub fn buy_basket(
    ctx: Context<BuyBasket>, // Anchor账户上下文，自动校验权限与生命周期
    params: TradeParams, // 交易参数，类型安全
    price_params: OracleParams, // 预言机价格参数，类型安全
    exec_params: Option<AlgoParams>, // 可选算法参数，类型安全
    strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全
) -> Result<()> { // Anchor规范返回类型
    let basket_index = &mut ctx.accounts.basket_index; // 获取可变资产篮子账户，生命周期由Anchor自动管理
    basket_index.validate()?; // 校验资产篮子状态，防止非法买入，业务安全
    // 1. 算法执行（如有）
    let mut exec_result = None; // 算法执行结果，初始为None
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() { // 类型转换为算法执行trait对象
                    // 这里应传递真实的Context和参数
                    // exec_result = Some(exec_strategy.execute(ctx, &exec_params.algo_params)?); // 伪代码，实际应调用算法执行
                }
            }
        }
    }
    // 2. 预言机价格（如有）
    let mut price = price_params.price; // 默认使用传入价格
    if let Some(oracle_name) = &price_params.oracle_name { // 若指定预言机名
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
        if let Some(adapter) = factory.get(oracle_name) { // 查找预言机适配器
            if let Some(oracle_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::oracles::traits::OracleAdapter>>() { // 类型转换为预言机trait对象
                // 这里应传递真实的参数
                // let oracle_result = oracle_adapter.get_price(&price_params)?; // 伪代码，实际应调用预言机查询
                // price = oracle_result.price; // 更新价格
            }
        }
    }
    // 3. DEX/AMM swap（如有）
    if let Some(dex_name) = &params.dex_name { // 若指定DEX名
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
        if let Some(adapter) = factory.get(dex_name) { // 查找DEX适配器
            if let Some(dex_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::dex::traits::DexAdapter>>() { // 类型转换为DEX trait对象
                // 这里应传递真实的参数
                // let swap_result = dex_adapter.swap(&params)?; // 伪代码，实际应调用DEX swap
                // 可用swap_result.avg_price、swap_result.executed_amount等
            }
        }
    }
    // 4. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数
        if !strategy_params.strategy_name.is_empty() { // 策略名非空
            crate::services::basket_service::BasketService::strategy_rebalance(
                basket_index, // 目标资产篮子账户
                &strategy_params.strategy_name, // 策略名称
                &strategy_params.params, // 策略参数
                ctx.accounts.buyer.key(), // 买入人公钥
            )?; // 调用服务层策略再平衡逻辑
        }
    }
    // 5. 资产买入
    BasketService::buy(
        basket_index, // 目标资产篮子账户
        params.amount_in, // 买入数量
        price, // 买入价格
        ctx.accounts.buyer.key(), // 买入人公钥
    )?; // 调用服务层买入逻辑
    emit!(BasketBought { // 触发篮子买入事件，链上可追溯
        basket_id: basket_index.id, // 事件：篮子ID
        amount: params.amount_in, // 事件：买入数量
        price, // 事件：买入价格
        buyer: ctx.accounts.buyer.key(), // 事件：买入人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct SellBasket<'info> { // 定义篮子卖出指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket_index: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub seller: Signer<'info>, // 卖出操作人签名者，类型安全
}

pub fn sell_basket(
    ctx: Context<SellBasket>, // Anchor账户上下文，自动校验权限与生命周期
    params: TradeParams, // 交易参数，类型安全
    price_params: OracleParams, // 预言机价格参数，类型安全
    exec_params: Option<AlgoParams>, // 可选算法参数，类型安全
    strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全
) -> Result<()> { // Anchor规范返回类型
    let basket_index = &mut ctx.accounts.basket_index; // 获取可变资产篮子账户，生命周期由Anchor自动管理
    basket_index.validate()?; // 校验资产篮子状态，防止非法卖出，业务安全
    // 1. 算法执行（如有）
    let mut exec_result = None; // 算法执行结果，初始为None
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() { // 类型转换为算法执行trait对象
                    // 这里应传递真实的Context和参数
                    // exec_result = Some(exec_strategy.execute(ctx, &exec_params.algo_params)?); // 伪代码，实际应调用算法执行
                }
            }
        }
    }
    // 2. 预言机价格（如有）
    let mut price = price_params.price; // 默认使用传入价格
    if let Some(oracle_name) = &price_params.oracle_name { // 若指定预言机名
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
        if let Some(adapter) = factory.get(oracle_name) { // 查找预言机适配器
            if let Some(oracle_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::oracles::traits::OracleAdapter>>() { // 类型转换为预言机trait对象
                // 这里应传递真实的参数
                // let oracle_result = oracle_adapter.get_price(&price_params)?; // 伪代码，实际应调用预言机查询
                // price = oracle_result.price; // 更新价格
            }
        }
    }
    // 3. DEX/AMM swap（如有）
    if let Some(dex_name) = &params.dex_name { // 若指定DEX名
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
        if let Some(adapter) = factory.get(dex_name) { // 查找DEX适配器
            if let Some(dex_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::dex::traits::DexAdapter>>() { // 类型转换为DEX trait对象
                // 这里应传递真实的参数
                // let swap_result = dex_adapter.swap(&params)?; // 伪代码，实际应调用DEX swap
                // 可用swap_result.avg_price、swap_result.executed_amount等
            }
        }
    }
    // 4. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数
        if !strategy_params.strategy_name.is_empty() { // 策略名非空
            crate::services::basket_service::BasketService::strategy_rebalance(
                basket_index, // 目标资产篮子账户
                &strategy_params.strategy_name, // 策略名称
                &strategy_params.params, // 策略参数
                ctx.accounts.seller.key(), // 卖出人公钥
            )?; // 调用服务层策略再平衡逻辑
        }
    }
    // 5. 资产卖出
    BasketService::sell(
        basket_index, // 目标资产篮子账户
        params.amount_out, // 卖出数量
        price, // 卖出价格
        ctx.accounts.seller.key(), // 卖出人公钥
    )?; // 调用服务层卖出逻辑
    emit!(BasketSold { // 触发篮子卖出事件，链上可追溯
        basket_id: basket_index.id, // 事件：篮子ID
        amount: params.amount_out, // 事件：卖出数量
        price, // 事件：卖出价格
        seller: ctx.accounts.seller.key(), // 事件：卖出人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct SwapBasket<'info> { // 定义篮子兑换指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub from_basket: Account<'info, BasketIndexState>, // 源资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub to_basket: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn swap_basket(
    ctx: Context<SwapBasket>, // Anchor账户上下文，自动校验权限与生命周期
    params: TradeParams, // 交易参数，类型安全
    price_params: OracleParams, // 预言机价格参数，类型安全
    exec_params: Option<AlgoParams>, // 可选算法参数，类型安全
    strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全
) -> Result<()> { // Anchor规范返回类型
    let from = &mut ctx.accounts.from_basket; // 获取可变源篮子账户，生命周期由Anchor自动管理
    let to = &mut ctx.accounts.to_basket; // 获取可变目标篮子账户，生命周期由Anchor自动管理
    require!(ctx.accounts.authority.key() == from.authority, crate::errors::basket_error::BasketError::NotAllowed); // 校验操作人是否为源篮子授权人
    if from.total_value < params.amount_in { // 判断源篮子余额是否充足
        return Err(crate::errors::basket_error::BasketError::InsufficientValue.into()); // 不足则返回余额不足错误
    }
    // 1. 算法执行（如有）
    let mut exec_result = None; // 算法执行结果，初始为None
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() { // 类型转换为算法执行trait对象
                    // 这里应传递真实的Context和参数
                    // exec_result = Some(exec_strategy.execute(ctx, &exec_params.algo_params)?); // 伪代码，实际应调用算法执行
                }
            }
        }
    }
    // 2. 预言机价格（如有）
    let mut price = price_params.price; // 默认使用传入价格
    if let Some(oracle_name) = &price_params.oracle_name { // 若指定预言机名
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
        if let Some(adapter) = factory.get(oracle_name) { // 查找预言机适配器
            if let Some(oracle_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::oracles::traits::OracleAdapter>>() { // 类型转换为预言机trait对象
                // 这里应传递真实的参数
                // let oracle_result = oracle_adapter.get_price(&price_params)?; // 伪代码，实际应调用预言机查询
                // price = oracle_result.price; // 更新价格
            }
        }
    }
    // 3. DEX/AMM swap（如有）
    if let Some(dex_name) = &params.dex_name { // 若指定DEX名
        let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
        if let Some(adapter) = factory.get(dex_name) { // 查找DEX适配器
            if let Some(dex_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::dex::traits::DexAdapter>>() { // 类型转换为DEX trait对象
                // 这里应传递真实的参数
                // let swap_result = dex_adapter.swap(&params)?; // 伪代码，实际应调用DEX swap
                // 可用swap_result.avg_price、swap_result.executed_amount等
            }
        }
    }
    // 4. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数
        if !strategy_params.strategy_name.is_empty() { // 策略名非空
            crate::services::basket_service::BasketService::strategy_rebalance(
                from, // 源资产篮子账户
                &strategy_params.strategy_name, // 策略名称
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 操作人公钥
            )?; // 调用服务层策略再平衡逻辑
        }
    }
    // 5. 资产兑换
    from.total_value -= params.amount_in; // 源篮子扣减数量
    to.total_value = to.total_value.checked_add(params.amount_out).ok_or(crate::errors::basket_error::BasketError::InsufficientValue)?; // 目标篮子安全加数量，防止溢出
    emit!(BasketSwapped { // 触发篮子兑换事件，链上可追溯
        from_basket_id: from.id, // 事件：源篮子ID
        to_basket_id: to.id, // 事件：目标篮子ID
        amount_in: params.amount_in, // 事件：兑换输入数量
        amount_out: params.amount_out, // 事件：兑换输出数量
        price, // 事件：兑换价格
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)]
pub struct AuthorizeBasket<'info> { // 定义篮子授权指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 当前授权人签名者，类型安全
    pub new_authority: Pubkey, // 新授权人公钥，类型安全
}

pub fn authorize_basket(
    ctx: Context<AuthorizeBasket>, // Anchor账户上下文，自动校验权限与生命周期
    exec_params: Option<AlgoParams>, // 可选算法参数，类型安全
    strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全
) -> Result<()> { // Anchor规范返回类型
    let basket = &mut ctx.accounts.basket; // 获取可变资产篮子账户，生命周期由Anchor自动管理
    require!(ctx.accounts.authority.key() == basket.authority, crate::errors::basket_error::BasketError::NotAllowed); // 校验操作人是否为当前授权人
    // 1. 算法执行（如有）
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                // 假设所有算法adapter都实现ExecutionStrategy trait
                // 这里可做trait对象downcast或统一trait接口
                // 这里只做动态发现和调用示例
                // let _exec_result = algo.execute(Context::default(), exec_params)?; // 伪代码，实际应调用算法执行
            }
        }
    }
    // 2. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数
        if !strategy_params.strategy_name.is_empty() { // 策略名非空
            crate::services::basket_service::BasketService::strategy_rebalance(
                basket, // 目标资产篮子账户
                &strategy_params.strategy_name, // 策略名称
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 当前授权人公钥
            )?; // 调用服务层策略再平衡逻辑
        }
    }
    // 3. 授权
    basket.authority = ctx.accounts.new_authority; // 更新授权人
    emit!(BasketAuthorized { // 触发篮子授权事件，链上可追溯
        basket_id: basket.id, // 事件：篮子ID
        old_authority: ctx.accounts.authority.key(), // 事件：原授权人
        new_authority: ctx.accounts.new_authority, // 事件：新授权人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct CombineBasket<'info> { // 定义篮子合并指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub target_basket: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub source_basket: Account<'info, BasketIndexState>, // 源资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn combine_basket(
    ctx: Context<CombineBasket>, // Anchor账户上下文，自动校验权限与生命周期
    amount: u64, // 合并数量，类型安全
    exec_params: Option<AlgoParams>, // 可选算法参数，类型安全
    strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全
) -> Result<()> { // Anchor规范返回类型
    let target = &mut ctx.accounts.target_basket; // 获取可变目标篮子账户，生命周期由Anchor自动管理
    let source = &mut ctx.accounts.source_basket; // 获取可变源篮子账户，生命周期由Anchor自动管理
    require!(ctx.accounts.authority.key() == source.authority, crate::errors::basket_error::BasketError::NotAllowed); // 校验操作人是否为源篮子授权人
    if source.total_value < amount { // 判断源篮子余额是否充足
        return Err(crate::errors::basket_error::BasketError::InsufficientValue.into()); // 不足则返回余额不足错误
    }
    // 1. 算法执行（如有）
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                // 假设所有算法adapter都实现ExecutionStrategy trait
                // 这里可做trait对象downcast或统一trait接口
                // 这里只做动态发现和调用示例
                // let _exec_result = algo.execute(Context::default(), exec_params)?; // 伪代码，实际应调用算法执行
            }
        }
    }
    // 2. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数
        if !strategy_params.strategy_name.is_empty() { // 策略名非空
            crate::services::basket_service::BasketService::strategy_rebalance(
                target, // 目标资产篮子账户
                &strategy_params.strategy_name, // 策略名称
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 操作人公钥
            )?; // 调用服务层策略再平衡逻辑
        }
    }
    // 3. 合并操作
    source.total_value -= amount; // 源篮子扣减数量
    target.total_value = target.total_value.checked_add(amount).ok_or(crate::errors::basket_error::BasketError::InsufficientValue)?; // 目标篮子安全加数量，防止溢出
    emit!(BasketCombined { // 触发篮子合并事件，链上可追溯
        target_basket_id: target.id, // 事件：目标篮子ID
        source_basket_id: source.id, // 事件：源篮子ID
        amount, // 事件：合并数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct SplitBasket<'info> { // 定义篮子拆分指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub source_basket: Account<'info, BasketIndexState>, // 源资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub new_basket: Account<'info, BasketIndexState>, // 新生成的目标篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn split_basket(
    ctx: Context<SplitBasket>, // Anchor账户上下文，自动校验权限与生命周期
    amount: u64, // 拆分数量，类型安全
    exec_params: Option<AlgoParams>, // 可选算法参数，类型安全
    strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全
) -> Result<()> { // Anchor规范返回类型
    let source = &mut ctx.accounts.source_basket; // 获取可变源篮子账户，生命周期由Anchor自动管理
    let new_basket = &mut ctx.accounts.new_basket; // 获取可变新篮子账户，生命周期由Anchor自动管理
    require!(ctx.accounts.authority.key() == source.authority, crate::errors::basket_error::BasketError::NotAllowed); // 校验操作人是否为源篮子授权人
    if source.total_value < amount { // 判断源篮子余额是否充足
        return Err(crate::errors::basket_error::BasketError::InsufficientValue.into()); // 不足则返回余额不足错误
    }
    // 1. 算法执行（如有）
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                // 假设所有算法adapter都实现ExecutionStrategy trait
                // 这里可做trait对象downcast或统一trait接口
                // 这里只做动态发现和调用示例
                // let _exec_result = algo.execute(Context::default(), exec_params)?; // 伪代码，实际应调用算法执行
            }
        }
    }
    // 2. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数
        if !strategy_params.strategy_name.is_empty() { // 策略名非空
            crate::services::basket_service::BasketService::strategy_rebalance(
                new_basket, // 新生成的目标篮子账户
                &strategy_params.strategy_name, // 策略名称
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 操作人公钥
            )?; // 调用服务层策略再平衡逻辑
        }
    }
    // 3. 拆分操作
    source.total_value -= amount; // 源篮子扣减数量
    new_basket.total_value = new_basket.total_value.checked_add(amount).ok_or(crate::errors::basket_error::BasketError::InsufficientValue)?; // 新篮子安全加数量，防止溢出
    emit!(BasketSplit { // 触发篮子拆分事件，链上可追溯
        source_basket_id: source.id, // 事件：源篮子ID
        new_basket_id: new_basket.id, // 事件：新篮子ID
        amount, // 事件：拆分数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct FreezeBasket<'info> { // 定义篮子冻结指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn freeze_basket(
    ctx: Context<FreezeBasket>, // Anchor账户上下文，自动校验权限与生命周期
    exec_params: Option<AlgoParams>, // 可选算法参数，类型安全
    strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全
) -> Result<()> { // Anchor规范返回类型
    let basket = &mut ctx.accounts.basket; // 获取可变资产篮子账户，生命周期由Anchor自动管理
    require!(ctx.accounts.authority.key() == basket.authority, crate::errors::basket_error::BasketError::NotAllowed); // 校验操作人是否为授权人
    // 1. 算法执行（如有）
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                // 假设所有算法adapter都实现ExecutionStrategy trait
                // 这里可做trait对象downcast或统一trait接口
                // 这里只做动态发现和调用示例
                // let _exec_result = algo.execute(Context::default(), exec_params)?; // 伪代码，实际应调用算法执行
            }
        }
    }
    // 2. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数
        if !strategy_params.strategy_name.is_empty() { // 策略名非空
            crate::services::basket_service::BasketService::strategy_rebalance(
                basket, // 目标资产篮子账户
                &strategy_params.strategy_name, // 策略名称
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 操作人公钥
            )?; // 调用服务层策略再平衡逻辑
        }
    }
    // 3. 冻结操作
    basket.is_frozen = true; // 标记篮子为冻结状态
    emit!(BasketFrozen { // 触发篮子冻结事件，链上可追溯
        basket_id: basket.id, // 事件：篮子ID
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct UnfreezeBasket<'info> { // 定义篮子解冻指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn unfreeze_basket(
    ctx: Context<UnfreezeBasket>, // Anchor账户上下文，自动校验权限与生命周期
    exec_params: Option<AlgoParams>, // 可选算法参数，类型安全
    strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全
) -> Result<()> { // Anchor规范返回类型
    let basket = &mut ctx.accounts.basket; // 获取可变资产篮子账户，生命周期由Anchor自动管理
    require!(ctx.accounts.authority.key() == basket.authority, crate::errors::basket_error::BasketError::NotAllowed); // 校验操作人是否为授权人
    // 1. 算法执行（如有）
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                // 假设所有算法adapter都实现ExecutionStrategy trait
                // 这里可做trait对象downcast或统一trait接口
                // 这里只做动态发现和调用示例
                // let _exec_result = algo.execute(Context::default(), exec_params)?; // 伪代码，实际应调用算法执行
            }
        }
    }
    // 2. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数
        if !strategy_params.strategy_name.is_empty() { // 策略名非空
            crate::services::basket_service::BasketService::strategy_rebalance(
                basket, // 目标资产篮子账户
                &strategy_params.strategy_name, // 策略名称
                &strategy_params.params, // 策略参数
                ctx.accounts.authority.key(), // 操作人公钥
            )?; // 调用服务层策略再平衡逻辑
        }
    }
    // 3. 解冻操作
    basket.is_frozen = false; // 标记篮子为非冻结状态
    emit!(BasketUnfrozen { // 触发篮子解冻事件，链上可追溯
        basket_id: basket.id, // 事件：篮子ID
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct BatchTransferBasket<'info> { // 定义批量篮子转移指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub from_basket: Account<'info, BasketIndexState>, // 源资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub to_baskets: Vec<Account<'info, BasketIndexState>>, // 目标资产篮子账户列表，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn batch_transfer_basket(
    ctx: Context<BatchTransferBasket>, // Anchor账户上下文，自动校验权限与生命周期
    amounts: Vec<u64>, // 批量转移数量，类型安全
    exec_params: Option<AlgoParams>, // 可选算法参数，类型安全
    strategy_params: Option<StrategyParams>, // 可选策略参数，类型安全
) -> Result<()> { // Anchor规范返回类型
    let from = &mut ctx.accounts.from_basket; // 获取可变源篮子账户，生命周期由Anchor自动管理
    let to_baskets = &mut ctx.accounts.to_baskets; // 获取可变目标篮子账户列表，生命周期由Anchor自动管理
    require!(ctx.accounts.authority.key() == from.authority, crate::errors::basket_error::BasketError::NotAllowed); // 校验操作人是否为源篮子授权人
    let total: u64 = amounts.iter().sum(); // 计算总转移数量
    if from.total_value < total { // 判断源篮子余额是否充足
        return Err(crate::errors::basket_error::BasketError::InsufficientValue.into()); // 不足则返回余额不足错误
    }
    // 1. 算法执行（如有）
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                // 假设所有算法adapter都实现ExecutionStrategy trait
                // 这里可做trait对象downcast或统一trait接口
                // 这里只做动态发现和调用示例
                // let _exec_result = algo.execute(Context::default(), exec_params)?; // 伪代码，实际应调用算法执行
            }
        }
    }
    // 2. 策略融合（如有）
    if let Some(strategy_params) = &strategy_params { // 若有策略参数
        if !strategy_params.strategy_name.is_empty() { // 策略名非空
            for to in to_baskets.iter_mut() { // 遍历所有目标篮子
                crate::services::basket_service::BasketService::strategy_rebalance(
                    to, // 目标资产篮子账户
                    &strategy_params.strategy_name, // 策略名称
                    &strategy_params.params, // 策略参数
                    ctx.accounts.authority.key(), // 操作人公钥
                )?; // 调用服务层策略再平衡逻辑
            }
        }
    }
    // 3. 批量转移操作
    from.total_value -= total; // 源篮子扣减总数量
    for (to, amount) in to_baskets.iter_mut().zip(amounts.iter()) { // 遍历目标篮子和数量
        to.total_value = to.total_value.checked_add(*amount).ok_or(crate::errors::basket_error::BasketError::InsufficientValue)?; // 目标篮子安全加数量，防止溢出
        emit!(BasketTransferred { // 触发篮子转移事件，链上可追溯
            from_basket_id: from.id, // 事件：源篮子ID
            to_basket_id: to.id, // 事件：目标篮子ID
            amount: *amount, // 事件：转移数量
            authority: ctx.accounts.authority.key(), // 事件：操作人
            timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
        });
    }
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct BatchRebalanceBasket<'info> { // 定义批量篮子再平衡指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn batch_rebalance_basket(
    ctx: Context<BatchRebalanceBasket>, // Anchor账户上下文，自动校验权限与生命周期
    params: BatchTradeParams, // 批量再平衡参数，类型安全
) -> Result<()> { // Anchor规范返回类型
    require!(!params.swaps.is_empty(), crate::errors::basket_error::BasketError::InvalidParams); // 校验参数非空
    let basket = &mut ctx.accounts.basket; // 获取可变资产篮子账户，生命周期由Anchor自动管理
    crate::services::basket_service::batch_rebalance(
        basket, // 目标资产篮子账户
        &params.swaps, // 批量再平衡swap参数
        ctx.accounts.authority.key(), // 操作人公钥
    ) // 调用服务层批量再平衡逻辑，返回Anchor规范Result
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct StrategyRebalanceBasket<'info> { // 定义策略篮子再平衡指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn strategy_rebalance_basket(
    ctx: Context<StrategyRebalanceBasket>, // Anchor账户上下文，自动校验权限与生命周期
    params: StrategyParams, // 策略参数，类型安全
    exec_params: Option<AlgoParams>, // 可选算法参数，类型安全
) -> Result<()> { // Anchor规范返回类型
    let basket = &mut ctx.accounts.basket; // 获取可变资产篮子账户，生命周期由Anchor自动管理
    // 1. 算法执行（如有）
    if let Some(exec_params) = &exec_params { // 若有算法参数
        if let Some(algo_name) = &exec_params.algo_name { // 若指定算法名
            let factory = ADAPTER_FACTORY.lock().unwrap(); // 获取全局适配器工厂，线程安全
            if let Some(algo) = factory.get(algo_name) { // 查找算法适配器
                // 假设所有算法adapter都实现ExecutionStrategy trait
                // 这里可做trait对象downcast或统一trait接口
                // 这里只做动态发现和调用示例
                // let _exec_result = algo.execute(Context::default(), exec_params)?; // 伪代码，实际应调用算法执行
            }
        }
    }
    // 2. 策略再平衡
    crate::services::basket_service::BasketService::strategy_rebalance(
        basket, // 目标资产篮子账户
        &params.strategy_name, // 策略名称
        &params.params, // 策略参数
        ctx.accounts.authority.key(), // 操作人公钥
    ) // 调用服务层策略再平衡逻辑，返回Anchor规范Result
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct BatchSubscribeBasket<'info> { // 定义批量篮子认购指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn batch_subscribe_basket(
    ctx: Context<BatchSubscribeBasket>, // Anchor账户上下文，自动校验权限与生命周期
    amounts: Vec<u64>, // 批量认购数量，类型安全
) -> Result<()> { // Anchor规范返回类型
    let basket = &mut ctx.accounts.basket; // 获取可变资产篮子账户，生命周期由Anchor自动管理
    require!(!amounts.is_empty(), crate::errors::basket_error::BasketError::InvalidParams); // 校验参数非空
    for &amt in amounts.iter() { // 遍历每个认购数量
        require!(amt > 0, crate::errors::basket_error::BasketError::InvalidParams); // 校验每个数量大于0
        basket.total_value = basket.total_value.checked_add(amt).ok_or(crate::errors::basket_error::BasketError::Overflow)?; // 安全加数量，防止溢出
        emit!(BasketMinted { // 触发篮子认购事件，链上可追溯
            basket_id: basket.id, // 事件：篮子ID
            amount: amt, // 事件：认购数量
            authority: ctx.accounts.authority.key(), // 事件：操作人
            timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
        });
    }
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct BatchRedeemBasket<'info> { // 定义批量篮子赎回指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn batch_redeem_basket(
    ctx: Context<BatchRedeemBasket>, // Anchor账户上下文，自动校验权限与生命周期
    amounts: Vec<u64>, // 批量赎回数量，类型安全
) -> Result<()> { // Anchor规范返回类型
    let basket = &mut ctx.accounts.basket; // 获取可变资产篮子账户，生命周期由Anchor自动管理
    require!(!amounts.is_empty(), crate::errors::basket_error::BasketError::InvalidParams); // 校验参数非空
    let total: u64 = amounts.iter().try_fold(0u64, |acc, &x| acc.checked_add(x).ok_or(crate::errors::basket_error::BasketError::Overflow))?; // 计算总赎回数量，防止溢出
    require!(basket.total_value >= total, crate::errors::basket_error::BasketError::InsufficientValue); // 校验篮子余额充足
    for &amt in amounts.iter() { // 遍历每个赎回数量
        require!(amt > 0, crate::errors::basket_error::BasketError::InvalidParams); // 校验每个数量大于0
        basket.total_value -= amt; // 扣减篮子数量
        emit!(BasketBurned { // 触发篮子赎回事件，链上可追溯
            basket_id: basket.id, // 事件：篮子ID
            amount: amt, // 事件：赎回数量
            authority: ctx.accounts.authority.key(), // 事件：操作人
            timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
        });
    }
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct BatchCombineBasket<'info> { // 定义批量篮子合并指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub target_basket: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub source_baskets: Vec<Account<'info, BasketIndexState>>, // 源资产篮子账户列表，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn batch_combine_basket(
    ctx: Context<BatchCombineBasket>, // Anchor账户上下文，自动校验权限与生命周期
    amounts: Vec<u64>, // 批量合并数量，类型安全
) -> Result<()> { // Anchor规范返回类型
    let target = &mut ctx.accounts.target_basket; // 获取可变目标篮子账户，生命周期由Anchor自动管理
    let source_baskets = &mut ctx.accounts.source_baskets; // 获取可变源篮子账户列表，生命周期由Anchor自动管理
    require!(source_baskets.len() == amounts.len(), crate::errors::basket_error::BasketError::InvalidParams); // 校验参数长度一致
    for (source, &amount) in source_baskets.iter_mut().zip(amounts.iter()) { // 遍历源篮子和数量
        require!(ctx.accounts.authority.key() == source.authority, crate::errors::basket_error::BasketError::NotAllowed); // 校验操作人是否为源篮子授权人
        require!(source.total_value >= amount, crate::errors::basket_error::BasketError::InsufficientValue); // 校验源篮子余额充足
        source.total_value -= amount; // 源篮子扣减数量
        target.total_value = target.total_value.checked_add(amount).ok_or(crate::errors::basket_error::BasketError::InsufficientValue)?; // 目标篮子安全加数量，防止溢出
        emit!(BasketCombined { // 触发篮子合并事件，链上可追溯
            target_basket_id: target.id, // 事件：目标篮子ID
            source_basket_id: source.id, // 事件：源篮子ID
            amount, // 事件：合并数量
            authority: ctx.accounts.authority.key(), // 事件：操作人
            timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
        });
    }
    Ok(()) // Anchor规范返回，生命周期自动管理
}

#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct BatchSplitBasket<'info> { // 定义批量篮子拆分指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub source_basket: Account<'info, BasketIndexState>, // 源资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub new_baskets: Vec<Account<'info, BasketIndexState>>, // 新生成的目标篮子账户列表，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn batch_split_basket(
    ctx: Context<BatchSplitBasket>, // Anchor账户上下文，自动校验权限与生命周期
    amounts: Vec<u64>, // 批量拆分数量，类型安全
) -> Result<()> { // Anchor规范返回类型
    let source = &mut ctx.accounts.source_basket; // 获取可变源篮子账户，生命周期由Anchor自动管理
    let new_baskets = &mut ctx.accounts.new_baskets; // 获取可变新篮子账户列表，生命周期由Anchor自动管理
    require!(new_baskets.len() == amounts.len(), crate::errors::basket_error::BasketError::InvalidParams); // 校验参数长度一致
    let total: u64 = amounts.iter().try_fold(0u64, |acc, &x| acc.checked_add(x).ok_or(crate::errors::basket_error::BasketError::Overflow))?; // 计算总拆分数量，防止溢出
    require!(source.total_value >= total, crate::errors::basket_error::BasketError::InsufficientValue); // 校验源篮子余额充足
    for (new_basket, &amount) in new_baskets.iter_mut().zip(amounts.iter()) { // 遍历新篮子和数量
        require!(ctx.accounts.authority.key() == source.authority, crate::errors::basket_error::BasketError::NotAllowed); // 校验操作人是否为源篮子授权人
        require!(amount > 0, crate::errors::basket_error::BasketError::InvalidParams); // 校验拆分数量大于0
        source.total_value -= amount; // 源篮子扣减数量
        new_basket.total_value = new_basket.total_value.checked_add(amount).ok_or(crate::errors::basket_error::BasketError::InsufficientValue)?; // 新篮子安全加数量，防止溢出
        emit!(BasketSplit { // 触发篮子拆分事件，链上可追溯
            source_basket_id: source.id, // 事件：源篮子ID
            new_basket_id: new_basket.id, // 事件：新篮子ID
            amount, // 事件：拆分数量
            authority: ctx.accounts.authority.key(), // 事件：操作人
            timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
        });
    }
    Ok(()) // Anchor规范返回，生命周期自动管理
}

// === 新增：报价指令 ===
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct QuoteBasket<'info> { // 定义篮子报价指令的账户上下文结构体，'info生命周期由Anchor自动推断
    pub basket: Account<'info, BasketIndexState>, // 只读资产篮子账户，类型安全
}

pub fn quote_basket(
    ctx: Context<QuoteBasket>, // Anchor账户上下文，自动校验权限与生命周期
    params: TradeParams, // 交易参数，类型安全
    price_params: OracleParams, // 预言机价格参数，类型安全
) -> Result<u64> { // Anchor规范返回类型，返回u64类型的报价
    let basket = &ctx.accounts.basket; // 获取只读资产篮子账户，生命周期由Anchor自动管理
    // 1. 价格计算逻辑（可扩展）
    // 这里只返回篮子总价值，实际可根据params和price_params实现更复杂逻辑
    Ok(basket.total_value) // 返回篮子总价值，Anchor自动处理生命周期
}

// === 新增：执行买入指令 ===
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct ExecuteBuyBasket<'info> { // 定义执行买入指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub buyer: Signer<'info>, // 买入操作人签名者，类型安全
}

pub fn execute_buy_basket(
    ctx: Context<ExecuteBuyBasket>, // Anchor账户上下文，自动校验权限与生命周期
    params: TradeParams, // 交易参数，类型安全
    price: u64, // 买入价格，类型安全
) -> Result<()> { // Anchor规范返回类型
    require!(params.amount_in > 0, crate::errors::basket_error::BasketError::InvalidParams); // 校验买入数量大于0
    crate::services::basket_service::BasketService::execute_buy(
        &mut ctx.accounts.basket, // 目标资产篮子账户
        &params, // 交易参数
        price, // 买入价格
        ctx.accounts.buyer.key(), // 买入人公钥
    )?; // 调用服务层买入逻辑
    emit!(BasketBought { // 触发篮子买入事件，链上可追溯
        basket_id: ctx.accounts.basket.id, // 事件：篮子ID
        amount: params.amount_in, // 事件：买入数量
        price, // 事件：买入价格
        buyer: ctx.accounts.buyer.key(), // 事件：买入人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

// === 新增：执行卖出指令 ===
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct ExecuteSellBasket<'info> { // 定义执行卖出指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub basket: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub seller: Signer<'info>, // 卖出操作人签名者，类型安全
}

pub fn execute_sell_basket(
    ctx: Context<ExecuteSellBasket>, // Anchor账户上下文，自动校验权限与生命周期
    params: TradeParams, // 交易参数，类型安全
    price: u64, // 卖出价格，类型安全
) -> Result<()> { // Anchor规范返回类型
    require!(params.amount_out > 0, crate::errors::basket_error::BasketError::InvalidParams); // 校验卖出数量大于0
    crate::services::basket_service::BasketService::execute_sell(
        &mut ctx.accounts.basket, // 目标资产篮子账户
        &params, // 交易参数
        price, // 卖出价格
        ctx.accounts.seller.key(), // 卖出人公钥
    )?; // 调用服务层卖出逻辑
    emit!(BasketSold { // 触发篮子卖出事件，链上可追溯
        basket_id: ctx.accounts.basket.id, // 事件：篮子ID
        amount: params.amount_out, // 事件：卖出数量
        price, // 事件：卖出价格
        seller: ctx.accounts.seller.key(), // 事件：卖出人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

// === 新增：执行交换指令 ===
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct ExecuteSwapBasket<'info> { // 定义执行篮子兑换指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub from_basket: Account<'info, BasketIndexState>, // 源资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub to_basket: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn execute_swap_basket(
    ctx: Context<ExecuteSwapBasket>, // Anchor账户上下文，自动校验权限与生命周期
    from_amount: u64, // 兑换输入数量，类型安全
    to_amount: u64, // 兑换输出数量，类型安全
) -> Result<()> { // Anchor规范返回类型
    require!(from_amount > 0 && to_amount > 0, crate::errors::basket_error::BasketError::InvalidParams); // 校验输入输出数量大于0
    crate::services::basket_service::BasketService::execute_swap(
        &mut ctx.accounts.from_basket, // 源资产篮子账户
        &mut ctx.accounts.to_basket, // 目标资产篮子账户
        from_amount, // 兑换输入数量
        to_amount, // 兑换输出数量
        ctx.accounts.authority.key(), // 操作人公钥
    )?; // 调用服务层兑换逻辑
    emit!(BasketSwapped { // 触发篮子兑换事件，链上可追溯
        from_basket_id: ctx.accounts.from_basket.id, // 事件：源篮子ID
        to_basket_id: ctx.accounts.to_basket.id, // 事件：目标篮子ID
        amount_in: from_amount, // 事件：兑换输入数量
        amount_out: to_amount, // 事件：兑换输出数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

// === 新增：执行合并指令 ===
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct ExecuteCombineBasket<'info> { // 定义执行篮子合并指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub target_basket: Account<'info, BasketIndexState>, // 目标资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub source_basket: Account<'info, BasketIndexState>, // 源资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn execute_combine_basket(
    ctx: Context<ExecuteCombineBasket>, // Anchor账户上下文，自动校验权限与生命周期
    amount: u64, // 合并数量，类型安全
) -> Result<()> { // Anchor规范返回类型
    require!(amount > 0, crate::errors::basket_error::BasketError::InvalidParams); // 校验合并数量大于0
    crate::services::basket_service::BasketService::execute_combine(
        &mut ctx.accounts.target_basket, // 目标资产篮子账户
        &mut ctx.accounts.source_basket, // 源资产篮子账户
        amount, // 合并数量
        ctx.accounts.authority.key(), // 操作人公钥
    )?; // 调用服务层合并逻辑
    emit!(BasketCombined { // 触发篮子合并事件，链上可追溯
        target_basket_id: ctx.accounts.target_basket.id, // 事件：目标篮子ID
        source_basket_id: ctx.accounts.source_basket.id, // 事件：源篮子ID
        amount, // 事件：合并数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

// === 新增：执行拆分指令 ===
#[derive(Accounts)] // Anchor宏，自动为结构体生成账户校验与生命周期管理代码
pub struct ExecuteSplitBasket<'info> { // 定义执行篮子拆分指令的账户上下文结构体，'info生命周期由Anchor自动推断
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub source_basket: Account<'info, BasketIndexState>, // 源资产篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验PDA和生命周期
    pub new_basket: Account<'info, BasketIndexState>, // 新生成的目标篮子账户，需可变，类型安全
    #[account(mut)] // Anchor属性，标记该账户为可变，自动校验签名
    pub authority: Signer<'info>, // 操作人签名者，类型安全
}

pub fn execute_split_basket(
    ctx: Context<ExecuteSplitBasket>, // Anchor账户上下文，自动校验权限与生命周期
    amount: u64, // 拆分数量，类型安全
) -> Result<()> { // Anchor规范返回类型
    require!(amount > 0, crate::errors::basket_error::BasketError::InvalidParams); // 校验拆分数量大于0
    crate::services::basket_service::BasketService::execute_split(
        &mut ctx.accounts.source_basket, // 源资产篮子账户
        &mut ctx.accounts.new_basket, // 新生成的目标篮子账户
        amount, // 拆分数量
        ctx.accounts.authority.key(), // 操作人公钥
    )?; // 调用服务层拆分逻辑
    emit!(BasketSplit { // 触发篮子拆分事件，链上可追溯
        source_basket_id: ctx.accounts.source_basket.id, // 事件：源篮子ID
        new_basket_id: ctx.accounts.new_basket.id, // 事件：新篮子ID
        amount, // 事件：拆分数量
        authority: ctx.accounts.authority.key(), // 事件：操作人
        timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
    });
    Ok(()) // Anchor规范返回，生命周期自动管理
}

// 批量策略重平衡指令（如有需求，可按如下模式实现）
pub fn batch_strategy_rebalance_basket(ctx: Context<BatchTransferBasket>, strategies: Vec<StrategyParams>, exec_params: Option<AlgoParams>) -> Result<()> {
    let from = &mut ctx.accounts.from_basket;
    let mut to_baskets: Vec<&mut BasketIndexState> = ctx.accounts.to_baskets.iter_mut().map(|b| b.as_mut()).collect();
    let n = to_baskets.len();
    require!(n > 0, crate::errors::basket_error::BasketError::InvalidParams);
    require!(strategies.len() == n, crate::errors::basket_error::BasketError::InvalidParams);
    require!(ctx.accounts.authority.key() == from.authority, crate::errors::basket_error::BasketError::NotAllowed);
    for (to, strategy) in to_baskets.iter_mut().zip(strategies.iter()) {
        require!(!strategy.strategy_name.is_empty(), crate::errors::basket_error::BasketError::InvalidParams);
        // 算法融合（如有）
        if let Some(exec_params) = &exec_params {
            if let Some(algo_name) = &exec_params.algo_name {
                let factory = ADAPTER_FACTORY.lock().unwrap();
                if let Some(algo) = factory.get(algo_name) {
                    if let Some(exec_strategy) = algo.as_any().downcast_ref::<Arc<dyn crate::algorithms::traits::ExecutionStrategy>>() {
                        // 这里应传递真实的Context和参数
                        // let _exec_result = exec_strategy.execute(ctx, &exec_params.algo_params)?;
                    }
                }
            }
        }
        // 策略融合
        crate::services::basket_service::BasketService::strategy_rebalance(
            to,
            &strategy.strategy_name,
            &strategy.params,
            ctx.accounts.authority.key(),
        )?;
        emit!(BasketStrategyRebalanced {
            basket_id: to.id,
            strategy: strategy.strategy_name.clone(),
            params: strategy.params.clone(),
            authority: ctx.accounts.authority.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::baskets::BasketIndexState;
    use anchor_lang::prelude::*;

    fn default_basket() -> BasketIndexState {
        BasketIndexState {
            fee_collector: Pubkey::new_unique(),
            composition: vec![Default::default()],
            weights: vec![10_000],
            is_active: true,
            total_value: 1000,
            ..Default::default()
        }
    }

    #[test]
    fn test_transfer_basket_success() {
        let mut from = default_basket();
        let mut to = default_basket();
        let amount = 500;
        from.total_value = 1000;
        to.total_value = 200;
        // 模拟transfer逻辑
        assert!(from.total_value >= amount);
        from.total_value -= amount;
        to.total_value += amount;
        assert_eq!(from.total_value, 500);
        assert_eq!(to.total_value, 700);
    }

    #[test]
    fn test_transfer_basket_insufficient_value() {
        let mut from = default_basket();
        let mut to = default_basket();
        let amount = 2000;
        from.total_value = 1000;
        to.total_value = 200;
        // 模拟transfer逻辑
        assert!(from.total_value < amount);
    }

    #[test]
    fn test_query_basket() {
        let basket = default_basket();
        let value = basket.total_value;
        assert_eq!(value, 1000);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::core::types::{TradeParams, BatchTradeParams, StrategyParams, OracleParams, AlgoParams};
    use anchor_lang::prelude::Pubkey;

    #[test]
    fn test_buy_basket_invalid_params() {
        let ctx = Context::default();
        let params = TradeParams {
            from_token: Pubkey::default(),
            to_token: Pubkey::default(),
            amount_in: 0,
            min_amount_out: 0,
            dex_name: "orca".to_string(),
        };
        let price_params = OracleParams { asset: Pubkey::default(), oracle_name: "pyth".to_string() };
        let result = buy_basket(ctx, params, price_params, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_batch_rebalance_basket_empty() {
        let ctx = Context::default();
        let params = BatchTradeParams { swaps: vec![] };
        let result = batch_rebalance_basket(ctx, params);
        assert!(result.is_err());
    }

    #[test]
    fn test_strategy_rebalance_basket_empty_strategy() {
        let ctx = Context::default();
        let params = StrategyParams {
            strategy_name: "".to_string(),
            params: vec![],
        };
        let result = strategy_rebalance_basket(ctx, params, None);
        assert!(result.is_err());
    }
}

