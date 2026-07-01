use n4n5_share::cli_main;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    cli_main().await?;
    Ok(())
}
