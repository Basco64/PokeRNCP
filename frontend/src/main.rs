use yew::prelude::*;use yew::prelude::*;

mod components;mod components;

use components::{Layout, LoginForm, Pokedex, SignUpForm, Profile};use components::{Layout, LoginForm, Pokedex, SignUpForm, Profile};

use gloo_net::http::Request;use gloo_net::http::Request;

use wasm_bindgen_futures::spawn_local;use wasm_bindgen_futures::spawn_local;



#[function_component]#[function_component]

fn App() -> Html {fn App() -> Html {

    let logged_in = use_state(|| false);    let logged_in = use_state(|| false);

    #[derive(Clone, Copy, PartialEq)]    #[derive(Clone, Copy, PartialEq)]

    enum AuthMode {    enum AuthMode {

        Login,        Login,

        Signup,        Signup,

    }    }

    let mode = use_state(|| AuthMode::Login);    let mode = use_state(|| AuthMode::Login);

    #[derive(Clone, Copy, PartialEq)]    #[derive(Clone, Copy, PartialEq)]

    enum AppView { Pokedex, Profile }    enum AppView { Pokedex, Profile }

    let view = use_state(|| AppView::Pokedex);    let view = use_state(|| AppView::Pokedex);



    {    {

        let logged_in = logged_in.clone();        let logged_in = logged_in.clone();

        use_effect_with((), move |_| {        use_effect_with((), move |_| {

            let logged_in = logged_in.clone();            let logged_in = logged_in.clone();

            spawn_local(async move {            spawn_local(async move {

                let me = Request::get("/api/auth/me")                let me = Request::get("/api/auth/me")

                    .credentials(web_sys::RequestCredentials::Include)                    .credentials(web_sys::RequestCredentials::Include)

                    .send().await;                    .send().await;

                let ok = match me {                let ok = match me {

                    Ok(r) if r.status() == 200 => true,                    Ok(r) if r.status() == 200 => true,

                    _ => {                    _ => {

                        match Request::post("/api/auth/refresh-token")                        match Request::post("/api/auth/refresh-token")

                            .credentials(web_sys::RequestCredentials::Include)                            .credentials(web_sys::RequestCredentials::Include)

                            .send().await {                            .send().await {

                                Ok(rf) if rf.status() == 200 => true,                                Ok(rf) if rf.status() == 200 => true,

                                _ => false,                                _ => false,

                            }                            }

                    }                    }

                };                };

                if ok { logged_in.set(true); }                if ok { logged_in.set(true); }

            });            });

            || {}            || {}

        });        });

    }    }



    let on_logout = {    let on_logout = {

        let logged_in = logged_in.clone();        let logged_in = logged_in.clone();

        Callback::from(move |_| {        Callback::from(move |_| {

            let logged_in = logged_in.clone();            let logged_in = logged_in.clone();

            spawn_local(async move {            spawn_local(async move {

                let _ = Request::post("/api/auth/logout")                let _ = Request::post("/api/auth/logout")

                    .credentials(web_sys::RequestCredentials::Include)                    .credentials(web_sys::RequestCredentials::Include)

                    .send().await;                    .send().await;

                logged_in.set(false);                logged_in.set(false);

            });            });

        })        })

    };    };



    html! {    html! {

        <Layout>        <Layout>

            if !*logged_in {            if !*logged_in {

                <div class="page-header-actions" style="margin-bottom: 16px;">                <div class="page-header-actions" style="margin-bottom: 16px;">

                    <button class="loginbutton" onclick={{ let mode = mode.clone(); Callback::from(move |_| mode.set(AuthMode::Login)) }}>{"Se connecter"}</button>                    <button class="loginbutton" onclick={{ let mode = mode.clone(); Callback::from(move |_| mode.set(AuthMode::Login)) }}>{"Se connecter"}</button>

                    <button class="inscriptionbutton" onclick={{ let mode = mode.clone(); Callback::from(move |_| mode.set(AuthMode::Signup)) }}>{"Créer un compte"}</button>                    <button class="inscriptionbutton" onclick={{ let mode = mode.clone(); Callback::from(move |_| mode.set(AuthMode::Signup)) }}>{"Créer un compte"}</button>

                </div>                </div>

                { match *mode {                { match *mode {

                    AuthMode::Login => html!{ <LoginForm on_logged_in={{ let logged_in = logged_in.clone(); Callback::from(move |_| logged_in.set(true)) }} /> },                    AuthMode::Login => html!{ <LoginForm on_logged_in={{ let logged_in = logged_in.clone(); Callback::from(move |_| logged_in.set(true)) }} /> },

                    AuthMode::Signup => html!{ <SignUpForm on_logged_in={{ let logged_in = logged_in.clone(); Callback::from(move |_| logged_in.set(true)) }} /> },                    AuthMode::Signup => html!{ <SignUpForm on_logged_in={{ let logged_in = logged_in.clone(); Callback::from(move |_| logged_in.set(true)) }} /> },

                }}                }}

            } else {            } else {

                <div class="page-header-actions" style="margin-bottom: 16px; gap: 8px;">                <div class="page-header-actions" style="margin-bottom: 16px; gap: 8px;">

                    <button class="loginbutton" onclick={{ let view = view.clone(); Callback::from(move |_| view.set(AppView::Pokedex)) }}>{"Pokédex"}</button>                    <button class="loginbutton" onclick={{ let view = view.clone(); Callback::from(move |_| view.set(AppView::Pokedex)) }}>{"Pokédex"}</button>

                    <button class="inscriptionbutton" onclick={{ let view = view.clone(); Callback::from(move |_| view.set(AppView::Profile)) }}>{"Profil"}</button>                    <button class="inscriptionbutton" onclick={{ let view = view.clone(); Callback::from(move |_| view.set(AppView::Profile)) }}>{"Profil"}</button>

                    <button class="logoutbutton" onclick={on_logout}>{"Se déconnecter"}</button>                    <button class="logoutbutton" onclick={on_logout}>{"Se déconnecter"}</button>

                </div>                </div>

                { match *view {                { match *view {

                    AppView::Pokedex => html!{ <Pokedex /> },                    AppView::Pokedex => html!{ <Pokedex /> },

                    AppView::Profile => html!{ <Profile on_logged_out={{ let logged_in = logged_in.clone(); Callback::from(move |_| logged_in.set(false)) }} /> },                    AppView::Profile => html!{ <Profile on_logged_out={{ let logged_in = logged_in.clone(); Callback::from(move |_| logged_in.set(false)) }} /> },

                }}                }}

            }            }

        </Layout>        </Layout>

    }    }

}}



fn main() {fn main() {

    yew::Renderer::<App>::new().render();    yew::Renderer::<App>::new().render();

}}

