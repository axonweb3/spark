mod config;

use std::{env, panic::PanicInfo, sync::Arc};

use api::{run_server, DefaultAPIAdapter};
use backtrace::Backtrace;
use config::SparkConfig;
use storage::{SmtManager, TransactionHistory};
#[cfg(unix)]
use tokio::signal::unix as os_impl;

#[tokio::main]
async fn main() {
    let args = env::args().nth(1).expect("Missing env variable");
    let config: SparkConfig = config::parse_file(args).expect("Failed to parse config file");

    let rdb = Arc::new(TransactionHistory::new(&config.rdb_url).await);
    let kvdb = Arc::new(SmtManager::new(&config.kvdb_path));
    let api_adapter = Arc::new(DefaultAPIAdapter::new(rdb, kvdb));
    let handle = run_server(api_adapter, config.rpc_listen_address)
        .await
        .unwrap();

    tokio::spawn(handle.stopped());

    set_ctrl_c_handle().await;
}

async fn set_ctrl_c_handle() {
    let ctrl_c_handler = tokio::spawn(async {
        #[cfg(windows)]
        let _ = tokio::signal::ctrl_c().await;
        #[cfg(unix)]
        {
            let mut sigtun_int = os_impl::signal(os_impl::SignalKind::interrupt()).unwrap();
            let mut sigtun_term = os_impl::signal(os_impl::SignalKind::terminate()).unwrap();
            tokio::select! {
                _ = sigtun_int.recv() => {}
                _ = sigtun_term.recv() => {}
            };
        }
    });

    // register channel of panic
    let (panic_sender, mut panic_receiver) = tokio::sync::mpsc::channel::<()>(1);

    std::panic::set_hook(Box::new(move |info: &PanicInfo| {
        let panic_sender = panic_sender.clone();
        panic_log(info);
        panic_sender.try_send(()).expect("panic_receiver is droped");
    }));

    tokio::select! {
        _ = ctrl_c_handler => { log::info!("ctrl + c is pressed, quit.") },
        _ = panic_receiver.recv() => { log::info!("child thread panic, quit.") },
    };
}

fn panic_log(info: &PanicInfo) {
    let backtrace = Backtrace::new();
    let thread = std::thread::current();
    let name = thread.name().unwrap_or("unnamed");
    let location = info.location().unwrap(); // The current implementation always returns Some
    let msg = match info.payload().downcast_ref::<&'static str>() {
        Some(s) => *s,
        None => match info.payload().downcast_ref::<String>() {
            Some(s) => &**s,
            None => "Box<Any>",
        },
    };
    log::error!(
        target: "panic", "thread '{}' panicked at '{}': {}:{} {:?}",
        name,
        msg,
        location.file(),
        location.line(),
        backtrace,
    );
}
