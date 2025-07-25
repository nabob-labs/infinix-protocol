//!
//! DEX Offchain REST API Module
//!
//! 本模块实现 DEX 离线服务的 RESTful API 客户端，支持 HTTP 请求、响应解析、错误处理等，确保与 DEX 后端安全、合规、高效交互。

use std::collections::HashMap;
use std::time::Duration;
use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION};

/// REST 客户端结构体。
pub struct RestClient {
    pub base_url: String,         // API 基础 URL
    pub api_key: String,          // API 密钥
    pub timeout_secs: u64,        // 超时时间（秒）
}

impl RestClient {
    /// 创建新的 REST 客户端。
    pub fn new(base_url: &str, api_key: &str, timeout_secs: u64) -> Self {
        Self {
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            timeout_secs,
        }
    }
    /// 构建 HTTP 客户端。
    fn build_client(&self) -> Client {
        Client::builder()
            .timeout(Duration::from_secs(self.timeout_secs))
            .build()
            .expect("Failed to build HTTP client")
    }
    /// 构建请求头。
    fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if !self.api_key.is_empty() {
            headers.insert(AUTHORIZATION, HeaderValue::from_str(&self.api_key).unwrap());
        }
        headers
    }
    /// 发送 GET 请求。
    pub fn get(&self, endpoint: &str, params: Option<&HashMap<&str, &str>>) -> Result<Response, reqwest::Error> {
        let client = self.build_client();
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        let req = client.get(&url).headers(self.build_headers());
        let req = if let Some(p) = params {
            req.query(p)
        } else {
            req
        };
        req.send()
    }
    /// 发送 POST 请求。
    pub fn post(&self, endpoint: &str, body: &str) -> Result<Response, reqwest::Error> {
        let client = self.build_client();
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        client.post(&url)
            .headers(self.build_headers())
            .body(body.to_string())
            .send()
    }
}
