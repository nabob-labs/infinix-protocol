use dex_offchain::factory::OffchainDexFactory;
use dex_offchain::traits::*;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use crate::rest;
    use actix_web::{test, App};

    #[actix_rt::test]
    async fn test_health_check() {
        let mut app = test::init_service(App::new().configure(rest::configure)).await;
        let req = test::TestRequest::get().uri("/health/dex").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }
}

struct DummyDex;
#[async_trait::async_trait]
impl OffchainDexAdapter for DummyDex {
    async fn swap(&self, req: OffchainSwapRequest) -> Result<OffchainSwapResponse, String> {
        Ok(OffchainSwapResponse {
            amount_out: req.amount_in * 2,
            tx_hash: "dummyhash".to_string(),
        })
    }
    fn name(&self) -> &'static str {
        "dummy"
    }
}

#[tokio::test]
async fn test_offchain_dex_factory_register_and_swap() {
    let factory = OffchainDexFactory::new();
    let adapter = Arc::new(DummyDex);
    factory.register("dummy", adapter.clone());
    let got = factory.get("dummy").unwrap();
    let req = OffchainSwapRequest {
        token_in: "A".to_string(),
        token_out: "B".to_string(),
        amount_in: 10,
    };
    let resp = got.swap(req).await.unwrap();
    assert_eq!(resp.amount_out, 20);
    assert_eq!(resp.tx_hash, "dummyhash");
}
