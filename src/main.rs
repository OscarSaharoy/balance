use leptos::prelude::*;
use leptos::web_sys;
use leptos_use::signal_debounced;
use itertools::Itertools;

mod csv_parse;
use csv_parse::{Food, Nutrient, get_foods, lookup_foods, sum_nutrients, recommend_foods};

async fn get_data() -> Result<(Vec<Nutrient>, Vec<Food>)> {
    let res = reqwasm::http::Request::get("/assets/cofid.csv")
        .send().await?;
    let text = res.text().await?;
    let (nutrients, foods) = get_foods(text);
    Ok((nutrients, foods))
}

fn get_response(
    foods: String, data: Option<Result<(Vec<Nutrient>, Vec<Food>)>>
) -> String {
    if foods.len() == 0 {
        return "".to_string();
    }
    if let None = data {
        return "Searching...".to_string();
    }
    let (nutrients, foods) = data.unwrap().unwrap();
    let found_foods = lookup_foods(
        &foods,
        "Ackee, Amla, Apples".to_string()
    );
    let nutrients_sum = sum_nutrients(
        &nutrients,
        &found_foods
    );
    let recommended_foods = recommend_foods(
        &nutrients,
        &foods,
        &nutrients_sum
    );
    recommended_foods
        .iter()
        .map(|f| f.name.to_string())
        .intersperse(", ".to_string())
        .collect::<String>()
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
    let foods_debounced: Signal<String> = signal_debounced(foods, 1000.0);
    let data = LocalResource::new(move || get_data());

    view! {
        <input
            on:input:target=move |e| set_foods.set(e.target().value())
            value=foods
            placeholder="eg. Bread, Brazil Nuts, Strawberry Milkshake"
            style="font-size: 1rem;"
        />
        <p> {move || get_response(foods_debounced.get(), data.get().as_deref().cloned())} </p>
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
