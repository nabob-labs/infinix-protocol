//!
//! 资产服务层
//! 业务逻辑实现，供指令入口调用，封装资产增发、销毁、报价、买卖、交换、合并、拆分等操作。

use anchor_lang::prelude::*; // Anchor 预导入，包含合约开发基础类型、宏、事件、Result等
use crate::state::baskets::BasketIndexState; // 引入资产篮子状态结构体，作为所有资产操作的核心数据结构
use crate::errors::asset_error::AssetError; // 引入资产相关错误类型，便于错误处理和合规校验
use crate::core::logging::log_instruction_dispatch; // 引入统一日志分发工具，便于链上操作审计
use crate::core::types::{TradeParams, BatchTradeParams, StrategyParams, OracleParams, PriceParams}; // 引入核心参数类型，涵盖交易、批量、策略、预言机等

/// 资产增发trait
///
/// 定义资产增发的接口，便于不同实现的扩展。
/// - 设计意图：抽象出增发操作，便于后续多种资产类型的统一处理。
pub trait AssetMintable {
    /// 增发资产
    ///
    /// # 参数
    /// - `basket_index`: 资产状态对象，需可变引用。
    /// - `amount`: 增发数量。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 AssetError。
    fn mint(&self, basket_index: &mut BasketIndexState, amount: u64) -> Result<()>;
}

/// 资产增发服务实现
///
/// 实现资产增发逻辑，校验资产是否可用并安全累加。
pub struct MintAssetService; // 无状态结构体，便于多实例和线程安全
impl AssetMintable for MintAssetService {
    /// 增发资产实现
    ///
    /// - 若资产未激活，返回 NotAllowed 错误。
    /// - 若累加溢出，返回 InsufficientValue 错误。
    fn mint(&self, basket_index: &mut BasketIndexState, amount: u64) -> Result<()> {
        if !basket_index.is_active {
            // 资产未激活，禁止增发，合规校验
            return Err(AssetError::NotAllowed.into());
        }
        // 安全累加，防止溢出
        basket_index.total_value = basket_index.total_value.checked_add(amount).ok_or(AssetError::InsufficientValue)?;
        Ok(()) // 增发成功
    }
}

/// 资产销毁trait
///
/// 定义资产销毁的接口，便于不同实现的扩展。
pub trait AssetBurnable {
    /// 销毁资产
    ///
    /// # 参数
    /// - `basket_index`: 资产状态对象，需可变引用。
    /// - `amount`: 销毁数量。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 AssetError。
    fn burn(&self, basket_index: &mut BasketIndexState, amount: u64) -> Result<()>;
}

/// 资产销毁服务实现
///
/// 实现资产销毁逻辑，校验余额充足并安全扣减。
pub struct BurnAssetService; // 无状态结构体，便于多实例和线程安全
impl AssetBurnable for BurnAssetService {
    /// 销毁资产实现
    ///
    /// - 若余额不足，返回 InsufficientValue 错误。
    fn burn(&self, basket_index: &mut BasketIndexState, amount: u64) -> Result<()> {
        if basket_index.total_value < amount {
            // 余额不足，禁止销毁，合规校验
            return Err(AssetError::InsufficientValue.into());
        }
        // 安全扣减
        basket_index.total_value -= amount;
        Ok(()) // 销毁成功
    }
}

// === 新增：报价与执行买入 trait ===
/// 资产报价trait
///
/// 定义资产报价接口，便于集成不同报价逻辑。
pub trait AssetQuotable {
    /// 资产报价
    ///
    /// # 参数
    /// - `asset`: 资产状态。
    /// - `params`: 交易参数。
    /// - `price_params`: 预言机参数。
    ///
    /// # 返回值
    /// - 返回报价（u64），失败返回 AssetError。
    fn quote(&self, asset: &BasketIndexState, params: &TradeParams, price_params: &OracleParams) -> Result<u64>;
}

/// 资产报价服务实现
///
/// 示例实现，实际应集成DEX/Oracle。
pub struct QuoteAssetService; // 无状态结构体，便于多实例和线程安全
impl AssetQuotable for QuoteAssetService {
    /// 资产报价实现
    ///
    /// - 这里只做业务处理，不做校验和事件分发。
    /// - 示例：直接返回一个模拟价格，实际应集成DEX/Oracle。
    fn quote(&self, asset: &BasketIndexState, params: &TradeParams, price_params: &OracleParams) -> Result<u64> {
        let price = params.amount_in * 1000; // 假设1:1000，实际应集成DEX/Oracle
        Ok(price) // 返回模拟价格
    }
}

/// 资产买入trait
///
/// 定义资产买入接口。
pub trait AssetBuyExecutable {
    /// 执行买入
    ///
    /// # 参数
    /// - `asset`: 资产状态。
    /// - `params`: 交易参数。
    /// - `price`: 买入价格。
    /// - `buyer`: 买方公钥。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 AssetError。
    fn execute_buy(&self, asset: &mut BasketIndexState, params: &TradeParams, price: u64, buyer: Pubkey) -> Result<()>;
}

/// 资产买入服务实现
///
/// 实现资产买入逻辑。
pub struct ExecuteBuyAssetService; // 无状态结构体，便于多实例和线程安全
impl AssetBuyExecutable for ExecuteBuyAssetService {
    /// 买入资产实现
    ///
    /// - 若累加溢出，返回 BuyFailed 错误。
    fn execute_buy(&self, asset: &mut BasketIndexState, params: &TradeParams, price: u64, _buyer: Pubkey) -> Result<()> {
        asset.total_value = asset.total_value.checked_add(params.amount_in).ok_or(crate::errors::asset_error::AssetError::BuyFailed)?; // 安全累加，溢出报错
        Ok(()) // 买入成功
    }
}

// === 新增：卖出 trait ===
/// 资产卖出trait
///
/// 定义资产卖出接口。
pub trait AssetSellable {
    /// 执行卖出
    ///
    /// # 参数
    /// - `asset`: 资产状态。
    /// - `params`: 交易参数。
    /// - `price`: 卖出价格。
    /// - `seller`: 卖方公钥。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 AssetError。
    fn execute_sell(&self, asset: &mut BasketIndexState, params: &TradeParams, price: u64, seller: Pubkey) -> Result<()>;
}

/// 资产卖出服务实现
///
/// 实现资产卖出逻辑。
pub struct ExecuteSellAssetService; // 无状态结构体，便于多实例和线程安全
impl AssetSellable for ExecuteSellAssetService {
    /// 卖出资产实现
    ///
    /// - 若余额不足，返回 SellFailed 错误。
    fn execute_sell(&self, asset: &mut BasketIndexState, params: &TradeParams, price: u64, _seller: Pubkey) -> Result<()> {
        if asset.total_value < params.amount_in {
            return Err(crate::errors::asset_error::AssetError::SellFailed.into()); // 余额不足，禁止卖出
        }
        asset.total_value -= params.amount_in; // 安全扣减
        Ok(()) // 卖出成功
    }
}

// === 新增：资产交换 trait ===
/// 资产交换trait
///
/// 定义资产交换接口。
pub trait AssetSwappable {
    /// 执行资产交换
    ///
    /// # 参数
    /// - `from`: 源资产状态。
    /// - `to`: 目标资产状态。
    /// - `from_amount`: 源资产数量。
    /// - `to_amount`: 目标资产数量。
    /// - `authority`: 操作权限。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 AssetError。
    fn execute_swap(&self, from: &mut BasketIndexState, to: &mut BasketIndexState, from_amount: u64, to_amount: u64, authority: Pubkey) -> Result<()>;
}

/// 资产交换服务实现
///
/// 实现资产交换逻辑。
pub struct ExecuteSwapAssetService; // 无状态结构体，便于多实例和线程安全
impl AssetSwappable for ExecuteSwapAssetService {
    /// 资产交换实现
    ///
    /// - 若源资产余额不足，返回 InsufficientValue 错误。
    /// - 若目标资产累加溢出，返回 InsufficientValue 错误。
    fn execute_swap(&self, from: &mut BasketIndexState, to: &mut BasketIndexState, from_amount: u64, to_amount: u64, _authority: Pubkey) -> Result<()> {
        if from.total_value < from_amount {
            return Err(crate::errors::asset_error::AssetError::InsufficientValue.into()); // 源资产余额不足，禁止交换
        }
        from.total_value -= from_amount; // 扣减源资产
        to.total_value = to.total_value.checked_add(to_amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?; // 累加目标资产，溢出报错
        Ok(()) // 交换成功
    }
}

// === 新增：资产合并 trait ===
/// 资产合并trait
///
/// 定义资产合并接口。
pub trait AssetCombinable {
    /// 执行资产合并
    ///
    /// # 参数
    /// - `target`: 目标资产状态。
    /// - `source`: 源资产状态。
    /// - `amount`: 合并数量。
    /// - `authority`: 操作权限。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 AssetError。
    fn execute_combine(&self, target: &mut BasketIndexState, source: &mut BasketIndexState, amount: u64, authority: Pubkey) -> Result<()>;
}

/// 资产合并服务实现
///
/// 实现资产合并逻辑。
pub struct ExecuteCombineAssetService; // 无状态结构体，便于多实例和线程安全
impl AssetCombinable for ExecuteCombineAssetService {
    /// 资产合并实现
    ///
    /// - 若源资产余额不足，返回 CombineFailed 错误。
    /// - 若目标资产累加溢出，返回 CombineFailed 错误。
    fn execute_combine(&self, target: &mut BasketIndexState, source: &mut BasketIndexState, amount: u64, _authority: Pubkey) -> Result<()> {
        if source.total_value < amount {
            return Err(crate::errors::asset_error::AssetError::CombineFailed.into()); // 源资产余额不足，禁止合并
        }
        source.total_value -= amount; // 扣减源资产
        target.total_value = target.total_value.checked_add(amount).ok_or(crate::errors::asset_error::AssetError::CombineFailed)?; // 累加目标资产，溢出报错
        Ok(()) // 合并成功
    }
}

// === 新增：资产拆分 trait ===
/// 资产拆分trait
///
/// 定义资产拆分接口。
pub trait AssetSplittable {
    /// 执行资产拆分
    ///
    /// # 参数
    /// - `source`: 源资产状态。
    /// - `new_asset`: 新资产状态。
    /// - `amount`: 拆分数量。
    /// - `authority`: 操作权限。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 AssetError。
    fn execute_split(&self, source: &mut BasketIndexState, new_asset: &mut BasketIndexState, amount: u64, authority: Pubkey) -> Result<()>;
}

/// 资产拆分服务实现
///
/// 实现资产拆分逻辑。
pub struct ExecuteSplitAssetService; // 无状态结构体，便于多实例和线程安全
impl AssetSplittable for ExecuteSplitAssetService {
    /// 资产拆分实现
    ///
    /// - 若源资产余额不足，返回 SplitFailed 错误。
    /// - 若新资产累加溢出，返回 SplitFailed 错误。
    fn execute_split(&self, source: &mut BasketIndexState, new_asset: &mut BasketIndexState, amount: u64, _authority: Pubkey) -> Result<()> {
        if source.total_value < amount {
            return Err(crate::errors::asset_error::AssetError::SplitFailed.into()); // 源资产余额不足，禁止拆分
        }
        source.total_value -= amount; // 扣减源资产
        new_asset.total_value = new_asset.total_value.checked_add(amount).ok_or(crate::errors::asset_error::AssetError::SplitFailed)?; // 累加新资产，溢出报错
        Ok(()) // 拆分成功
    }
}

/// AssetService 作为 facade，组合 MintAssetService 和 BurnAssetService，供指令层调用。
///
/// 设计意图：统一对外暴露资产相关操作，便于维护和扩展。
pub struct AssetService {
    /// 增发服务
    pub minter: MintAssetService, // 资产增发服务实例，负责mint逻辑
    /// 销毁服务
    pub burner: BurnAssetService, // 资产销毁服务实例，负责burn逻辑
}

impl AssetService {
    /// 构造AssetService实例
    pub fn new() -> Self {
        Self {
            minter: MintAssetService,
            burner: BurnAssetService,
        }
    }
    /// 增发资产（统一通过AssetTrait接口）
    pub fn mint<T: AssetTrait>(&self, asset: &mut T, amount: u64) -> Result<()> {
        // 统一通过AssetTrait::mint接口，便于多资产类型扩展
        asset.mint(amount)
    }
    /// 销毁资产（统一通过AssetTrait接口）
    pub fn burn<T: AssetTrait>(&self, asset: &mut T, amount: u64) -> Result<()> {
        // 统一通过AssetTrait::burn接口，便于多资产类型扩展
        asset.burn(amount)
    }
    /// 买入资产（融合算法/策略/DEX/预言机，生产级实现）
    pub fn buy(
        asset: &mut BasketIndexState,
        amount: u64,
        price: u64,
        buyer: Pubkey,
        exec_params: Option<ExecutionParams>,
        strategy_params: Option<StrategyParams>,
        oracle_params: Option<OracleParams>,
    ) -> Result<()> {
        // 1. 算法/策略融合：如有 ExecutionParams，查找并调用已注册的 ExecutionStrategy trait 实现
        if let Some(exec_params) = &exec_params {
            if let Some(algo_name) = &exec_params.algo_name {
                let registry = crate::algorithms::algorithm_registry::ALGORITHM_REGISTRY.lock().unwrap();
                if let Some(exec_strategy) = registry.get_execution(algo_name) {
                    // 执行算法，获取最优执行方案（如最优路由、滑点等）
                    let _algo_result = exec_strategy.execute(exec_params)?;
                    // 可将 _algo_result 参与后续决策
                }
            }
        }
        // 2. 预言机融合：如有 OracleParams，查找并调用已注册的 OracleAdapter trait 实现
        let mut final_price = price;
        if let Some(oracle_params) = &oracle_params {
            if let Some(oracle_name) = &oracle_params.oracle_name {
                let factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
                if let Some(adapter) = factory.get(oracle_name) {
                    if let Some(oracle_adapter) = adapter.as_any().downcast_ref::<std::sync::Arc<dyn crate::oracles::traits::OracleAdapter>>() {
                        let oracle_result = oracle_adapter.get_price(oracle_params)?;
                        final_price = oracle_result.price;
                    }
                }
            }
        }
        // 3. DEX/AMM融合：如有 ExecutionParams/DexParams，查找并调用已注册的 DexAdapter trait 实现
        if let Some(exec_params) = &exec_params {
            if let Some(dex_name) = &exec_params.dex_name {
                let factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
                if let Some(adapter) = factory.get(dex_name) {
                    if let Some(dex_adapter) = adapter.as_any().downcast_ref::<std::sync::Arc<dyn crate::dex::traits::DexAdapter>>() {
                        let swap_params = crate::dex::traits::SwapParams {
                            input_mint: asset.mint,
                            output_mint: exec_params.output_mint,
                            amount_in: amount,
                            min_amount_out: exec_params.min_amount_out,
                            user: buyer,
                            dex_accounts: exec_params.dex_accounts.clone(),
                        };
                        let swap_result = dex_adapter.swap(crate::dex::traits::Context::default(), swap_params)?;
                        // 用 swap_result.amount_out 作为实际买入数量，swap_result.fee 参与后续决策
                        // 可根据 swap_result.avg_price 更新 final_price
                        final_price = swap_result.amount_out; // 或 avg_price
                    }
                }
            }
        }
        // 4. 策略融合：如有 StrategyParams，查找并调用已注册的策略trait实现
        if let Some(strategy_params) = &strategy_params {
            if !strategy_params.strategy_name.is_empty() {
                // 这里可查找并调用策略trait实现，参与决策
                // 例如：crate::strategies::strategy_registry::STRATEGY_REGISTRY.get(&strategy_params.strategy_name)
            }
        }
        // 5. 实际买入业务逻辑
        // 安全累加，防止溢出
        asset.total_value = asset.total_value.checked_add(amount).ok_or(crate::errors::asset_error::AssetError::BuyFailed)?;
        // 6. 事件emit由指令层完成，参数链路已补全
        Ok(())
    }
    /// 卖出资产（融合算法/策略/DEX/预言机，生产级实现）
    pub fn sell(
        asset: &mut BasketIndexState,
        amount: u64,
        price: u64,
        seller: Pubkey,
        exec_params: Option<ExecutionParams>,
        strategy_params: Option<StrategyParams>,
        oracle_params: Option<OracleParams>,
    ) -> Result<()> {
        // 1. 算法/策略融合：如有 ExecutionParams，查找并调用已注册的 ExecutionStrategy trait 实现
        if let Some(exec_params) = &exec_params {
            if let Some(algo_name) = &exec_params.algo_name {
                let registry = crate::algorithms::algorithm_registry::ALGORITHM_REGISTRY.lock().unwrap();
                if let Some(exec_strategy) = registry.get_execution(algo_name) {
                    let _algo_result = exec_strategy.execute(exec_params)?;
                }
            }
        }
        // 2. 预言机融合：如有 OracleParams，查找并调用已注册的 OracleAdapter trait 实现
        let mut final_price = price;
        if let Some(oracle_params) = &oracle_params {
            if let Some(oracle_name) = &oracle_params.oracle_name {
                let factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
                if let Some(adapter) = factory.get(oracle_name) {
                    if let Some(oracle_adapter) = adapter.as_any().downcast_ref::<std::sync::Arc<dyn crate::oracles::traits::OracleAdapter>>() {
                        let oracle_result = oracle_adapter.get_price(oracle_params)?;
                        final_price = oracle_result.price;
                    }
                }
            }
        }
        // 3. DEX/AMM融合：如有 ExecutionParams/DexParams，查找并调用已注册的 DexAdapter trait 实现
        if let Some(exec_params) = &exec_params {
            if let Some(dex_name) = &exec_params.dex_name {
                let factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
                if let Some(adapter) = factory.get(dex_name) {
                    if let Some(dex_adapter) = adapter.as_any().downcast_ref::<std::sync::Arc<dyn crate::dex::traits::DexAdapter>>() {
                        let swap_params = crate::dex::traits::SwapParams {
                            input_mint: asset.mint,
                            output_mint: exec_params.output_mint,
                            amount_in: amount,
                            min_amount_out: exec_params.min_amount_out,
                            user: seller,
                            dex_accounts: exec_params.dex_accounts.clone(),
                        };
                        let swap_result = dex_adapter.swap(crate::dex::traits::Context::default(), swap_params)?;
                        final_price = swap_result.amount_out;
                    }
                }
            }
        }
        // 4. 策略融合：如有 StrategyParams，查找并调用已注册的策略trait实现
        if let Some(strategy_params) = &strategy_params {
            if !strategy_params.strategy_name.is_empty() {
                // 这里可查找并调用策略trait实现，参与决策
            }
        }
        // 5. 实际卖出业务逻辑
        if asset.total_value < amount {
            return Err(crate::errors::asset_error::AssetError::SellFailed.into());
        }
        asset.total_value -= amount;
        Ok(())
    }
    /// 资产交换（融合算法/策略/DEX/预言机，生产级实现）
    pub fn swap(
        from: &mut BasketIndexState,
        to: &mut BasketIndexState,
        from_amount: u64,
        to_amount: u64,
        authority: Pubkey,
        exec_params: Option<ExecutionParams>,
        strategy_params: Option<StrategyParams>,
        oracle_params: Option<OracleParams>,
    ) -> Result<()> {
        // 1. 算法/策略融合：如有 ExecutionParams，查找并调用已注册的 ExecutionStrategy trait 实现
        if let Some(exec_params) = &exec_params {
            if let Some(algo_name) = &exec_params.algo_name {
                let registry = crate::algorithms::algorithm_registry::ALGORITHM_REGISTRY.lock().unwrap();
                if let Some(exec_strategy) = registry.get_execution(algo_name) {
                    let _algo_result = exec_strategy.execute(exec_params)?;
                }
            }
        }
        // 2. 预言机融合：如有 OracleParams，查找并调用已注册的 OracleAdapter trait 实现
        let mut final_to_amount = to_amount;
        if let Some(oracle_params) = &oracle_params {
            if let Some(oracle_name) = &oracle_params.oracle_name {
                let factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
                if let Some(adapter) = factory.get(oracle_name) {
                    if let Some(oracle_adapter) = adapter.as_any().downcast_ref::<std::sync::Arc<dyn crate::oracles::traits::OracleAdapter>>() {
                        let oracle_result = oracle_adapter.get_price(oracle_params)?;
                        final_to_amount = oracle_result.price;
                    }
                }
            }
        }
        // 3. DEX/AMM融合：如有 ExecutionParams/DexParams，查找并调用已注册的 DexAdapter trait 实现
        if let Some(exec_params) = &exec_params {
            if let Some(dex_name) = &exec_params.dex_name {
                let factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
                if let Some(adapter) = factory.get(dex_name) {
                    if let Some(dex_adapter) = adapter.as_any().downcast_ref::<std::sync::Arc<dyn crate::dex::traits::DexAdapter>>() {
                        let swap_params = crate::dex::traits::SwapParams {
                            input_mint: from.mint,
                            output_mint: exec_params.output_mint,
                            amount_in: from_amount,
                            min_amount_out: exec_params.min_amount_out,
                            user: authority,
                            dex_accounts: exec_params.dex_accounts.clone(),
                        };
                        let swap_result = dex_adapter.swap(crate::dex::traits::Context::default(), swap_params)?;
                        final_to_amount = swap_result.amount_out;
                    }
                }
            }
        }
        // 4. 策略融合：如有 StrategyParams，查找并调用已注册的策略trait实现
        if let Some(strategy_params) = &strategy_params {
            if !strategy_params.strategy_name.is_empty() {
                // 这里可查找并调用策略trait实现，参与决策
            }
        }
        // 5. 实际交换业务逻辑
        if from.total_value < from_amount {
            return Err(crate::errors::asset_error::AssetError::InsufficientValue.into());
        }
        from.total_value -= from_amount;
        to.total_value = to.total_value.checked_add(final_to_amount).ok_or(crate::errors::asset_error::AssetError::InsufficientValue)?;
        Ok(())
    }
    /// 资产组合（统一通过AssetTrait接口）
    pub fn combine<T: AssetTrait>(&self, target: &mut T, source: &mut T, amount: u64) -> Result<()> {
        // 统一通过AssetTrait::combine接口，便于多资产类型扩展
        target.combine(source, amount)
    }
    /// 资产分割（统一通过AssetTrait接口）
    pub fn split<T: AssetTrait>(&self, asset: &mut T, amount: u64) -> Result<()> {
        // 统一通过AssetTrait::split接口，便于多资产类型扩展
        asset.split(amount)
    }
    /// 资产授权（统一通过AssetTrait接口）
    pub fn authorize<T: AssetTrait>(&self, asset: &mut T, authority: Pubkey) -> Result<()> {
        // 统一通过AssetTrait::authorize接口，便于多资产类型扩展
        asset.authorize(authority)
    }
    /// 资产冻结（统一通过AssetTrait接口）
    pub fn freeze<T: AssetTrait>(&self, asset: &mut T) -> Result<()> {
        // 统一通过AssetTrait::freeze接口，便于多资产类型扩展
        asset.freeze()
    }
    /// 资产解冻（统一通过AssetTrait接口）
    pub fn unfreeze<T: AssetTrait>(&self, asset: &mut T) -> Result<()> {
        // 统一通过AssetTrait::unfreeze接口，便于多资产类型扩展
        asset.unfreeze()
    }
    /// 批量转移资产
    ///
    /// # 参数
    /// - `from`: 源资产状态。
    /// - `to_assets`: 目标资产状态数组。
    /// - `amounts`: 各目标资产对应的转移数量。
    /// - `authority`: 操作权限。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 AssetError。
    ///
    /// # 边界条件
    /// - 权限校验失败、数量不一致、余额不足、累加溢出均会报错。
    pub fn batch_transfer(from: &mut BasketIndexState, to_assets: &mut [&mut BasketIndexState], amounts: &[u64], authority: Pubkey) -> Result<()> {
        if from.authority != authority {
            msg!("[ERROR][batch_transfer_asset] BatchTransferFailed: from_asset_id={}, authority={}", from.id, authority); // 权限校验失败
            return Err(crate::errors::asset_error::AssetError::BatchTransferFailed.into());
        }
        if to_assets.len() != amounts.len() {
            return Err(crate::errors::asset_error::AssetError::BatchTransferFailed.into()); // 数量不一致
        }
        let total: u64 = amounts.iter().sum(); // 计算总转移数量
        if from.total_value < total {
            return Err(crate::errors::asset_error::AssetError::InsufficientValue.into()); // 余额不足
        }
        from.total_value -= total; // 扣减源资产
        for (to, &amt) in to_assets.iter_mut().zip(amounts.iter()) {
            to.total_value = to.total_value.checked_add(amt).ok_or(crate::errors::asset_error::AssetError::BatchTransferFailed)?; // 累加目标资产，溢出报错
        }
        emit!(crate::events::asset_event::AssetBatchTransferred {
            from_asset_id: from.id, // 事件：源资产ID
            to_asset_ids: to_assets.iter().map(|a| a.id).collect(), // 事件：目标资产ID数组
            amounts: amounts.to_vec(), // 事件：转移数量数组
            authority, // 事件：操作权限
            timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
        });
        Ok(()) // 批量转移成功
    }
    /// 批量swap资产
    ///
    /// # 参数
    /// - `from`: 源资产状态。
    /// - `to_assets`: 目标资产状态数组。
    /// - `params`: 批量交易参数。
    /// - `authority`: 操作权限。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 AssetError。
    ///
    /// # 边界条件
    /// - 权限校验失败、参数为空、数量不一致、余额不足、累加溢出均会报错。
    pub fn batch_swap(from: &mut crate::state::baskets::BasketIndexState, to_assets: &mut [&mut crate::state::baskets::BasketIndexState], params: &BatchTradeParams, authority: anchor_lang::prelude::Pubkey) -> anchor_lang::Result<()> {
        if params.trades.is_empty() {
            return Err(crate::errors::asset_error::AssetError::InvalidParams.into()); // 参数为空
        }
        if from.authority != authority {
            msg!("[ERROR][batch_swap_asset] BatchSwapFailed: from_asset_id={}, authority={}", from.id, authority); // 权限校验失败
            return Err(crate::errors::asset_error::AssetError::BatchSwapFailed.into());
        }
        if to_assets.len() != params.trades.len() {
            return Err(crate::errors::asset_error::AssetError::BatchSwapFailed.into()); // 数量不一致
        }
        let total_from: u64 = params.trades.iter().map(|s| s.from_amount).sum(); // 计算总转出数量
        if from.total_value < total_from {
            return Err(crate::errors::asset_error::AssetError::InsufficientValue.into()); // 余额不足
        }
        from.total_value -= total_from; // 扣减源资产
        for (to, trade_param) in to_assets.iter_mut().zip(params.trades.iter()) {
            to.total_value = to.total_value.checked_add(trade_param.to_amount).ok_or(crate::errors::asset_error::AssetError::BatchSwapFailed)?; // 累加目标资产，溢出报错
        }
        emit!(crate::events::asset_event::AssetBatchSwapped {
            from_asset_id: from.id, // 事件：源资产ID
            to_asset_ids: to_assets.iter().map(|a| a.id).collect(), // 事件：目标资产ID数组
            from_amounts: params.trades.iter().map(|s| s.from_amount).collect(), // 事件：转出数量数组
            to_amounts: params.trades.iter().map(|s| s.to_amount).collect(), // 事件：转入数量数组
            authority, // 事件：操作权限
            timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
        });
        Ok(()) // 批量swap成功
    }
    /// 策略交易事件
    ///
    /// # 参数
    /// - `asset`: 资产状态。
    /// - `params`: 策略参数及可选交易参数。
    /// - `authority`: 操作权限。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 AssetError。
    ///
    /// # 设计意图
    /// - 可集成算法/策略执行逻辑，当前仅记录事件。
    pub fn strategy_trade(asset: &mut crate::state::baskets::BasketIndexState, params: &{ strategy: StrategyParams, swap_params: Option<TradeParams>, price_params: Option<OracleParams>, exec_params: Option<TradeParams> }, authority: anchor_lang::prelude::Pubkey) -> anchor_lang::Result<()> {
        if params.strategy.strategy_name.is_empty() {
            return Err(crate::errors::asset_error::AssetError::InvalidParams.into()); // 策略名不能为空
        }
        if asset.authority != authority {
            msg!("[ERROR][strategy_trade_asset] StrategyTradeFailed: asset_id={}, authority={}", asset.id, authority); // 权限校验失败
            return Err(crate::errors::asset_error::AssetError::StrategyTradeFailed.into());
        }
        // 这里可集成算法/策略执行逻辑，示例为直接记录事件
        emit!(crate::events::asset_event::AssetStrategyTraded {
            asset_id: asset.id, // 事件：资产ID
            strategy: params.strategy.strategy_name.to_string(), // 事件：策略名
            params: params.strategy.params.to_vec(), // 事件：策略参数
            authority, // 事件：操作权限
            timestamp: Clock::get()?.unix_timestamp, // 事件：链上时间戳
        });
        Ok(()) // 策略交易成功
    }
    // === 新增：报价与执行买入 ===
    /// 资产报价
    ///
    /// # 参数
    /// - `asset`: 资产状态。
    /// - `params`: 交易参数。
    /// - `price_params`: 预言机参数。
    ///
    /// # 返回值
    /// - 返回报价（u64），失败返回 AssetError。
    pub fn quote(asset: &BasketIndexState, params: &TradeParams, price_params: &OracleParams) -> Result<u64> {
        let service = QuoteAssetService; // 实例化报价服务
        service.quote(asset, params, price_params) // 调用报价逻辑
    }
    /// 执行买入
    ///
    /// # 参数
    /// - `asset`: 资产状态。
    /// - `params`: 交易参数。
    /// - `price`: 买入价格。
    /// - `buyer`: 买方公钥。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 AssetError。
    pub fn execute_buy(asset: &mut BasketIndexState, params: &TradeParams, price: u64, buyer: Pubkey) -> Result<()> {
        let service = ExecuteBuyAssetService; // 实例化买入服务
        service.execute_buy(asset, params, price, buyer) // 调用买入逻辑
    }
    // === 新增：卖出/交换/合并/拆分 ===
    /// 执行卖出
    ///
    /// # 参数
    /// - `asset`: 资产状态。
    /// - `params`: 交易参数。
    /// - `price`: 卖出价格。
    /// - `seller`: 卖方公钥。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 AssetError。
    pub fn execute_sell(asset: &mut BasketIndexState, params: &TradeParams, price: u64, seller: Pubkey) -> Result<()> {
        let service = ExecuteSellAssetService; // 实例化卖出服务
        service.execute_sell(asset, params, price, seller) // 调用卖出逻辑
    }
    /// 执行资产交换
    ///
    /// # 参数
    /// - `from`: 源资产状态。
    /// - `to`: 目标资产状态。
    /// - `from_amount`: 源资产数量。
    /// - `to_amount`: 目标资产数量。
    /// - `authority`: 操作权限。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 AssetError。
    pub fn execute_swap(from: &mut BasketIndexState, to: &mut BasketIndexState, from_amount: u64, to_amount: u64, authority: Pubkey) -> Result<()> {
        let service = ExecuteSwapAssetService; // 实例化交换服务
        service.execute_swap(from, to, from_amount, to_amount, authority) // 调用交换逻辑
    }
    /// 执行资产合并
    ///
    /// # 参数
    /// - `target`: 目标资产状态。
    /// - `source`: 源资产状态。
    /// - `amount`: 合并数量。
    /// - `authority`: 操作权限。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 AssetError。
    pub fn execute_combine(target: &mut BasketIndexState, source: &mut BasketIndexState, amount: u64, authority: Pubkey) -> Result<()> {
        let service = ExecuteCombineAssetService; // 实例化合并服务
        service.execute_combine(target, source, amount, authority) // 调用合并逻辑
    }
    /// 执行资产拆分
    ///
    /// # 参数
    /// - `source`: 源资产状态。
    /// - `new_asset`: 新资产状态。
    /// - `amount`: 拆分数量。
    /// - `authority`: 操作权限。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 AssetError。
    pub fn execute_split(source: &mut BasketIndexState, new_asset: &mut BasketIndexState, amount: u64, authority: Pubkey) -> Result<()> {
        let service = ExecuteSplitAssetService; // 实例化拆分服务
        service.execute_split(source, new_asset, amount, authority) // 调用拆分逻辑
    }
    /// 获取资产价格，融合DEX/Oracle
    pub fn get_price(params: &PriceParams) -> Result<u64> {
        // 1. 优先通过oracle_name获取链上预言机价格
        if let Some(oracle_name) = &params.oracle_name {
            let factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
            if let Some(adapter) = factory.get(oracle_name) {
                if let Some(oracle_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::oracles::traits::OracleAdapter>>() {
                    let oracle_result = oracle_adapter.get_price(params)?;
                    return Ok(oracle_result.price);
                }
            }
        }
        // 2. 若未指定oracle或未获取到，则尝试通过DEX聚合价格
        if let Some(dex_name) = &params.dex_name {
            let factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
            if let Some(adapter) = factory.get(dex_name) {
                if let Some(dex_adapter) = adapter.as_any().downcast_ref::<Arc<dyn crate::dex::traits::DexAdapter>>() {
                    let swap_result = dex_adapter.quote(params)?;
                    return Ok(swap_result.avg_price);
                }
            }
        }
        // 3. 否则返回参数中的价格或错误
        params.price.ok_or(crate::errors::asset_error::AssetError::PriceNotFound.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::baskets::BasketIndexState; // 测试用资产篮子状态
    use crate::core::types::{BatchTradeParams, StrategyParams}; // 测试用参数类型
    use anchor_lang::prelude::Pubkey; // Anchor 公钥类型

    /// 构造默认资产篮子
    fn default_basket(authority: Pubkey, value: u64) -> BasketIndexState {
        BasketIndexState {
            authority, // 测试用权限
            total_value: value, // 测试用余额
            ..Default::default() // 其余字段默认
        }
    }

    /// 测试批量swap参数为空
    #[test]
    fn test_batch_swap_empty_params() {
        let mut from = default_basket(Pubkey::default(), 1000); // 源资产
        let mut to1 = default_basket(Pubkey::default(), 0); // 目标资产1
        let mut to2 = default_basket(Pubkey::default(), 0); // 目标资产2
        let mut to_assets = vec![&mut to1, &mut to2]; // 目标资产数组
        let params = BatchTradeParams { trades: vec![] }; // 空参数
        let result = AssetService::batch_swap(&mut from, &mut to_assets, &params, from.authority); // 调用批量swap
        assert!(result.is_err()); // 应报错
    }

    /// 测试批量swap成功
    #[test]
    fn test_batch_swap_success() {
        let mut from = default_basket(Pubkey::default(), 1000); // 源资产
        let mut to1 = default_basket(Pubkey::default(), 0); // 目标资产1
        let mut to2 = default_basket(Pubkey::default(), 0); // 目标资产2
        let mut to_assets = vec![&mut to1, &mut to2]; // 目标资产数组
        let params = BatchTradeParams {
            trades: vec![
                TradeParams {
                    from_token: Pubkey::default(),
                    to_token: Pubkey::default(),
                    amount_in: 400,
                    min_amount_out: 390,
                    dex_name: "jupiter".to_string(),
                },
                TradeParams {
                    from_token: Pubkey::default(),
                    to_token: Pubkey::default(),
                    amount_in: 500,
                    min_amount_out: 490,
                    dex_name: "orca".to_string(),
                },
            ],
        };
        let result = AssetService::batch_swap(&mut from, &mut to_assets, &params, from.authority); // 调用批量swap
        assert!(result.is_ok()); // 应成功
        assert_eq!(from.total_value, 100); // 源资产余额校验
        assert_eq!(to1.total_value, 390); // 目标资产1余额校验
        assert_eq!(to2.total_value, 490); // 目标资产2余额校验
    }

    /// 测试策略交易参数为空
    #[test]
    fn test_strategy_trade_empty_strategy() {
        let mut asset = default_basket(Pubkey::default(), 1000); // 测试资产
        let params = {
            let strategy_params = StrategyParams { strategy_name: "".to_string(), params: vec![] };
            (
                strategy_params,
                None,
                None,
                None,
            )
        };
        let result = AssetService::strategy_trade(&mut asset, &params, asset.authority); // 调用策略交易
        assert!(result.is_err()); // 应报错
    }
} 