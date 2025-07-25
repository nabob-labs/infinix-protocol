/*!
 * 高级执行优化器模块
 *
 * ## 算法简介
 * 高级执行优化器，融合遗传算法、机器学习、动态规划等多种优化策略，自动寻找最优分单、时序、风险参数，最小化市场冲击和执行成本。
 *
 * ## 主要特性
 * - **多策略可插拔**：支持遗传算法、模拟退火、动态规划等多种优化策略，trait接口可扩展
 * - **极端风险保护**：高波动、低流动性、极端分单参数等自动熔断
 * - **AI/ML融合**：支持机器学习预测、历史回放、动态权重调整
 * - **参数校验与溢出保护**：所有输入参数和数值运算均有严格校验
 * - **可观测性**：优化过程有详细日志输出，便于链上追踪和监控
 * - **单元测试**：覆盖极端行情、异常输入、边界条件、熔断触发等场景
 *
 * ## 关键可插拔点
 * - `ExecutionOptimizer` trait：支持自定义优化策略
 * - `MarketImpactModel`、`MLPredictor`等：支持自定义市场冲击建模与AI预测
 *
 * ## 极端场景保护
 * - 波动率超80%自动中止优化
 * - 流动性极端低自动中止
 * - 分单参数超限自动中止
 *
 * ## 扩展方式
 * - 实现自定义优化策略、市场冲击模型、AI/ML模块等
 * - 可扩展更多风险控制、性能优化、外部数据源等
 *
 * ## 用法示例
 * ```rust
 * let mut optimizer = ExecutionOptimizer::new();
 * let input = ExecutionOptimizationInput { order_size: 1000 };
 * let config = OptimizationConfig::default();
 * let market_data = EnhancedMarketData::default();
 * let result = optimizer.optimize(input, &config, &market_data);
 * ```
 */

use crate::algorithms::{AlgorithmMetrics, TradingAlgorithm}; // 引入算法指标与通用算法 trait
use crate::core::*; // 引入核心类型与工具
use crate::error::StrategyError; // 引入统一错误类型
use anchor_lang::prelude::*; // Anchor 预导入，包含 Context、Result、账户等

// 拆分内容：
// 1. mod.rs：主结构体、trait、mod声明、核心入口
// 2. genetic.rs：遗传算法主流程与相关函数
// 3. ml.rs：机器学习相关结构与实现
// 4. types.rs：参数、配置、结果等类型定义
// 5. tests.rs：单元测试
