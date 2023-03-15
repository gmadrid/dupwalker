use argh;
use argh::FromArgs;
use std::path::PathBuf;
use std::thread;

use crossbeam_channel;
use crossbeam_channel::Receiver;
use ignore::{DirEntry, WalkBuilder};

/// Tool to find near-duplicates
#[derive(FromArgs)]
struct Args {
    /// the directories to walk.
    #[argh(positional)]
    directories: Vec<PathBuf>,
}

struct FileWalker;

fn skip_lightroom(entry: &DirEntry) -> bool {
    entry
        .path()
        .extension()
        .map(|ext| ext != "lrdata" && ext != "lrcat" && ext != "lrcat-data")
        .unwrap_or(true)
}

impl FileWalker {
    pub fn start(roots: &Vec<PathBuf>) -> Receiver<PathBuf> {
        let (sender, receiver) = crossbeam_channel::bounded(100);

        let mut builder = WalkBuilder::new(roots.first().unwrap_or(&".".into()).to_path_buf());
        roots.iter().skip(1).for_each(|p| {
            builder.add(p);
        });

        thread::spawn(move || {
            // We filter out Lightroom directories and ignore any errors.
            // TODO: don't ignore errors
            for entry in builder
                .filter_entry(skip_lightroom)
                .build()
                .into_iter()
                .filter_map(|r| r.ok())
            {
                // Don't send any directory names.
                if !entry.metadata().map(|e| e.is_dir()).unwrap_or_default() {
                    sender.send(entry.into_path()).unwrap();
                }
            }
        });

        receiver
    }
}

fn main() {
    let args: Args = argh::from_env();
    let engine = dupwalker::init_engine();
    engine.run();

    let receiver = FileWalker::start(&args.directories);

    for path in receiver {
        println!("Received: {}", path.display())
    }
}
