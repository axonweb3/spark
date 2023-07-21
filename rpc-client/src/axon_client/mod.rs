use std::sync::{atomic::AtomicU64, Arc};

use anyhow::Result;
use async_trait::async_trait;
use common::{
    traits::axon_rpc_client::{AxonRpc, AxonWsRpc, SubmitProcess},
    types::{
        api::ChainState,
        axon_rpc_client::{Block, Header, LatestCheckPointInfo, Metadata},
        ckb_rpc_client::Cell,
    },
};
use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
    ws_client::WsClientBuilder,
};
use reqwest::Url;

macro_rules! request {
    ($client:expr, $method:expr $(, $param:expr)*) => {
        {
            $client
                .request($method, rpc_params![$($param),*])
                .await
                .unwrap()
        }
    };
}

pub struct RpcSubmit;

#[async_trait]
impl SubmitProcess for RpcSubmit {
    fn is_closed(&self) -> bool {
        false
    }

    async fn notify_axon(&mut self, cell: &Cell) -> bool {
        println!("cell: {:?}", cell);
        true
    }
}

#[derive(Debug)]
pub struct AxonRpcClient {
    http_client: HttpClient,
    _id:         Arc<AtomicU64>,
}

impl AxonRpcClient {
    pub async fn new(axon_http_url: &str, _axon_ws_url: &str) -> Self {
        let axon_http_url =
            Url::parse(axon_http_url).expect("axon http url, e.g. \"http://localhost:8000\"");
        // todo: modify axon ws server
        // let ws = WsClientBuilder::default().build(axon_ws_url).await.unwrap();
        let http = HttpClientBuilder::default().build(axon_http_url).unwrap();
        AxonRpcClient {
            http_client: http,
            _id:         Arc::new(AtomicU64::new(0)),
        }
    }
}

#[async_trait]
impl AxonWsRpc for AxonRpcClient {
    async fn sub_axon_header(&self) -> Result<ChainState> {
        // todo: create rpc ws connection to axon
        let ws = WsClientBuilder::default()
            .build("ws://localhost:8000/socket")
            .await
            .unwrap();
        let _response: Header = ws.request("eth_subscription", rpc_params![]).await?;
        Ok(ChainState::default())
    }
}

#[async_trait]
impl AxonRpc for AxonRpcClient {
    async fn get_checkpoint_info(&self) -> Result<LatestCheckPointInfo> {
        let metadata: Metadata = request!(self.http_client, "axon_getCurrentMetadata");
        let block: Block = request!(self.http_client, "axon_getBlockById", "latest");
        let last_block_number = format!("0x{:X}", block.header.number - 1);
        let last_block: Block = request!(self.http_client, "axon_getBlockById", last_block_number);
        Ok(LatestCheckPointInfo::new(
            &last_block.header,
            &block.header,
            &metadata,
        ))
    }
}

mod tests {

    // #[tokio::test]
    async fn _test_http_client() {
        use super::*;
        let client = HttpClientBuilder::default()
            .build("http://localhost:8000")
            .unwrap();
        let params = rpc_params![];
        let metadata: Metadata = client
            .request("axon_getCurrentMetadata", params)
            .await
            .unwrap();
        println!("r: {:?}", metadata);
        let params = rpc_params!["latest"];
        let block: Block = client.request("axon_getBlockById", params).await.unwrap();
        println!("r: {:?}", block);
    }

    // #[tokio::test]
    async fn _test_http_client_macro() {
        use super::*;
        let http = HttpClientBuilder::default()
            .build("http://localhost:8000")
            .unwrap();
        // let metadata: Metadata = request!(http, "axon_getCurrentMetadata");
        // println!("r: {:?}", metadata);

        let block: Block = request!(http, "axon_getBlockById", "latest");
        let last_block_number = format!("0x{:X}", block.header.number - 1);
        println!("last_block_number: {:?}", last_block_number);
        println!("current_block_number: {:?}", block.header.number);
        let block: Block = request!(http, "axon_getBlockById", last_block_number);
        println!("verify: {:?}", block.header.number);
    }

    // #[tokio::test]
    async fn _test_ws_client() {
        use super::*;
        let url = "ws://localhost:8000/socket";
        let client = WsClientBuilder::default().build(&url).await.unwrap();
        let params = rpc_params![];
        let metadata: Metadata = client
            .request("axon_getCurrentMetadata", params)
            .await
            .unwrap();
        println!("r: {:?}", metadata);
        let params = rpc_params!["latest"];
        let block: Block = client.request("axon_getBlockById", params).await.unwrap();
        println!("r: {:?}", block);
    }
}
