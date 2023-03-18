use argh::FromArgs;
use std::path::PathBuf;

/// Tool to find near-duplicates
#[derive(FromArgs)]
struct Args {
    /// the directories to walk.
    #[argh(positional)]
    directories: Vec<PathBuf>,

    /// location for the cache file.
    /// (Defaults to system 'cache' directory.)
    #[argh(option)]
    cache_file: Option<PathBuf>,
}

fn main() {
    let args: Args = argh::from_env();
    let cache_file = args.cache_file.unwrap_or_else(|| PathBuf::from("./cache_file.js"));
    dupwalker::Engine.run(&args.directories, &cache_file);
}
