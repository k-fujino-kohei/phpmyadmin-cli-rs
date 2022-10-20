use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = pma_cli::Opt::parse();
    pma_cli::run(opt).await
}
