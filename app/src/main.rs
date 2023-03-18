use app::*;
use gloo_net::http::Request;
use shared::{DWComparingStatus, DWScanningStatus, DWStatus};
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
    let state = use_state(DWStatus::default);
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
            if !matches!(*state, DWStatus::Initializing | DWStatus::Scanning(_)) {
                0
            } else {
                500
            },
        );
    }

    dwstatus_summary(&state)
}

fn dwstatus_summary(status: &DWStatus) -> Html {
    let contents = match status {
        DWStatus::Initializing => html! {},
        DWStatus::Scanning(scanning) => dwscanning_summary(scanning),
        DWStatus::Comparing(comparing) => dwcomparing_summary(comparing),
        DWStatus::Ready => {
            html! { <b>{"Ready placeholder"}</b>}
        }
    };
    html! {
        <>
        <section class="section">
          <div class="container">
            {contents}
          </div>
        </section>
        </>
    }
}

fn dwcomparing_summary(comparing: &DWComparingStatus) -> Html {
    let progress_str = format!("{} / {}", comparing.image_scanning, comparing.total_images);
    let current_image_number = comparing.image_scanning.to_string();
    let total_images = comparing.total_images.to_string();
    html! {
        <div class="panel is-primary">
          <p class="panel-heading">{"Looking for near duplicates..."}</p>
          <div class="panel-block">
            <label>{"Progress: "}<b>{progress_str}</b></label>
          </div>
          <div class="panel-block">
            <progress class="progress is-primary" value={current_image_number} max={total_images}/>
          </div>
        </div>
    }
}

fn dwscanning_summary(scanning: &DWScanningStatus) -> Html {
    let last_image = scanning
        .last_scanned
        .as_ref()
        .and_then(|pb| pb.file_name())
        .map(|pb| pb.to_string_lossy())
        .unwrap_or_default();

    html! {
        <div class="panel is-primary">
          <p class="panel-heading">{"Scanning image files..."}</p>
          <div class="panel-block">
            <label>{"Images scanned: "}<b>{scanning.count}</b></label>
          </div>
          <div class="panel-block">
            <label>{"Last image: "}<b>{last_image}</b></label>
          </div>
        </div>
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
