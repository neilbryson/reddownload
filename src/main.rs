use clap::Parser;
use reqwest::{ClientBuilder, Result};
use std::time::Duration;

/// Reddit video downloader
#[derive(Parser)]
#[clap(name = "rdl")]
#[clap(version, about)]
struct Cli {
    url: String,
    save_to_path: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Cli = Cli::parse();
    let json_url = format!("{}.json", args.url);
    let timeout = Duration::new(20, 0);
    let client = ClientBuilder::new().timeout(timeout).build()?;

    println!("Requesting from {}", &json_url);

    let check_url_response = client.head(&json_url).send().await?;

    if check_url_response.status().is_success() {
        let api_response = client.get(&json_url).send().await?;
        println!("{:?}", api_response);
    } else {
        println!("Invalid URL");
    }

    Ok(())
}
