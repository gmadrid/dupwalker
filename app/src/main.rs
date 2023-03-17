use app::*;
use gloo_net::http::Request;
use shared::DWStatus;
use std::path::PathBuf;
use yew::prelude::*;
use yew_hooks::use_interval;

#[function_component]
fn DWNavBar() -> Html {
    html! {
        <NavBar>
          <NavBarBrand>
            <NavBarItem>
              <div><b>{"DupWalker"}</b></div>
            </NavBarItem>
            <NavBarBurger />
          </NavBarBrand>
        </NavBar>

    }
}

#[function_component]
fn DWBody() -> Html {
    let state = use_state(|| DWStatus::default());
    {
        let state2 = state.clone();
        use_interval(
            move || {
                let state3 = state2.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let foobar = Request::get("/api/status")
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    state3.set(foobar);
                });
            },
            if state.finished { 0 } else { 500 },
        );
    }

    let last_image = state
        .last_scanned
        .as_ref()
        .and_then(|pb| pb.file_name())
        .map(|pb| pb.to_string_lossy())
        .unwrap_or_default()
        .to_string();

    html! {
        <>
        <section class="section">
        <div class="container">
        <div class="panel">
        <p class="panel-heading">{"Scan status"}</p>
        <div class="panel-block">
        <label>{"Images scanned: "}<b>{state.count}</b></label>
        </div>
        <div class="panel-block">
        <p><label>{"Last image scanned:"}<b>{last_image}</b></label></p>
        </div>
        <div class="panel-block">
        <p><label>{"Finished:"}<b>{state.finished}</b></label></p>
        </div>
        </div>
        </div>
        </section>
        </>
    }
}

#[function_component]
fn App() -> Html {
    html! {
        <>
        <DWNavBar/>
        <DWBody/>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
