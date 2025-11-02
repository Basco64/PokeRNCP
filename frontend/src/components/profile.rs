use yew::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_logged_out: Callback<()>,
}

#[function_component]
pub fn Profile(props: &Props) -> Html {
    let me = use_state(|| Ok::<String, String>("Chargement...".into()));

    {
        let me = me.clone();
        use_effect_with((), move |_| {
            let me = me.clone();
            spawn_local(async move {
                let res = Request::get("/api/auth/me")
                    .credentials(web_sys::RequestCredentials::Include)
                    .send().await;
                match res {
                    Ok(r) => match r.text().await {
                        Ok(txt) => me.set(Ok(txt)),
                        Err(e) => me.set(Err(format!("Erreur lecture: {}", e))),
                    },
                    Err(e) => me.set(Err(format!("Erreur requête: {}", e))),
                }
            });
            || {}
        });
    }

    let logout = {
        let on_logged_out = props.on_logged_out.clone();
        Callback::from(move |_| {
            let on_logged_out = on_logged_out.clone();
            spawn_local(async move {
                let _ = Request::post("/api/auth/logout")
                    .credentials(web_sys::RequestCredentials::Include)
                    .send().await;
                on_logged_out.emit(());
            });
        })
    };

    html! {
        <section>
            <h2>{"Profil"}</h2>
            {
                match &*me {
                    Ok(txt) => html!{ <pre class="profile-json">{ txt }</pre> },
                    Err(err) => html!{ <p class="error">{ err }</p> },
                }
            }
            <button class="logoutbutton" onclick={logout}>{"Se déconnecter"}</button>
        </section>
    }
}
