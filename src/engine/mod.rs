use std::path::PathBuf;

mod file_walker;

pub struct Engine;

impl Engine {
    pub fn run(self, roots: &Vec<PathBuf>) {
        let file_recv = file_walker::start(roots);

        for path in file_recv {
            println!("Received: {}", path.display())
        }
    }
}
