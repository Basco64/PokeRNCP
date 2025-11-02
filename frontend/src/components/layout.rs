use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub children: Children,
}

#[function_component]
pub fn Layout(props: &Props) -> Html {
    html! {
        <div class="layout">
            <header class="header">
                <h1>{"PokeRNCP"}</h1>
            </header>
            <main class="content">
                { for props.children.iter() }
            </main>
        </div>
    }
}
