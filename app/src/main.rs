use yew::prelude::*;
use yew_hooks::use_interval;
use gloo_net::http::Request;
use std::ops::Deref;

use app::*;

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
    let state = use_state(|| "".to_string());
    {
        let state2 = state.clone();
        use_interval(
             move || {
                 let state3 = state2.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let foobar = Request::get("/api/count").send().await.unwrap();
                    let val = foobar.text().await.unwrap();
                    state3.set(val);
                });
            },
            5000,
        );
    }

    html! {
        <>
        <section class="section">
        <div class="container">
        <h1 class="title">{"George something"}</h1>
        <p class="subtitle">{"A subtitle"}</p>
        <b>{state.deref()}</b>
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
