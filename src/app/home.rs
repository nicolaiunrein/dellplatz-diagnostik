use std::collections::BTreeSet;

use crate::app::error_box::ErrorBox;
use crate::types::*;
use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;

use super::ServerFnResult;

#[server]
async fn create_user(tests: BTreeSet<String>) -> ServerFnResult<User> {
    let user = crate::db::Db::get()
        .create_user(tests)
        .await
        .map_err(ServerFnError::new)?;
    Ok(user)
}

#[server]
async fn get_available_tests() -> ServerFnResult<Vec<Test>> {
    let user = crate::db::Db::get()
        .get_available_tests()
        .await
        .map_err(ServerFnError::new)?;
    Ok(user)
}

#[derive(serde::Deserialize, Clone, Debug)]
struct FormData {
    tests: BTreeSet<String>,
}

#[component]
pub(crate) fn Page() -> impl IntoView {
    let create_user_action = Action::new(|tests: &BTreeSet<String>| {
        let tests = tests.clone();
        async move { create_user(tests).await }
    });
    let available_tests = Resource::new(|| (), |_| async move { get_available_tests().await });
    let make_test_opt = |(index, Test { id, name }): (usize, Test)| {
        view! {
            <label>
                <input type="checkbox" name=format!("tests[{index}]") value=id />
                <span>{name}</span>
            </label>
        }
    };

    let user_links = |User { id, retrieval_id }| {
        let patienten_href = format!("/tests/{}", id);
        let abruf_href = format!("/tests/abruf/{}", retrieval_id);
        view! {
            <a href=patienten_href>Patienten Link</a>
            <a href=abruf_href>Abruf Link</a>
        }
    };

    view! {
        <ErrorBox>
            <form on:submit=move |ev| {
                ev.prevent_default();
                ev.stop_propagation();
                let data = FormData::from_event(&ev);
                create_user_action.dispatch(data.unwrap().tests);
            }>
                <Suspense fallback=move || {
                    "Loading..."
                }>
                    {Suspend::new(async move {
                        available_tests
                            .await
                            .map(|opt| {
                                opt.into_iter().enumerate().map(make_test_opt).collect_view()
                            })
                    })}
                </Suspense>
                <button class="btn">Generieren</button>

            </form>
            <div>{move || create_user_action.value().get().map(|res| res.map(user_links))}</div>

        </ErrorBox>
    }
}
