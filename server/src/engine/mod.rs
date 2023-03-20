use crate::engine::status_mgr::StatusMgr;
use crossbeam_channel::{Receiver};
use rocket::fs::FileServer;
use rocket::routes;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

mod api;
mod dupfinder;
mod file_walker;
mod hasher;
mod image_loader;
mod status_mgr;

pub fn first_or_default<T: Default>(recv: Receiver<T>) -> T {
    recv.iter().next().unwrap_or_default()
}

#[rocket::main]
async fn start_rocket() -> Result<(), rocket::Error> {
    // We pass this in a global since Rocket won't let me add parameters to this function.
    let status_mgr = unsafe { STATUS_MGR.take().unwrap() };

    let _r = rocket::build()
        .mount("/", routes![api::root])
        .mount("/api", routes![api::api_noop, api::api_status])
        .mount("/app", FileServer::from("dist-app"))
        .manage(status_mgr)
        .launch()
        .await?;

    Ok(())
}

pub struct Engine;

// TODO: move stuff around to try to get rid of the globals.
static mut STATUS_MGR: Option<Arc<Mutex<StatusMgr>>> = None;

impl Engine {
    pub fn run(self, roots: &[PathBuf], cache_file: &Path) {
        let status_mgr = Arc::new(Mutex::new(StatusMgr::load_or_default(cache_file)));

        unsafe {
            // AFAIK, we have no way to pass this to the Rocket builder func, so we use a global.
            STATUS_MGR = Some(status_mgr.clone());
        }

        let file_recv = file_walker::start(roots);
        let loader_recv = image_loader::start(file_recv, status_mgr.clone());
        let hasher_done_recv = hasher::start(loader_recv, status_mgr.clone());
        let dupfinder_done_recv = dupfinder::start(hasher_done_recv, status_mgr.clone());

        start_rocket().unwrap();
    }
}
