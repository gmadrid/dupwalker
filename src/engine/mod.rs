use std::path::PathBuf;

mod file_walker;
mod hasher;
mod image_loader;
mod status_mgr;

pub struct Engine;

impl Engine {
    pub fn run(self, roots: &[PathBuf]) {
        let status_sndr = status_mgr::start();

        let file_recv = file_walker::start(roots);
        let loader_recv = image_loader::start(file_recv);

        let done_recv = hasher::start(loader_recv, status_sndr);
        done_recv.iter().count();
    }
}
