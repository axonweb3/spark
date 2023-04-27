#[cfg(test)]
mod tests {
    use jsonrpsee::RpcModule;
    use jsonrpsee::core::async_trait;
    use jsonrpsee::core::client::ClientT;
    use jsonrpsee::http_client::HttpClientBuilder;

    use jsonrpsee::server::{ServerBuilder, ServerHandle};

    use jsonrpsee::{core::Error, proc_macros::rpc}; 
    use std::io::Read;
    use std::net::{TcpListener, SocketAddr};
    pub use ethereum_types::{
        BigEndianHash, Bloom, Public, Secret, Signature, H128, H160, H256, H512, H520, H64, U128, U256,
        U512, U64,
    };
    use jsonrpsee::rpc_params;

    use crate::QueryError;


    async fn run_server_01() -> Result<SocketAddr, QueryError>  {
        // let server = jsonrpsee::server::ServerBuilder::new().http_only()
        // .build("127.0.0.1:0".parse::<SocketAddr>().unwrap())
        // .await
        // .map_err(|e| QueryError::HttpServer(e.to_string()))?;
        let server = ServerBuilder::default()
        .build("127.0.0.1:22".parse::<SocketAddr>().unwrap())
        .await.map_err(|e| QueryError::HttpServer(e.to_string()))?;

        // println!("===={}==", server);

        let mut module = RpcModule::new(());

        module.register_method("say_hello", |_, _| "lo").unwrap();
    
        let addr = server.local_addr().unwrap();
        let handle = server.start(module).unwrap();
    
        tokio::spawn(handle.stopped());
    
        Ok(addr)
    }

    async fn run_server_02() -> Result<SocketAddr, QueryError> {
        let mut module = Method{}.into_rpc();
        println!("++++++++++++++++++++++++++++++++++");
        let server = jsonrpsee::server::ServerBuilder::new().http_only()
        .build("127.0.0.1:0".parse::<SocketAddr>().unwrap())
        .await
        .map_err(|e| QueryError::HttpServer(e.to_string()))?;
        println!("++++++++++++++++++++++++++++++++++");
        println!("addr: {:?}", server.local_addr().unwrap());
        // let _a = server.local_addr().unwrap();
        // eprintln!("{:?}", server.unwrap_err());
        // if let Err(e) = server.start(method) {
        //     println!("Error starting server: {:?}", e);
        // } else {
        //     println!("Server started successfully");
        // }
        
        // let mut module = RpcModule::new(());

        module.register_method("say_hello", |_, _| "lo").unwrap();
    
        let addr = server.local_addr().unwrap();
        let handle = server.start(module).unwrap();
    
        tokio::spawn(handle.stopped());
    
        Ok(addr)
    //     curl http://127.0.0.1:43275 \
    // -X POST \
    // -H "Content-Type: application/json" \
    // --data '{"jsonrpc":"2.0","method":"eth_sendRawTransaction","params":[{}],"id":1}'
    }

    #[tokio::test]
    async fn run_jsonrpc_server_01() -> Result<(), QueryError>{

        let server_addr = run_server_01().await?;
        let url = format!("http://{:?}", server_addr);
        println!("addr: {:?}", url);

        let client = HttpClientBuilder::default().build(url).unwrap();
        let params = rpc_params![];
        // curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "say_hello", "params": []}' http://localhost:33121/
        let response: Result<String, _> = client.request("say_hello", params).await;
        tracing::info!("r: {:?}", response);
        println!("result: {:?}", response);

        Ok(())
    }

    #[tokio::test]
    async fn run_jsonrpc_server_02() -> Result<(), QueryError>{

        let server_addr = run_server_02().await?;
        let url = format!("http://{:?}", server_addr);
        println!("addr: {:?}", url);

        let client = HttpClientBuilder::default().build(url).unwrap();
        let params = rpc_params![];
        // curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "say_hello", "params": []}' http://localhost:33121/
        let response: Result<String, _> = client.request("say_hello", params).await;
        tracing::info!("r: {:?}", response);
        println!("result: {:?}", response);

        Ok(())
    }

    #[tokio::test]
    async fn run_jsonrpc_server_03() -> Result<(), QueryError>{

        let server_addr = run_server_02().await?;
        let url = format!("http://{:?}", server_addr);
        println!("addr: {:?}", url);

        let client = HttpClientBuilder::default().build(url).unwrap();
        let params = rpc_params![];
        // curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "say_hello", "params": []}' http://localhost:33121/
        let response: Result<Vec<H256>, _> = client.request("eth_sendRawTransaction", params).await;
        tracing::info!("r: {:?}", response);
        println!("result: {:?}", response);

        Ok(())
    }


    struct Method {}

    #[async_trait]
    impl TestRpcServer for Method {

        async fn send_raw_transaction(&self) -> Vec<H256> {
            println!("!!!!!!!!!!!!!!!");
            let res = H256::default();
            vec![res]
        }
    }

    #[rpc(server)]
    pub trait TestRpc {
        /// Sends signed transaction, returning its hash.
        #[method(name = "eth_sendRawTransaction")]
        async fn send_raw_transaction(&self) -> Vec<H256>;
    }

    
}

