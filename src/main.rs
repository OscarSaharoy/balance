use leptos::prelude::*;
use leptos::web_sys;

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
fn NutrientRow(
    nutrient: Nutrient,
    food: Food,
) -> impl IntoView {
    let percentage = 100. * food.nutrients[&nutrient.name] / nutrient.recommended_intake;
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
            { food.nutrients[&nutrient.name] }{ nutrient.units.clone() }
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
fn FoodModal(
    food: Food,
    nutrients: Vec<Nutrient>,
    open: bool,
    mut close: impl FnMut() -> () + 'static,
) -> impl IntoView {
    if open {
        view! {
            <div
                style="position: fixed; display: grid; place-items: center; top: 0; left: 0; right: 0; bottom: 0; width: 100vw; min-height: 100vh; background: #0004; z-index: 2; padding: 2rem; overflow: scroll;"
            >
                <div
                    style="display: grid; padding: 2rem; background: var(--bg); border: 1px solid var(--fg); border-radius: 2rem; width: 100%; max-width: 38rem; gap: 0.25rem;"
                >
                    <div style="display: flex;">
                        <h1 style="flex-grow: 1;">
                            { food.emoji.clone() }" "{ food.display_name.clone() }
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
                    <h3><em> { food.name.clone() } </em></h3>
                    <p style="margin: 1rem 0">
                        "Here is the nutritional composition for 100 grams of "{ food.display_name.clone() }:
                    </p>

                    <div
                        style="display: grid; grid-template-columns: 1fr max-content max-content; column-gap: 0.5rem; align-items: center;"
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
                                <NutrientRow nutrient={n.clone()} food=food.clone() />
                            })
                            .collect::<Vec<_>>()
                        }
                    </div>
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
                    <FoodModal
                        food={food}
                        nutrients={nutrients}
                        open={modal_open.get()}
                        close={move || set_modal_open.set(false)}
                    />
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
                { move || {
                    if search.read().len() == 0 {
                        return vec![view!{<p></p>}.into_any()];
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
                    let (highest_nutrient, _) =
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
                                            on_remove={None::<fn() -> ()>}
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
fn SelectedFoods(
    selected_foods: ReadSignal<Vec<Food>>,
    set_selected_foods: WriteSignal<Vec<Food>>,
    data: LocalResource<Result<(Vec<Nutrient>, Vec<Food>)>>,
) -> impl IntoView {
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
        <div id="leptos-root" style="min-width: 100vw; min-height: 100vh; padding: 2rem; display: grid; place-items: center; grid-template-rows: auto max-content;">
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
