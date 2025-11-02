use gloo_net::http::Request;
use gloo_timers::callback::Timeout;
use serde::Deserialize;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Clone, PartialEq, Deserialize)]
struct PokemonItem {
    id: i32,
    name: String,
    #[allow(dead_code)]
    type1: String,
    #[allow(dead_code)]
    type2: Option<String>,
    #[allow(dead_code)]
    dex_no: Option<i32>,
    image_url: Option<String>,
    caught: bool,
}

#[derive(Clone, PartialEq, Deserialize)]
struct PokemonDetail {
    id: i32,
    name: String,
    #[allow(dead_code)]
    type1: String,
    #[allow(dead_code)]
    type2: Option<String>,
    #[allow(dead_code)]
    dex_no: Option<i32>,
    image_url: Option<String>,
    height_m: Option<f64>,
    weight_kg: Option<f64>,
    description: Option<String>,
    base_hp: Option<i32>,
    base_attack: Option<i32>,
    base_defense: Option<i32>,
    base_sp_attack: Option<i32>,
    base_sp_defense: Option<i32>,
    base_speed: Option<i32>,
    #[allow(dead_code)]
    caught: bool,
}

#[function_component]
pub fn Pokedex() -> Html {
    let pokemons = use_state(|| Vec::<PokemonItem>::new());
    let error = use_state(|| None as Option<String>);
    let query = use_state(|| String::new());
    let detail_open = use_state(|| false);
    let detail = use_state(|| None as Option<PokemonDetail>);
    let detail_error = use_state(|| None as Option<String>);

    // Charger la liste selon la recherche (vide => tout) avec un petit debounce
    {
        let pokemons = pokemons.clone();
        let error = error.clone();
        let query = query.clone();
        use_effect_with(query.clone(), move |q| {
            let term = (*q).clone();
            let pokemons = pokemons.clone();
            let error = error.clone();
            let handle = Timeout::new(300, move || {
                let term = term.clone();
                let pokemons = pokemons.clone();
                let error = error.clone();
                spawn_local(async move {
                    let make_req = |term: &String| {
                        if term.trim().is_empty() {
                            Request::get("/api/pokemons")
                                .credentials(web_sys::RequestCredentials::Include)
                        } else {
                            let url =
                                format!("/api/pokemons/search?q={}", urlencoding::encode(term));
                            Request::get(&url).credentials(web_sys::RequestCredentials::Include)
                        }
                    };
                    let mut did_refresh = false;
                    let mut resp = make_req(&term).send().await;
                    if let Ok(r) = &resp {
                        if r.status() == 401 || r.status() == 403 {
                            if let Ok(rr) = Request::post("/api/auth/refresh-token")
                                .credentials(web_sys::RequestCredentials::Include)
                                .send()
                                .await
                            {
                                if rr.status() == 200 {
                                    did_refresh = true;
                                    resp = make_req(&term).send().await;
                                }
                            }
                        }
                    }
                    match resp {
                        Ok(r) if r.status() == 200 => match r.json::<Vec<PokemonItem>>().await {
                            Ok(list) => pokemons.set(list),
                            Err(e) => error.set(Some(format!("Erreur parsing: {}", e))),
                        },
                        Ok(r) => {
                            let extra = if did_refresh { " (après refresh)" } else { "" };
                            error.set(Some(format!(
                                "Échec chargement (status: {}){}",
                                r.status(),
                                extra
                            )))
                        }
                        Err(e) => error.set(Some(format!("Erreur requête: {}", e))),
                    }
                });
            });
            move || drop(handle)
        });
    }

    let on_catch = {
        let pokemons = pokemons.clone();
        let error = error.clone();
        Callback::from(move |name: String| {
            let pokemons = pokemons.clone();
            let error = error.clone();
            spawn_local(async move {
                let body = serde_json::json!({ "name": name.clone() });
                let target_name = name.clone();
                let mut did_refresh = false;
                let mut resp = Request::post("/api/pokemons/catch")
                    .credentials(web_sys::RequestCredentials::Include)
                    .json(&body)
                    .unwrap()
                    .send()
                    .await;
                if let Ok(r) = &resp {
                    if r.status() == 401 || r.status() == 403 {
                        if let Ok(rr) = Request::post("/api/auth/refresh-token")
                            .credentials(web_sys::RequestCredentials::Include)
                            .send()
                            .await
                        {
                            if rr.status() == 200 {
                                did_refresh = true;
                                resp = Request::post("/api/pokemons/catch")
                                    .credentials(web_sys::RequestCredentials::Include)
                                    .json(&body)
                                    .unwrap()
                                    .send()
                                    .await;
                            }
                        }
                    }
                }
                match resp {
                    Ok(r) if r.status() == 201 || r.status() == 200 => {
                        // Marque comme attrapé côté UI
                        let current = (*pokemons).clone();
                        let updated: Vec<PokemonItem> = current
                            .into_iter()
                            .map(|mut p| {
                                if p.name == target_name {
                                    p.caught = true
                                }
                                p
                            })
                            .collect();
                        pokemons.set(updated);
                    }
                    Ok(r) => {
                        let extra = if did_refresh { " (après refresh)" } else { "" };
                        error.set(Some(format!(
                            "Impossible d'attraper ({}).{}",
                            r.status(),
                            extra
                        )))
                    }
                    Err(e) => error.set(Some(format!("Erreur réseau: {}", e))),
                }
            });
        })
    };

    // Ouvrir la fiche détaillée (seulement pour les pokémons attrapés)
    let on_open_detail = {
        let detail_open = detail_open.clone();
        let detail = detail.clone();
        let detail_error = detail_error.clone();
        Callback::from(move |id: i32| {
            detail_open.set(true);
            detail.set(None);
            detail_error.set(None);
            let detail = detail.clone();
            let detail_error = detail_error.clone();
            spawn_local(async move {
                let url = format!("/api/pokemons/{}", id);
                let mut did_refresh = false;
                let mut resp = Request::get(&url)
                    .credentials(web_sys::RequestCredentials::Include)
                    .send()
                    .await;
                if let Ok(r) = &resp {
                    if r.status() == 401 || r.status() == 403 {
                        if let Ok(rr) = Request::post("/api/auth/refresh-token")
                            .credentials(web_sys::RequestCredentials::Include)
                            .send()
                            .await
                        {
                            if rr.status() == 200 {
                                did_refresh = true;
                                resp = Request::get(&url)
                                    .credentials(web_sys::RequestCredentials::Include)
                                    .send()
                                    .await;
                            }
                        }
                    }
                }
                match resp {
                    Ok(r) if r.status() == 200 => match r.json::<PokemonDetail>().await {
                        Ok(d) => detail.set(Some(d)),
                        Err(e) => detail_error.set(Some(format!("Erreur parsing: {}", e))),
                    },
                    Ok(r) => {
                        let extra = if did_refresh { " (après refresh)" } else { "" };
                        detail_error.set(Some(format!(
                            "Chargement impossible ({}).{}",
                            r.status(),
                            extra
                        )))
                    }
                    Err(e) => detail_error.set(Some(format!("Erreur réseau: {}", e))),
                }
            });
        })
    };

    let on_close_detail = {
        let detail_open = detail_open.clone();
        Callback::from(move |_| detail_open.set(false))
    };

    html! {
        <>
        <section>
            <h2>{"Pokédex"}</h2>
            <div class="searchbar">
                <input
                    type="search"
                    placeholder="Rechercher par nom..."
                    value={(*query).clone()}
                    oninput={{ let query = query.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<web_sys::HtmlInputElement>() { query.set(t.value()); } }) }}
                />
            </div>
            if let Some(err) = &*error { <p class="error">{err}</p> }
            <div class="grid">
                {
                    for pokemons.iter().map(|p| {
                        let name = p.name.clone();
                        let caught = p.caught;
                        let image_url = p.image_url.clone();
                        let on_catch = on_catch.clone();
                        let open_detail = on_open_detail.clone();

                        // Affichage du nom: si non attrapé, première lettre + "..."
                        let display_name = if caught {
                            name.clone()
                        } else {
                            let first = name.chars().next().unwrap_or('?').to_ascii_uppercase();
                            format!("{}...", first)
                        };

                        // Gestion du clic sur "Attraper"
                        let click_name = name.clone();
                        let onclick = Callback::from(move |_| {
                            if !caught { on_catch.emit(click_name.clone()); }
                        });

                        // Classe supplémentaire pour griser l'image si non attrapé
                        let thumb_class = if caught { "pokemon-thumb" } else { "pokemon-thumb uncaught" };

                        // Click sur la carte seulement si attrapé
                        let card_onclick = if caught {
                            let id = p.id;
                            Some(Callback::from(move |_| open_detail.emit(id)))
                        } else { None };
                        let card_class = if caught { "pokemon-card clickable" } else { "pokemon-card" };

                        html!{
                            <article class={card_class} onclick={card_onclick}>
                                <div class={thumb_class}>
                                    {
                                        if let Some(url) = image_url {
                                            html!{ <img src={url} alt={display_name.clone()} loading="lazy" /> }
                                        } else {
                                            html!{ <div class="noimg">{"?"}</div> }
                                        }
                                    }
                                </div>
                                <div class="pokemon-meta">
                                    <h3 class="pokemon-name">{ display_name }</h3>
                                    {
                                        if caught { html!{ <span class="caught">{"Attrapé"}</span> } }
                                        else { html!{ <button class="inscriptionbutton" onclick={onclick}>{"Attraper"}</button> } }
                                    }
                                </div>
                            </article>
                        }
                    })
                }
            </div>
        </section>
        { if *detail_open {
            html!{
                <div class="overlay">
                    <div class="backdrop" onclick={on_close_detail.clone()}></div>
                    <div class="modal">
                        <div class="modal-header">
                            <h3>{ detail.as_ref().map(|d| d.name.clone()).unwrap_or_else(|| "Chargement...".into()) }</h3>
                            <button class="closebtn" onclick={on_close_detail.clone()}>{"✕"}</button>
                        </div>
                        <div class="modal-body">
                            {
                                if let Some(err) = &*detail_error { html!{ <p class="error">{err}</p> } }
                                else if let Some(d) = &*detail {
                                    html!{
                                        <div class="detail">
                                            <div class="detail-left">
                                                {
                                                    if let Some(url) = &d.image_url { html!{ <img src={url.clone()} alt={d.name.clone()} /> } }
                                                    else { html!{ <div class="noimg">{"?"}</div> } }
                                                }
                                            </div>
                                            <div class="detail-right">
                                                <p><strong>{"Type: "}</strong>{ d.type2.as_ref().map(|t2| format!("{} / {}", d.type1, t2)).unwrap_or_else(|| d.type1.clone()) }</p>
                                                <p><strong>{"Taille: "}</strong>{ d.height_m.map(|v| format!("{:.1} m", v)).unwrap_or("?".into()) }</p>
                                                <p><strong>{"Poids: "}</strong>{ d.weight_kg.map(|v| format!("{:.1} kg", v)).unwrap_or("?".into()) }</p>
                                                { if let Some(desc) = &d.description { html!{ <p class="desc">{desc}</p> } } else { html!{} } }
                                                <div class="stats">
                                                    <div><span>{"PV"}</span><b>{ d.base_hp.unwrap_or(0) }</b></div>
                                                    <div><span>{"ATT"}</span><b>{ d.base_attack.unwrap_or(0) }</b></div>
                                                    <div><span>{"DEF"}</span><b>{ d.base_defense.unwrap_or(0) }</b></div>
                                                    <div><span>{"SPA"}</span><b>{ d.base_sp_attack.unwrap_or(0) }</b></div>
                                                    <div><span>{"SPD"}</span><b>{ d.base_sp_defense.unwrap_or(0) }</b></div>
                                                    <div><span>{"VIT"}</span><b>{ d.base_speed.unwrap_or(0) }</b></div>
                                                </div>
                                            </div>
                                        </div>
                                    }
                                } else { html!{ <p>{"Chargement des détails..."}</p> } }
                            }
                        </div>
                    </div>
                </div>
            }
        } else { html!{} } }
        </>
    }
}
