use crate::engine::status_mgr::StatusMgrMsg;
use crossbeam_channel::{Receiver, Sender};
use rocket::fs::FileServer;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{routes, State};
use shared::DWStatus;
use std::path::{Path, PathBuf};

mod dupfinder;
mod file_walker;
mod hasher;
mod image_loader;
mod status_mgr;

pub fn first_or_default<T: Default>(recv: Receiver<T>) -> T {
    recv.iter().next().unwrap_or_default()
}

#[get("/")]
fn root() -> Redirect {
    Redirect::to("/app")
}

#[get("/noop")]
fn api_noop(status_sndr: &State<Sender<StatusMgrMsg>>) -> String {
    (*status_sndr).clone().send(StatusMgrMsg::NoOp).unwrap();
    "NoOp".to_string()
}

#[get("/status")]
fn api_status(status_sndr: &State<Sender<StatusMgrMsg>>) -> Json<DWStatus> {
    let (sndr, recv) = crossbeam_channel::bounded(1);
    let send_result = (*status_sndr)
        .clone()
        .send(StatusMgrMsg::StatusRequest(sndr));
    if let Err(err) = send_result {
        println!("ERROR in api_status: {:?}", err);
    }
    Json(first_or_default(recv))
}

#[rocket::main]
async fn start_rocket() -> Result<(), rocket::Error> {
    // We pass this in a global since Rocket won't let me add parameters to this function.
    // We 'take' the Sender so that we don't leave any unmanaged references to it.
    let sndr = unsafe { STATUS_MSG_SENDER.take().unwrap() };

    let _r = rocket::build()
        .mount("/", routes![root])
        .mount("/api", routes![api_noop, api_status])
        .mount("/app", FileServer::from("dist-app"))
        .manage(sndr)
        .launch()
        .await?;

    Ok(())
}

pub struct Engine;

// TODO: move stuff around to try to get rid of the globals.
static mut STATUS_MSG_SENDER: Option<Sender<StatusMgrMsg>> = None;

impl Engine {
    pub fn run(self, roots: &[PathBuf], cache_file: &Path) {
        let status_sndr = status_mgr::start(cache_file);
        unsafe {
            // AFAIK, we have no way to pass this to the Rocket builder func, so we use a global.
            STATUS_MSG_SENDER = Some(status_sndr.clone());
        }

        let file_recv = file_walker::start(roots);
        let loader_recv = image_loader::start(file_recv, status_sndr.clone());
        let hasher_done_recv = hasher::start(loader_recv, status_sndr.clone());
        let dupfinder_done_recv = dupfinder::start(hasher_done_recv, status_sndr);

        start_rocket().unwrap();
    }
}
