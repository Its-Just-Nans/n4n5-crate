use n4n5_ocr::cli_main;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    cli_main().await?;
    Ok(())
}
