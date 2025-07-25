//!
//! 遗传算法执行器实现模块
//! 实现 ExecutionStrategy trait，支持基于遗传算法的订单执行优化。
//! 支持 Anchor 自动注册，便于工厂/注册表动态调用。

use crate::algorithms::traits::{ExecutionStrategy, ExecutionResult, AlgorithmError}; // 引入执行策略 trait 及相关类型，便于类型安全和接口统一
use crate::core::types::AlgoParams; // 引入通用算法参数类型，便于算法通用化
use crate::core::adapter::AdapterTrait; // 引入适配器 trait，便于统一管理和注册
use anchor_lang::prelude::*; // Anchor 预导入，包含 Context、Result、账户等，确保与Anchor兼容

/// 遗传算法执行器实现结构体
pub struct GeneticImpl; // 主结构体，无状态实现，所有逻辑在trait实现中，提升安全性和可复用性

/// AdapterTrait 实现，便于统一管理和注册
impl AdapterTrait for GeneticImpl {
    fn name(&self) -> &'static str { "genetic" } // 算法唯一名称，便于注册表/工厂识别
    fn version(&self) -> &'static str { "1.0.0" } // 算法版本号，便于升级和兼容性管理
    fn supported_assets(&self) -> Vec<String> { vec!["SOL".to_string(), "USDC".to_string()] } // 支持资产类型，便于资产适配
    fn status(&self) -> Option<String> { Some("active".to_string()) } // 激活状态，便于运维监控
}

/// Anchor 自动注册宏，模块加载时自动注册到工厂
#[ctor::ctor] // ctor宏，模块加载时自动执行，便于自动注册
fn auto_register_genetic_impl() {
    let adapter = GeneticImpl; // 创建遗传算法适配器实例，无状态实现
    let mut factory = crate::core::registry::ADAPTER_FACTORY.lock().unwrap(); // 获取全局工厂锁，保证线程安全
    factory.register(adapter); // 注册适配器到工厂，支持热插拔和动态扩展
}

/// ExecutionStrategy trait 实现
impl ExecutionStrategy for GeneticImpl {
    /// 执行遗传算法主入口
    /// - 参数 ctx: Anchor 上下文，包含账户、权限等
    /// - 参数 params: 算法参数（需序列化为 (order_size, slippage_tolerance)）
    /// - 返回 ExecutionResult，包含优化后成交量、预期成本等
    fn execute(&self, _ctx: Context<crate::algorithms::traits::Execute>, params: &AlgoParams) -> Result<ExecutionResult> {
        // 解析 AlgoParams，获取 order_size、slippage_tolerance
        let (order_size, slippage_tolerance): (u64, u64) = bincode::deserialize(&params.params)
            .map_err(|_| AlgorithmError::InvalidInput)?; // 反序列化参数，错误则返回 InvalidInput，防止恶意输入
        if order_size == 0 || slippage_tolerance == 0 {
            return Err(AlgorithmError::InvalidInput.into()); // 输入参数校验，订单量和滑点必须大于0，防止无效或恶意调用
        }
        // 生产级遗传算法核心流程
        let population_size = 32; // 种群规模，影响搜索空间和收敛速度
        let generations = 20;     // 迭代次数，影响算法精度和性能
        let mut rng = anchor_lang::solana_program::keccak::hashv(&[&order_size.to_le_bytes()]); // 随机种子，提升多样性
        let mut population: Vec<u64> = (0..population_size)
            .map(|i| (order_size / population_size as u64) * (i as u64 + 1))
            .collect(); // 初始化种群，均匀分布
        let mut best = population[0]; // 当前最优解，初始化为第一个个体
        let mut best_fitness = Self::fitness(best, order_size, slippage_tolerance); // 当前最优适应度
        for _ in 0..generations {
            // 计算适应度
            let mut fitness: Vec<u64> = population.iter().map(|&x| Self::fitness(x, order_size, slippage_tolerance)).collect(); // 适应度向量
            // 选择
            let mut selected = Vec::with_capacity(population_size); // 选择后的个体
            for _ in 0..population_size {
                let idx = (rng.0[0] as usize) % population_size; // 随机选择索引
                selected.push(population[idx]); // 选择个体
            }
            // 交叉
            for i in (0..population_size).step_by(2) {
                if i + 1 < population_size {
                    let a = selected[i];
                    let b = selected[i + 1];
                    let child = (a + b) / 2; // 简单均值交叉
                    selected[i] = child; // 替换父代
                }
            }
            // 变异
            for x in &mut selected {
                if (rng.0[1] as usize) % 10 == 0 {
                    *x = x.saturating_add(1); // 随机变异，防止下溢
                }
            }
            population = selected; // 更新种群
            // 更新最优解
            for &x in &population {
                let fit = Self::fitness(x, order_size, slippage_tolerance); // 计算适应度
                if fit > best_fitness {
                    best = x; // 更新最优解
                    best_fitness = fit; // 更新最优适应度
                }
            }
        }
        Ok(ExecutionResult {
            optimized_size: best,      // 优化后成交量，类型安全
            expected_cost: best_fitness, // 适应度作为预期成本（示例），便于链上链下审计
        })
    }
    /// 算法名称
    fn name(&self) -> &'static str { "Genetic" } // 算法名称常量，便于注册表/工厂识别
}

impl GeneticImpl {
    /// 适应度函数
    /// - 参数 candidate: 候选成交量
    /// - 参数 order_size: 总订单量
    /// - 参数 slippage_tolerance: 滑点容忍度
    /// - 返回 u64，适应度分数，越大越优
    fn fitness(candidate: u64, order_size: u64, slippage_tolerance: u64) -> u64 {
        // 简化适应度函数：成交量越大、滑点越小越优
        candidate * 1000 / (slippage_tolerance + 1) // 防止除零，提升健壮性
    }
} 