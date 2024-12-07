use leptos::prelude::*;

#[component]
fn Space<'a>(height: &'a str) -> impl IntoView + use<'a> {
    view! {
        <div style:margin-top=height />
    }
}

fn get_response(foods: String) -> String {
    "test".to_string() + &foods
}

#[component]
fn Intro() -> impl IntoView {
    view! {
        <h1> "balance ⚖️ " </h1>
        <p> What have you eaten today? </p>
    }
}

#[component]
fn Foods() -> impl IntoView {
    let (foods, set_foods) = signal("".to_string());

    view! {
        <input
            on:input:target=move |e| set_foods.set(e.target().value()) 
            value=foods
            placeholder="eg. Bread, Brazil Nuts, Milk"
            style="font-size: 1rem;"
        />
        <p> {move || get_response(foods.get())} </p>
    }
}


#[component]
fn App() -> impl IntoView {
    view! {
        <div style="min-width: 100vw; min-height: 100vh; padding: 2rem; display: grid; place-items: center;">
            <main style="width: 100%; max-width: 50rem; display: grid; gap: 1rem;">
                <Intro />
                <Foods />
            </main>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App)
}
