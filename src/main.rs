mod utils;

use anyhow::{Context, Result};
use clap::Parser;
use reqwest::ClientBuilder;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::canonicalize;
use std::time::Duration;
use utils::{build_video, download_file};

/// Reddit video downloader
#[derive(Parser)]
#[clap(name = "reddownload")]
#[clap(version, about)]
struct Cli {
    url: String,
    save_to_path: String,
}

#[derive(Deserialize, Debug)]
struct SecureMedia {
    fallback_url: String,
    height: i16,
    width: i16,
}

#[derive(Deserialize, Debug)]
struct PostData {
    subreddit: String,
    secure_media: Option<HashMap<String, SecureMedia>>,
}

#[derive(Deserialize, Debug)]
struct ListingDataChild {
    data: PostData,
}

#[derive(Deserialize, Debug)]
struct ListingData {
    children: Vec<ListingDataChild>,
}

#[derive(Deserialize, Debug)]
struct RootResponse {
    data: ListingData,
}

const USER_AGENT: &str = "reddownload";

#[tokio::main]
async fn main() -> Result<()> {
    let args: Cli = Cli::parse();
    let json_url = format!("{}.json", args.url);
    let timeout = Duration::new(20, 0);
    let client = ClientBuilder::new().timeout(timeout).build()?;
    let check_url_response = client.head(&json_url).send().await?;

    if check_url_response.status().is_success() {
        let api_response = client.get(&json_url).send().await?;
        let json: Vec<RootResponse> = api_response.json().await?;

        for root_response in json {
            for listing_data in root_response.data.children {
                if let Some(media_info) = listing_data.data.secure_media {
                    if let Some(reddit_video) = media_info.get("reddit_video") {
                        let temp_dir = tempfile::Builder::new()
                            .prefix("reddownload-")
                            .tempdir()
                            .context("Unable to create temporary directory")?;

                        println!(
                            "Found a {}x{} video (/r/{})",
                            reddit_video.height, reddit_video.width, listing_data.data.subreddit
                        );

                        println!("Downloading video at {}", reddit_video.fallback_url);
                        let video_file_path = download_file(
                            &client,
                            &temp_dir,
                            &reddit_video.fallback_url,
                            "tmp.mp4",
                            "video/mp4",
                        )
                        .await?;

                        println!(
                            "Temporary file saved at {}",
                            video_file_path.to_string_lossy()
                        );

                        let base_url: Vec<&str> = reddit_video.fallback_url.split("DASH").collect();
                        let audio_url = format!("{}HLS_AUDIO_160_K.aac", base_url.get(0).unwrap());

                        println!("Downloading audio at {}", &audio_url);
                        let audio_file_path =
                            download_file(&client, &temp_dir, &audio_url, "tmp.aac", "audio/aac")
                                .await?;
                        println!(
                            "Temporary file saved at {}",
                            audio_file_path.to_string_lossy()
                        );

                        build_video(video_file_path, audio_file_path, &args.save_to_path)?;

                        println!(
                            "Video saved at {}",
                            canonicalize(&args.save_to_path).unwrap().to_string_lossy()
                        );

                        return Ok(());
                    }
                }
            }
        }

        eprintln!("No media to download")
    } else {
        eprintln!("Invalid URL");
    }

    Ok(())
}
