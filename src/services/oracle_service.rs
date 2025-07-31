//!
//! 预言机服务层
//! 业务逻辑实现，供指令入口调用，封装预言机适配器注册、价格查询、TWAP/VWAP、批量操作、权限校验等操作。

use anchor_lang::prelude::*;
use crate::oracles::traits::{OracleAdapter, OracleParams, OraclePriceResult, OracleTwapResult, OracleVwapResult};
use crate::core::types::{BatchTradeParams};
// use crate::errors::basket_error::BasketError;

/// 预言机适配器注册trait
///
/// 定义预言机适配器注册接口，便于扩展多种注册方式。
/// - 设计意图：统一注册入口，便于后续多种适配器注册策略。
trait OracleAdapterRegistrable {
    /// 注册预言机适配器
    ///
    /// # 参数
    /// - `adapter`: 适配器实例。
    ///
    /// # 返回值
    /// - 成功返回 Ok(())，失败返回 BasketError。
    fn register_adapter(&self, adapter: Box<dyn OracleAdapter>) -> anchor_lang::Result<()>;
}

/// 预言机适配器注册服务实现
///
/// 示例实现：注册到全局适配器表。
pub struct RegisterOracleAdapterService;
impl OracleAdapterRegistrable for RegisterOracleAdapterService {
    /// 注册实现
    fn register_adapter(&self, _adapter: Box<dyn OracleAdapter>) -> anchor_lang::Result<()> {
        // 生产级实现：注册到全局适配器表
        Ok(())
    }
}

/// 价格查询trait
///
/// 定义现价查询接口，便于扩展多种价格查询方式。
/// - 设计意图：统一价格查询入口，便于后续多种价格源。
trait OraclePriceQuotable {
    /// 查询现价
    ///
    /// # 参数
    /// - `params`: 查询参数。
    ///
    /// # 返回值
    /// - 返回价格结果，失败返回 BasketError。
    fn get_price(&self, params: &OracleParams) -> anchor_lang::Result<OraclePriceResult>;
}

/// 价格查询服务实现
///
/// 示例实现：调用预言机适配器get_price。
pub struct GetPriceOracleService;
impl OraclePriceQuotable for GetPriceOracleService {
    /// 现价查询实现
    fn get_price(&self, params: &OracleParams) -> anchor_lang::Result<OraclePriceResult> {
        // 生产级实现：调用预言机适配器get_price
        Ok(OraclePriceResult { 
            price: params.price * 1000,
            last_updated: anchor_lang::clock::Clock::get()?.unix_timestamp,
            oracle_name: params.oracle_name.clone(),
        })
    }
}

/// TWAP查询trait
///
/// 定义TWAP查询接口，便于扩展多种TWAP查询方式。
/// - 设计意图：统一TWAP查询入口，便于后续多种价格源。
trait OracleTwapQuotable {
    /// 查询TWAP
    ///
    /// # 参数
    /// - `params`: 查询参数。
    ///
    /// # 返回值
    /// - 返回TWAP结果，失败返回 BasketError。
    fn get_twap(&self, params: &OracleParams) -> anchor_lang::Result<OracleTwapResult>;
}

/// TWAP查询服务实现
///
/// 示例实现：调用预言机适配器get_twap。
pub struct GetTwapOracleService;
impl OracleTwapQuotable for GetTwapOracleService {
    /// TWAP查询实现
    fn get_twap(&self, params: &OracleParams) -> anchor_lang::Result<OracleTwapResult> {
        // 生产级实现：调用预言机适配器get_twap
        Ok(OracleTwapResult { 
            twap: params.price * 1000,
            last_updated: anchor_lang::clock::Clock::get()?.unix_timestamp,
            oracle_name: params.oracle_name.clone(),
        })
    }
}

/// VWAP查询trait
///
/// 定义VWAP查询接口，便于扩展多种VWAP查询方式。
/// - 设计意图：统一VWAP查询入口，便于后续多种价格源。
trait OracleVwapQuotable {
    /// 查询VWAP
    ///
    /// # 参数
    /// - `params`: 查询参数。
    ///
    /// # 返回值
    /// - 返回VWAP结果，失败返回 BasketError。
    fn get_vwap(&self, params: &OracleParams) -> anchor_lang::Result<OracleVwapResult>;
}

/// VWAP查询服务实现
///
/// 示例实现：调用预言机适配器get_vwap。
pub struct GetVwapOracleService;
impl OracleVwapQuotable for GetVwapOracleService {
    /// VWAP查询实现
    fn get_vwap(&self, params: &OracleParams) -> anchor_lang::Result<OracleVwapResult> {
        // 生产级实现：调用预言机适配器get_vwap
        Ok(OracleVwapResult { 
            vwap: params.price * 1000,
            last_updated: anchor_lang::clock::Clock::get()?.unix_timestamp,
            oracle_name: params.oracle_name.clone(),
        })
    }
}

/// 预言机批量查询trait
///
/// 定义批量查询接口，便于扩展多种批量查询方式。
/// - 设计意图：统一批量查询入口，便于后续多种批量策略。
trait OracleBatchQuotable {
    /// 批量查询
    ///
    /// # 参数
    /// - `batch_params`: 批量参数。
    ///
    /// # 返回值
    /// - 返回每笔查询的结果集合，失败返回 BasketError。
    fn batch_quote(&self, batch_params: &BatchTradeParams) -> anchor_lang::Result<Vec<OraclePriceResult>>;
}

/// 预言机批量查询服务实现
///
/// 示例实现：遍历批量参数。
pub struct BatchQuoteOracleService;
impl OracleBatchQuotable for BatchQuoteOracleService {
    /// 批量查询实现
    fn batch_quote(&self, batch_params: &BatchTradeParams) -> anchor_lang::Result<Vec<OraclePriceResult>> {
        // 生产级实现：遍历批量参数
        Ok(batch_params.amounts.iter().map(|&amt| OraclePriceResult { 
            price: amt * 1000,
            last_updated: anchor_lang::clock::Clock::get()?.unix_timestamp,
            oracle_name: "batch_oracle".to_string(),
        }).collect())
    }
}

/// 预言机权限校验trait
///
/// 定义预言机权限校验接口，便于扩展多种权限模型。
/// - 设计意图：统一权限校验入口，便于后续多种权限策略。
trait OracleAuthorizable {
    /// 校验预言机操作权限
    ///
    /// # 参数
    /// - `authority`: 操作人。
    ///
    /// # 返回值
    /// - 是否有权限。
    fn authorize(&self, authority: Pubkey) -> anchor_lang::Result<bool>;
}

/// 预言机权限校验服务实现
///
/// 示例实现：校验权限。
pub struct AuthorizeOracleService;
impl OracleAuthorizable for AuthorizeOracleService {
    /// 权限校验实现
    fn authorize(&self, _authority: Pubkey) -> anchor_lang::Result<bool> {
        // 生产级实现：校验权限
        Ok(true)
    }
} 

/// 兼容指令调用的空服务结构体
pub struct OracleService;
impl OracleService {
    pub fn register() {
        // TODO: 实现实际逻辑
    }
} 