use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use std::fs;
use std::io::{copy, Cursor};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

pub async fn download_file(
    client: &Client,
    temp_dir: &TempDir,
    url: &String,
    temp_file_name: &str,
    content_type: &str,
) -> Result<PathBuf> {
    let response = client
        .get(url)
        .header("Content-Type", content_type)
        .send()
        .await?;
    let file_path = temp_dir.path().join(temp_file_name);
    let mut file = fs::File::create(&file_path).context("Unable to create file")?;
    let mut content = Cursor::new(response.bytes().await?);
    copy(&mut content, &mut file)?;
    Ok(file_path)
}

pub fn build_video(
    video_file_path: PathBuf,
    audio_file_path: PathBuf,
    output_path: &String,
) -> Result<()> {
    // Check ffmpeg first
    if let Ok(_) = Command::new("ffmpeg").arg("--help").output() {
        let output = Command::new("ffmpeg")
            .arg("-i")
            .arg(&video_file_path.into_os_string().into_string().unwrap())
            .arg("-i")
            .arg(&audio_file_path.into_os_string().into_string().unwrap())
            .arg("-vcodec")
            .arg("libx264")
            .arg("-acodec")
            .arg("aac")
            .arg(&output_path)
            .output()?;

        if output.status.success() {
            println!(
                "Video saved at {} (Size: {} B)",
                fs::canonicalize(&output_path).unwrap().to_string_lossy(),
                fs::metadata(&output_path).unwrap().len()
            );
        } else {
            return Err(anyhow!("Video generation failed"));
        }
    } else {
        println!("ffmpeg is not installed. Copying the mp4 file without audio.");
        fs::copy(
            Path::new(&video_file_path.into_os_string()),
            Path::new(&output_path),
        )?;
    }

    Ok(())
}
