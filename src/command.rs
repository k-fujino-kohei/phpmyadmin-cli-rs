use clap::{Parser, ValueHint};

#[derive(Debug, Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
pub struct Opt {
    /// Location of phpmyadmin
    #[clap(short, long, value_hint = ValueHint::Url)]
    pub url: String,

    /// Language of phpmyadmin
    #[clap(short, long, default_value = "ja")]
    pub lang: String,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Parser)]
pub enum Command {
    /// Export the specified tables.
    Export {
        /// Export the specified table names.
        tables: Vec<String>,

        #[clap(flatten)]
        export_option: ExportOption,
    },
    /// Export all tables.
    ExportAll {
        /// Export table names that match the specified regex.
        #[clap(short, long, value_name = "regex")]
        filter_table: Option<String>,

        #[clap(flatten)]
        export_option: ExportOption,
    },
}

#[derive(Debug, Parser)]
pub struct ExportOption {
    /// Database name
    #[clap(long, help_heading = "Required")]
    pub db: String,

    /// Include all data.
    #[clap(short, long, group = "include_data", default_value = "false")]
    pub all_data: bool,

    /// Include data in the specified table names.
    #[clap(short, long, group = "include_data", value_name = "table name")]
    pub data: Option<Vec<String>>,

    /// Include data from a table with conditions matching the specified prefix.
    #[clap(long, group = "include_data", value_name = "table name prefix")]
    pub data_prefix: Option<String>,

    /// Destination of exported data.
    #[clap(short, long, default_value = "./")]
    pub output: String,

    /// Separate exported data.
    #[clap(short, long, default_value = "false")]
    pub separate_files: bool,
}
