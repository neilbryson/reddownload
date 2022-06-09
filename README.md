# reddownload

Downloader for Reddit-hosted videos (v.redd.it).

## Requirements
* ffmpeg (without this, the video would have no audio)

## Installation

The latest release binaries can be downloaded here
* [Linux](https://github.com/neilbryson/reddownload/releases/latest/download/reddownload-linux.tar.gz)
* [macOS](https://github.com/neilbryson/reddownload/releases/latest/download/reddownload-macos.tar.gz)
* [Windows](https://github.com/neilbryson/reddownload/releases/latest/download/reddownload-windows.zip)

These can be added to your `PATH` for easier usage in the command line.

## Usage
```bash
reddownload <URL> <SAVE_TO_PATH>
```

### Sample
```bash
reddownload https://www.reddit.com/r/Saul_Goodman_3d/comments/tz7nqy/3d_saul_goodman/ 3d_saul.mp4 
```

## Building from source

[rustup](https://rustup.rs/) must be installed.

Run `cargo build --release` to generate a release binary. It will be located on `target/release`.

## License
[MIT](LICENSE)
