use gloo_net::http::Request;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_logged_out: Callback<()>,
}

#[derive(Clone, PartialEq, Deserialize)]
struct Me {
    id: String,
    username: String,
    email: Option<String>,
}

#[derive(Clone, PartialEq, Deserialize)]
struct PokemonSummary {
    caught: bool,
}

#[function_component]
pub fn Profile(props: &Props) -> Html {
    let me = use_state(|| None as Option<Me>);
    let me_error = use_state(|| None as Option<String>);
    let caught_count = use_state(|| None as Option<usize>);
    let caught_error = use_state(|| None as Option<String>);
    let success = use_state(|| None as Option<String>);

    {
        let me = me.clone();
        let me_error = me_error.clone();
        use_effect_with((), move |_| {
            let me = me.clone();
            let me_error = me_error.clone();
            spawn_local(async move {
                let res = Request::get("/api/auth/me")
                    .credentials(web_sys::RequestCredentials::Include)
                    .send()
                    .await;
                match res {
                    Ok(r) if r.status() == 200 => match r.json::<Me>().await {
                        Ok(data) => me.set(Some(data)),
                        Err(e) => me_error.set(Some(format!("Réponse invalide: {}", e))),
                    },
                    Ok(r) => {
                        me_error.set(Some(format!("Échec chargement profil ({}).", r.status())))
                    }
                    Err(e) => me_error.set(Some(format!("Erreur requête: {}", e))),
                }
            });
            || {}
        });
    }

    // Charger le nombre de pokémons attrapés
    {
        let caught_count = caught_count.clone();
        let caught_error = caught_error.clone();
        use_effect_with((), move |_| {
            let caught_count = caught_count.clone();
            let caught_error = caught_error.clone();
            spawn_local(async move {
                match Request::get("/api/pokemons")
                    .credentials(web_sys::RequestCredentials::Include)
                    .send()
                    .await
                {
                    Ok(r) if r.status() == 200 => match r.json::<Vec<PokemonSummary>>().await {
                        Ok(list) => {
                            caught_count.set(Some(list.iter().filter(|p| p.caught).count()))
                        }
                        Err(e) => caught_error.set(Some(format!("Réponse invalide: {}", e))),
                    },
                    Ok(r) => caught_error
                        .set(Some(format!("Échec chargement Pokédex ({}).", r.status()))),
                    Err(e) => caught_error.set(Some(format!("Erreur réseau: {}", e))),
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
                    .send()
                    .await;
                on_logged_out.emit(());
            });
        })
    };

    // Changement de mot de passe
    let current_pwd = use_state(|| String::new());
    let new_pwd = use_state(|| String::new());
    let confirm_pwd = use_state(|| String::new());
    let pwd_error = use_state(|| None as Option<String>);

    let on_change_password = {
        let current_pwd = current_pwd.clone();
        let new_pwd = new_pwd.clone();
        let confirm_pwd = confirm_pwd.clone();
        let pwd_error = pwd_error.clone();
        let success = success.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            pwd_error.set(None);
            success.set(None);
            if new_pwd.is_empty() || current_pwd.is_empty() {
                pwd_error.set(Some("Veuillez remplir tous les champs.".into()));
                return;
            }
            if *new_pwd != *confirm_pwd {
                pwd_error.set(Some("La confirmation ne correspond pas.".into()));
                return;
            }
            let body = serde_json::json!({
                "current_password": (*current_pwd).clone(),
                "new_password": (*new_pwd).clone(),
            });
            let pwd_error = pwd_error.clone();
            let success = success.clone();
            spawn_local(async move {
                match Request::put("/api/auth/change-password")
                    .credentials(web_sys::RequestCredentials::Include)
                    .json(&body)
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(r) if r.status() == 200 => {
                        success.set(Some("Mot de passe mis à jour.".into()))
                    }
                    Ok(r) => pwd_error.set(Some(format!("Échec mise à jour ({}).", r.status()))),
                    Err(e) => pwd_error.set(Some(format!("Erreur réseau: {}", e))),
                }
            });
        })
    };

    html! {
        <section>
            <h2>{"Profil"}</h2>
            <div class="profile-card">
                {
                    if let Some(err) = &*me_error { html!{ <p class="error">{err}</p> } }
                    else if let Some(m) = &*me {
                        html!{
                            <div class="profile-info">
                                <div><span class="lbl">{"Utilisateur"}</span><b class="val">{ &m.username }</b></div>
                                <div><span class="lbl">{"Email"}</span><b class="val">{ m.email.clone().unwrap_or("—".into()) }</b></div>
                            </div>
                        }
                    } else { html!{ <p>{"Chargement du profil..."}</p> } }
                }

                <div class="profile-stats">
                    {
                        if let Some(err) = &*caught_error { html!{ <p class="error">{err}</p> } }
                        else if let Some(c) = *caught_count { html!{ <div><span>{"Pokémons attrapés"}</span><b>{ c }</b></div> } }
                        else { html!{ <p>{"Chargement des statistiques..."}</p> } }
                    }
                </div>

                <form class="form change-password" onsubmit={on_change_password}>
                    <h3>{"Changer le mot de passe"}</h3>
                    <div class="field"><label>{"Mot de passe actuel"}</label>
                        <input type="password" value={(*current_pwd).clone()} oninput={{ let s=current_pwd.clone(); Callback::from(move |e: InputEvent| { if let Some(t)=e.target_dyn_into::<web_sys::HtmlInputElement>(){ s.set(t.value()); } }) }} />
                    </div>
                    <div class="field"><label>{"Nouveau mot de passe"}</label>
                        <input type="password" value={(*new_pwd).clone()} oninput={{ let s=new_pwd.clone(); Callback::from(move |e: InputEvent| { if let Some(t)=e.target_dyn_into::<web_sys::HtmlInputElement>(){ s.set(t.value()); } }) }} />
                    </div>
                    <div class="field"><label>{"Confirmer le nouveau"}</label>
                        <input type="password" value={(*confirm_pwd).clone()} oninput={{ let s=confirm_pwd.clone(); Callback::from(move |e: InputEvent| { if let Some(t)=e.target_dyn_into::<web_sys::HtmlInputElement>(){ s.set(t.value()); } }) }} />
                    </div>
                    { if let Some(err) = &*pwd_error { html!{ <p class="error">{err}</p> } } else { html!{} } }
                    { if let Some(msg) = &*success { html!{ <p class="success">{msg}</p> } } else { html!{} } }
                    <button class="primary" type="submit">{"Mettre à jour"}</button>
                </form>

                <div class="actions">
                    <button class="logoutbutton" onclick={logout}>{"Se déconnecter"}</button>
                </div>
            </div>
        </section>
    }
}
