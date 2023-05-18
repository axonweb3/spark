use async_trait::async_trait;
use common::{traits::axon_rpc_client::SubmitProcess, types::ckb_rpc_client::Cell};

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
