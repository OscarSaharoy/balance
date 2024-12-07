use leptos::prelude::*;
use leptos::web_sys;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct Food {
    food_name: String,
}

async fn get_data() -> Result<String> {
    let res = reqwasm::http::Request::get("/assets/cofid.csv").send().await?;
    let data = res.text().await?;
    parse_data(data.clone());
    Ok(data)
}

fn parse_data(data: String) -> Result<()> {
    let mut rdr = csv::Reader::from_reader(data.as_bytes());
    leptos::logging::log!("ok");
    leptos::logging::log!("{data}");
    for result in rdr.deserialize() {
        let record: Food = result?;
        leptos::logging::log!("{record:?}");
    }
    leptos::logging::log!("9");
    Ok(())
}

fn get_response(foods: String) -> String {
    "test".to_string() + &foods
}

#[component]
fn Space<'a>(height: &'a str) -> impl IntoView + use<'a> {
    view! {
        <div style:margin-top=height />
    }
}

#[component]
fn Intro() -> impl IntoView {
    view! {
        <h1> "balance ⚖️ " </h1>
        <p> "What have you eaten today?" </p>
    }
}

#[component]
fn Foods() -> impl IntoView {
    let (foods, set_foods) = signal("".to_string());
    let data = LocalResource::new(move || get_data());

    //leptos::logging::log!("{data}");

    view! {
        <input
            on:input:target=move |e| set_foods.set(e.target().value()) 
            value=foods
            placeholder="eg. Bread, Brazil Nuts, Strawberry Milkshake"
            style="font-size: 1rem;"
        />
        <p> {move || get_response(foods.get())} </p>
    }
}


#[component]
fn App() -> impl IntoView {
    view! {
        <div id="leptos-root" style="min-width: 100vw; min-height: 100vh; padding: 2rem; display: grid; place-items: center;">
            <main style="width: 100%; max-width: 40rem; display: grid; gap: 1rem;">
                <Intro />
                <Foods />
            </main>
        </div>
    }
}

fn main() -> () {
    console_error_panic_hook::set_once();
    leptos::logging::log!("balance ⚖️ ");

    leptos::mount::mount_to_body(App);
    web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("loading-placeholder"))
        .map(|loading_placeholder| loading_placeholder.remove());
}
