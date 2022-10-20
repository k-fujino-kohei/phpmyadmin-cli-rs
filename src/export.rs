use crate::{
    command::ExportOption,
    pma_client::{PMAClient, PMAExportConfig, Table},
    read_zip::read_zip,
};
use anyhow::bail;
use itertools::Itertools;
use std::path::Path;
use tokio::{fs::File, io::AsyncWriteExt};

pub async fn export(
    client: PMAClient,
    tables: Vec<String>,
    option: ExportOption,
) -> anyhow::Result<()> {
    let actual_tables = client.fetch_tables(&option.db).await?;
    check_if_table_exists(&actual_tables, &tables)?;
    _export(client, tables, option).await?;
    Ok(())
}

pub async fn export_all(client: PMAClient, option: ExportOption) -> anyhow::Result<()> {
    let tables = client.fetch_tables(&option.db).await?;
    _export(client, tables, option).await?;
    Ok(())
}

async fn _export(
    client: PMAClient,
    tables: Vec<String>,
    option: ExportOption,
) -> anyhow::Result<()> {
    let config = opt_to_config(tables, option).await?;
    let (token, cookie) = client.fetch_token().await?;
    let exported_bin = client.export(&token, &cookie, config.pma).await?;
    let files = read_zip(exported_bin).await?;
    let create_files = files.into_iter().map(|f| {
        println!("Exported: {}", f.header.file_name);
        let path = config.output_path.clone();
        tokio::spawn(async move {
            let root = Path::new(&path).to_path_buf();
            let path = root.clone().join(f.header.file_name);
            let mut file = File::create(path).await?;
            file.write_all(&f.content).await?;
            anyhow::Ok(())
        })
    });
    for h in create_files {
        h.await??;
    }

    Ok(())
}

fn check_if_table_exists(actual: &[String], check_tables: &[String]) -> anyhow::Result<()> {
    let invalid_tables = check_tables
        .iter()
        .filter(|table| !actual.contains(table))
        .collect::<Vec<_>>();
    if !invalid_tables.is_empty() {
        let tables = invalid_tables.iter().join(",");
        bail!("Not Found table. {}", tables);
    }
    Ok(())
}

#[derive(Debug)]
struct ExportConfig {
    pma: PMAExportConfig,
    output_path: String,
}

async fn opt_to_config(tables: Vec<String>, option: ExportOption) -> anyhow::Result<ExportConfig> {
    let ExportOption {
        db,
        all_data,
        ref data,
        ref data_prefix,
        separate_files,
        output,
    } = option;
    if let Some(d) = data {
        check_if_table_exists(&tables, d)?;
    }
    let tables = tables
        .into_iter()
        .map(|t| {
            let data = match (all_data, data, data_prefix) {
                (true, _, _) => true,
                (_, Some(data), _) => data.contains(&t),
                (_, _, Some(prefix)) => t.starts_with(prefix),
                _ => false,
            };
            Table {
                name: t,
                structure: true,
                data,
            }
        })
        .collect();
    Ok(ExportConfig {
        pma: PMAExportConfig {
            db,
            tables,
            separate_files,
        },
        output_path: output,
    })
}
