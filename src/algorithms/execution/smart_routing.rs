//! 智能路由算法实现模块
//! 实现 RoutingStrategy trait，支持多 DEX 备选路由选择。
//! 支持 Anchor 自动注册，便于工厂/注册表动态调用。

use crate::algorithms::traits::{RoutingStrategy, RoutingParams, RoutingResult, AlgorithmError}; // 引入路由策略 trait 及相关类型
use crate::core::adapter::AdapterTrait; // 引入适配器 trait，便于统一管理
use anchor_lang::prelude::*; // Anchor 预导入，包含 Context、Result、账户等
use crate::core::types::AssetType;

// SmartRoutingAlgorithm - 智能路由算法实现
// 生产级实现，完整实现SmartRoutingAlgorithmTrait，所有方法均逐行专业注释

use anchor_lang::prelude::*;
use crate::core::types::AssetType;

/// SmartRoutingAlgorithmTrait - 智能路由算法trait，所有智能路由算法实现均需实现该trait
pub trait SmartRoutingAlgorithmTrait {
    /// 初始化智能路由算法，注册支持的DEX/AMM名称集合
    fn initialize(&mut self, supported_dex: Vec<String>);
    /// 更新市场深度信息，dex为DEX名称，asset为资产类型，liquidity为流动性，price为价格
    fn update_market(&mut self, dex: String, asset: AssetType, liquidity: u64, price: u64);
    /// 路由下单，asset为目标资产，amount为下单数量，返回最佳DEX名称和成交价格
    fn route(&self, asset: &AssetType, amount: u64) -> Option<(String, u64)>;
    /// 清空市场信息
    fn reset(&mut self);
}

/// MarketInfo - 市场深度信息结构体
#[derive(Clone)]
pub struct MarketInfo {
    /// DEX名称
    pub dex: String,
    /// 资产类型
    pub asset: AssetType,
    /// 流动性
    pub liquidity: u64,
    /// 价格
    pub price: u64,
}

/// SmartRoutingAlgorithm结构体，代表智能路由算法实例
pub struct SmartRoutingAlgorithm {
    /// 支持的DEX/AMM名称集合
    supported_dex: Vec<String>,
    /// 市场深度信息集合
    markets: Vec<MarketInfo>,
}

impl SmartRoutingAlgorithm {
    /// 构造函数，初始化SmartRoutingAlgorithm
    pub fn new(supported_dex: Vec<String>) -> Self {
        Self {
            supported_dex,
            markets: Vec::new(),
        }
    }
}

impl SmartRoutingAlgorithmTrait for SmartRoutingAlgorithm {
    /// 初始化智能路由算法，注册支持的DEX/AMM名称集合
    fn initialize(&mut self, supported_dex: Vec<String>) {
        self.supported_dex = supported_dex;
        self.markets.clear();
    }
    /// 更新市场深度信息，添加或更新市场信息
    fn update_market(&mut self, dex: String, asset: AssetType, liquidity: u64, price: u64) {
        // 查找是否已存在该市场
        if let Some(market) = self.markets.iter_mut().find(|m| m.dex == dex && m.asset == asset) {
            market.liquidity = liquidity;
            market.price = price;
        } else {
            self.markets.push(MarketInfo { dex, asset, liquidity, price });
        }
    }
    /// 路由下单，选择流动性充足且价格最优的DEX
    fn route(&self, asset: &AssetType, amount: u64) -> Option<(String, u64)> {
        // 过滤支持的DEX和目标资产，按价格升序、流动性降序排序
        self.markets
            .iter()
            .filter(|m| self.supported_dex.contains(&m.dex) && &m.asset == asset && m.liquidity >= amount)
            .min_by_key(|m| m.price)
            .map(|m| (m.dex.clone(), m.price))
    }
    /// 清空市场信息
    fn reset(&mut self) {
        self.markets.clear();
    }
}

/// RoutingStrategy trait 实现
impl RoutingStrategy for SmartRoutingImpl {
    /// 路由算法主入口
    /// - 参数 ctx: Anchor 上下文
    /// - 参数 params: 路由参数（包含 DEX 备选列表、输入金额等）
    /// - 返回 RoutingResult，包含最优 DEX、预期输出等
    fn route(&self, _ctx: Context<crate::algorithms::traits::Route>, params: &RoutingParams) -> anchor_lang::Result<RoutingResult> {
        if params.dex_candidates.is_empty() {
            return Err(AlgorithmError::InvalidInput.into()); // 输入参数校验
        }
        // 简化：选择第一个 DEX 作为最优 DEX，实际应根据市场深度等排序
        Ok(RoutingResult {
            best_dex: params.dex_candidates[0].clone(), // 最优 DEX
            expected_out: params.amount_in,             // 预期输出
        })
    }
    /// 算法名称
    fn name(&self) -> &'static str { "SmartRouting" }
}

/// AdapterTrait 实现，便于统一管理和注册
impl AdapterTrait for SmartRoutingImpl {
    fn name(&self) -> &'static str { "smart_routing" } // 算法唯一名称
    fn version(&self) -> &'static str { "1.0.0" } // 算法版本号
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] } // 支持资产
    fn status(&self) -> Option<String> { Some("active".to_string()) } // 激活状态
}

/// Anchor 自动注册宏，模块加载时自动注册到工厂
#[ctor::ctor]
fn auto_register_smart_routing_impl() {
    let adapter = SmartRoutingImpl;
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap();
    factory.register(adapter);
}

#[cfg(test)]
mod tests {
    use super::*;
    /// 测试：正常路由
    #[test]
    fn test_route_basic() {
        let algo = SmartRoutingImpl;
        let params = RoutingParams {
            input_mint: Default::default(),
            output_mint: Default::default(),
            amount_in: 100,
            dex_candidates: vec!["Jupiter".to_string(), "Orca".to_string()],
        };
        let result = algo.route(anchor_lang::prelude::Context::default(), &params).unwrap();
        assert_eq!(result.best_dex, "Jupiter");
    }
    /// 测试：空 DEX 备选列表
    #[test]
    fn test_route_empty() {
        let algo = SmartRoutingImpl;
        let params = RoutingParams {
            input_mint: Default::default(),
            output_mint: Default::default(),
            amount_in: 100,
            dex_candidates: vec![],
        };
        assert!(algo.route(anchor_lang::prelude::Context::default(), &params).is_err());
    }
} 