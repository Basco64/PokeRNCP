use yew::prelude::*;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;

#[function_component]
pub fn Pokedex() -> Html {
    let data = use_state(|| Ok::<String, String>("Chargement...".into()));

    {
        let data = data.clone();
        use_effect_with((), move |_| {
            let data = data.clone();
            spawn_local(async move {
                let res = Request::get("/api/pokemons")
                    .credentials(web_sys::RequestCredentials::Include)
                    .send().await;
                match res {
                    Ok(r) => match r.text().await {
                        Ok(txt) => data.set(Ok(txt)),
                        Err(e) => data.set(Err(format!("Erreur lecture: {}", e))),
                    },
                    Err(e) => data.set(Err(format!("Erreur requête: {}", e))),
                }
            });
            || {}
        });
    }

    html! {
        <section>
            <h2>{"Pokédex"}</h2>
            {
                match &*data {
                    Ok(txt) => html!{ <pre class="pokedex-json">{ txt }</pre> },
                    Err(err) => html!{ <p class="error">{ err }</p> },
                }
            }
        </section>
    }
}
