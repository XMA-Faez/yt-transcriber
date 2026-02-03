use clap::{Parser, ValueEnum};
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::process::{Command, ExitCode};
use tempfile::TempDir;

#[derive(Parser)]
#[command(name = "yt-transcriber")]
#[command(version = "1.0.0")]
#[command(about = "Extract YouTube video transcripts with timestamps")]
struct Cli {
    /// YouTube URL or video ID
    url: String,

    /// Output format
    #[arg(short, long, default_value = "txt", value_enum)]
    format: OutputFormat,

    /// Output file path (default: stdout)
    #[arg(short, long)]
    output: Option<String>,

    /// Language code for transcript
    #[arg(short, long, default_value = "en")]
    language: String,

    /// Exclude timestamps from TXT output
    #[arg(long)]
    no_timestamps: bool,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Txt,
    Srt,
    Json,
}

#[derive(Serialize)]
struct TranscriptSegment {
    index: usize,
    text: String,
    start_seconds: f64,
    end_seconds: f64,
    duration_seconds: f64,
}

#[derive(Serialize)]
struct TranscriptResult {
    video_id: String,
    language: String,
    segments: Vec<TranscriptSegment>,
    metadata: Metadata,
}

#[derive(Serialize)]
struct Metadata {
    total_segments: usize,
    extracted_at: String,
}

fn extract_video_id(input: &str) -> Option<String> {
    let trimmed = input.trim();
    let id_regex = Regex::new(r"^[a-zA-Z0-9_-]{11}$").unwrap();

    if id_regex.is_match(trimmed) {
        return Some(trimmed.to_string());
    }

    if let Ok(url) = url::Url::parse(trimmed) {
        let host = url.host_str().unwrap_or("");
        let clean_host = host
            .trim_start_matches("www.")
            .trim_start_matches("m.")
            .trim_start_matches("music.");

        if clean_host == "youtu.be" {
            let path = url.path().trim_start_matches('/');
            let id = path.split('/').next().unwrap_or("");
            if id_regex.is_match(id) {
                return Some(id.to_string());
            }
        }

        if clean_host == "youtube.com" {
            if let Some(v) = url.query_pairs().find(|(k, _)| k == "v") {
                if id_regex.is_match(&v.1) {
                    return Some(v.1.to_string());
                }
            }

            let segments: Vec<&str> = url.path().split('/').filter(|s| !s.is_empty()).collect();
            let patterns = ["watch", "embed", "v", "shorts", "live", "clip"];

            for i in 0..segments.len() {
                if patterns.contains(&segments[i]) {
                    if let Some(id) = segments.get(i + 1) {
                        if id_regex.is_match(id) {
                            return Some(id.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

fn check_yt_dlp() -> bool {
    Command::new("yt-dlp").arg("--version").output().is_ok()
}

fn install_yt_dlp() -> bool {
    eprintln!("yt-dlp not found. Attempting to install...");

    if Command::new("pip").arg("--version").output().is_ok() {
        let status = Command::new("pip")
            .args(["install", "--user", "yt-dlp"])
            .status();
        if status.map(|s| s.success()).unwrap_or(false) {
            return true;
        }
    }

    if Command::new("pipx").arg("--version").output().is_ok() {
        let status = Command::new("pipx").args(["install", "yt-dlp"]).status();
        if status.map(|s| s.success()).unwrap_or(false) {
            return true;
        }
    }

    if Command::new("brew").arg("--version").output().is_ok() {
        let status = Command::new("brew").args(["install", "yt-dlp"]).status();
        if status.map(|s| s.success()).unwrap_or(false) {
            return true;
        }
    }

    false
}

fn parse_vtt_timestamp(ts: &str) -> f64 {
    let parts: Vec<&str> = ts.split(':').collect();
    match parts.len() {
        2 => {
            let mins: f64 = parts[0].parse().unwrap_or(0.0);
            let secs: f64 = parts[1].parse().unwrap_or(0.0);
            mins * 60.0 + secs
        }
        3 => {
            let hours: f64 = parts[0].parse().unwrap_or(0.0);
            let mins: f64 = parts[1].parse().unwrap_or(0.0);
            let secs: f64 = parts[2].parse().unwrap_or(0.0);
            hours * 3600.0 + mins * 60.0 + secs
        }
        _ => 0.0,
    }
}

fn parse_vtt(content: &str) -> Vec<TranscriptSegment> {
    let mut segments = Vec::new();
    let timestamp_re = Regex::new(r"(\d{1,2}:\d{2}:\d{2}\.\d{3}|\d{1,2}:\d{2}\.\d{3})\s*-->\s*(\d{1,2}:\d{2}:\d{2}\.\d{3}|\d{1,2}:\d{2}\.\d{3})").unwrap();
    let tag_re = Regex::new(r"<[^>]+>").unwrap();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if let Some(caps) = timestamp_re.captures(line) {
            let start = parse_vtt_timestamp(&caps[1]);
            let end = parse_vtt_timestamp(&caps[2]);

            let mut text_lines = Vec::new();
            i += 1;

            while i < lines.len() && !lines[i].trim().is_empty() && !timestamp_re.is_match(lines[i]) {
                let text_line = lines[i].trim();
                if !text_line.starts_with("WEBVTT") && !text_line.starts_with("Kind:") && !text_line.starts_with("Language:") {
                    let clean = tag_re.replace_all(text_line, "").to_string();
                    if !clean.is_empty() {
                        text_lines.push(clean);
                    }
                }
                i += 1;
            }

            if !text_lines.is_empty() {
                let text = text_lines.join(" ");
                if !text.trim().is_empty() {
                    segments.push(TranscriptSegment {
                        index: segments.len(),
                        text,
                        start_seconds: start,
                        end_seconds: end,
                        duration_seconds: end - start,
                    });
                }
            }
        } else {
            i += 1;
        }
    }

    segments
}

fn format_timestamp_bracket(seconds: f64) -> String {
    let mins = (seconds / 60.0).floor() as u32;
    let secs = (seconds % 60.0).floor() as u32;
    format!("[{:02}:{:02}]", mins, secs)
}

fn format_timestamp_srt(seconds: f64) -> String {
    let hours = (seconds / 3600.0).floor() as u32;
    let mins = ((seconds % 3600.0) / 60.0).floor() as u32;
    let secs = (seconds % 60.0).floor() as u32;
    let millis = ((seconds % 1.0) * 1000.0).floor() as u32;
    format!("{:02}:{:02}:{:02},{:03}", hours, mins, secs, millis)
}

fn format_txt(result: &TranscriptResult, include_timestamps: bool) -> String {
    result
        .segments
        .iter()
        .map(|seg| {
            if include_timestamps {
                format!("{} {}", format_timestamp_bracket(seg.start_seconds), seg.text)
            } else {
                seg.text.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_srt(result: &TranscriptResult) -> String {
    result
        .segments
        .iter()
        .enumerate()
        .map(|(i, seg)| {
            format!(
                "{}\n{} --> {}\n{}",
                i + 1,
                format_timestamp_srt(seg.start_seconds),
                format_timestamp_srt(seg.end_seconds),
                seg.text
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn format_json(result: &TranscriptResult) -> String {
    serde_json::to_string_pretty(result).unwrap_or_default()
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let video_id = match extract_video_id(&cli.url) {
        Some(id) => id,
        None => {
            eprintln!("Error: Invalid YouTube URL or video ID");
            return ExitCode::from(1);
        }
    };

    if !check_yt_dlp() {
        if !install_yt_dlp() {
            eprintln!("Error: yt-dlp is required but could not be installed");
            eprintln!("Please install it manually: pip install yt-dlp");
            return ExitCode::from(1);
        }
        if !check_yt_dlp() {
            eprintln!("Error: yt-dlp installation succeeded but command not found in PATH");
            eprintln!("Try restarting your terminal or adding ~/.local/bin to PATH");
            return ExitCode::from(1);
        }
    }

    let temp_dir = match TempDir::new() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: Failed to create temp directory - {}", e);
            return ExitCode::from(4);
        }
    };

    let url = format!("https://www.youtube.com/watch?v={}", video_id);
    let output_template = temp_dir.path().join("%(id)s");

    let output = Command::new("yt-dlp")
        .args([
            "--write-sub",
            "--write-auto-sub",
            "--sub-lang",
            &cli.language,
            "--sub-format",
            "vtt",
            "--skip-download",
            "--no-warnings",
            "-o",
            output_template.to_str().unwrap_or("%(id)s"),
            &url,
        ])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Error: Failed to run yt-dlp - {}", e);
            return ExitCode::from(3);
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("unavailable") || stderr.contains("private") || stderr.contains("deleted") {
            eprintln!("Error: Video is unavailable (private/deleted/restricted)");
        } else {
            eprintln!("Error: yt-dlp failed - {}", stderr.trim());
        }
        return ExitCode::from(2);
    }

    let vtt_patterns = [
        format!("{}.{}.vtt", video_id, cli.language),
        format!("{}.{}-orig.vtt", video_id, cli.language),
    ];

    let mut vtt_content = None;

    for pattern in &vtt_patterns {
        let vtt_path = temp_dir.path().join(pattern);
        if vtt_path.exists() {
            if let Ok(content) = fs::read_to_string(&vtt_path) {
                vtt_content = Some(content);
                break;
            }
        }
    }

    if vtt_content.is_none() {
        if let Ok(entries) = fs::read_dir(temp_dir.path()) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "vtt").unwrap_or(false) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        vtt_content = Some(content);
                        break;
                    }
                }
            }
        }
    }

    let vtt_content = match vtt_content {
        Some(c) => c,
        None => {
            eprintln!("Error: No subtitles available for this video in '{}' language", cli.language);
            return ExitCode::from(2);
        }
    };

    let segments = parse_vtt(&vtt_content);

    if segments.is_empty() {
        eprintln!("Error: No transcript content found");
        return ExitCode::from(2);
    }

    let result = TranscriptResult {
        video_id: video_id.clone(),
        language: cli.language.clone(),
        metadata: Metadata {
            total_segments: segments.len(),
            extracted_at: chrono::Utc::now().to_rfc3339(),
        },
        segments,
    };

    let output = match cli.format {
        OutputFormat::Txt => format_txt(&result, !cli.no_timestamps),
        OutputFormat::Srt => format_srt(&result),
        OutputFormat::Json => format_json(&result),
    };

    if let Some(path) = cli.output {
        if let Err(e) = fs::write(&path, &output) {
            eprintln!("Error: Failed to write file - {}", e);
            return ExitCode::from(4);
        }
        eprintln!("Transcript saved to {}", path);
    } else {
        println!("{}", output);
    }

    ExitCode::SUCCESS
}
