use async_trait::async_trait;

use crate::types::ckb_rpc_client::Cell;

#[async_trait]
pub trait SubmitProcess {
    fn is_closed(&self) -> bool;
    // if false return, it means this cell process should be shutdown
    async fn notify_axon(&mut self, cell: &Cell) -> bool;
}
