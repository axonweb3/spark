use std::sync::{
    atomic::{AtomicPtr, Ordering},
    Arc,
};

use ckb_jsonrpc_types::BlockNumber;
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};

use common::types::ckb_rpc_client::{RpcSearchKey, TipState};

#[derive(Clone)]
pub struct State {
    pub cell_states: Arc<dashmap::DashMap<RpcSearchKey, ScanTip>>,
}

impl Serialize for State {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("State", 2)?;
        state.serialize_field(
            "cell_states",
            &self
                .cell_states
                .iter()
                .map(|kv| (kv.key().clone(), kv.value().clone()))
                .collect::<Vec<_>>(),
        )?;
        state.end()
    }
}

impl<'a> Deserialize<'a> for State {
    fn deserialize<D>(deserializer: D) -> Result<State, D::Error>
    where
        D: Deserializer<'a>,
    {
        #[derive(Deserialize, Serialize)]
        struct StateVisitor {
            cell_states: Vec<(RpcSearchKey, ScanTip)>,
        }

        let v: StateVisitor = Deserialize::deserialize(deserializer)?;
        Ok(State {
            cell_states: Arc::new(v.cell_states.into_iter().collect()),
        })
    }
}

pub struct ScanTipInner(pub AtomicPtr<BlockNumber>);

pub struct ScanTip(pub Arc<ScanTipInner>);

impl Drop for ScanTipInner {
    fn drop(&mut self) {
        unsafe { drop(Box::from_raw(self.0.load(Ordering::Relaxed))) }
    }
}

impl Clone for ScanTip {
    fn clone(&self) -> Self {
        ScanTip(Arc::new(ScanTipInner(AtomicPtr::new(Box::into_raw(
            Box::new(*self.load()),
        )))))
    }
}

impl TipState for ScanTip {
    fn load(&self) -> &BlockNumber {
        unsafe { &*self.0 .0.load(Ordering::Acquire) }
    }

    fn update(&mut self, current: BlockNumber) {
        let raw = self
            .0
             .0
            .swap(Box::into_raw(Box::new(current)), Ordering::AcqRel);

        unsafe {
            drop(Box::from_raw(raw));
        }
    }
}

impl Serialize for ScanTip {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let inner = unsafe { &*self.0 .0.load(Ordering::Acquire) };

        inner.serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for ScanTip {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let inner = BlockNumber::deserialize(deserializer)?;

        Ok(ScanTip(Arc::new(ScanTipInner(AtomicPtr::new(
            Box::into_raw(Box::new(inner)),
        )))))
    }
}
