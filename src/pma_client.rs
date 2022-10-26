use anyhow::{bail, Context};
use bytes::Bytes;
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::header;

static TOKEN_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("token:\"(([[:alpha:]]|\\d|)+)\",").unwrap());

static COOKIE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new("phpMyAdmin=(([[:alpha:]]|\\d)+);").unwrap());

static TABLE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new("table=(([[:alpha:]]|_)+)&").unwrap());

#[derive(Debug)]
pub struct PMAExportConfig {
    pub db: String,
    pub tables: Vec<Table>,
    pub separate_files: bool,
    pub create_database: bool,
}

#[derive(Debug)]
pub struct Table {
    pub name: String,
    pub structure: bool,
    pub data: bool,
}

pub struct PMAConfig {
    pub url: String,
    pub lang: String,
}

pub struct PMAClient {
    http_client: reqwest::Client,
    config: PMAConfig,
}

macro_rules! to_str_tuple {
    ($k:expr, $v:expr) => {
        ($k, $v.to_string())
    };
}

impl PMAClient {
    pub fn new(config: PMAConfig) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            config,
        }
    }

    pub async fn fetch_token(&self) -> anyhow::Result<(String, String)> {
        let PMAConfig { url, lang } = &self.config;
        let req = self
            .http_client
            .get(url)
            .header(reqwest::header::COOKIE, format!("pma_lang={}", lang))
            .build()?;
        let resp = self.http_client.execute(req).await?;
        let cookie = resp.headers().get("set-cookie").unwrap().to_str()?;
        let cookie = COOKIE_REGEX
            .captures(cookie)
            .and_then(|x| x.get(1))
            .map(|x| x.as_str().to_string())
            .context(format!("cannot get the session cookie from {}.", cookie))?;
        let resp = resp.text().await?;
        let token = TOKEN_REGEX
            .captures(&resp)
            .and_then(|x| x.get(1))
            .map(|x| x.as_str().to_string())
            .context("cannot get token from response.")?;
        Ok((token, cookie))
    }

    pub async fn fetch_tables(&self, db: &str) -> anyhow::Result<Vec<String>> {
        let PMAConfig { url, lang: _ } = &self.config;
        let req = self
            .http_client
            .get(format!("{}/db_structure.php", url))
            .query(&[("db", db)])
            .build()?;
        let resp = self.http_client.execute(req).await?;
        let text = resp.text().await?;
        let tables = TABLE_REGEX
            .captures_iter(&text)
            .flat_map(|capt| capt.get(1))
            .map(|x| x.as_str().to_owned())
            .unique()
            .collect::<Vec<_>>();
        Ok(tables)
    }

    pub async fn export(
        &self,
        token: &str,
        cookie: &str,
        config: PMAExportConfig,
    ) -> anyhow::Result<Bytes> {
        let PMAConfig { url, lang: _ } = &self.config;
        let tables = self.fetch_tables(&config.db).await?;
        if tables.is_empty() {
            bail!("Not found tables in '{}'", config.db)
        }
        let req = self
            .http_client
            .post(format!("{}/export.php", url))
            .header(header::COOKIE, format!("phpMyAdmin={}", cookie))
            .header(header::ACCEPT_ENCODING, "gzip, deflate, br")
            .form(&ExportPayload::new(token, config).form())
            .build()?;
        let resp = self.http_client.execute(req).await?;
        let bytes = resp.bytes().await?;
        Ok(bytes)
    }
}

struct ExportPayload<'a>(Vec<(&'a str, String)>);

impl<'a> ExportPayload<'a> {
    fn new(token: &str, config: PMAExportConfig) -> Self {
        let db: &str = &config.db;
        let as_separate_files = if config.separate_files {
            "database"
        } else {
            ""
        };
        let tables: Vec<(&str, String)> = config
            .tables
            .iter()
            .flat_map(|t| {
                if !(t.structure || t.data) {
                    return vec![];
                }
                let mut table = vec![("table_select[]", t.name.clone())];
                if t.structure {
                    table.push(("table_structure[]", t.name.clone()));
                }
                if t.data {
                    table.push(("table_data[]", t.name.clone()));
                }
                table
            })
            .collect();

        let mut payload = ExportPayload::default();
        payload.extend(&[
            ("db", db),
            ("token", token),
            ("as_separate_files", as_separate_files),
        ]);
        if config.create_database {
            payload.extend(&[("sql_create_database", "something")]);
        }
        payload.extend(&tables);
        payload
    }

    fn extend(&mut self, pairs: &[(&'a str, impl ToString)]) {
        self.0
            .extend(pairs.iter().map(|(k, v)| to_str_tuple!(*k, v)));
    }

    fn form(self) -> Vec<(&'a str, String)> {
        self.0
    }
}

impl<'a> Default for ExportPayload<'a> {
    fn default() -> Self {
        Self(vec![
            to_str_tuple!("export_type", "database"),
            to_str_tuple!("export_method", "quick"),
            to_str_tuple!("template_id", ""),
            to_str_tuple!("quick_or_custom", "custom"),
            to_str_tuple!("what", "sql"),
            to_str_tuple!("structure_or_data_forced", "0"),
            to_str_tuple!("aliases_new", ""),
            to_str_tuple!("output_format", "sendit"),
            to_str_tuple!("filename_template", "@DATABASE@"),
            to_str_tuple!("remember_template", "on"),
            to_str_tuple!("charset", "utf-8"),
            to_str_tuple!("compression", "zip"),
            to_str_tuple!("maxsize", ""),
            to_str_tuple!("sql_include_comments", "something"),
            to_str_tuple!("sql_header_comment", ""),
            to_str_tuple!("sql_use_transaction", "something"),
            to_str_tuple!("sql_compatibility", "NONE"),
            to_str_tuple!("sql_structure_or_data", "structure_and_data"),
            to_str_tuple!("sql_create_table", "something"),
            to_str_tuple!("sql_drop_table", "something"),
            to_str_tuple!("sql_auto_increment", "something"),
            to_str_tuple!("sql_create_view", "something"),
            to_str_tuple!("sql_create_trigger", "something"),
            to_str_tuple!("sql_backquotes", "something"),
            to_str_tuple!("sql_type", "INSERT"),
            to_str_tuple!("sql_insert_syntax", "both"),
            to_str_tuple!("sql_max_query_size", "50000"),
            to_str_tuple!("sql_hex_for_binary", "something"),
            to_str_tuple!("sql_utc_time", "something"),
            to_str_tuple!("knjenc", ""),
        ])
    }
}
