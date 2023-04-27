#[cfg(test)]
mod tests {
    use jsonrpsee::RpcModule;
    use jsonrpsee::core::async_trait;
    use jsonrpsee::core::client::ClientT;
    use jsonrpsee::http_client::HttpClientBuilder;
    // pub async fn run_jsonrpc_server<Adapter: APIAdapter + 'static>(
    //     config: Config,
    //     adapter: Arc<Adapter>,
    // ) -> ProtocolResult<(Option<ServerHandle>)> {
    //     let mut rpc = r#impl::Web3RpcImpl::new(Arc::clone(&adapter), config.rpc.gas_cap).into_rpc();
    
    
    //     if let Some(addr) = config.rpc.http_listening_address {
    //         let server = ServerBuilder::new()
    //             .http_only()
    //             .max_request_body_size(config.rpc.max_payload_size)
    //             .max_response_body_size(config.rpc.max_payload_size)
    //             .max_connections(config.rpc.maxconn)
    //             .build(addr)
    //             .await
    //             .map_err(|e| APIError::HttpServer(e.to_string()))?;
    
    //         ret.0 = Some(
    //             server
    //                 .start(rpc.clone())
    //                 .map_err(|e| APIError::HttpServer(e.to_string()))?,
    //         );
    //     }
    
    
    //     Ok(ret)
    // }
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
    #[tokio::test]
    async fn run_jsonrpc_server() {
        let listener = std::net::TcpListener::bind("127.0.0.1:1234").unwrap();
        let occupied_addr = listener.local_addr().unwrap();
        let addrs: &[std::net::SocketAddr] = &[
            occupied_addr,
            "127.0.0.1:0".parse().unwrap(),
        ];
        assert!(ServerBuilder::default().build(occupied_addr).await.is_err());
        assert!(ServerBuilder::default().build(addrs).await.is_ok());
        
    }

    #[tokio::test]
    async fn run_server() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let occupied_addr = listener.local_addr().unwrap();

        println!("Listening on {}", occupied_addr);

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buffer = [0; 1024];
                    match stream.read(&mut buffer) {
                        Ok(size) => {
                            println!("Received: {}", String::from_utf8_lossy(&buffer[..size]));
                        }
                        Err(e) => {
                            println!("Error reading from stream: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Error establishing connection: {:?}", e);
                }
            }
        }


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


    #[tokio::test]
    async fn run_rpc_server() -> Result<(), QueryError> {
        let method = Method{}.into_rpc();
        println!("++++++++++++++++++++++++++++++++++");
        let server = jsonrpsee::server::ServerBuilder::new().http_only()
        .build("127.0.0.1:0".parse::<SocketAddr>().unwrap())
        .await
        .map_err(|e| QueryError::HttpServer(e.to_string()))?;
        println!("++++++++++++++++++++++++++++++++++");
        println!("addr: {:?}", server.local_addr().unwrap());
        // let _a = server.local_addr().unwrap();
        // eprintln!("{:?}", server.unwrap_err());
        if let Err(e) = server.start(method) {
            println!("Error starting server: {:?}", e);
        } else {
            println!("Server started successfully");
        }
        
        Ok(())
    //     curl http://127.0.0.1:43275 \
    // -X POST \
    // -H "Content-Type: application/json" \
    // --data '{"jsonrpc":"2.0","method":"eth_sendRawTransaction","params":[{}],"id":1}'
    }

    // #[tokio::test]
    // async fn run_rpc_loop_server() -> Result<(), QueryError> {
    //     let server = ServerBuilder::default().build(&"127.0.0.1:8080".parse().unwrap());
    //     let methods = eth_methods();

    //     loop {
    //         tokio::select! {
    //             event = server.next_event() => {
    //                 if let Ok(Some(request)) = event {
    //                     let response = methods.handle(request).await.unwrap();
    //                     let _ = server.respond(response).await;
    //                 }
    //             },
    //         }
    //     }
    // }

    // fn eth_methods() -> jsonrpsee::RpcModule<()> {
    //     jsonrpsee::RpcModule::new(serde_json::json!({
    //         "eth_sendRawTransaction": {
    //             "handler": async move |_params: Vec<String>| -> Result<String, jsonrpsee::core::Error> {
    //                 println!("hello");
    //                 Ok("hello".to_owned())
    //             },
    //             "params": [
    //                 {
    //                     "name": "data",
    //                     "type": "String"
    //                 }
    //             ],
    //             "returns": "String"
    //         }
    //     }))
    // }


    #[tokio::test]
    async fn run_jsonrpc_server_01() -> Result<(), QueryError>  {
        let server = ServerBuilder::default()
		.build("127.0.0.1:0".parse::<SocketAddr>().unwrap())
		.await.unwrap();

        let mut module = RpcModule::new(());
        module.register_method("say_hello", |_, _| {
            println!("say_hello method called!");
            "Hello there!!"
        }).unwrap();

        let addr = server.local_addr().unwrap();
        let handle = server.start(module).unwrap();
        println!("addr: {:?}", addr);

        // In this example we don't care about doing shutdown so let's it run forever.
        // You may use the `ServerHandle` to shut it down or manage it yourself.
        // tokio::spawn(handle.stopped());
        loop {
            
        }
        Ok(())
    }

    //=====================
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


// curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "say_hello", "params": []}' http://localhost:33121/