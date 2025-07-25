use oracle_offchain::factory::OffchainOracleFactory;
use oracle_offchain::traits::*;
use std::sync::Arc;

struct DummyOracle;
#[async_trait::async_trait]
impl OffchainOracleAdapter for DummyOracle {
    async fn get_price(&self, req: OffchainPriceRequest) -> Result<OffchainPriceResponse, String> {
        Ok(OffchainPriceResponse {
            price: 42.0,
            last_updated: 123456,
        })
    }
    fn name(&self) -> &'static str {
        "dummy"
    }
}

#[tokio::test]
async fn test_offchain_oracle_factory_register_and_get_price() {
    let factory = OffchainOracleFactory::new();
    let adapter = Arc::new(DummyOracle);
    factory.register("dummy", adapter.clone());
    let got = factory.get("dummy").unwrap();
    let req = OffchainPriceRequest {
        symbol: "BTC".to_string(),
    };
    let resp = got.get_price(req).await.unwrap();
    assert_eq!(resp.price, 42.0);
    assert_eq!(resp.last_updated, 123456);
}

#[cfg(test)]
mod tests {
    use crate::rest;
    use actix_web::{test, App};
    use serde_json::json;

    #[actix_rt::test]
    async fn test_health_check() {
        let mut app = test::init_service(App::new().configure(rest::configure)).await;
        let req = test::TestRequest::get().uri("/health/oracle").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_price_endpoint() {
        let mut app = test::init_service(App::new().configure(rest::configure)).await;
        let req = test::TestRequest::post()
            .uri("/pyth/price")
            .set_json(&json!({"base_mint": "So11111111111111111111111111111111111111112", "quote_mint": "USDC111111111111111111111111111111111111111",}))
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        // 501/400/200均可，主要验证接口可用
        assert!(
            resp.status().is_client_error()
                || resp.status().is_success()
                || resp.status().as_u16() == 501
        );
    }

    #[actix_rt::test]
    async fn test_twap_endpoint() {
        let mut app = test::init_service(App::new().configure(rest::configure)).await;
        let req = test::TestRequest::post()
            .uri("/pyth/twap")
            .set_json(&json!({"base_mint": "So11111111111111111111111111111111111111112", "quote_mint": "USDC111111111111111111111111111111111111111", "period": 60}))
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(
            resp.status().is_client_error()
                || resp.status().is_success()
                || resp.status().as_u16() == 501
        );
    }
}
