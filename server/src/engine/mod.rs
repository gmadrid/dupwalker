use crate::engine::status_mgr::StatusMgrMsg;
use crossbeam_channel::{Receiver, Sender};
use rocket::{routes, State};
use std::path::PathBuf;
use rocket::response::Redirect;

mod file_walker;
mod hasher;
mod image_loader;
mod status_mgr;

fn first_or_default<T: Default>(recv: Receiver<T>) -> T {
    recv.iter().next().unwrap_or_default()
}

#[get("/")]
fn root() -> Redirect {
    Redirect::to(uri!(app_root()))
}

#[get("/app")]
fn app_root(status_sndr: &State<Sender<StatusMgrMsg>>) -> String {
    let (sndr, recv) = crossbeam_channel::bounded(1);
    (*status_sndr).clone().send(StatusMgrMsg::TestMsg(sndr)).unwrap();
    format!("ROOT: {}", first_or_default(recv))
}

#[get("/noop")]
fn noop(status_sndr: &State<Sender<StatusMgrMsg>>) -> String {
    (*status_sndr).clone().send(StatusMgrMsg::NoOp).unwrap();
    "NoOp".to_string()
}

#[rocket::main]
async fn start_rocket() -> Result<(), rocket::Error> {
    // We pass this in a global since Rocket won't let me add parameters to this function.
    // We 'take' the Sender so that we don't leave any unmanaged references to it.
    let sndr = unsafe { STATUS_MSG_SENDER.take().unwrap() };

    let _r = rocket::build()
        .mount("/", routes![root, app_root])
        .mount("/api", routes![noop])
        .manage(sndr)
        .launch()
        .await?;

    Ok(())
}

pub struct Engine;

static mut STATUS_MSG_SENDER: Option<Sender<StatusMgrMsg>> = None;

impl Engine {
    pub fn run(self, roots: &[PathBuf]) {
        let status_sndr = status_mgr::start();
        unsafe {
            // AFAIK, we have no way to pass this to the Rocket builder func, so we use a global.
            STATUS_MSG_SENDER = Some(status_sndr.clone());
        }

        let file_recv = file_walker::start(roots);
        let loader_recv = image_loader::start(file_recv);
        hasher::start(loader_recv, status_sndr);

        start_rocket().unwrap();
    }
}
