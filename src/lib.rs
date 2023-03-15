mod engine;
mod hash_mgr;
mod status_mgr;
mod status_timer;

use engine::Engine;

pub fn init_engine() -> Engine {
    Engine
}
