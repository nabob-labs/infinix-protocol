//!
//! Oracle Offchain Library Entry
//!
//! 本模块为预言机离线服务库主入口，统一导出所有子模块，便于外部集成与调用。

// 导入并公开所有子模块。
pub mod chainlink;    // Chainlink 适配器模块
pub mod config;       // 配置模块
pub mod factory;      // 工厂模块
pub mod logging;      // 日志模块
pub mod pyth;         // Pyth 适配器模块
pub mod rest;         // REST API 客户端模块
pub mod switchboard;  // Switchboard 适配器模块
pub mod traits;       // 通用 trait 模块

// 重新导出常用类型和函数，便于外部访问。
pub use chainlink::*;
pub use config::*;
pub use factory::*;
pub use logging::*;
pub use pyth::*;
pub use rest::*;
pub use switchboard::*;
pub use traits::*;

/// 预言机离线服务库版本号。
pub const ORACLE_OFFCHAIN_VERSION: &str = "1.0.0";

use actix_web::{App, HttpServer};

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    // 日志初始化
    tracing_subscriber::fmt::init();
    // 配置热加载与Adapter自动初始化
    let config_path = "oracle_config.toml";
    tokio::spawn(async move {
        config::watch_config(config_path, |cfg| {
            tracing::info!("[Config] Oracle config changed: {:?}", cfg);
            // 这里可根据cfg.adapters动态注册/注销Adapter并调用init
            // 示例：遍历cfg.adapters，注册/注销Adapter
        })
        .await;
    });
    HttpServer::new(|| App::new().configure(rest::configure))
        .bind(("0.0.0.0", 8081))?
        .run()
        .await
}
