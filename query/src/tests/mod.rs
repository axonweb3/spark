#[cfg(test)]
mod tests {
    use anyhow::Error;
    use jsonrpsee::types::Params;
    use reqwest::Client as Cli;
    use serde_json::{json, Value};

    #[tokio::test]
    async fn test_eth_estimate_gas() {
        let url = "http://192.168.0.103:8000";
        let data = json!({
            "jsonrpc": "2.0",
            "method": "eth_estimateGas",
            "params": [
                {
                    "from": "0x2892467CdE7A331D367ce7f5eAe3dD99DFDe5f23",
                    "to": "0xa990077c3205cbDf861e17Fa532eeB069cE9fF96",
                    "value": {
                        "type": "BigNumber",
                        "hex": "0x011c37937e080000"
                    },
                    "accessList": null
                }
            ],
            "id": 1
        });

        let client = Cli::new();
        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .body(data.to_string())
            .send()
            .await
            .unwrap();

        let json = response.text().await.unwrap();
        let expected_json = "{expected_json}";

        assert_eq!(json, expected_json);
    }

    //     use jsonrpsee::{server::ServerBuilder, types::Error, RequestContext,
    // ServerResult};

    // async fn eth_estimate_gas(_ctx: RequestContext, _params:
    // Vec<serde_json::Value>) -> ServerResult<serde_json::Value> {     // Replace
    // this with your actual implementation     println!("hello");

    //     Ok(serde_json::Value::Null)
    // }

    // #[tokio::test]
    // async fn test_json_rpc() {
    //     let mut io = ServerBuilder::default().build("127.0.0.1:8080").unwrap();

    //     io.register_method("eth_estimateGas", eth_estimate_gas.into());

    //     loop {
    //         if let Err(e) = io.next().await {
    //             match e {
    //                 Error::Internal(_) => break,
    //                 e => eprintln!("Server error: {}", e),
    //             }
    //         }
    //     }
    // }

    use std::net::SocketAddr;

    use jsonrpsee::client_transport::ws::{Uri, WsTransportClientBuilder};
    use jsonrpsee::core::client::{Client, ClientBuilder, ClientT};
    use jsonrpsee::rpc_params;
    use jsonrpsee::server::{RpcModule, ServerBuilder};

    #[tokio::test]
    async fn test_history_json_rpc() -> anyhow::Result<()> {
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .try_init()
            .expect("setting default subscriber failed");

        let addr = run_server().await?;
        let uri: Uri = format!("ws://{}", addr).parse()?;

        let (tx, rx) = WsTransportClientBuilder::default().build(uri).await?;
        let client: Client = ClientBuilder::default().build_with_tokio(tx, rx);
        let response: String = client.request("say_hello", rpc_params![]).await?;
        tracing::info!("response: {:?}", response);

        Ok(())
    }

    fn get_reward_history(_a:u128, _b:u128)-> String {
        println!("history");
        String::from("3")
    }

    fn because_you() -> Result<Value, Error> {
        println!("i am used");
        Ok("hai".into())
    }

    async fn run_server() -> anyhow::Result<SocketAddr> {
        let server = ServerBuilder::default().build("127.0.0.1:0").await?;
        let mut module = RpcModule::new(());
        let input = Params::new(Some("1"));
        module.register_method("say_hello", |_, _| {
            because_you();
        })?;
        let addr = server.local_addr()?;

        let handle = server.start(module)?;

        // In this example we don't care about doing shutdown so let's it run forever.
        // You may use the `ServerHandle` to shut it down or manage it yourself.
        tokio::spawn(handle.stopped());

        Ok(addr)
    }
}
