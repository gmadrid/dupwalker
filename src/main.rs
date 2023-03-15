use argh;
use argh::FromArgs;
use std::path::PathBuf;

/// Tool to find near-duplicates
#[derive(FromArgs)]
struct Args {
    /// the directories to walk.
    #[argh(positional)]
    directories: Vec<PathBuf>,
}

fn main() {
    let args: Args = argh::from_env();
    dupwalker::init_engine().run(&args.directories);
}
