use crate::engine::status_mgr::StatusMgr;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::State;
use shared::{DWComparingStatus, DWScanningStatus, DWStatus};
use std::sync::{Arc, Mutex};

#[get("/")]
pub fn root() -> Redirect {
    Redirect::to("/app")
}

#[get("/noop")]
pub fn api_noop() -> String {
    "NoOp".to_string()
}

#[get("/status")]
pub fn api_status(status_mgr: &State<Arc<Mutex<StatusMgr>>>) -> Json<DWStatus> {
    let status_mgr = status_mgr.lock().unwrap();

    let s = if status_mgr.scan_finished() {
        DWStatus::Comparing(DWComparingStatus {
            total_images: status_mgr.len(),
            image_scanning: 0,
        })
    } else {
        DWStatus::Scanning(DWScanningStatus {
            count: status_mgr.len(),
            last_scanned: status_mgr
                .last_scanned()
                .as_ref()
                .map(|apb| apb.as_ref().clone()),
        })
    };
    Json(s)
}
