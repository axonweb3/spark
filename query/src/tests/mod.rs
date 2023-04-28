#[cfg(test)]
mod tests {
    use common::types::H256;
    use jsonrpsee::core::client::ClientT;
    use jsonrpsee::http_client::HttpClientBuilder;
    use jsonrpsee::server::ServerBuilder;

    use std::net::SocketAddr;

    use jsonrpsee::rpc_params;

    use crate::QueryError;

    use jsonrpsee::core::async_trait;

    use jsonrpsee::proc_macros::rpc;

    #[rpc(server)]
    pub trait TestKickerRpc {
        /// Sends signed transaction, returning its hash.
        #[method(name = "getStakeHistory")]
        async fn get_stake_history(&self) -> Vec<H256>;

        #[method(name = "getRewardHistory")]
        async fn get_reward_history(&self) -> Vec<H256>;

        #[method(name = "getStakeAmountByEpoch")]
        async fn get_stake_amount_by_epoch(&self) -> Vec<H256>;

        #[method(name = "getTopStakeAddress")]
        async fn get_top_stake_address(&self) -> Vec<H256>;

        #[method(name = "getStakeState")]
        async fn get_stake_state(&self) -> Vec<H256>;

        #[method(name = "getRewardState")]
        async fn get_reward_state(&self) -> Vec<H256>;

        #[method(name = "getChainState")]
        async fn get_chain_state(&self) -> Vec<H256>;
    }

    pub struct RpcModule {}

    impl RpcModule {
        pub fn new() -> Self {
            RpcModule {}
        }
    }

    #[async_trait]
    impl TestKickerRpcServer for RpcModule {
        async fn get_stake_history(&self) -> Vec<H256> {
            let res = H256::default();
            vec![res]
        }

        async fn get_reward_history(&self) -> Vec<H256> {
            let res = H256::default();
            vec![res]
        }

        async fn get_stake_amount_by_epoch(&self) -> Vec<H256> {
            let res = H256::default();
            vec![res]
        }

        async fn get_top_stake_address(&self) -> Vec<H256> {
            let res = H256::default();
            vec![res]
        }

        async fn get_stake_state(&self) -> Vec<H256> {
            let res = H256::default();
            vec![res]
        }

        async fn get_reward_state(&self) -> Vec<H256> {
            let res = H256::default();
            vec![res]
        }

        async fn get_chain_state(&self) -> Vec<H256> {
            let res = H256::default();
            vec![res]
        }
    }
    async fn run_server() -> Result<SocketAddr, QueryError> {
        let module = RpcModule::new().into_rpc();
        let server = ServerBuilder::new()
            .http_only()
            .build("127.0.0.1:0".parse::<SocketAddr>().unwrap())
            .await
            .map_err(|e| QueryError::HttpServer(e.to_string()))?;
        println!("addr: {:?}", server.local_addr().unwrap());
        // module.register_method("a_method", |_, _| "lo").unwrap();

        let addr = server.local_addr().unwrap();
        let handle = server.start(module).unwrap();

        tokio::spawn(handle.stopped());

        Ok(addr)
    }

    #[tokio::test]
    async fn mock_jsonrpc_server() -> Result<(), QueryError> {
        let server_addr = run_server().await?;
        let url = format!("http://{:?}", server_addr);
        // println!("addr: {:?}", url);
        let client = HttpClientBuilder::default().build(url).unwrap();

        let params = rpc_params![];
        // curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "say_hello", "params": []}' http://localhost:33121/
        let response: Result<Vec<H256>, _> = client.request("getStakeHistory", params).await;
        println!("result: {:?}", response);

        let params = rpc_params![];
        let response: Result<Vec<H256>, _> = client.request("getRewardHistory", params).await;
        println!("result: {:?}", response);

        let params = rpc_params![];
        let response: Result<Vec<H256>, _> = client.request("getStakeAmountByEpoch", params).await;
        println!("result: {:?}", response);

        let params = rpc_params![];
        let response: Result<Vec<H256>, _> = client.request("getTopStakeAddress", params).await;
        println!("result: {:?}", response);

        let params = rpc_params![];
        let response: Result<Vec<H256>, _> = client.request("getStakeState", params).await;
        println!("result: {:?}", response);

        let params = rpc_params![];
        let response: Result<Vec<H256>, _> = client.request("getRewardState", params).await;
        println!("result: {:?}", response);

        let params = rpc_params![];
        let response: Result<Vec<H256>, _> = client.request("getChainState", params).await;
        println!("result: {:?}", response);

        Ok(())
    }
}
