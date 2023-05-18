macro_rules! rpc_get {
    ($method:expr) => {{
        loop {
            match $method.await {
                Ok(r) => break r,
                Err(e) => panic!("rpc error: {:?}", e),
            }
        }
    }};
}

pub mod cell_process;
#[cfg(feature = "client")]
pub mod ckb_rpc_client;
pub mod ckb_subscription_client;
mod state_handle;
pub mod types;
