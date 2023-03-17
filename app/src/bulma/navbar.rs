use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct NavBarProps {
    pub children: Children,
}

#[function_component]
pub fn NavBar(props: &NavBarProps) -> Html {
    html! {
        <>
        <nav class="navbar is-fixed-top" role="navigation" aria-label="dropdown navigation">
          { for props.children.iter() }
        </nav>
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct NavBarBrandProps {
    pub children: Children,
}

#[function_component]
pub fn NavBarBrand(props: &NavBarBrandProps) -> Html {
    html! {
        <>
        <div class="navbar-brand">
          { for props.children.iter() }
        </div>
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct NavBarItemProps {
    pub children: Children,
}

#[function_component]
pub fn NavBarItem(props: &NavBarItemProps) -> Html {
    html! {
        <a class="navbar-item" href={"/"}>
          { for props.children.iter() }
        </a>
    }
}

#[function_component]
pub fn NavBarBurger() -> Html {
    html! {
        <a role="button" class="navbar-burger" aria-label="menu" aria-expanded="false" data-target="navbarBasicExample">
          <span aria-hidden="true"></span>
          <span aria-hidden="true"></span>
          <span aria-hidden="true"></span>
        </a>
    }
}
