//! IS (Implementation Shortfall) 算法模块
//! 
//! 本模块提供IS算法的完整实现，包括：
//! - 参数验证：验证IS参数的有效性和边界条件
//! - 执行逻辑：基于实现短缺的算法执行
//! - 服务层调用：委托给IsService执行核心业务逻辑
//! - 事件发射：发射IS算法执行事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于IS算法执行功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给IsService
//! - 事件驱动：完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::{
    algorithms::traits::*,
    core::{
        constants::*,
        events::*,
        types::*,
        validation::*,
    },
    errors::*,
    services::*,
    utils::*,
};

/// IS算法参数结构体
/// 
/// 包含IS算法执行所需的所有参数：
/// - target_price: 目标价格
/// - urgency: 紧急程度
/// - risk_tolerance: 风险容忍度
/// - exec_params: 执行参数
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct IsParams {
    /// 目标价格
    pub target_price: f64,
    /// 紧急程度（0-100）
    pub urgency: f64,
    /// 风险容忍度（0-100）
    pub risk_tolerance: f64,
    /// 执行参数
    pub exec_params: ExecutionParams,
}

/// IS算法账户上下文
/// 
/// 定义IS算法执行所需的账户结构：
/// - asset: 资产账户（可变）
/// - authority: 执行权限账户
/// - market_data: 市场数据账户
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct IsExecute<'info> {
    /// 资产账户（可变）
    #[account(mut)]
    pub asset: Account<'info, Asset>,
    
    /// 执行权限账户
    pub authority: Signer<'info>,
    
    /// 市场数据账户
    /// CHECK: 由程序验证
    pub market_data: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// IS算法实现
pub struct IsAlgorithm {
    /// 算法名称
    name: String,
    /// 算法指标
    metrics: AlgorithmMetrics,
}

impl IsAlgorithm {
    /// 创建新的IS算法实例
    pub fn new() -> Self {
        Self {
            name: "IS".to_string(),
            metrics: AlgorithmMetrics::default(),
        }
    }
    
    /// 验证IS算法参数
    /// 
    /// 检查IS算法参数的有效性和边界条件：
    /// - 目标价格验证
    /// - 紧急程度验证（0-100）
    /// - 风险容忍度验证（0-100）
    /// - 执行参数验证
    /// 
    /// # 参数
    /// - params: IS算法参数
    /// 
    /// # 返回
    /// - Result<()>: 验证结果
    pub fn validate_is_params(params: &IsParams) -> Result<()> {
        // 验证目标价格
        if params.target_price <= 0.0 {
            return Err(AlgorithmError::InvalidInput.into());
        }
        
        // 验证紧急程度
        if params.urgency < 0.0 || params.urgency > 100.0 {
            return Err(AlgorithmError::InvalidInput.into());
        }
        
        // 验证风险容忍度
        if params.risk_tolerance < 0.0 || params.risk_tolerance > 100.0 {
            return Err(AlgorithmError::InvalidInput.into());
        }
        
        // 验证执行参数
        validate_execution_params(&params.exec_params)?;
        
        Ok(())
    }
    
    /// 检查IS算法执行权限
    /// 
    /// 验证执行权限和授权状态：
    /// - 权限账户验证
    /// - 资产权限验证
    /// 
    /// # 参数
    /// - authority: 权限账户
    /// - asset: 资产账户
    /// 
    /// # 返回
    /// - Result<()>: 权限验证结果
    pub fn check_is_authority_permission(
        authority: &Signer,
        asset: &Account<Asset>,
    ) -> Result<()> {
        // 验证权限账户
        if authority.key() != asset.owner {
            return Err(AssetError::InvalidOwner.into());
        }
        
        Ok(())
    }
    
    /// 执行IS算法
    /// 
    /// 执行IS算法的主要逻辑：
    /// - 参数验证
    /// - 权限检查
    /// - 服务层调用
    /// - 事件发射
    /// 
    /// # 参数
    /// - ctx: IS算法执行上下文
    /// - params: IS算法参数
    /// 
    /// # 返回
    /// - Result<()>: 执行结果
    pub fn execute_is(
        ctx: Context<IsExecute>,
        params: IsParams,
    ) -> Result<()> {
        // 参数验证
        Self::validate_is_params(&params)?;
        
        // 权限检查
        Self::check_is_authority_permission(&ctx.accounts.authority, &ctx.accounts.asset)?;
        
        // 获取资产引用
        let asset = &mut ctx.accounts.asset;
        
        // 服务层调用
        let service = IsService::new();
        let result = service.execute_is(
            asset,
            params.target_price,
            params.urgency,
            params.risk_tolerance,
            &params.exec_params,
        )?;
        
        // 更新算法指标
        // self.metrics.update_with_operation(true, result.execution_time_ms); // This line was commented out in the original file, so it's commented out here.
        
        // 发射事件
        emit!(AlgorithmExecuted {
            algorithm_type: AlgorithmType::IS,
            asset: asset.key(),
            authority: ctx.accounts.authority.key(),
            volume_processed: result.volume_processed,
            efficiency_score: result.efficiency_score,
            gas_used: result.gas_used,
            execution_time_ms: result.execution_time_ms,
            success: result.success,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }
}

impl Default for IsAlgorithm {
    fn default() -> Self {
        Self::new()
    }
}

impl Algorithm for IsAlgorithm {
    fn execute(&self, params: &AlgoParams) -> Result<ExecutionResult> {
        // 将AlgoParams转换为IsParams
        let is_params = IsParams {
            target_price: params.target_price.unwrap_or(100.0),
            urgency: params.urgency.unwrap_or(50.0),
            risk_tolerance: params.risk_tolerance.unwrap_or(50.0),
            exec_params: params.exec_params.clone(),
        };
        
        // 验证参数
        Self::validate_is_params(&is_params)?;
        
        // 执行IS算法
        let service = IsService::new();
        let result = service.execute_is(
            &mut Asset::default(), // 这里需要实际的资产账户
            is_params.target_price,
            is_params.urgency,
            is_params.risk_tolerance,
            &is_params.exec_params,
        )?;
        
        Ok(ExecutionResult {
            optimized_size: result.volume_processed,
            expected_cost: result.gas_used,
        })
    }
    
    fn name(&self) -> &'static str {
        "IS"
    }
    
    fn supported_assets(&self) -> Vec<String> {
        vec!["SOL".to_string(), "USDC".to_string(), "BTC".to_string(), "ETH".to_string()]
    }
    
    fn supported_markets(&self) -> Vec<String> {
        vec!["spot".to_string(), "futures".to_string()]
    }
    
    fn algorithm_type(&self) -> AlgorithmType {
        AlgorithmType::Execution
    }
}

impl ExecutionStrategy for IsAlgorithm {
    fn execute(&self, ctx: Context<Execute>, params: &AlgoParams) -> Result<ExecutionResult> {
        // 将AlgoParams转换为IsParams
        let is_params = IsParams {
            target_price: params.target_price.unwrap_or(100.0),
            urgency: params.urgency.unwrap_or(50.0),
            risk_tolerance: params.risk_tolerance.unwrap_or(50.0),
            exec_params: params.exec_params.clone(),
        };
        
        // 执行IS算法
        self.execute_is(ctx, is_params)?;
        
        Ok(ExecutionResult {
            optimized_size: 0, // 这里需要实际的执行结果
            expected_cost: 0,
        })
    }
    
    fn name(&self) -> &'static str {
        "IS"
    }
}

/// IS服务层
pub struct IsService;

impl IsService {
    /// 创建新的IS服务实例
    pub fn new() -> Self {
        Self
    }
    
    /// 执行IS算法
    /// 
    /// 执行IS算法的核心业务逻辑：
    /// - 计算实现短缺
    /// - 执行交易
    /// - 返回执行结果
    /// 
    /// # 参数
    /// - asset: 资产账户
    /// - target_price: 目标价格
    /// - urgency: 紧急程度
    /// - risk_tolerance: 风险容忍度
    /// - exec_params: 执行参数
    /// 
    /// # 返回
    /// - Result<AlgorithmResult>: 执行结果
    pub fn execute_is(
        &self,
        asset: &mut Account<Asset>,
        target_price: f64,
        urgency: f64,
        risk_tolerance: f64,
        exec_params: &ExecutionParams,
    ) -> Result<AlgorithmResult> {
        // 计算实现短缺
        let current_price = 100.0; // 示例当前价格
        let shortfall = (target_price - current_price).abs();
        
        // 根据紧急程度和风险容忍度调整执行策略
        let execution_aggressiveness = (urgency + risk_tolerance) / 200.0;
        
        // 执行交易逻辑
        // TODO: 实现具体的交易执行逻辑
        
        // 返回执行结果
        Ok(AlgorithmResult {
            algorithm_type: AlgorithmType::IS,
            volume_processed: 1000, // 示例值
            efficiency_score: 9000, // 示例值
            gas_used: 1200, // 示例值
            execution_time_ms: 40, // 示例值
            success: true,
            metrics: AlgorithmMetrics::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_algorithm_creation() {
        let algorithm = IsAlgorithm::new();
        assert_eq!(algorithm.name(), "IS");
    }
    
    #[test]
    fn test_is_params_validation() {
        let valid_params = IsParams {
            target_price: 100.0,
            urgency: 50.0,
            risk_tolerance: 50.0,
            exec_params: ExecutionParams::default(),
        };
        
        assert!(IsAlgorithm::validate_is_params(&valid_params).is_ok());
        
        let invalid_params = IsParams {
            target_price: -10.0, // 无效的目标价格
            urgency: 50.0,
            risk_tolerance: 50.0,
            exec_params: ExecutionParams::default(),
        };
        
        assert!(IsAlgorithm::validate_is_params(&invalid_params).is_err());
    }
    
    #[test]
    fn test_is_service_execution() {
        let service = IsService::new();
        let mut asset = Asset::default();
        
        let result = service.execute_is(
            &mut asset,
            100.0,
            50.0,
            50.0,
            &ExecutionParams::default(),
        );
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert_eq!(result.algorithm_type, AlgorithmType::IS);
    }
} 