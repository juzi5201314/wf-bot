use std::str::FromStr;

use proc_qq::Authentication;
use proc_qq::ClientBuilder;
use proc_qq::DeviceSource::JsonFile;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::mods::{active_arb, eidolon, invite, wm};

mod mods;
mod timing;
pub mod wf_api;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;

    init_tracing_subscriber()?;
    let client = ClientBuilder::new()
        .version(proc_qq::re_exports::ricq::version::IPAD)
        .device(JsonFile("device.json".to_owned()))
        .authentication(Authentication::UinPassword(
            dotenv::var("number")?.parse()?,
            dotenv::var("password")?,
        ))
        .modules(vec![
            active_arb::module(),
            invite::module(),
            eidolon::module(),
            wm::module(),
        ])
        .build()
        .await?;

    let rq_client = &client.rq_client;
    timing::arbitration(rq_client.clone());
    timing::eidolon(rq_client.clone());

    client.start().await??;
    Ok(())
}

fn init_tracing_subscriber() -> anyhow::Result<()> {
    let lvl = Level::from_str(&dotenv::var("level")?)?;
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .without_time(),
        )
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("ricq", lvl)
                .with_target("proc_qq", lvl)
                .with_target("wf-bot", lvl),
        )
        .init();
    Ok(())
}
