use leptos::prelude::*;
use leptos::web_sys;
use leptos::ev::{Targeted, MouseEvent};
use crate::web_sys::HtmlButtonElement;

mod nutrition;
use nutrition::{Food, Nutrient, get_foods, lookup_food, sum_nutrients, recommend_foods, get_highest_and_lowest_nutrients};

fn get_url(path: String) -> String {
    let window = web_sys::window().expect("Missing Window");
    let href = window.location().href().expect("Missing location.href");
    format!("{href}{path}")
}

async fn get_data() -> Result<(Vec<Nutrient>, Vec<Food>)> {
    let res = reqwasm::http::Request::get(
        &get_url("assets/cofid.csv".to_string())
    ).send().await?;
    let text = res.text().await?;
    let (nutrients, foods) = get_foods(text);
    Ok((nutrients, foods))
}

#[component]
fn Match(
    food: Food,
    nutrients: Vec<Nutrient>,
    mut on_remove: Option<impl FnMut(Targeted<MouseEvent, HtmlButtonElement>) -> () + 'static>,
) -> impl IntoView {
    let name = food.display_name.clone();
    let show_x = on_remove.is_some();
    let (highest_nutrient, _) = get_highest_and_lowest_nutrients(nutrients, food.nutrients.clone());
    view! {
        <div
            style="padding: 0.5rem 0.6rem 0.5rem 1rem; border: 1px solid var(--fg); border-radius: 2rem; display: grid; grid-template-columns: max-content auto max-content max-content; gap: 0.25rem; align-items: center;"
            style:background=if show_x { "var(--bg2)" } else { "unset" }
        >
            <p style="transform: scale(1.2); margin-right: 0.32rem;"> { food.emoji } </p>
            <p style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap;"> { food.display_name } </p>
            <p style="font-weight: bold; font-size: 0.75rem;"> { highest_nutrient.display_name } </p>
            <button
                on:click:target={move |e| if let Some(ref mut f) = on_remove { f(e); }}
                style="padding: 0;"
            >
                <img
                    src={get_url("/assets/x.svg".to_string())}
                    style="height: 1.5rem;"
                    style:display=move || if show_x { "unset" } else { "none" }
                    class="invert" 
                />
            </button>
        </div>
    }
}

#[component]
fn FoodSearch(
    set_selected_foods: WriteSignal<Vec<Food>>,
    data: LocalResource<Result<(Vec<Nutrient>, Vec<Food>)>>,
) -> impl IntoView {
    let (search, set_search) = signal("".to_string());
    view! {
        <div class="search-outer">
            <div class="search-container">
                <input
                    on:input:target=move |e| set_search.set(e.target().value())
                    prop:value={search}
                    placeholder="+ Search foods"
                    style="font-size: 1rem;"
                />
                { move || {
                    if search.read().len() == 0 {
                        return vec![view!{<p></p>}.into_any()];
                    }
                    match data.read().as_deref() {
                        Some(Ok((nutrients,foods))) =>
                            lookup_food(foods, search.get())
                                .iter()
                                .map(|f| {
                                    let food = (*f).clone();
                                    view! {
                                        <button
                                            on:click:target=move |e| {
                                                let food = food.clone();
                                                set_selected_foods.update(move |sf| sf.push(food));
                                                set_search.set("".to_string());
                                            }
                                        >
                                            { f.display_name.to_string() }
                                        </button>
                                    }.into_any()
                                })
                                .collect::<Vec<_>>(),
                        _ =>
                            vec![view!{<p></p>}.into_any()],
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn FoodReport(
    selected_foods: ReadSignal<Vec<Food>>,
    data: LocalResource<Result<(Vec<Nutrient>, Vec<Food>)>>,
) -> impl IntoView {
    view! {
        { move || {
            if selected_foods.read().len() == 0 {
                return view!{<p></p>}.into_any();
            }
            match data.read().as_deref() {
                Some(Ok((nutrients,foods))) => {
                    let nutrients_sum = sum_nutrients(nutrients.clone(), selected_foods.get());                   
                    let recommended_foods = recommend_foods(
                        nutrients.clone(),
                        &foods,
                        nutrients_sum.clone(),
                    );
                    let (highest_nutrient, lowest_nutrient) =
                        get_highest_and_lowest_nutrients(
                            nutrients.clone(), nutrients_sum.clone(),
                        );
                    view! {
                        <p style="white-space: pre-wrap;">
                            { format!(
                                "Sounds delicious, you have had a lot of {} 😋 Try eating some of these foods to balance your diet:",
                                highest_nutrient.display_name
                            ) }
                        </p>
                        {
                            recommended_foods
                                .iter()
                                .map(|f| {
                                    let food = (*f).clone();
                                    view! {
                                        <Match
                                            food={food}
                                            nutrients={nutrients.clone()}
                                            on_remove={None::<fn(Targeted<MouseEvent, HtmlButtonElement>) -> ()>}
                                        />
                                    }.into_any()
                                })
                                .collect::<Vec<_>>()
                        }
                    }.into_any()
                },
                _ =>
                    view!{<p></p>}.into_any(),
            }
        }}
    }
}

#[component]
fn Foods() -> impl IntoView {
    let (selected_foods, set_selected_foods) = signal(Vec::<Food>::new());
    let data = LocalResource::new(move || get_data());

    view! {
        { move || {
            let nutrients = match data.read().as_deref() {
                Some(Ok((nutrients,_))) => nutrients.clone(),
                _ => Vec::<Nutrient>::new(),
            };
            selected_foods
                .read()
                .iter()
                .enumerate()
                .map(|(i,f)| {
                    let food = f.clone();
                    view! {
                        <Match
                            food={food}
                            nutrients={nutrients.clone()}
                            on_remove={Some(move |e|
                                set_selected_foods.update(|sf| {
                                    (*sf).remove(i);
                                })
                            )}
                        />
                    }
                })
                .collect::<Vec<_>>()
        }}
        <FoodSearch
            set_selected_foods={set_selected_foods}
            data={data}
        />
        <FoodReport
            selected_foods={selected_foods}
            data={data}
        />
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
fn App() -> impl IntoView {
    view! {
        <div id="leptos-root" style="min-width: 100vw; min-height: 100vh; padding: 2rem; display: grid; place-items: center; grid-template-rows: auto max-content;">
            <main style="width: 100%; max-width: 40rem; display: grid; gap: 0.8rem;">
                <Intro />
                <Foods />
            </main>
            <footer style="display: grid; grid-auto-flow: column; align-items: center; gap: .7rem; justify-self: start; align-self: end;">
                <a href="https://github.com/OscarSaharoy/balance" style="display: grid;" target="_blank" rel="noopener noreferrer">
                    <img src={get_url("/assets/github.svg".to_string())} style="height: 1.5rem;" class="invert" />
                </a>
                <div style="line-height: 1.27; font-size: 0.75rem;">
                    <a
                        target="_blank"
                        style="color: var(--fg);"
                        href="https://www.nhs.uk/conditions/vitamins-and-minerals/"
                    >
                        "🔗 NHS guide to vitamins and minerals"
                    </a>
                    <p> "Made by Oscar Saharoy. Don't rely on this for good nutrition advice!" </p>
                </div>
            </footer>
        </div>
    }
}

fn remove_loading_placeholder() -> () {
    web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("loading-placeholder"))
        .map(|loading_placeholder| loading_placeholder.remove());
}

fn main() -> () {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
    remove_loading_placeholder();
    leptos::logging::log!("balance ⚖️ ");
}
