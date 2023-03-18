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

    /// clear the cache file before starting
    #[argh(switch)]
    clear_cache: bool,
}

fn main() {
    let args: Args = argh::from_env();
    let cache_file = args
        .cache_file
        .unwrap_or_else(|| {
            directories::BaseDirs::new()
                .map(|bd| bd.cache_dir().join("dupwalker_cache.js"))
                .unwrap()
        });
    if args.clear_cache {
        if let Err(err) = std::fs::remove_file(&cache_file) {
            println!("Error clearing cache: {:?}", err);
        }
    }
    dupwalker::Engine.run(&args.directories, &cache_file);
}
