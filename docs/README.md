# yt-transcriber

A CLI tool for extracting YouTube video transcripts with timestamps in multiple formats.

## Installation

### From source (requires Rust)

```bash
git clone https://github.com/XMA-Faez/yt-transcriber.git
cd yt-transcriber
cargo build --release
sudo cp target/release/yt-transcriber /usr/local/bin/
```

### One-liner install

```bash
curl -fsSL https://raw.githubusercontent.com/XMA-Faez/yt-transcriber/main/install.sh | bash
```

## Requirements

- **yt-dlp**: The tool will attempt to install it automatically if not found, or you can install manually:
  ```bash
  pip install yt-dlp
  # or
  brew install yt-dlp
  ```

## Usage

```bash
yt-transcriber <url> [options]
```

### Arguments

- `url` - YouTube URL or video ID (required)

### Options

| Option | Alias | Description | Default |
|--------|-------|-------------|---------|
| `--format` | `-f` | Output format: txt, srt, json | txt |
| `--output` | `-o` | Output file path | stdout |
| `--language` | `-l` | Language code for transcript | en |
| `--no-timestamps` | | Exclude timestamps from TXT output | false |

### Examples

```bash
# Basic text output to stdout
yt-transcriber dQw4w9WgXcQ

# Full URL
yt-transcriber 'https://www.youtube.com/watch?v=dQw4w9WgXcQ'

# SRT subtitle format to file
yt-transcriber dQw4w9WgXcQ -f srt -o subtitles.srt

# JSON output
yt-transcriber dQw4w9WgXcQ --format json --output transcript.json

# Spanish transcript
yt-transcriber dQw4w9WgXcQ -l es

# Text without timestamps
yt-transcriber dQw4w9WgXcQ --no-timestamps
```

## Output Formats

### TXT (default)

```
[00:01] Hello and welcome to this video
[00:05] Today we're going to talk about...
```

### SRT

```
1
00:00:01,000 --> 00:00:04,500
Hello and welcome to this video

2
00:00:04,500 --> 00:00:11,200
Today we're going to talk about...
```

### JSON

```json
{
  "video_id": "VIDEO_ID",
  "language": "en",
  "segments": [
    {
      "index": 0,
      "text": "Hello and welcome",
      "start_seconds": 1.0,
      "end_seconds": 4.5,
      "duration_seconds": 3.5
    }
  ],
  "metadata": {
    "total_segments": 1,
    "extracted_at": "2026-02-03T12:00:00Z"
  }
}
```

## Supported URL Formats

- `dQw4w9WgXcQ` (video ID only)
- `https://youtube.com/watch?v=dQw4w9WgXcQ`
- `https://www.youtube.com/watch?v=dQw4w9WgXcQ`
- `https://m.youtube.com/watch?v=dQw4w9WgXcQ`
- `https://youtu.be/dQw4w9WgXcQ`
- `https://youtube.com/shorts/dQw4w9WgXcQ`
- `https://youtube.com/live/dQw4w9WgXcQ`
- `https://youtube.com/embed/dQw4w9WgXcQ`
- `https://music.youtube.com/watch?v=dQw4w9WgXcQ`

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Invalid arguments or yt-dlp not available |
| 2 | Video/transcript unavailable |
| 3 | Network error |
| 4 | File write error |

## Tech Stack

- **Language**: Rust
- **Subtitle Extraction**: yt-dlp (external dependency)
