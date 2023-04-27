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


    async fn run_server_01() -> anyhow::Result<SocketAddr> {
        let server = ServerBuilder::default().build("127.0.0.1:0".parse::<SocketAddr>()?).await?;
        let mut module = RpcModule::new(());
        module.register_method("say_hello", |_, _| "lo")?;
    
        let addr = server.local_addr()?;
        let handle = server.start(module)?;
    
        tokio::spawn(handle.stopped());
    
        Ok(addr)
    }

    #[tokio::test]
    async fn run_jsonrpc_server_02() -> anyhow::Result<()> {

        let server_addr = run_server_01().await?;
        let url = format!("http://{:?}", server_addr);
        println!("addr: {:?}", url);

        let client = HttpClientBuilder::default().build(url)?;
        let params = rpc_params![];
        // curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "say_hello", "params": []}' http://localhost:33121/
        let response: Result<String, _> = client.request("say_hello", params).await;
        tracing::info!("r: {:?}", response);
        println!("result: {:?}", response);

        Ok(())
    }

    
}

