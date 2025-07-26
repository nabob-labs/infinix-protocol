use anchor_lang::prelude::*;
use crate::core::adapter::AdapterTrait;

/// Oracle参数结构体
/// - 用于Oracle价格查询的统一参数接口
/// - 设计意图：标准化Oracle查询参数，便于统一处理
#[derive(Debug, Clone)]
pub struct OracleParams {
    /// 资产公钥
    pub asset: Pubkey,
    /// Oracle名称
    pub oracle_name: String,
    /// 价格（可选，用于某些Oracle的输入）
    pub price: u64,
}

/// Oracle价格结果结构体
/// - 用于返回Oracle价格查询结果
/// - 设计意图：标准化Oracle查询结果，便于统一处理
#[derive(Debug, Clone)]
pub struct OraclePriceResult {
    /// 价格数值
    pub price: u64,
    /// 最后更新时间戳
    pub last_updated: i64,
    /// Oracle名称
    pub oracle_name: String,
}

/// Oracle TWAP结果结构体
/// - 用于返回Oracle TWAP查询结果
/// - 设计意图：标准化Oracle TWAP查询结果
#[derive(Debug, Clone)]
pub struct OracleTwapResult {
    /// TWAP数值
    pub twap: u64,
    /// 最后更新时间戳
    pub last_updated: i64,
    /// Oracle名称
    pub oracle_name: String,
}

/// Oracle VWAP结果结构体
/// - 用于返回Oracle VWAP查询结果
/// - 设计意图：标准化Oracle VWAP查询结果
#[derive(Debug, Clone)]
pub struct OracleVwapResult {
    /// VWAP数值
    pub vwap: u64,
    /// 最后更新时间戳
    pub last_updated: i64,
    /// Oracle名称
    pub oracle_name: String,
}

/// Oracle适配器类型枚举
/// - 用于标识不同类型的Oracle适配器
/// - 设计意图：便于类型检查和适配器管理
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OracleAdapterType {
    /// Pyth预言机
    Pyth,
    /// Switchboard预言机
    Switchboard,
    /// Chainlink预言机
    Chainlink,
    /// 其他类型
    Other,
}

/// Oracle适配器核心trait
/// - 定义Oracle适配器的统一接口
/// - 设计意图：为所有Oracle适配器提供标准化的接口规范
pub trait OracleAdapter: AdapterTrait {
    /// 获取现价
    /// - params: OracleParams结构体，包含资产、oracle名称等
    /// - 返回：OraclePriceResult结构体
    fn get_price(&self, params: &OracleParams) -> anchor_lang::Result<OraclePriceResult>;
    
    /// 获取TWAP
    /// - params: OracleParams结构体
    /// - 返回：OracleTwapResult结构体
    fn get_twap(&self, params: &OracleParams) -> anchor_lang::Result<OracleTwapResult>;
    
    /// 获取VWAP
    /// - params: OracleParams结构体
    /// - 返回：OracleVwapResult结构体
    fn get_vwap(&self, params: &OracleParams) -> anchor_lang::Result<OracleVwapResult>;
    
    /// 返回支持的资产列表
    /// - 返回：资产名称列表
    fn supported_assets(&self) -> Vec<String> { vec![] }
    
    /// 返回支持的市场类型
    /// - 返回：市场类型名称列表
    fn supported_markets(&self) -> Vec<String> { vec![] }
    
    /// 返回适配器类型
    /// - 返回：OracleAdapterType枚举
    fn adapter_type(&self) -> OracleAdapterType { OracleAdapterType::Other }
}

/// Oracle价格查询上下文结构体
/// - 用于Oracle价格查询的账户声明
/// - 设计意图：标准化Oracle查询的账户结构
#[derive(Accounts)]
pub struct GetOraclePrice<'info> {
    /// Oracle价格账户
    pub price_account: AccountInfo<'info>,
    /// Oracle程序
    pub oracle_program: AccountInfo<'info>,
}

/// Oracle TWAP查询上下文结构体
/// - 用于Oracle TWAP查询的账户声明
#[derive(Accounts)]
pub struct GetOracleTwap<'info> {
    /// Oracle TWAP账户
    pub twap_account: AccountInfo<'info>,
    /// Oracle程序
    pub oracle_program: AccountInfo<'info>,
}

/// Oracle VWAP查询上下文结构体
/// - 用于Oracle VWAP查询的账户声明
#[derive(Accounts)]
pub struct GetOracleVwap<'info> {
    /// Oracle VWAP账户
    pub vwap_account: AccountInfo<'info>,
    /// Oracle程序
    pub oracle_program: AccountInfo<'info>,
}

/// Oracle聚合器trait
/// - 用于聚合多个Oracle的数据
/// - 设计意图：提供多Oracle数据源的聚合和容错机制
pub trait OracleAggregator: Send + Sync {
    /// 聚合多个Oracle的价格数据
    /// - params: OracleParams结构体
    /// - oracle_adapters: Oracle适配器列表
    /// - 返回：聚合后的价格结果
    fn aggregate_price(
        &self,
        params: &OracleParams,
        oracle_adapters: &[Box<dyn OracleAdapter>]
    ) -> anchor_lang::Result<OraclePriceResult>;
    
    /// 聚合多个Oracle的TWAP数据
    /// - params: OracleParams结构体
    /// - oracle_adapters: Oracle适配器列表
    /// - 返回：聚合后的TWAP结果
    fn aggregate_twap(
        &self,
        params: &OracleParams,
        oracle_adapters: &[Box<dyn OracleAdapter>]
    ) -> anchor_lang::Result<OracleTwapResult>;
    
    /// 聚合多个Oracle的VWAP数据
    /// - params: OracleParams结构体
    /// - oracle_adapters: Oracle适配器列表
    /// - 返回：聚合后的VWAP结果
    fn aggregate_vwap(
        &self,
        params: &OracleParams,
        oracle_adapters: &[Box<dyn OracleAdapter>]
    ) -> anchor_lang::Result<OracleVwapResult>;
}

/// 默认Oracle聚合器实现
/// - 提供基本的Oracle数据聚合功能
/// - 设计意图：为Oracle聚合提供默认实现
pub struct DefaultOracleAggregator;

impl OracleAggregator for DefaultOracleAggregator {
    /// 聚合多个Oracle的价格数据（取平均值）
    fn aggregate_price(
        &self,
        params: &OracleParams,
        oracle_adapters: &[Box<dyn OracleAdapter>]
    ) -> anchor_lang::Result<OraclePriceResult> {
        if oracle_adapters.is_empty() {
            return Err(crate::errors::oracle_error::OracleError::NoOracleAvailable.into());
        }
        
        let mut total_price = 0u64;
        let mut valid_count = 0u64;
        let mut last_updated = 0i64;
        
        for adapter in oracle_adapters {
            match adapter.get_price(params) {
                Ok(result) => {
                    total_price += result.price;
                    last_updated = last_updated.max(result.last_updated);
                    valid_count += 1;
                }
                Err(_) => continue, // 跳过失败的Oracle
            }
        }
        
        if valid_count == 0 {
            return Err(crate::errors::oracle_error::OracleError::NoOracleAvailable.into());
        }
        
        Ok(OraclePriceResult {
            price: total_price / valid_count,
            last_updated,
            oracle_name: "aggregated".to_string(),
        })
    }
    
    /// 聚合多个Oracle的TWAP数据（取平均值）
    fn aggregate_twap(
        &self,
        params: &OracleParams,
        oracle_adapters: &[Box<dyn OracleAdapter>]
    ) -> anchor_lang::Result<OracleTwapResult> {
        if oracle_adapters.is_empty() {
            return Err(crate::errors::oracle_error::OracleError::NoOracleAvailable.into());
        }
        
        let mut total_twap = 0u64;
        let mut valid_count = 0u64;
        let mut last_updated = 0i64;
        
        for adapter in oracle_adapters {
            match adapter.get_twap(params) {
                Ok(result) => {
                    total_twap += result.twap;
                    last_updated = last_updated.max(result.last_updated);
                    valid_count += 1;
                }
                Err(_) => continue,
            }
        }
        
        if valid_count == 0 {
            return Err(crate::errors::oracle_error::OracleError::NoOracleAvailable.into());
        }
        
        Ok(OracleTwapResult {
            twap: total_twap / valid_count,
            last_updated,
            oracle_name: "aggregated".to_string(),
        })
    }
    
    /// 聚合多个Oracle的VWAP数据（取平均值）
    fn aggregate_vwap(
        &self,
        params: &OracleParams,
        oracle_adapters: &[Box<dyn OracleAdapter>]
    ) -> anchor_lang::Result<OracleVwapResult> {
        if oracle_adapters.is_empty() {
            return Err(crate::errors::oracle_error::OracleError::NoOracleAvailable.into());
        }
        
        let mut total_vwap = 0u64;
        let mut valid_count = 0u64;
        let mut last_updated = 0i64;
        
        for adapter in oracle_adapters {
            match adapter.get_vwap(params) {
                Ok(result) => {
                    total_vwap += result.vwap;
                    last_updated = last_updated.max(result.last_updated);
                    valid_count += 1;
                }
                Err(_) => continue,
            }
        }
        
        if valid_count == 0 {
            return Err(crate::errors::oracle_error::OracleError::NoOracleAvailable.into());
        }
        
        Ok(OracleVwapResult {
            vwap: total_vwap / valid_count,
            last_updated,
            oracle_name: "aggregated".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试OracleParams结构体
    #[test]
    fn test_oracle_params() {
        let params = OracleParams {
            asset: Pubkey::default(),
            oracle_name: "test".to_string(),
            price: 1000,
        };
        assert_eq!(params.oracle_name, "test");
        assert_eq!(params.price, 1000);
    }
    
    /// 测试OraclePriceResult结构体
    #[test]
    fn test_oracle_price_result() {
        let result = OraclePriceResult {
            price: 1000,
            last_updated: 1234567890,
            oracle_name: "test".to_string(),
        };
        assert_eq!(result.price, 1000);
        assert_eq!(result.oracle_name, "test");
    }
    
    /// 测试OracleAdapterType枚举
    #[test]
    fn test_oracle_adapter_type() {
        assert_eq!(OracleAdapterType::Pyth, OracleAdapterType::Pyth);
        assert_ne!(OracleAdapterType::Pyth, OracleAdapterType::Switchboard);
    }
}
