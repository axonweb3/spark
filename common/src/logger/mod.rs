#![allow(clippy::uninlined_format_args)]

mod date_fixed_roller;

use std::path::PathBuf;

use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::json::JsonEncoder;
use log4rs::encode::pattern::PatternEncoder;

use date_fixed_roller::DateFixedWindowRoller;

pub fn init(
    filter: String,
    log_to_console: bool,
    console_show_file_and_line: bool,
    log_to_file: bool,
    log_path: PathBuf,
    file_size_limit: u64, // bytes
) {
    let console_appender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            if console_show_file_and_line {
                "[{d} {h({l})} {t} {f}:{L}] {m}{n}"
            } else {
                "[{d} {h({l})} {t}] {m}{n}"
            },
        )))
        .build();

    let spark_roller_pat = log_path.join("{date}.spark.{timestamp}.log");

    let file_appender = {
        let size_trigger = SizeTrigger::new(file_size_limit);
        let roller = DateFixedWindowRoller::builder()
            .build(&spark_roller_pat.to_string_lossy())
            .unwrap();
        let policy = CompoundPolicy::new(Box::new(size_trigger), Box::new(roller));

        RollingFileAppender::builder()
            .encoder(Box::new(JsonEncoder::new()))
            .build(log_path.join("spark.log"), Box::new(policy))
            .unwrap()
    };

    let mut root_builder = Root::builder();
    if log_to_console {
        root_builder = root_builder.appender("console");
    }
    if log_to_file {
        root_builder = root_builder.appender("file");
    }

    let level_filter = convert_level(filter.as_ref());
    let root = root_builder.build(level_filter);

    let config_builder = Config::builder()
        .appender(Appender::builder().build("console", Box::new(console_appender)))
        .appender(Appender::builder().build("file", Box::new(file_appender)));

    let config = config_builder.build(root).unwrap();

    log4rs::init_config(config).expect("");
}

fn convert_level(level: &str) -> LevelFilter {
    match level {
        "off" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        f => {
            println!("invalid logger.filter {}, use info", f);
            LevelFilter::Info
        }
    }
}
