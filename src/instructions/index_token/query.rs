//! IndexToken资产query指令
//! 指数代币查询指令实现，支持多种查询功能和数据聚合。

use anchor_lang::prelude::*;
use crate::state::baskets::BasketIndexState;
use crate::core::types::{AssetType, QueryParams};
use crate::services::index_token_service::IndexTokenService;
use crate::oracles::traits::{OracleAdapter, OracleParams};
use crate::dex::adapter::DexAdapter;

/// IndexToken资产query指令账户上下文
#[derive(Accounts)]
pub struct QueryIndexToken<'info> {
    /// 指数代币账户，只读
    pub index_token: Account<'info, BasketIndexState>,
    
    /// Oracle程序账户（可选）
    pub oracle_program: Option<AccountInfo<'info>>,
    
    /// DEX程序账户（可选）
    pub dex_program: Option<AccountInfo<'info>>,
}

/// 指数代币查询参数结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct QueryIndexTokenParams {
    /// 查询类型
    pub query_type: QueryType,
    /// 是否包含价格数据
    pub include_price: bool,
    /// 是否包含流动性数据
    pub include_liquidity: bool,
    /// 是否包含历史数据
    pub include_history: bool,
    /// Oracle名称（可选）
    pub oracle_name: Option<String>,
    /// DEX名称（可选）
    pub dex_name: Option<String>,
}

/// 查询类型枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum QueryType {
    /// 基本信息
    Basic,
    /// 价格信息
    Price,
    /// 流动性信息
    Liquidity,
    /// 历史数据
    History,
    /// 完整信息
    Full,
}

/// 指数代币查询结果结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct IndexTokenQueryResult {
    /// 指数代币ID
    pub index_token_id: Pubkey,
    /// 总价值
    pub total_value: u64,
    /// 当前价格
    pub current_price: Option<u64>,
    /// 24小时价格变化
    pub price_change_24h: Option<i64>,
    /// 总流动性
    pub total_liquidity: Option<u64>,
    /// 支持的DEX列表
    pub supported_dexes: Vec<String>,
    /// 支持的Oracle列表
    pub supported_oracles: Vec<String>,
    /// 查询时间戳
    pub timestamp: i64,
}

/// IndexToken资产query指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - params: 查询参数
/// - 返回: 查询结果
pub fn query_index_token(ctx: Context<QueryIndexToken>, params: QueryIndexTokenParams) -> Result<IndexTokenQueryResult> {
    let index_token = &ctx.accounts.index_token;
    
    // 验证资产类型
    require!(index_token.asset_type == AssetType::IndexToken, crate::errors::index_token_error::IndexTokenError::InvalidAssetType);
    
    // 验证指数代币状态
    index_token.validate()?;
    
    // 创建查询参数
    let query_params = QueryParams {
        asset: index_token.id,
        include_price: params.include_price,
        include_liquidity: params.include_liquidity,
        include_history: params.include_history,
    };
    
    // 执行指数代币查询逻辑
    let service = IndexTokenService::new();
    let result = service.query_index_token(
        index_token,
        &query_params,
        params.oracle_name.as_ref(),
        params.dex_name.as_ref(),
    )?;
    
    // 构建查询结果
    let query_result = IndexTokenQueryResult {
        index_token_id: index_token.id,
        total_value: index_token.total_value,
        current_price: result.current_price,
        price_change_24h: result.price_change_24h,
        total_liquidity: result.total_liquidity,
        supported_dexes: result.supported_dexes,
        supported_oracles: result.supported_oracles,
        timestamp: Clock::get()?.unix_timestamp,
    };
    
    Ok(query_result)
}

/// 指数代币查询错误码
#[error_code]
pub enum QueryIndexTokenError {
    /// 资产类型无效
    #[msg("Invalid asset type")] InvalidAssetType,
    /// 查询参数无效
    #[msg("Invalid query parameters")] InvalidQueryParams,
    /// Oracle不可用
    #[msg("Oracle not available")] OracleNotAvailable,
    /// DEX不可用
    #[msg("DEX not available")] DexNotAvailable,
    /// 查询失败
    #[msg("Query failed")] QueryFailed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::prelude::Pubkey;
    
    /// 测试指数代币查询参数验证
    #[test]
    fn test_query_index_token_params_validation() {
        let params = QueryIndexTokenParams {
            query_type: QueryType::Full,
            include_price: true,
            include_liquidity: true,
            include_history: false,
            oracle_name: Some("pyth".to_string()),
            dex_name: Some("jupiter".to_string()),
        };
        
        assert_eq!(params.query_type, QueryType::Full);
        assert!(params.include_price);
        assert!(params.include_liquidity);
        assert!(!params.include_history);
        assert_eq!(params.oracle_name, Some("pyth".to_string()));
        assert_eq!(params.dex_name, Some("jupiter".to_string()));
    }
    
    /// 测试查询类型枚举
    #[test]
    fn test_query_type_enum() {
        assert_eq!(QueryType::Basic, QueryType::Basic);
        assert_ne!(QueryType::Basic, QueryType::Price);
        assert_ne!(QueryType::Price, QueryType::Liquidity);
    }
} 