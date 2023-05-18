use common::{
    traits::{axon_rpc_client::SubmitProcess, ckb_rpc_client::CkbRpc},
    types::ckb_rpc_client::{Order, RpcSearchKey, TipState},
};

pub struct CellProcess<T, S, R> {
    key:      RpcSearchKey,
    scan_tip: T,
    rpc:      R,
    process:  S,
    stop:     bool,
}

impl<T, S, R> CellProcess<T, S, R>
where
    T: TipState,
    S: SubmitProcess,
    R: CkbRpc,
{
    pub fn new(key: RpcSearchKey, tip: T, rpc: R, process: S) -> Self {
        Self {
            key,
            scan_tip: tip,
            rpc,
            process,
            stop: false,
        }
    }

    pub async fn run(&mut self) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(8));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            if self.stop || self.process.is_closed() {
                break;
            }
            self.scan(&mut interval).await;
        }
    }

    async fn scan(&mut self, interval: &mut tokio::time::Interval) {
        let indexer_tip = rpc_get!(self.rpc.get_indexer_tip());
        let old_tip = *self.scan_tip.load();

        if indexer_tip.block_number.value().saturating_sub(24) > old_tip.value() {
            // use tip - 24 as new tip
            let new_tip = indexer_tip.block_number.value().saturating_sub(24).into();

            let search_key = self.key.clone().into_key(Some([old_tip, new_tip]));

            let txs = rpc_get!(self
                .rpc
                .get_cells(search_key.clone(), Order::Asc, 1.into(), None));

            if !txs.objects.is_empty() {
                let cell = txs.objects.first().unwrap();
                self.process.notify_axon(cell).await;
            }
            self.scan_tip.update(new_tip);
        } else {
            interval.tick().await;
        }
    }
}
