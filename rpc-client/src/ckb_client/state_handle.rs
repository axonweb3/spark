use std::{
    fs::{copy, create_dir_all, remove_file, rename, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use common::types::ckb_rpc_client::RpcSearchKey;

use crate::ckb_client::{cell_process::CellProcess, ckb_rpc_client::CkbClient, types::State};

use crate::axon_client::RpcSubmit;

pub struct GlobalState {
    pub state:        State,
    pub path:         PathBuf,
    pub cell_handles: Arc<dashmap::DashMap<RpcSearchKey, tokio::task::JoinHandle<()>>>,
}

impl Drop for GlobalState {
    fn drop(&mut self) {
        self.dump_to_dir(self.path.clone())
    }
}

impl GlobalState {
    pub fn new(path: PathBuf) -> Self {
        let state = Self::load_from_dir(path.clone());

        Self {
            cell_handles: Arc::new(dashmap::DashMap::with_capacity(state.cell_states.len())),
            state,
            path,
        }
    }

    pub async fn run(&mut self) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            // clean shutdown task
            let mut shutdown_task = Vec::new();
            self.cell_handles.retain(|k, v| {
                if v.is_finished() {
                    shutdown_task.push(k.clone());
                    false
                } else {
                    true
                }
            });
            shutdown_task.into_iter().for_each(|k| {
                self.state.cell_states.remove(&k);
            });

            self.dump_to_dir(self.path.clone());
        }
    }

    pub fn spawn_cells(
        &self,
        client: CkbClient,
    ) -> Arc<dashmap::DashMap<RpcSearchKey, tokio::task::JoinHandle<()>>> {
        if !self.state.cell_states.is_empty() {
            for kv in self.state.cell_states.iter() {
                let mut cell_process = CellProcess::new(
                    kv.key().clone(),
                    kv.value().clone(),
                    client.clone(),
                    RpcSubmit,
                );

                let handle = tokio::spawn(async move {
                    cell_process.run().await;
                });
                self.cell_handles.insert(kv.key().clone(), handle);
            }
        }
        Arc::<
            dashmap::DashMap<
                common::types::ckb_rpc_client::RpcSearchKey,
                tokio::task::JoinHandle<()>,
            >,
        >::clone(&self.cell_handles)
    }

    fn load_from_dir(path: PathBuf) -> State {
        let db_path = path.join("scan_state");

        match File::open(&db_path) {
            Ok(f) => serde_json::from_reader(f).unwrap_or(State {
                cell_states: Default::default(),
            }),
            Err(e) => {
                log::warn!(
                    "Failed to open state db, file: {:?}, error: {:?}",
                    db_path,
                    e
                );
                State {
                    cell_states: Default::default(),
                }
            }
        }
    }

    fn dump_to_dir<P: AsRef<Path>>(&self, path: P) {
        // create dir
        create_dir_all(&path).unwrap();
        // dump file to a temporary sub-directory
        let tmp_dir = path.as_ref().join("tmp");
        create_dir_all(&tmp_dir).unwrap();
        let tmp_scan_state = tmp_dir.join("scan_state");
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(false)
            .open(&tmp_scan_state)
            .unwrap();
        // empty file and dump the json string to it
        file.set_len(0)
            .and_then(|_| serde_json::to_string(&self.state).map_err(Into::into))
            .and_then(|json_string| file.write_all(json_string.as_bytes()))
            .and_then(|_| file.sync_all())
            .unwrap();
        move_file(tmp_scan_state, path.as_ref().join("scan_state")).unwrap();
    }
}

fn move_file<P: AsRef<Path>>(src: P, dst: P) -> Result<(), std::io::Error> {
    if rename(&src, &dst).is_err() {
        copy(&src, &dst)?;
        remove_file(&src)?;
    }
    Ok(())
}
