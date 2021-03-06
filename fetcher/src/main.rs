extern crate rustyrobot;

extern crate chrono;
extern crate fern;
extern crate serde;
extern crate serde_derive;
extern crate serde_json as json;
#[macro_use]
extern crate log;
extern crate ctrlc;
extern crate failure;

mod fetcher;
mod strategy;

use failure::Error;

use chrono::Duration;
use std::time::Duration as StdDuration;

fn init_fern() -> Result<(), Error> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level_for("fetcher", log::LevelFilter::Debug)
        .level_for("rustyrobot", log::LevelFilter::Debug)
        .level(log::LevelFilter::Warn)
        .chain(std::io::stdout())
        .apply()?;

    info!("logger initialised");

    Ok(())
}

use rustyrobot::{
    kafka::{
        topic,
        util::{producer::ThreadedProducer, state::StateHandler},
    },
    search::{
        query::SearchFor,
        query::{Lang, Query},
    },
    shutdown::GracefulShutdown,
};

use chrono::{NaiveDate, Utc};
use fetcher::Fetcher;
use std::thread;
use strategy::DateWindow;

fn main() {
    init_fern().unwrap();

    // Create graceful shutdown primitives
    let shutdown = GracefulShutdown::new();

    // Hook SIGINT signal
    let sigint_shutdown = shutdown.clone();
    ctrlc::set_handler(move || {
        info!("got SIGINT (Ctrl-C) signal, shutting down");
        sigint_shutdown.shutdown();
    })
    .expect("couldn't register SIGINT handler");

    // Fetch fetcher state
    let mut state = StateHandler::new(topic::FETCHER_STATE).expect("couldn't create StateHandler");
    state.restore().expect("couldn't restore state");

    // Create producer
    let producer = ThreadedProducer::new(topic::GITHUB_REQUEST, shutdown.thread_handle())
        .expect("couldn't start producer");

    // Create base query
    let query = Query::builder()
        .lang(Lang::Rust)
        .search_for(SearchFor::Repository)
        .owner("mersinvald")
        .count(100);

    // Setup fetching strategy
    let mut strategy = DateWindow {
        days_per_request: 1,
        start_date: Some(NaiveDate::from_ymd(2018, 8, 10)),
        ..Default::default()
    };

    let fetch_period = Duration::minutes(20);
    let mut fetch_time = Utc::now();

    while !shutdown.thread_handle().should_shutdown() {
        if Utc::now() >= fetch_time {
            let mut fetcher = Fetcher::new(
                &mut state,
                producer.handle(),
                shutdown.thread_handle(),
                strategy.clone(),
            );

            // Resetting start_date in strategy so we won't start over in next iteration
            strategy.start_date = None;

            // If that fails, fetcher will start from last successful data
            if let Err(e) = fetcher.fetch(query.clone()) {
                error!("failed to submit fetch requests: {}", e);
            } else {
                // If success, moving fetch_time one period into the future,
                // next iteration will start from Utc::today()
                fetch_time = Utc::now() + fetch_period;
            }
        }
        thread::sleep(StdDuration::from_secs(1));
    }
}
