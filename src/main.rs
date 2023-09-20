use clap::Parser;
use regex::Regex;
use shrimple::Shrimple;
use std::process::Command;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    video: String,

    #[arg(short, long)]
    subs: String,

    // TODO Use Vec<String> (but how does it parse :thinking:)
    #[arg(short = 'e', long, default_value = "mkv")]
    video_extensions: String,

    #[arg(long, default_value = "ass,srt")]
    sub_extensions: String,

    #[arg(short = 'n', long = "dry-run", default_value_t = false)]
    dry_run: bool,

    #[arg(short = 'f', long, default_value_t = 1)]
    first_ep: usize,

    #[arg(short = 'm', long, default_value_t = 26)]
    max_ep: usize,
}

fn collect_files<'a>(extensions: impl Iterator<Item = &'a str>) -> Vec<String> {
    extensions
        .filter_map(|e| {
            Command::new("fd")
                .args(["--extension", e])
                .shrimp_vec()
                .ok()
        })
        .flatten()
        .collect::<Vec<String>>()
}

fn pair_videos(regex: &Regex, min: usize, max: usize, files: Vec<String>) -> Vec<(usize, String)> {
    files
        .into_iter()
        .filter_map(|s| {
            Some((
                regex
                    .captures(&s)?
                    .get(1)?
                    .as_str()
                    .parse()
                    .ok()
                    .filter(|v| *v >= min && *v <= max)?,
                s,
            ))
        })
        .collect()
}

fn main() {
    let args = Args::parse();
    let video_regex = Regex::new(args.video.as_str()).unwrap();
    let sub_regex = Regex::new(args.subs.as_str()).unwrap();

    let video_extensions = args.video_extensions.as_str().split(',');
    let sub_extensions = args.sub_extensions.as_str().split(',');

    let videos = pair_videos(
        &video_regex,
        args.first_ep,
        args.max_ep,
        collect_files(video_extensions),
    );
    let subs = pair_videos(
        &sub_regex,
        args.first_ep,
        args.max_ep,
        collect_files(sub_extensions),
    );

    for (ep, video) in videos {
        let sub = match subs.iter().find(|(n, _)| *n == ep) {
            Some(v) => &v.1,
            None => {
                eprintln!("WARNING:Could not pair \"{video}\": Missing sub file");
                continue;
            }
        };

        if !args.dry_run {
            Command::new("mkv_sub")
                .args([video.as_str(), sub.as_str(), "-b"])
                .shrimp_exec()
                .unwrap();
        } else {
            println!("mkv_sub '{video}' '{sub}' -b");
        }
    }
}
