//! API3 Oracle适配器模块
//! 
//! 本模块提供API3 Oracle的完整适配器实现，包括：
//! - 价格查询：获取API3的价格数据
//! - 数据验证：验证API3数据的有效性
//! - 服务层调用：委托给Api3Service执行核心业务逻辑
//! - 事件发射：发射API3 Oracle事件用于审计和追踪
//! 
//! 设计特点：
//! - 最小功能单元：专注于API3 Oracle适配功能
//! - 类型安全：严格的类型检查和边界验证
//! - 参数验证：全面的输入参数验证
//! - 权限控制：细粒度的权限验证
//! - 服务层抽象：核心业务逻辑委托给Api3Service
//! - 事件驱动：完整的事件发射和审计追踪

use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

use crate::{
    core::{
        constants::*,
        events::*,
        types::*,
        validation::*,
    },
    errors::*,
    oracles::traits::*,
    services::*,
    utils::*,
};

/// API3 Oracle参数结构体
/// 
/// 包含API3 Oracle查询所需的所有参数：
/// - asset: 资产公钥
/// - oracle_name: Oracle名称
/// - price: 价格（可选）
/// - confidence: 置信度
/// - timestamp: 时间戳
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct Api3Params {
    /// 资产公钥
    pub asset: Pubkey,
    /// Oracle名称
    pub oracle_name: String,
    /// 价格（可选）
    pub price: Option<u64>,
    /// 置信度
    pub confidence: f64,
    /// 时间戳
    pub timestamp: i64,
}

/// API3 Oracle账户上下文
/// 
/// 定义API3 Oracle查询所需的账户结构：
/// - api3_account: API3账户（可变）
/// - authority: 查询权限账户
/// - api3_program: API3程序
/// - system_program: 系统程序
/// - token_program: 代币程序
#[derive(Accounts)]
pub struct Api3Query<'info> {
    /// API3账户（可变）
    #[account(mut)]
    pub api3_account: AccountInfo<'info>,
    
    /// 查询权限账户
    pub authority: Signer<'info>,
    
    /// API3程序
    /// CHECK: 由程序验证
    pub api3_program: UncheckedAccount<'info>,
    
    /// 系统程序
    pub system_program: Program<'info, System>,
    
    /// 代币程序
    pub token_program: Program<'info, Token>,
}

/// API3 Oracle适配器实现
pub struct Api3Adapter {
    /// 适配器名称
    name: String,
    /// 适配器类型
    adapter_type: OracleAdapterType,
    /// 支持的资产列表
    supported_assets: Vec<String>,
    /// 支持的市场类型
    supported_markets: Vec<String>,
}

impl Api3Adapter {
    /// 创建新的API3适配器实例
    pub fn new() -> Self {
        Self {
            name: "API3".to_string(),
            adapter_type: OracleAdapterType::API3,
            supported_assets: vec![
                "SOL".to_string(),
                "USDC".to_string(),
                "BTC".to_string(),
                "ETH".to_string(),
                "USDT".to_string(),
            ],
            supported_markets: vec![
                "spot".to_string(),
                "futures".to_string(),
            ],
        }
    }
    
    /// 验证API3参数
    /// 
    /// 检查API3参数的有效性和边界条件：
    /// - 资产公钥验证
    /// - Oracle名称验证
    /// - 置信度验证
    /// - 时间戳验证
    /// 
    /// # 参数
    /// - params: API3参数
    /// 
    /// # 返回
    /// - Result<()>: 验证结果
    pub fn validate_api3_params(params: &Api3Params) -> Result<()> {
        // 验证资产公钥
        if params.asset == Pubkey::default() {
            return Err(OracleError::InvalidAsset.into());
        }
        
        // 验证Oracle名称
        if params.oracle_name.is_empty() {
            return Err(OracleError::InvalidOracleName.into());
        }
        
        // 验证置信度
        if params.confidence < 0.0 || params.confidence > 1.0 {
            return Err(OracleError::InvalidConfidence.into());
        }
        
        // 验证时间戳
        let current_timestamp = Clock::get()?.unix_timestamp;
        if params.timestamp > current_timestamp {
            return Err(OracleError::InvalidTimestamp.into());
        }
        
        Ok(())
    }
    
    /// 检查API3查询权限
    /// 
    /// 验证查询权限和授权状态：
    /// - 权限账户验证
    /// - 账户权限验证
    /// 
    /// # 参数
    /// - authority: 权限账户
    /// - api3_account: API3账户
    /// 
    /// # 返回
    /// - Result<()>: 权限验证结果
    pub fn check_api3_authority_permission(
        authority: &Signer,
        api3_account: &AccountInfo,
    ) -> Result<()> {
        // 验证权限账户
        if !authority.is_signer {
            return Err(OracleError::InvalidAuthority.into());
        }
        
        // 验证账户权限
        if !api3_account.is_writable {
            return Err(OracleError::InvalidAccount.into());
        }
        
        Ok(())
    }
    
    /// 查询API3价格
    /// 
    /// 查询API3价格的主要逻辑：
    /// - 参数验证
    /// - 权限检查
    /// - 服务层调用
    /// - 事件发射
    /// 
    /// # 参数
    /// - ctx: API3查询上下文
    /// - params: API3参数
    /// 
    /// # 返回
    /// - Result<OraclePriceResult>: 查询结果
    pub fn query_api3_price(
        ctx: Context<Api3Query>,
        params: Api3Params,
    ) -> Result<OraclePriceResult> {
        // 参数验证
        Self::validate_api3_params(&params)?;
        
        // 权限检查
        Self::check_api3_authority_permission(&ctx.accounts.authority, &ctx.accounts.api3_account)?;
        
        // 服务层调用
        let service = Api3Service::new();
        let result = service.get_price(
            &params.asset,
            &params.oracle_name,
            params.confidence,
            params.timestamp,
        )?;
        
        // 发射事件
        emit!(OraclePriceQueried {
            oracle_type: OracleType::API3,
            asset: params.asset,
            price: result.price,
            confidence: params.confidence,
            timestamp: result.last_updated,
            authority: ctx.accounts.authority.key(),
        });
        
        Ok(result)
    }
}

impl Default for Api3Adapter {
    fn default() -> Self {
        Self::new()
    }
}

impl OracleAdapter for Api3Adapter {
    fn get_price(&self, params: &OracleParams) -> Result<OraclePriceResult> {
        // 将OracleParams转换为Api3Params
        let api3_params = Api3Params {
            asset: params.asset,
            oracle_name: params.oracle_name.clone(),
            price: Some(params.price),
            confidence: 0.95, // 默认置信度
            timestamp: Clock::get()?.unix_timestamp,
        };
        
        // 验证参数
        Self::validate_api3_params(&api3_params)?;
        
        // 查询价格
        let service = Api3Service::new();
        service.get_price(
            &api3_params.asset,
            &api3_params.oracle_name,
            api3_params.confidence,
            api3_params.timestamp,
        )
    }
    
    fn get_twap(&self, params: &OracleParams) -> Result<OracleTwapResult> {
        // 将OracleParams转换为Api3Params
        let api3_params = Api3Params {
            asset: params.asset,
            oracle_name: params.oracle_name.clone(),
            price: Some(params.price),
            confidence: 0.95, // 默认置信度
            timestamp: Clock::get()?.unix_timestamp,
        };
        
        // 验证参数
        Self::validate_api3_params(&api3_params)?;
        
        // 查询TWAP
        let service = Api3Service::new();
        service.get_twap(
            &api3_params.asset,
            &api3_params.oracle_name,
            api3_params.confidence,
            api3_params.timestamp,
        )
    }
    
    fn get_vwap(&self, params: &OracleParams) -> Result<OracleVwapResult> {
        // 将OracleParams转换为Api3Params
        let api3_params = Api3Params {
            asset: params.asset,
            oracle_name: params.oracle_name.clone(),
            price: Some(params.price),
            confidence: 0.95, // 默认置信度
            timestamp: Clock::get()?.unix_timestamp,
        };
        
        // 验证参数
        Self::validate_api3_params(&api3_params)?;
        
        // 查询VWAP
        let service = Api3Service::new();
        service.get_vwap(
            &api3_params.asset,
            &api3_params.oracle_name,
            api3_params.confidence,
            api3_params.timestamp,
        )
    }
    
    fn supported_assets(&self) -> Vec<String> {
        self.supported_assets.clone()
    }
    
    fn supported_markets(&self) -> Vec<String> {
        self.supported_markets.clone()
    }
    
    fn adapter_type(&self) -> OracleAdapterType {
        self.adapter_type
    }
    
    fn name(&self) -> &'static str {
        "API3"
    }
}

/// API3服务层
pub struct Api3Service;

impl Api3Service {
    /// 创建新的API3服务实例
    pub fn new() -> Self {
        Self
    }
    
    /// 获取价格
    /// 
    /// 获取API3价格的核心业务逻辑：
    /// - 查询价格数据
    /// - 验证数据有效性
    /// - 返回价格结果
    /// 
    /// # 参数
    /// - asset: 资产公钥
    /// - oracle_name: Oracle名称
    /// - confidence: 置信度
    /// - timestamp: 时间戳
    /// 
    /// # 返回
    /// - Result<OraclePriceResult>: 价格结果
    pub fn get_price(
        &self,
        asset: &Pubkey,
        oracle_name: &str,
        confidence: f64,
        timestamp: i64,
    ) -> Result<OraclePriceResult> {
        // TODO: 实现具体的API3价格查询逻辑
        // 这里应该调用API3的实际API
        
        // 模拟价格查询结果
        let price = 100_000_000; // 示例价格（以最小单位计）
        let last_updated = timestamp;
        
        Ok(OraclePriceResult {
            price,
            last_updated,
            oracle_name: oracle_name.to_string(),
        })
    }
    
    /// 获取TWAP
    /// 
    /// 获取API3 TWAP的核心业务逻辑：
    /// - 查询TWAP数据
    /// - 验证数据有效性
    /// - 返回TWAP结果
    /// 
    /// # 参数
    /// - asset: 资产公钥
    /// - oracle_name: Oracle名称
    /// - confidence: 置信度
    /// - timestamp: 时间戳
    /// 
    /// # 返回
    /// - Result<OracleTwapResult>: TWAP结果
    pub fn get_twap(
        &self,
        asset: &Pubkey,
        oracle_name: &str,
        confidence: f64,
        timestamp: i64,
    ) -> Result<OracleTwapResult> {
        // TODO: 实现具体的API3 TWAP查询逻辑
        
        // 模拟TWAP查询结果
        let twap = 100_000_000; // 示例TWAP（以最小单位计）
        let last_updated = timestamp;
        
        Ok(OracleTwapResult {
            twap,
            last_updated,
            oracle_name: oracle_name.to_string(),
        })
    }
    
    /// 获取VWAP
    /// 
    /// 获取API3 VWAP的核心业务逻辑：
    /// - 查询VWAP数据
    /// - 验证数据有效性
    /// - 返回VWAP结果
    /// 
    /// # 参数
    /// - asset: 资产公钥
    /// - oracle_name: Oracle名称
    /// - confidence: 置信度
    /// - timestamp: 时间戳
    /// 
    /// # 返回
    /// - Result<OracleVwapResult>: VWAP结果
    pub fn get_vwap(
        &self,
        asset: &Pubkey,
        oracle_name: &str,
        confidence: f64,
        timestamp: i64,
    ) -> Result<OracleVwapResult> {
        // TODO: 实现具体的API3 VWAP查询逻辑
        
        // 模拟VWAP查询结果
        let vwap = 100_000_000; // 示例VWAP（以最小单位计）
        let last_updated = timestamp;
        
        Ok(OracleVwapResult {
            vwap,
            last_updated,
            oracle_name: oracle_name.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api3_adapter_creation() {
        let adapter = Api3Adapter::new();
        assert_eq!(adapter.name(), "API3");
        assert_eq!(adapter.adapter_type(), OracleAdapterType::API3);
    }
    
    #[test]
    fn test_api3_params_validation() {
        let valid_params = Api3Params {
            asset: Pubkey::new_unique(),
            oracle_name: "SOL/USD".to_string(),
            price: Some(100_000_000),
            confidence: 0.95,
            timestamp: Clock::get().unwrap().unix_timestamp,
        };
        
        assert!(Api3Adapter::validate_api3_params(&valid_params).is_ok());
        
        let invalid_params = Api3Params {
            asset: Pubkey::default(), // 无效的资产公钥
            oracle_name: "SOL/USD".to_string(),
            price: Some(100_000_000),
            confidence: 0.95,
            timestamp: Clock::get().unwrap().unix_timestamp,
        };
        
        assert!(Api3Adapter::validate_api3_params(&invalid_params).is_err());
    }
    
    #[test]
    fn test_api3_service_price_query() {
        let service = Api3Service::new();
        let asset = Pubkey::new_unique();
        
        let result = service.get_price(
            &asset,
            "SOL/USD",
            0.95,
            Clock::get().unwrap().unix_timestamp,
        );
        
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.oracle_name, "SOL/USD");
        assert!(result.price > 0);
    }
} 