mod command;
mod export;
mod pma_client;
mod read_zip;

use crate::pma_client::PMAClient;
use clap::Parser;
use command::{Command, Opt};
use export::{export, export_all};
use pma_client::PMAConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    run(opt).await
}

async fn run(opt: Opt) -> anyhow::Result<()> {
    let client = PMAClient::new(PMAConfig {
        url: opt.url,
        lang: opt.lang,
    });
    match opt.command {
        Command::Export {
            tables,
            export_option,
        } => export(client, tables, export_option).await?,

        Command::ExportAll { export_option } => export_all(client, export_option).await?,
    }
    Ok(())
}
