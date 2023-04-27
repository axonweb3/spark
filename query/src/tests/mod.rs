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
        module.register_method("say_hello", |_, _| "l222222222o")?;
    
        let addr = server.local_addr()?;
        let handle = server.start(module)?;
    
        // In this example we don't care about doing shutdown so let's it run forever.
        // You may use the `ServerHandle` to shut it down or manage it yourself.
        tokio::spawn(handle.stopped());
    
        Ok(addr)
    }

    #[tokio::test]
    async fn run_jsonrpc_server_02() -> anyhow::Result<()> {
        // let filter = tracing_subscriber::EnvFilter::try_from_default_env()?
        //     .add_directive("jsonrpsee[method_call{name = \"say_hello\"}]=trace".parse()?);
        // tracing_subscriber::FmtSubscriber::builder().with_env_filter(filter).finish().try_init()?;

        let server_addr = run_server_01().await?;
        let url = format!("http://{:?}", server_addr);
        println!("!!!!!!!!!!!!addr: {:?}", url);
        // let middleware = tower::ServiceBuilder::new()
        // .layer(
        //     TraceLayer::new_for_http()
        //         .on_request(
        //             |request: &hyper::Request<hyper::Body>, _span: &tracing::Span| tracing::info!(request = ?request, "on_request"),
        //         )
        //         .on_body_chunk(|chunk: &Bytes, latency: Duration, _: &tracing::Span| {
        //             tracing::info!(size_bytes = chunk.len(), latency = ?latency, "sending body chunk")
        //         })
        //         .make_span_with(DefaultMakeSpan::new().include_headers(true))
        //         .on_response(DefaultOnResponse::new().include_headers(true).latency_unit(LatencyUnit::Micros)),
        // );
       
        // let client = HttpClientBuilder::default().set_middleware(middleware).build(url)?;
        let client = HttpClientBuilder::default().build(url)?;
        let params = rpc_params![1_u64, 2, 3, 1999];
        let response: Result<String, _> = client.request("say_hello", params).await;
        tracing::info!("r: {:?}", response);
        println!("!!!!!!!!!!!!result: {:?}", response);

        Ok(())
    }

}
