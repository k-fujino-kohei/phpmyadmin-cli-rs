[![Crates.io](https://img.shields.io/crates/v/pma-cli?style=for-the-badge)](https://crates.io/crates/pma-cli)

# phpmyadmin-cli-rs
You can use phpmyadmin by command line.

## Install

``` sh
cargo install pma-cli
```

## Usage

### Export all tables

``` sh
pma --url http://localhost export-all --db my_db
```

### Export specified tables

``` sh
pma --url http://localhost export table_1 table_2 --db my_db
```

Options are as follows

``` sh
Usage: pma --url <URL> export [OPTIONS] --db <DB> [TABLES]...

Arguments:
  [TABLES]...  Export the specified table names

Options:
  -a, --all-data
          Include all data
  -d, --data <table name>
          Include data in the specified table names
      --data-prefix <table name prefix>
          Include data from a table with conditions matching the specified prefix
  -o, --output <OUTPUT>
          Destination of exported data [default: ./]
  -s, --separate-files
          Separate exported data
  -h, --help
          Print help information

Required:
      --db <DB>  Database name
```
