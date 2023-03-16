use crossbeam_channel::Receiver;
use ignore::{DirEntry, WalkBuilder};
use std::path::PathBuf;
use std::thread;

fn skip_lightroom(entry: &DirEntry) -> bool {
    entry
        .path()
        .extension()
        .map(|ext| ext != "lrdata" && ext != "lrcat" && ext != "lrcat-data")
        .unwrap_or(true)
}

pub fn start(roots: &[PathBuf]) -> Receiver<PathBuf> {
    let (sender, receiver) = crossbeam_channel::bounded(100);

    let mut builder = WalkBuilder::new(roots.first().unwrap_or(&".".into()));
    roots.iter().skip(1).for_each(|p| {
        builder.add(p);
    });

    thread::spawn(move || {
        // We filter out Lightroom directories and ignore any errors.
        // TODO: don't ignore errors
        for entry in builder
            .filter_entry(skip_lightroom)
            .build()
            .filter_map(|r| r.ok())
        {
            // Don't send any directory names or non-Image files.
            if !entry.metadata().map(|e| e.is_dir()).unwrap_or_default()
                && mime_guess::from_path(entry.path())
                    .iter()
                    .any(|m| m.type_() == mime::IMAGE)
            {
                sender
                    .send(entry.into_path().canonicalize().unwrap())
                    .unwrap();
            }
        }
    });

    receiver
}
