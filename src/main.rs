use leptos::prelude::*;

#[component]
fn Space(height: &str) -> impl IntoView {
    view! {
        <span style:height=height />
    }
}

#[component]
fn Intro() -> impl IntoView {
    view! {
        <h1> balance </h1>
        <space height="1rem" />
        <p> What have you eaten today? </p>
    }
}

#[component]
fn App() -> impl IntoView {
    view! {
        <div style="min-width: 100vw; min-height: 100vh; padding: 2rem; display: grid; place-items: center;">
            <main style="width: 100%; max-width: 50rem;">
                <Intro />
            </main>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App)
}
