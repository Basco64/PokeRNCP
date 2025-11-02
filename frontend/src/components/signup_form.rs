use gloo_net::http::Request;
use serde::Serialize;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub on_logged_in: Callback<()>,
}

#[derive(Serialize)]
struct SignupBody {
    email: String,
    password: String,
}

#[function_component]
pub fn SignUpForm(props: &Props) -> Html {
    let email = use_state(|| String::new());
    let password = use_state(|| String::new());
    let error = use_state(|| None as Option<String>);

    let on_submit = {
        let email = email.clone();
        let password = password.clone();
        let on_logged_in = props.on_logged_in.clone();
        let error = error.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let body = SignupBody {
                email: (*email).clone(),
                password: (*password).clone(),
            };
            let on_logged_in = on_logged_in.clone();
            let error = error.clone();
            spawn_local(async move {
                match Request::post("/api/auth/signup")
                    .credentials(web_sys::RequestCredentials::Include)
                    .json(&body)
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(resp) if resp.status() == 201 || resp.status() == 200 => {
                        on_logged_in.emit(())
                    }
                    Ok(resp) => error.set(Some(format!("Échec inscription ({}).", resp.status()))),
                    Err(err) => error.set(Some(format!("Erreur réseau: {}", err))),
                }
            });
        })
    };

    html! {
        <form onsubmit={on_submit} class="form auth-form">
            <div class="field">
                <label>{"Email"}</label>
                <input type="email" value={(*email).clone()} oninput={{ let email = email.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<web_sys::HtmlInputElement>() { email.set(t.value()); } }) }} />
            </div>
            <div class="field">
                <label>{"Mot de passe"}</label>
                <input type="password" value={(*password).clone()} oninput={{ let password = password.clone(); Callback::from(move |e: InputEvent| { if let Some(t) = e.target_dyn_into::<web_sys::HtmlInputElement>() { password.set(t.value()); } }) }} />
            </div>
            if let Some(err) = &*error { <p class="error">{err}</p> }
            <button class="inscriptionbutton" type="submit">{"Créer un compte"}</button>
        </form>
    }
}
