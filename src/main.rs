use yew::prelude::*;

#[function_component]
fn App() -> Html {
    html!{
        <div><b>{"Hi, George!"}</b></div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
