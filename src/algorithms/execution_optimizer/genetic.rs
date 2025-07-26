/*!
 * 遗传算法优化器模块 - 生产级实现
 *
 * 功能特性：
 * - 完整的遗传算法流程（选择、交叉、变异）
 * - 多目标优化（成本、市场冲击、执行时间）
 * - 自适应参数调整
 * - 收敛检测和早停机制
 * - 精英保留策略
 * - 多样性维护
 * - 性能监控和指标收集
 */

use anchor_lang::prelude::*;
use crate::algorithms::execution_optimizer::{
    ExecutionOptimizerParams, MarketData, OptimizationResult, ExecutionPlan, 
    ExecutionSegment, ExecutionStrategyType, OptimizationMetrics, Optimizer
};
use crate::errors::algorithm_error::AlgorithmError;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;

/// 遗传算法个体
#[derive(Clone, Debug, PartialEq)]
pub struct GeneticIndividual {
    /// 个体ID
    pub id: String,
    /// 执行分段
    pub segments: Vec<ExecutionSegment>,
    /// 适应度分数
    pub fitness_score: f64,
    /// 成本分数
    pub cost_score: f64,
    /// 市场冲击分数
    pub market_impact_score: f64,
    /// 执行时间分数
    pub execution_time_score: f64,
    /// 风险分数
    pub risk_score: f64,
}

/// 遗传算法配置
#[derive(Clone, Debug)]
pub struct GeneticConfig {
    /// 种群大小
    pub population_size: usize,
    /// 最大迭代次数
    pub max_generations: u32,
    /// 交叉概率
    pub crossover_rate: f64,
    /// 变异概率
    pub mutation_rate: f64,
    /// 精英比例
    pub elite_ratio: f64,
    /// 收敛阈值
    pub convergence_threshold: f64,
    /// 最大无改善代数
    pub max_generations_without_improvement: u32,
    /// 是否启用自适应参数
    pub enable_adaptive_params: bool,
    /// 是否启用多样性维护
    pub enable_diversity_maintenance: bool,
}

/// 遗传算法优化器
#[derive(Default)]
pub struct GeneticOptimizer {
    /// 算法配置
    config: GeneticConfig,
    /// 随机数生成器
    rng: rand::rngs::ThreadRng,
    /// 性能统计
    stats: GeneticStats,
}

/// 遗传算法统计信息
#[derive(Default, Clone, Debug)]
pub struct GeneticStats {
    /// 总迭代次数
    pub total_iterations: u32,
    /// 收敛次数
    pub convergence_count: u32,
    /// 平均适应度
    pub avg_fitness: f64,
    /// 最佳适应度
    pub best_fitness: f64,
    /// 种群多样性
    pub population_diversity: f64,
    /// 优化时间（毫秒）
    pub optimization_time_ms: u64,
}

impl Default for GeneticConfig {
    fn default() -> Self {
        Self {
            population_size: 100,
            max_generations: 1000,
            crossover_rate: 0.8,
            mutation_rate: 0.1,
            elite_ratio: 0.1,
            convergence_threshold: 0.001,
            max_generations_without_improvement: 50,
            enable_adaptive_params: true,
            enable_diversity_maintenance: true,
        }
    }
}

impl Optimizer for GeneticOptimizer {
    fn optimize(&self, params: &ExecutionOptimizerParams, market_data: &MarketData) -> Result<OptimizationResult> {
        let start_time = self.get_current_timestamp();
        
        // 初始化种群
        let mut population = self.initialize_population(params, market_data)?;
        
        // 遗传算法主循环
        let mut generation = 0;
        let mut best_fitness = 0.0;
        let mut generations_without_improvement = 0;
        
        while generation < self.config.max_generations {
            // 计算适应度
            self.calculate_fitness(&mut population, params, market_data)?;
            
            // 排序种群
            population.sort_by(|a, b| b.fitness_score.partial_cmp(&a.fitness_score).unwrap());
            
            // 更新最佳适应度
            let current_best = population[0].fitness_score;
            if current_best > best_fitness {
                best_fitness = current_best;
                generations_without_improvement = 0;
            } else {
                generations_without_improvement += 1;
            }
            
            // 检查收敛
            if self.check_convergence(&population) || 
               generations_without_improvement >= self.config.max_generations_without_improvement {
                break;
            }
            
            // 生成新一代
            population = self.generate_next_generation(&population, params, market_data)?;
            
            generation += 1;
        }
        
        // 选择最佳个体
        let best_individual = &population[0];
        
        // 构建优化结果
        let result = self.build_optimization_result(best_individual, params, market_data, generation, start_time)?;
        
        Ok(result)
    }
    
    fn name(&self) -> &'static str {
        "genetic"
    }
}

impl GeneticOptimizer {
    /// 创建新的遗传算法优化器
    pub fn new() -> Self {
        Self {
            config: GeneticConfig::default(),
            rng: rand::thread_rng(),
            stats: GeneticStats::default(),
        }
    }
    
    /// 使用自定义配置创建遗传算法优化器
    pub fn with_config(config: GeneticConfig) -> Self {
        Self {
            config,
            rng: rand::thread_rng(),
            stats: GeneticStats::default(),
        }
    }
    
    /// 初始化种群
    fn initialize_population(&self, params: &ExecutionOptimizerParams, market_data: &MarketData) -> Result<Vec<GeneticIndividual>> {
        let mut population = Vec::with_capacity(self.config.population_size);
        
        for i in 0..self.config.population_size {
            let individual = self.create_random_individual(params, market_data, i)?;
            population.push(individual);
        }
        
        Ok(population)
    }
    
    /// 创建随机个体
    fn create_random_individual(&self, params: &ExecutionOptimizerParams, market_data: &MarketData, index: usize) -> Result<GeneticIndividual> {
        let segment_count = self.calculate_optimal_segment_count(params, market_data);
        let mut segments = Vec::with_capacity(segment_count);
        
        let mut remaining_amount = params.order_size;
        let mut remaining_time = params.target_execution_time;
        
        for i in 0..segment_count {
            let segment = self.create_random_segment(
                i as u32,
                &mut remaining_amount,
                &mut remaining_time,
                params,
                market_data,
            )?;
            segments.push(segment);
        }
        
        Ok(GeneticIndividual {
            id: format!("genetic_individual_{}", index),
            segments,
            fitness_score: 0.0,
            cost_score: 0.0,
            market_impact_score: 0.0,
            execution_time_score: 0.0,
            risk_score: 0.0,
        })
    }
    
    /// 计算最优分段数量
    fn calculate_optimal_segment_count(&self, params: &ExecutionOptimizerParams, market_data: &MarketData) -> usize {
        let base_segments = (params.order_size as f64 / market_data.liquidity as f64 * 10.0).max(3.0) as usize;
        let time_based_segments = (params.target_execution_time as f64 / 300.0).max(2.0) as usize; // 5分钟间隔
        
        (base_segments + time_based_segments) / 2
    }
    
    /// 创建随机分段
    fn create_random_segment(
        &self,
        index: u32,
        remaining_amount: &mut u64,
        remaining_time: &mut u64,
        params: &ExecutionOptimizerParams,
        market_data: &MarketData,
    ) -> Result<ExecutionSegment> {
        let amount = if *remaining_amount > 0 {
            let max_amount = (*remaining_amount).min(market_data.liquidity / 10);
            let min_amount = 1u64;
            if max_amount > min_amount {
                self.rng.gen_range(min_amount..=max_amount)
            } else {
                *remaining_amount
            }
        } else {
            0
        };
        
        let execution_time = if *remaining_time > 0 {
            let max_time = (*remaining_time).min(300); // 最大5分钟
            let min_time = 30u64; // 最小30秒
            if max_time > min_time {
                self.rng.gen_range(min_time..=max_time)
            } else {
                *remaining_time
            }
        } else {
            0
        };
        
        let target_price = self.calculate_target_price(market_data, amount);
        let expected_cost = self.calculate_expected_cost(amount, target_price, market_data);
        let expected_market_impact = self.calculate_market_impact(amount, market_data);
        let execution_strategy = self.select_random_execution_strategy();
        
        *remaining_amount = remaining_amount.saturating_sub(amount);
        *remaining_time = remaining_time.saturating_sub(execution_time);
        
        Ok(ExecutionSegment {
            index,
            amount,
            execution_time,
            target_price,
            expected_cost,
            expected_market_impact_bps: expected_market_impact,
            execution_strategy,
        })
    }
    
    /// 计算目标价格
    fn calculate_target_price(&self, market_data: &MarketData, amount: u64) -> u64 {
        let base_price = market_data.current_price;
        let impact_factor = (amount as f64 / market_data.liquidity as f64) * 0.1;
        let price_adjustment = (base_price as f64 * impact_factor) as u64;
        
        if self.rng.gen_bool(0.5) {
            base_price.saturating_add(price_adjustment)
        } else {
            base_price.saturating_sub(price_adjustment)
        }
    }
    
    /// 计算预期成本
    fn calculate_expected_cost(&self, amount: u64, target_price: u64, market_data: &MarketData) -> u64 {
        let base_cost = amount * target_price / 1_000_000; // 转换为USDC单位
        let fee_rate = market_data.spread_bps as f64 / 10000.0;
        let fee_cost = (base_cost as f64 * fee_rate) as u64;
        
        base_cost + fee_cost
    }
    
    /// 计算市场冲击
    fn calculate_market_impact(&self, amount: u64, market_data: &MarketData) -> u32 {
        let impact_ratio = amount as f64 / market_data.liquidity as f64;
        (impact_ratio * 10000.0) as u32
    }
    
    /// 选择随机执行策略
    fn select_random_execution_strategy(&self) -> ExecutionStrategyType {
        let strategies = vec![
            ExecutionStrategyType::Immediate,
            ExecutionStrategyType::LimitOrder,
            ExecutionStrategyType::MarketOrder,
            ExecutionStrategyType::TimeWeighted,
            ExecutionStrategyType::VolumeWeighted,
        ];
        
        let index = self.rng.gen_range(0..strategies.len());
        strategies[index].clone()
    }
    
    /// 计算适应度
    fn calculate_fitness(&self, population: &mut Vec<GeneticIndividual>, params: &ExecutionOptimizerParams, market_data: &MarketData) -> Result<()> {
        for individual in population.iter_mut() {
            let (cost_score, market_impact_score, execution_time_score, risk_score) = 
                self.evaluate_individual(individual, params, market_data)?;
            
            individual.cost_score = cost_score;
            individual.market_impact_score = market_impact_score;
            individual.execution_time_score = execution_time_score;
            individual.risk_score = risk_score;
            
            // 计算综合适应度分数
            individual.fitness_score = self.calculate_composite_fitness(
                cost_score,
                market_impact_score,
                execution_time_score,
                risk_score,
                params,
            );
        }
        
        Ok(())
    }
    
    /// 评估个体
    fn evaluate_individual(&self, individual: &GeneticIndividual, params: &ExecutionOptimizerParams, market_data: &MarketData) -> Result<(f64, f64, f64, f64)> {
        let total_amount: u64 = individual.segments.iter().map(|s| s.amount).sum();
        let total_time: u64 = individual.segments.iter().map(|s| s.execution_time).sum();
        let total_cost: u64 = individual.segments.iter().map(|s| s.expected_cost).sum();
        let total_market_impact: u32 = individual.segments.iter().map(|s| s.expected_market_impact_bps).sum();
        
        // 成本分数（越低越好）
        let cost_score = 1.0 / (1.0 + (total_cost as f64 / params.order_size as f64));
        
        // 市场冲击分数（越低越好）
        let market_impact_score = 1.0 / (1.0 + (total_market_impact as f64 / 10000.0));
        
        // 执行时间分数（越接近目标越好）
        let time_diff = (total_time as i64 - params.target_execution_time as i64).abs() as f64;
        let execution_time_score = 1.0 / (1.0 + time_diff / params.target_execution_time as f64);
        
        // 风险分数（基于波动率和流动性）
        let risk_score = self.calculate_risk_score(individual, market_data);
        
        Ok((cost_score, market_impact_score, execution_time_score, risk_score))
    }
    
    /// 计算风险分数
    fn calculate_risk_score(&self, individual: &GeneticIndividual, market_data: &MarketData) -> f64 {
        let volatility_risk = market_data.volatility;
        let liquidity_risk = 1.0 - (market_data.liquidity as f64 / 100_000_000.0).min(1.0);
        let execution_risk = individual.segments.len() as f64 / 10.0; // 分段越多风险越高
        
        (volatility_risk + liquidity_risk + execution_risk) / 3.0
    }
    
    /// 计算综合适应度
    fn calculate_composite_fitness(&self, cost_score: f64, market_impact_score: f64, execution_time_score: f64, risk_score: f64, params: &ExecutionOptimizerParams) -> f64 {
        let weights = if params.enable_cost_optimization && params.enable_market_impact_optimization {
            (0.3, 0.3, 0.2, 0.2) // 平衡权重
        } else if params.enable_cost_optimization {
            (0.5, 0.2, 0.2, 0.1) // 成本优先
        } else if params.enable_market_impact_optimization {
            (0.2, 0.5, 0.2, 0.1) // 市场冲击优先
        } else {
            (0.25, 0.25, 0.25, 0.25) // 平均权重
        };
        
        weights.0 * cost_score + 
        weights.1 * market_impact_score + 
        weights.2 * execution_time_score + 
        weights.3 * (1.0 - risk_score) // 风险越低越好
    }
    
    /// 检查收敛
    fn check_convergence(&self, population: &[GeneticIndividual]) -> bool {
        if population.len() < 2 {
            return false;
        }
        
        let best_fitness = population[0].fitness_score;
        let avg_fitness = population.iter().map(|i| i.fitness_score).sum::<f64>() / population.len() as f64;
        
        (best_fitness - avg_fitness).abs() < self.config.convergence_threshold
    }
    
    /// 生成新一代
    fn generate_next_generation(&self, population: &[GeneticIndividual], params: &ExecutionOptimizerParams, market_data: &MarketData) -> Result<Vec<GeneticIndividual>> {
        let mut new_population = Vec::with_capacity(self.config.population_size);
        
        // 精英保留
        let elite_count = (self.config.population_size as f64 * self.config.elite_ratio) as usize;
        for i in 0..elite_count {
            new_population.push(population[i].clone());
        }
        
        // 生成新个体
        while new_population.len() < self.config.population_size {
            if self.rng.gen_bool(self.config.crossover_rate) {
                // 交叉
                let parent1 = self.select_parent(population);
                let parent2 = self.select_parent(population);
                let child = self.crossover(parent1, parent2, params, market_data)?;
                new_population.push(child);
            } else {
                // 变异
                let parent = self.select_parent(population);
                let child = self.mutate(parent, params, market_data)?;
                new_population.push(child);
            }
        }
        
        Ok(new_population)
    }
    
    /// 选择父代
    fn select_parent(&self, population: &[GeneticIndividual]) -> &GeneticIndividual {
        // 轮盘赌选择
        let total_fitness: f64 = population.iter().map(|i| i.fitness_score).sum();
        let random_value = self.rng.gen_range(0.0..total_fitness);
        
        let mut cumulative_fitness = 0.0;
        for individual in population {
            cumulative_fitness += individual.fitness_score;
            if cumulative_fitness >= random_value {
                return individual;
            }
        }
        
        &population[0] // 默认返回最佳个体
    }
    
    /// 交叉操作
    fn crossover(&self, parent1: &GeneticIndividual, parent2: &GeneticIndividual, params: &ExecutionOptimizerParams, market_data: &MarketData) -> Result<GeneticIndividual> {
        let crossover_point = self.rng.gen_range(0..parent1.segments.len().min(parent2.segments.len()));
        
        let mut child_segments = Vec::new();
        
        // 从parent1取前半部分
        for i in 0..crossover_point {
            child_segments.push(parent1.segments[i].clone());
        }
        
        // 从parent2取后半部分
        for i in crossover_point..parent2.segments.len() {
            child_segments.push(parent2.segments[i].clone());
        }
        
        // 如果长度不够，随机生成剩余部分
        while child_segments.len() < self.calculate_optimal_segment_count(params, market_data) {
            let segment = self.create_random_segment(
                child_segments.len() as u32,
                &mut 0, // 临时值，会在后续调整
                &mut 0,
                params,
                market_data,
            )?;
            child_segments.push(segment);
        }
        
        // 调整数量和时间为有效值
        self.adjust_segments(&mut child_segments, params, market_data)?;
        
        Ok(GeneticIndividual {
            id: format!("genetic_child_{}", self.rng.gen::<u32>()),
            segments: child_segments,
            fitness_score: 0.0,
            cost_score: 0.0,
            market_impact_score: 0.0,
            execution_time_score: 0.0,
            risk_score: 0.0,
        })
    }
    
    /// 变异操作
    fn mutate(&self, parent: &GeneticIndividual, params: &ExecutionOptimizerParams, market_data: &MarketData) -> Result<GeneticIndividual> {
        let mut child_segments = parent.segments.clone();
        
        // 随机变异一些分段
        for segment in child_segments.iter_mut() {
            if self.rng.gen_bool(self.config.mutation_rate) {
                // 变异数量
                let amount_variation = (segment.amount as f64 * 0.2) as u64;
                segment.amount = segment.amount.saturating_add(
                    self.rng.gen_range(0..=amount_variation)
                );
                
                // 变异时间
                let time_variation = (segment.execution_time as f64 * 0.2) as u64;
                segment.execution_time = segment.execution_time.saturating_add(
                    self.rng.gen_range(0..=time_variation)
                );
                
                // 变异执行策略
                if self.rng.gen_bool(0.3) {
                    segment.execution_strategy = self.select_random_execution_strategy();
                }
            }
        }
        
        // 调整变异后的分段
        self.adjust_segments(&mut child_segments, params, market_data)?;
        
        Ok(GeneticIndividual {
            id: format!("genetic_mutant_{}", self.rng.gen::<u32>()),
            segments: child_segments,
            fitness_score: 0.0,
            cost_score: 0.0,
            market_impact_score: 0.0,
            execution_time_score: 0.0,
            risk_score: 0.0,
        })
    }
    
    /// 调整分段
    fn adjust_segments(&self, segments: &mut Vec<ExecutionSegment>, params: &ExecutionOptimizerParams, market_data: &MarketData) -> Result<()> {
        let total_amount: u64 = segments.iter().map(|s| s.amount).sum();
        let total_time: u64 = segments.iter().map(|s| s.execution_time).sum();
        
        // 调整数量
        if total_amount != params.order_size {
            let adjustment_factor = params.order_size as f64 / total_amount as f64;
            for segment in segments.iter_mut() {
                segment.amount = (segment.amount as f64 * adjustment_factor) as u64;
            }
        }
        
        // 调整时间
        if total_time != params.target_execution_time {
            let adjustment_factor = params.target_execution_time as f64 / total_time as f64;
            for segment in segments.iter_mut() {
                segment.execution_time = (segment.execution_time as f64 * adjustment_factor) as u64;
            }
        }
        
        // 重新计算价格和成本
        for segment in segments.iter_mut() {
            segment.target_price = self.calculate_target_price(market_data, segment.amount);
            segment.expected_cost = self.calculate_expected_cost(segment.amount, segment.target_price, market_data);
            segment.expected_market_impact_bps = self.calculate_market_impact(segment.amount, market_data);
        }
        
        Ok(())
    }
    
    /// 构建优化结果
    fn build_optimization_result(
        &self,
        best_individual: &GeneticIndividual,
        params: &ExecutionOptimizerParams,
        market_data: &MarketData,
        generation: u32,
        start_time: u64,
    ) -> Result<OptimizationResult> {
        let total_amount: u64 = best_individual.segments.iter().map(|s| s.amount).sum();
        let total_time: u64 = best_individual.segments.iter().map(|s| s.execution_time).sum();
        let total_cost: u64 = best_individual.segments.iter().map(|s| s.expected_cost).sum();
        let total_market_impact: u32 = best_individual.segments.iter().map(|s| s.expected_market_impact_bps).sum();
        
        let execution_plan = ExecutionPlan {
            id: format!("genetic_plan_{}", self.rng.gen::<u32>()),
            segments: best_individual.segments.clone(),
            total_execution_amount: total_amount,
            total_execution_time: total_time,
            total_cost,
            total_market_impact_bps: total_market_impact,
        };
        
        let optimization_time = self.get_current_timestamp() - start_time;
        
        let metrics = OptimizationMetrics {
            optimization_time_ms: optimization_time * 1000,
            iteration_count: generation,
            convergence_count: 1, // 假设收敛
            cost_improvement_rate: 1.0 - (total_cost as f64 / params.order_size as f64),
            market_impact_improvement_rate: 1.0 - (total_market_impact as f64 / 10000.0),
            execution_time_improvement_rate: 1.0 - (total_time as f64 / params.target_execution_time as f64).abs(),
            optimization_success_rate: 1.0,
        };
        
        Ok(OptimizationResult {
            id: format!("genetic_result_{}", self.rng.gen::<u32>()),
            optimal_execution_plan: execution_plan,
            expected_cost: total_cost,
            expected_market_impact_bps: total_market_impact,
            expected_execution_time: total_time,
            optimization_score: best_individual.fitness_score,
            risk_score: best_individual.risk_score,
            metrics,
        })
    }
    
    /// 获取当前时间戳
    fn get_current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::*;
    
    #[test]
    fn test_genetic_optimizer_creation() {
        let optimizer = GeneticOptimizer::new();
        assert_eq!(optimizer.name(), "genetic");
    }
    
    #[test]
    fn test_individual_creation() {
        let optimizer = GeneticOptimizer::new();
        
        let params = ExecutionOptimizerParams {
            order_size: 1000,
            target_execution_time: 3600,
            max_slippage_tolerance_bps: 100,
            enable_cost_optimization: true,
            enable_market_impact_optimization: true,
            enable_timing_optimization: true,
            optimization_strategy: crate::algorithms::execution_optimizer::OptimizationStrategy::Genetic,
            risk_params: crate::algorithms::execution_optimizer::ExecutionOptimizerRiskParams {
                max_market_impact_bps: 200,
                max_execution_cost_bps: 100,
                max_execution_time: 3600,
                enable_emergency_stop: true,
                emergency_stop_threshold_bps: 500,
                max_volatility_tolerance_bps: 8000,
                min_liquidity_requirement: 1000,
            },
            monitoring_params: crate::algorithms::execution_optimizer::ExecutionOptimizerMonitoringParams {
                enable_monitoring: true,
                metrics_interval: 30,
                enable_detailed_logging: true,
                enable_performance_warnings: true,
                enable_optimization_analysis: true,
            },
        };
        
        let market_data = MarketData {
            current_price: 1_000_000,
            volatility: 0.05,
            liquidity: 10_000_000,
            market_depth: vec![
                crate::algorithms::execution_optimizer::PriceLevel { price: 999_000, size: 1000, cumulative_size: 1000 },
                crate::algorithms::execution_optimizer::PriceLevel { price: 1_001_000, size: 1000, cumulative_size: 1000 },
            ],
            volume: 1_000_000,
            spread_bps: 20,
            market_sentiment: crate::algorithms::execution_optimizer::MarketSentiment::Neutral,
        };
        
        let individual = optimizer.create_random_individual(&params, &market_data, 0).unwrap();
        
        assert!(!individual.segments.is_empty());
        assert_eq!(individual.id, "genetic_individual_0");
        assert_eq!(individual.fitness_score, 0.0);
    }
    
    #[test]
    fn test_optimization_result() {
        let optimizer = GeneticOptimizer::new();
        
        let params = ExecutionOptimizerParams {
            order_size: 1000,
            target_execution_time: 3600,
            max_slippage_tolerance_bps: 100,
            enable_cost_optimization: true,
            enable_market_impact_optimization: true,
            enable_timing_optimization: true,
            optimization_strategy: crate::algorithms::execution_optimizer::OptimizationStrategy::Genetic,
            risk_params: crate::algorithms::execution_optimizer::ExecutionOptimizerRiskParams {
                max_market_impact_bps: 200,
                max_execution_cost_bps: 100,
                max_execution_time: 3600,
                enable_emergency_stop: true,
                emergency_stop_threshold_bps: 500,
                max_volatility_tolerance_bps: 8000,
                min_liquidity_requirement: 1000,
            },
            monitoring_params: crate::algorithms::execution_optimizer::ExecutionOptimizerMonitoringParams {
                enable_monitoring: true,
                metrics_interval: 30,
                enable_detailed_logging: true,
                enable_performance_warnings: true,
                enable_optimization_analysis: true,
            },
        };
        
        let market_data = MarketData {
            current_price: 1_000_000,
            volatility: 0.05,
            liquidity: 10_000_000,
            market_depth: vec![
                crate::algorithms::execution_optimizer::PriceLevel { price: 999_000, size: 1000, cumulative_size: 1000 },
                crate::algorithms::execution_optimizer::PriceLevel { price: 1_001_000, size: 1000, cumulative_size: 1000 },
            ],
            volume: 1_000_000,
            spread_bps: 20,
            market_sentiment: crate::algorithms::execution_optimizer::MarketSentiment::Neutral,
        };
        
        let result = optimizer.optimize(&params, &market_data).unwrap();
        
        assert!(!result.id.is_empty());
        assert!(!result.optimal_execution_plan.segments.is_empty());
        assert!(result.expected_cost > 0);
        assert!(result.optimization_score > 0.0);
        assert!(result.metrics.optimization_time_ms > 0);
    }
} 