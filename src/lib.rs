mod command;
mod export;
mod pma_client;
mod read_zip;

use crate::pma_client::PMAClient;
use command::Command;
use export::{export, export_all};
use pma_client::PMAConfig;

pub use command::Opt;

pub async fn run(opt: Opt) -> anyhow::Result<()> {
    let client = PMAClient::new(PMAConfig {
        url: opt.url,
        lang: opt.lang,
    });
    match opt.command {
        Command::Export {
            tables,
            export_option,
        } => export(client, tables, export_option).await?,

        Command::ExportAll {
            filter_table,
            export_option,
        } => export_all(client, filter_table, export_option).await?,
    }
    Ok(())
}
