//! Asset资产query指令
//! 资产查询指令实现，支持多种查询功能和数据聚合。

use anchor_lang::prelude::*;
use crate::core::types::*;
use crate::services::asset_service::AssetService;
use crate::oracles::traits::{OracleAdapter, OracleParams};
use crate::dex::adapter::DexAdapter;
use crate::state::baskets::BasketIndexState; // 篮子状态类型

/// Asset资产query指令账户上下文
#[derive(Accounts)]
pub struct QueryAsset<'info> {
    /// 资产账户，只读
    pub asset: Account<'info, BasketIndexState>,
    
    /// Oracle程序账户（可选）
    pub oracle_program: Option<AccountInfo<'info>>,
    
    /// DEX程序账户（可选）
    pub dex_program: Option<AccountInfo<'info>>,
}

/// 资产查询参数结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct QueryAssetParams {
    /// 查询类型
    pub query_type: AssetQueryType,
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

/// 资产查询类型枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum AssetQueryType {
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

/// 资产查询结果结构体
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AssetQueryResult {
    /// 资产ID
    pub asset_id: Pubkey,
    /// 资产类型
    pub asset_type: AssetType,
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

/// Asset资产query指令实现
/// - ctx: Anchor账户上下文，自动校验权限与生命周期
/// - params: 查询参数
/// - 返回: 查询结果
pub fn query_asset(ctx: Context<QueryAsset>, params: QueryAssetParams) -> anchor_lang::Result<AssetQueryResult> {
    let asset = &ctx.accounts.asset;
    
    // 验证资产状态
    asset.validate()?;
    
    // 创建查询参数
    let query_params = QueryParams {
        asset: asset.id,
        include_price: params.include_price,
        include_liquidity: params.include_liquidity,
        include_history: params.include_history,
    };
    
    // 执行资产查询逻辑
    let service = AssetService::new();
    let result = service.query_asset(
        asset,
        &query_params,
        params.oracle_name.as_ref(),
        params.dex_name.as_ref(),
    )?;
    
    // 构建查询结果
    let query_result = AssetQueryResult {
        asset_id: asset.id,
        asset_type: asset.asset_type,
        total_value: asset.total_value,
        current_price: result.current_price,
        price_change_24h: result.price_change_24h,
        total_liquidity: result.total_liquidity,
        supported_dexes: result.supported_dexes,
        supported_oracles: result.supported_oracles,
        timestamp: Clock::get()?.unix_timestamp,
    };
    
    Ok(query_result)
}

/// 资产查询错误码
#[error_code]
pub enum QueryAssetError {
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
    
    /// 测试资产查询参数验证
    #[test]
    fn test_query_asset_params_validation() {
        let params = QueryAssetParams {
            query_type: AssetQueryType::Full,
            include_price: true,
            include_liquidity: true,
            include_history: false,
            oracle_name: Some("pyth".to_string()),
            dex_name: Some("jupiter".to_string()),
        };
        
        assert_eq!(params.query_type, AssetQueryType::Full);
        assert!(params.include_price);
        assert!(params.include_liquidity);
        assert!(!params.include_history);
        assert_eq!(params.oracle_name, Some("pyth".to_string()));
        assert_eq!(params.dex_name, Some("jupiter".to_string()));
    }
    
    /// 测试查询类型枚举
    #[test]
    fn test_asset_query_type_enum() {
        assert_eq!(AssetQueryType::Basic, AssetQueryType::Basic);
        assert_ne!(AssetQueryType::Basic, AssetQueryType::Price);
        assert_ne!(AssetQueryType::Price, AssetQueryType::Liquidity);
    }
} 