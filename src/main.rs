use std::collections::HashMap;
use leptos::prelude::*;
use leptos::web_sys;

mod nutrition;
use nutrition::{Food, Nutrient, get_foods, lookup_food, sum_nutrients, recommend_foods, get_highest_and_lowest_nutrients, format_float};

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
fn NutrientRow(
    nutrient: Nutrient,
    nutrient_value: f32,
) -> impl IntoView {
    let percentage = 100. * nutrient_value / nutrient.recommended_intake;
    let color = if nutrient.recommended_intake > 0.1 && percentage >= 20. {
        "#0d0"
    } else {
        "unset"
    };
    view! {
        <tr
            style="grid-column: 1/4; border: none; border-bottom: 1px solid var(--fg); margin: 0.1rem 0; opacity: 0.7;"
        />
        <p style:color={color}> { nutrient.display_name.clone() } </p>
        <p style="text-align: right;">
            { format_float(nutrient_value) }{ nutrient.units.clone() }
        </p>
        {
            if nutrient.recommended_intake > 0.1 {
                view! {
                    <p style="text-align: right;">
                        { nutrient.recommended_intake }{ nutrient.units.clone() }
                        " | "
                        <span style:color={color} >
                            { format!( "{:.0}", percentage ) }"%"
                        </span>
                    </p>
                }.into_any()
            } else {
                view! {
                    <p style="text-align: right;"> - </p>
                }.into_any()
            }
        }
    }
}

#[component]
fn NutrientTable(
    nutrients: Vec<Nutrient>,
    nutrient_values: HashMap<String, f32>,
) -> impl IntoView {
    view! {
        <div
            style="display: grid; grid-template-columns: 1fr max-content max-content; column-gap: 0.5rem; align-items: center; font-size: min(1rem, calc((100vw - 5rem) / 22));"
        >
            <p style="font-weight: bold;">
                Nutrient
            </p>
            <p style="text-align: right; font-weight: bold;">
                Content
            </p>
            <p style="text-align: right; font-weight: bold;">
                RI
            </p>
            { nutrients
                .iter()
                .map(|n| view! {
                    <NutrientRow nutrient=n.clone() nutrient_value=nutrient_values[&n.name] />
                })
                .collect::<Vec<_>>()
            }
        </div>
    }
}

#[component]
fn Modal(
    title: String,
    open: bool,
    mut close: impl FnMut() -> () + 'static,
    children: Children,
) -> impl IntoView {
    if open {
        view! {
            <div
                style="position: fixed; display: grid; place-items: center; top: 0; left: 0; right: 0; bottom: 0; width: 100vw; min-height: 100vh; background: #0004; z-index: 2; padding: 2rem 1rem; overflow: scroll;"
            >
                <div
                    style="display: grid; padding: 1.5rem; background: var(--bg); border: 1px solid var(--fg); border-radius: 2rem; width: 100%; max-width: 38rem; gap: 0.25rem;"
                >
                    <div style="display: flex; gap: 0.75rem;">
                        <h1 style="flex-grow: 1;">
                            { title }
                        </h1>
                        <button
                            on:click:target=move |_| close()
                            style="padding: 0; align-self: start; margin-top: 0.8rem;"
                        >
                            <img
                                src={get_url("/assets/x.svg".to_string())}
                                style="height: 2rem; display: grid;"
                                class="invert" 
                            />
                        </button>
                    </div>
                    { children() }
                </div>
            </div>
        }.into_any()
    } else {
        view!{}.into_any()
    }
}

#[component]
fn Match(
    food: Food,
    nutrients: Vec<Nutrient>,
    mut on_remove: Option<impl FnMut() -> () + 'static>,
) -> impl IntoView {
    let (modal_open, set_modal_open) = signal(false);
    let show_x = on_remove.is_some();
    let (highest_nutrient, _) = get_highest_and_lowest_nutrients(nutrients.clone(), food.nutrients.clone());
    view! {
        <div
            style="padding: 0 0.6rem 0 1rem; border: 1px solid var(--fg); border-radius: 2rem; display: grid; grid-template-columns: max-content auto max-content max-content; gap: 0.25rem; align-items: center;"
            style:background=if show_x { "var(--bg2)" } else { "unset" }
        >
            <p style="transform: scale(1.2); margin-right: 0.32rem;">
                { food.emoji.clone() }
            </p>
            <button
                style="display: grid; align-items: center; grid-template-columns: auto max-content; justify-content: left; gap: 0.25rem; cursor: pointer; padding: 0.5rem 0;"
                class="hover-line"
                on:click:target=move |_| set_modal_open.set(true)
            >
                <p
                    style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: .9rem;"
                >
                    { food.display_name.clone() }
                </p>
                <img
                    src={get_url("/assets/info.svg".to_string())}
                    style="height: .9rem; display: grid;"
                    class="invert" 
                />
            </button>
            <p style="font-weight: bold; font-size: 0.75rem;">
                "📊 "{ highest_nutrient.display_name }
            </p>
            <button
                on:click:target={move |_| if let Some(ref mut f) = on_remove { f(); }}
                style="padding: 0;"
                style:display=move || if show_x { "unset" } else { "none" }
            >
                <img
                    src={get_url("/assets/x.svg".to_string())}
                    style="height: 1.5rem; display: grid;"
                    class="invert" 
                />
            </button>
            { move || {
                let food = food.clone();
                let nutrients = nutrients.clone();
                view!{
                    <Modal
                        title={ format!("{} {}", food.emoji.clone(), food.display_name.clone()) }
                        open={modal_open.get()}
                        close={move || set_modal_open.set(false)}
                    >
                        <h3><em> { food.name.clone() } </em></h3>
                        <p style="margin: 1rem 0">
                            "Here is the nutritional composition for 100 grams of "{ food.display_name.clone() }:
                        </p>
                        <NutrientTable nutrients={nutrients} nutrient_values={food.nutrients} />
                    </Modal>
                }
            } }
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
                <div class="search-options">
                    { move || {
                        if search.read().len() == 0 {
                            return vec![view!{}.into_any()];
                        }
                        match data.read().as_deref() {
                            Some(Ok((_, foods))) =>
                                lookup_food(foods, search.get())
                                    .iter()
                                    .map(|f| {
                                        let food = (*f).clone();
                                        view! {
                                            <button
                                                on:click:target=move |_| {
                                                    let food = food.clone();
                                                    set_selected_foods.update(move |sf| sf.push(food));
                                                    set_search.set("".to_string());
                                                }
                                                style="font-size: 0.9rem; white-space: pre;"
                                            >
                                                { f.emoji.clone() }"  "{ f.display_name.to_string() }
                                            </button>
                                        }.into_any()
                                    })
                                    .collect::<Vec<_>>(),
                            _ =>
                                vec![view!{}.into_any()],
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

fn get_tasty_message(selected_foods: Vec<Food>) -> String {
    let seed = selected_foods
        .iter()
        .fold(0, |a, f| a + f.name.len() * f.nutrients["water_g"] as usize);
    [
        "Sounds delicious",
        "Sounds delectable",
        "Sounds tasty",
        "Delicious and nutritious",
        "Those are some of my favourite foods",
    ][seed % 5].to_string()
}

#[component]
fn FoodReport(
    selected_foods: ReadSignal<Vec<Food>>,
    data: LocalResource<Result<(Vec<Nutrient>, Vec<Food>)>>,
) -> impl IntoView {
    let (modal_open, set_modal_open) = signal(false);
    view! {
        { move || {
            if selected_foods.read().len() == 0 {
                return view!{}.into_any();
            }
            match data.read().as_deref() {
                Some(Ok((nutrients,foods))) => {
                    let nutrients_sum = sum_nutrients(nutrients.clone(), selected_foods.get());                   
                    let recommended_foods = recommend_foods(
                        nutrients.clone(),
                        &foods,
                        nutrients_sum.clone(),
                    );
                    let (highest_nutrient, _) =
                        get_highest_and_lowest_nutrients(
                            nutrients.clone(), nutrients_sum.clone(),
                        );
                    let nutrients1 = nutrients.clone();
                    let nutrients_sum1 = nutrients_sum.clone();
                    view! {
                        <button
                            style="white-space: pre-wrap; margin: 0 -1rem -0.75rem -1rem; font-size: 1rem;"
                            on:click:target=move |_| set_modal_open.set(true)
                        >
                            <p> 
                                { get_tasty_message(selected_foods.get()) }
                                "! You have had a lot of "
                                { highest_nutrient.display_name }" 😋 "
                                <span style="text-decoration: underline;">
                                    Click here
                                </span>" to view your overall nutrient breakdown for today."
                            </p>
                        </button>
                        <p>
                            Try eating some of these foods to balance your diet:
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
                                            on_remove={None::<fn() -> ()>}
                                        />
                                    }.into_any()
                                })
                                .collect::<Vec<_>>()
                        }
                        <Modal
                            title="⚖️  Nutrition Breakdown".to_string()
                            open={modal_open.get()}
                            close={move || set_modal_open.set(false)}
                        >
                            <div style="display: grid; gap: 0.75rem;">
                                <p style="margin: 1rem 0"> 
                                    This shows the combined breakdown of the nutrients you 
                                    have eaten today, assuming you ate around 100 grams of
                                    each selected food. </p>
                                <NutrientTable nutrients={nutrients1} nutrient_values={nutrients_sum1} />
                            </div>
                        </Modal>
                    }.into_any()
                },
                _ =>
                    view!{}.into_any(),
            }
        }}
    }
}

#[component]
fn SelectedFoods(
    selected_foods: ReadSignal<Vec<Food>>,
    set_selected_foods: WriteSignal<Vec<Food>>,
    data: LocalResource<Result<(Vec<Nutrient>, Vec<Food>)>>,
) -> impl IntoView {
    let nutrients = move || match data.read().as_deref() {
        Some(Ok((nutrients,_))) => nutrients.clone(),
        _ => Vec::<Nutrient>::new(),
    };
    view! {
        { move || {
            selected_foods
                .read()
                .iter()
                .enumerate()
                .map(|(i,f)| {
                    let food = f.clone();
                    view! {
                        <Match
                            food={food}
                            nutrients={nutrients()}
                            on_remove={Some(move ||
                                set_selected_foods.update(|sf| {
                                    (*sf).remove(i);
                                })
                            )}
                        />
                    }
                })
                .collect::<Vec<_>>()
        }}
    }
}


#[component]
fn Foods() -> impl IntoView {
    let (selected_foods, set_selected_foods) = signal(Vec::<Food>::new());
    let data = LocalResource::new(move || get_data());

    view! {
        <SelectedFoods
            selected_foods={selected_foods}
            set_selected_foods={set_selected_foods}
            data={data}
        />
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
        <div id="leptos-root" style="min-width: 100vw; min-height: 100vh; padding: 2rem; display: grid; place-items: center; grid-template-rows: auto max-content; gap: 2rem;">
            <main style="width: 100%; max-width: 40rem; display: grid; gap: 0.8rem;">
                <Intro />
                <Foods />
            </main>
            <footer
                style=
                    "display: grid; grid-auto-flow: column; align-items: center; gap: .7rem; justify-self: start; align-self: end;"
                >
                <a
                    href="https://github.com/OscarSaharoy/balance"
                    style="display: grid; align-self: end; margin-bottom: 0.2rem;"
                    target="_blank"
                    rel="noopener noreferrer"
                >
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
