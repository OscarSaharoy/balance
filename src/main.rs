use leptos::prelude::*;
use leptos::web_sys;

mod nutrition;
use nutrition::{Food, Nutrient, get_foods, lookup_foods, sum_nutrients, recommend_foods, get_highest_and_lowest_nutrients};

fn get_url(path: String) -> String {
    let window = web_sys::window().expect("Missing Window");
    let href = window.location().href().expect("Missing location.href");
    format!("{href}{path}")
}

async fn get_data() -> Result<(Vec<Nutrient>, Vec<Food>)> {
    let res = reqwasm::http::Request::get(&get_url("assets/cofid.csv".to_string()))
        .send().await?;
    let text = res.text().await?;
    let (nutrients, foods) = get_foods(text);
    Ok((nutrients, foods))
}

fn get_response(
    input: String, data: Option<Result<(Vec<Nutrient>, Vec<Food>)>>
) -> String {
    if input.len() == 0 {
        return "".to_string();
    }
    if let None = data {
        return "Searching...".to_string();
    }
    let (nutrients, foods) = data.unwrap().unwrap();
    let found_foods = lookup_foods(&foods, input);
    let nutrients_sum = sum_nutrients(
        &nutrients,
        &found_foods
    );
    let (highest_nutrient, lowest_nutrient) =
        get_highest_and_lowest_nutrients(
            &nutrients, &nutrients_sum
        );
    let recommended_foods = recommend_foods(
        &nutrients,
        &foods,
        &nutrients_sum
    );
    let recommended = recommended_foods
        .iter()
        .map(|f| format!(
            "{} {} - high in {}",
            f.emoji.to_string(),
            f.display_name.to_string(),
            get_highest_and_lowest_nutrients(&nutrients, &f.nutrients).0.display_name)
        )
        .collect::<Vec<String>>();
    format!(
        "Sounds delicious, you have had a lot of {} 😋 Try eating some of these foods to balance your diet:\n\n{}\n{}\n{}",
        highest_nutrient.display_name, recommended[0], recommended[1], recommended[2]
    )
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
fn Match(text: String) -> impl IntoView {
    view! {
        <div style="padding: 0.5rem 0.6rem 0.5rem 1rem; background: var(--bg2); border: 1px solid var(--fg); border-radius: 2rem; display: grid; grid-template-columns: auto max-content; gap: 0.25rem;">
            <p style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap;"> { text } </p>
            <img src={get_url("/assets/x.svg".to_string())} style="height: 1.5rem;" class="invert" />
        </div>
    }
}

#[component]
fn FoodSearch(foods: ReadSignal<Vec<Food>>, set_foods: WriteSignal<Vec<Food>>) -> impl IntoView {
    let (search, set_search) = signal("".to_string());
    view! {
        <Match text="🥮 Cake".to_string() />
        <Match text="🥧 Pie".to_string() />
        <Match text="🥕 Vegatal adwadawda dawdawdawdaw dawdawdawdawda wdawdawdawdawdaw dawdawdawd".to_string() />
        <div class="search-outer">
            <div class="search-container">
                <input
                    on:input:target=move |e| set_search.set(e.target().value())
                    placeholder="eg. Bread, Brazil Nuts, Strawberry Milkshake"
                    style="font-size: 1rem;"
                />
                <button> test1 </button>
                <button> test1 </button>
                <button> test1 </button>
                <button> test1 </button>
            </div>
        </div>
    }
}

#[component]
fn Foods() -> impl IntoView {
    let (foods, set_foods) = signal(Vec::<Food>::new());
    let data = LocalResource::new(move || get_data());

    view! {
        <FoodSearch foods={foods} set_foods={set_foods} />
        <p style="white-space: pre-wrap;">
            { move || get_response("nut".to_string(), data.get().as_deref().cloned()) }
        </p>
    }
}


#[component]
fn App() -> impl IntoView {
    view! {
        <div id="leptos-root" style="min-width: 100vw; min-height: 100vh; padding: 2rem; display: grid; place-items: center; grid-template-rows: auto max-content;">
            <main style="width: 100%; max-width: 40rem; display: grid; gap: 1rem;">
                <Intro />
                <Foods />
            </main>
            <footer style="display: grid; grid-auto-flow: column; align-items: center; gap: .7rem; justify-self: start; align-self: end;">
                <a href="https://github.com/OscarSaharoy/balance" style="display: grid;" target="_blank" rel="noopener noreferrer">
                    <img src={get_url("/assets/github.svg".to_string())} style="height: 1.5rem;" class="invert" />
                </a>
                <p style="font-size: 0.8rem;"> "Made by Oscar Saharoy. Don't rely on this for good nutrition advice!" </p>
            </footer>
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
