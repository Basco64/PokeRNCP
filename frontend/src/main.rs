use yew::prelude::*;
mod components;
use components::{Layout, LoginForm, Pokedex, Profile, SignUpForm};
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen_futures::spawn_local;

#[function_component]
fn App() -> Html {
    let logged_in = use_state(|| false);
    const SESSION_HINT_KEY: &str = "session_hint";

    #[derive(Clone, Copy, PartialEq)]
    enum AuthMode {
        Login,
        Signup,
    }
    let mode = use_state(|| AuthMode::Login);

    #[derive(Clone, Copy, PartialEq)]
    enum AppView {
        Pokedex,
        Profile,
    }
    let view = use_state(|| AppView::Pokedex);

    {
        let logged_in = logged_in.clone();
        use_effect_with((), move |_| {
            let should_probe = LocalStorage::get::<String>(SESSION_HINT_KEY)
                .ok()
                .as_deref()
                == Some("1");
            if should_probe {
                spawn_local(async move {
                    match Request::get("/api/auth/me")
                        .credentials(web_sys::RequestCredentials::Include)
                        .send()
                        .await
                    {
                        Ok(r) if r.status() == 200 => logged_in.set(true),
                        Ok(r) if r.status() == 401 || r.status() == 403 => {
                            if let Ok(rf) = Request::post("/api/auth/refresh-token")
                                .credentials(web_sys::RequestCredentials::Include)
                                .send()
                                .await
                            {
                                if rf.status() == 200 {
                                    logged_in.set(true);
                                }
                            }
                        }
                        _ => {}
                    }
                });
            }
            || {}
        });
    }

    let on_logout = {
        let logged_in = logged_in.clone();
        Callback::from(move |_| {
            let logged_in = logged_in.clone();
            spawn_local(async move {
                let _ = Request::post("/api/auth/logout")
                    .credentials(web_sys::RequestCredentials::Include)
                    .send()
                    .await;
                logged_in.set(false);
                let _ = LocalStorage::delete(SESSION_HINT_KEY);
            });
        })
    };

    html! {
        <Layout>
            if !*logged_in {
                <div class="page-header-actions" style="margin-bottom: 16px;">
                    <button class="loginbutton" onclick={{ let mode = mode.clone(); Callback::from(move |_| mode.set(AuthMode::Login)) }}>{"Se connecter"}</button>
                    <button class="inscriptionbutton" onclick={{ let mode = mode.clone(); Callback::from(move |_| mode.set(AuthMode::Signup)) }}>{"Créer un compte"}</button>
                </div>
                { match *mode {
                    AuthMode::Login => html! { <LoginForm on_logged_in={{ let logged_in = logged_in.clone(); Callback::from(move |_| { let _ = LocalStorage::set(SESSION_HINT_KEY, "1"); logged_in.set(true) }) }} /> },
                    AuthMode::Signup => html! { <SignUpForm on_logged_in={{ let logged_in = logged_in.clone(); Callback::from(move |_| { let _ = LocalStorage::set(SESSION_HINT_KEY, "1"); logged_in.set(true) }) }} /> },
                }}
            } else {
                <div class="page-header-actions" style="margin-bottom: 16px; gap: 8px;">
                    <button class="loginbutton" onclick={{ let view = view.clone(); Callback::from(move |_| view.set(AppView::Pokedex)) }}>{"Pokédex"}</button>
                    <button class="inscriptionbutton" onclick={{ let view = view.clone(); Callback::from(move |_| view.set(AppView::Profile)) }}>{"Profil"}</button>
                    <button class="logoutbutton" onclick={on_logout}>{"Se déconnecter"}</button>
                </div>
                { match *view {
                    AppView::Pokedex => html! { <Pokedex /> },
                    AppView::Profile => html! { <Profile on_logged_out={{ let logged_in = logged_in.clone(); Callback::from(move |_| logged_in.set(false)) }} /> },
                }}
            }
        </Layout>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
