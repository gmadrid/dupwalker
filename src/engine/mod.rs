use crate::engine::status_mgr::StatusMgrMsg;
use crossbeam_channel::{Receiver, Sender};
use rocket::request::{FromRequest, Outcome};
use rocket::{routes, Request};
use std::path::PathBuf;

mod file_walker;
mod hasher;
mod image_loader;
mod status_mgr;

fn first_or_default<T: Default>(mut i: impl Iterator<Item = T>) -> T {
    i.next().unwrap_or_default()
}

#[get("/world")]
fn test_foo(status_sndr: StatusMsgSender) -> String {
    first_or_default(status_sndr.test().iter())
}

#[rocket::main]
async fn start_rocket() -> Result<(), rocket::Error> {
    let _r = rocket::build()
        .mount("/", routes![test_foo])
        .launch()
        .await?;

    Ok(())
}

pub struct Engine;

static mut STATUS_MSG_SENDER: Option<Sender<StatusMgrMsg>> = None;

struct StatusMsgSender(Sender<StatusMgrMsg>);

impl StatusMsgSender {
    fn noop(&self) {
        self.0.send(StatusMgrMsg::NoOp).unwrap();
    }

    fn test(&self) -> Receiver<String> {
        let (sndr, recv) = crossbeam_channel::bounded(1);
        self.0.send(StatusMgrMsg::TestMsg(sndr)).unwrap();
        recv
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for StatusMsgSender {
    type Error = ();

    async fn from_request(_: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let sndr = unsafe { STATUS_MSG_SENDER.as_ref().cloned().unwrap() };
        Outcome::Success(StatusMsgSender(sndr))
    }
}

impl Engine {
    pub fn run(self, roots: &[PathBuf]) {
        let status_sndr = status_mgr::start();
        unsafe {
            STATUS_MSG_SENDER = Some(status_sndr.clone());
        }

        let file_recv = file_walker::start(roots);
        let loader_recv = image_loader::start(file_recv);
        hasher::start(loader_recv, status_sndr);

        start_rocket().unwrap();
    }
}
