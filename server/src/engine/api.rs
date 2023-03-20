use rocket::response::Redirect;
use rocket::State;
use crossbeam_channel::Sender;
use rocket::serde::json::Json;
use shared::DWStatus;
use crate::engine;
use crate::engine::status_mgr::StatusMgrMsg;

#[get("/")]
pub fn root() -> Redirect {
    Redirect::to("/app")
}

#[get("/noop")]
pub fn api_noop(status_sndr: &State<Sender<StatusMgrMsg>>) -> String {
    (*status_sndr).clone().send(StatusMgrMsg::NoOp).unwrap();
    "NoOp".to_string()
}

#[get("/status")]
pub fn api_status(status_sndr: &State<Sender<StatusMgrMsg>>) -> Json<DWStatus> {
    let (sndr, recv) = crossbeam_channel::bounded(1);
    let send_result = (*status_sndr)
        .clone()
        .send(StatusMgrMsg::StatusRequest(sndr));
    if let Err(err) = send_result {
        println!("ERROR in api_status: {:?}", err);
    }
    Json(engine::first_or_default(recv))
}
