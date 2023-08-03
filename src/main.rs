mod reddit;
mod utils;

use crate::reddit::*;
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use reqwest::ClientBuilder;
use std::time::Duration;
use utils::{build_video, download_file};

/// Reddit video downloader
#[derive(Parser)]
#[clap(name = "reddownload")]
#[clap(version, about)]
#[clap(arg_required_else_help = true)]
struct Cli {
    /// The URL to the Reddit post
    url: String,
    /// Output file name
    file_name: Option<String>,
}

const USER_AGENT: &str = "reddownload";

#[tokio::main]
async fn main() -> Result<()> {
    let args: Cli = Cli::parse();
    let json_url = format!("{}.json", args.url);
    let timeout = Duration::new(20, 0);
    let client = ClientBuilder::new()
        .timeout(timeout)
        .user_agent(USER_AGENT)
        .build()?;
    let check_url_response = client.head(&json_url).send().await?;

    return if check_url_response.status().is_success() {
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
                            "Found a {}x{} video",
                            reddit_video.height, reddit_video.width
                        );

                        println!("Downloading video from {}", reddit_video.fallback_url);
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
                        let audio_url = format!("{}HLS_AUDIO_128.aac", base_url.get(0).unwrap());

                        println!("Downloading audio from {}", &audio_url);
                        let audio_file_path = download_file(
                            &client,
                            &temp_dir,
                            &audio_url,
                            "tmp_audio.mp4",
                            "video/mp4",
                        )
                        .await?;
                        println!(
                            "Temporary file saved at {}",
                            audio_file_path.to_string_lossy()
                        );

                        let file_name = match args.file_name {
                            Some(path) => path,
                            None => {
                                let segments: Vec<&str> =
                                    reddit_video.fallback_url.split("/").collect();
                                let final_name = format!(
                                    "{}.mp4",
                                    segments.get(3).unwrap_or(&"reddownload-video.mp4")
                                );
                                final_name.to_string()
                            }
                        };

                        build_video(video_file_path, audio_file_path, &file_name)?;

                        return Ok(());
                    }
                }
            }
        }

        Err(anyhow!("No Reddit video to download"))
    } else {
        Err(anyhow!("Invalid URL"))
    };
}
