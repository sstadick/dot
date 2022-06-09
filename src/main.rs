use std::{process::exit, str::FromStr};

use anyhow::{anyhow, bail, Result};
use camino::{Utf8Path, Utf8PathBuf};
use chrono::{DateTime, Local, Utc};
use clap::Parser;
use env_logger::Env;
use human_bytes::human_bytes;
use rayon::iter::{ParallelBridge, ParallelIterator};
use walkdir::WalkDir;

#[derive(Debug)]
struct DirPath {
    path: Utf8PathBuf,
}

impl DirPath {
    fn new<P: AsRef<Utf8Path>>(path: &P) -> Result<Self> {
        if path.as_ref().is_file() {
            bail!("{} is a file, dir must be provided.", path.as_ref());
        }
        Ok(Self {
            path: Utf8PathBuf::from(path.as_ref()),
        })
    }

    fn data_over_time(
        &self,
        start: Option<DateTime<Local>>,
        end: Option<DateTime<Local>>,
        follow_links: bool,
    ) -> Result<u64> {
        let bytes = WalkDir::new(self.path.as_str())
            .follow_links(follow_links)
            .into_iter()
            .par_bridge()
            .map(|item| {
                let path = item?;
                let meta = path.metadata()?;
                let local_time = DateTime::<Local>::from(meta.modified()?);

                let size = match (start, end) {
                    (Some(start), Some(end)) if local_time >= start && local_time <= end => {
                        Some(meta.len())
                    }
                    (Some(start), None) if local_time >= start => Some(meta.len()),
                    (None, Some(end)) if local_time <= end => Some(meta.len()),
                    (None, None) => Some(meta.len()),
                    _ => None,
                };
                Ok::<Option<u64>, anyhow::Error>(size)
            })
            .try_reduce(
                || Some(0),
                |a: Option<u64>, b: Option<u64>| match (a, b) {
                    (Some(a), Some(b)) => Ok(Some(a + b)),
                    (None, None) => Ok(None),
                    (None, Some(b)) => Ok(Some(b)),
                    (Some(a), None) => Ok(Some(a)),
                },
            )?;
        if let Some(bytes) = bytes {
            Ok(bytes)
        } else {
            Ok(0)
        }
    }
}

impl FromStr for DirPath {
    type Err = anyhow::Error;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let path = Utf8PathBuf::from(raw);

        if path.is_file() {
            bail!("{} is a file, dir must be provided.", path);
        }
        Ok(Self { path })
    }
}

/// Data-over-time
///
/// Compute the amount of data generated in a directory between two time points.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Args {
    // The start time used to filter files
    #[clap(long, short)]
    start_time: Option<DateTime<Local>>,

    /// The end time used to filter files
    #[clap(long, short)]
    end_time: Option<DateTime<Local>>,

    /// Follow symbolic links
    #[clap(long, short)]
    follow_links: bool,

    /// The directory to search
    search_path: DirPath,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Args::parse();
    let bytes_found =
        args.search_path
            .data_over_time(args.start_time, args.end_time, args.follow_links)?;
    println!("{}b", human_bytes(bytes_found as f64));
    Ok(())
}
