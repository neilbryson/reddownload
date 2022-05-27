use clap::Parser;
use reqwest::{header, ClientBuilder, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{canonicalize, File};
use std::io::{copy, Cursor};
use std::process::Command;
use std::time::Duration;

/// Reddit video downloader
#[derive(Parser)]
#[clap(name = "rdl")]
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

const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:100.0) Gecko/20100101 Firefox/100.0";

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
                            .expect("Unable to create temporary directory");

                        println!(
                            "Found a {}x{} video (/r/{})",
                            reddit_video.height, reddit_video.width, listing_data.data.subreddit
                        );

                        let video_response = client
                            .get(&reddit_video.fallback_url)
                            .header(header::USER_AGENT, USER_AGENT)
                            .header("Content-Type", "video/mp4")
                            .send()
                            .await?;
                        println!("Downloading video at {}", reddit_video.fallback_url);
                        let video_file_path = temp_dir.path().join("tmp.mp4");
                        let mut video_file =
                            File::create(&video_file_path).expect("Unable to create file");
                        let mut video_content = Cursor::new(video_response.bytes().await?);
                        copy(&mut video_content, &mut video_file).unwrap();
                        println!(
                            "Temporary file saved at {}",
                            video_file_path.to_string_lossy()
                        );

                        let audio_url_build = match &reddit_video.fallback_url.split("DASH").next()
                        {
                            Some(url) => Some(format!("{}HLS_AUDIO_160_K.aac", url)),
                            None => None,
                        };

                        if let Some(audio_url) = audio_url_build {
                            println!("Downloading audio at {}", &audio_url);
                            let audio_response = client
                                .get(&audio_url)
                                .header(header::USER_AGENT, USER_AGENT)
                                .header("Content-Type", "audio/aac")
                                .send()
                                .await?;
                            let audio_file_path = temp_dir.path().join("tmp.aac");
                            let mut audio_file =
                                File::create(&audio_file_path).expect("Unable to create file");
                            let mut audio_content = Cursor::new(audio_response.bytes().await?);
                            copy(&mut audio_content, &mut audio_file).unwrap();
                            println!(
                                "Temporary file saved at {}",
                                audio_file_path.to_string_lossy()
                            );

                            println!("Building video with ffmpeg");

                            Command::new("ffmpeg")
                                .arg("-i")
                                .arg(video_file_path.into_os_string().into_string().unwrap())
                                .arg("-i")
                                .arg(audio_file_path.into_os_string().into_string().unwrap())
                                .arg(&args.save_to_path)
                                .output()
                                .expect("Failed to execute process");

                            println!(
                                "Video saved at {}",
                                canonicalize(args.save_to_path).unwrap().to_string_lossy()
                            );
                        }

                        return Ok(());
                    }
                }
            }
        }

        println!("No media to download")
    } else {
        println!("Invalid URL");
    }

    Ok(())
}
