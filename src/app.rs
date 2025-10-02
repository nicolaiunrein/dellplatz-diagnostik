mod error_box;
mod form;
mod home;
mod user_home;

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

pub(crate) type ServerFnResult<T> = Result<T, ServerFnError>;

const PAGE_TITLE: &str = "Willkommen in der Dellplatz Praxis";

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/dellplatz-diag.css"/>

        // sets the document title
        <Title text=PAGE_TITLE/>

        // content for this welcome page
        <Router>
                <img class="logo" src="/logo.webp" />
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=path!("/") view=home::Page/>
                    <Route path=path!("/tests/:user") view=user_home::Page/>
                    <Route path=path!("/tests/:user/:test") view=form::Page/>
                </Routes>
            </main>
        </Router>
    }
}
