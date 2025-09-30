use leptos_router::hooks::use_params_map;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::ServerFnResult;
use crate::types::*;
use leptos::{
    leptos_dom::logging::{console_error, console_log},
    prelude::*,
};
use serde::{Deserialize, Serialize};

#[server]
async fn get_questions(user: Uuid) -> ServerFnResult<Vec<Question>> {
    let questions = crate::db::Db::get()
        .get_questions()
        .await
        .map_err(ServerFnError::new)?;
    Ok(questions)
}

#[server]
async fn submit(user_id: Uuid, data: Data) -> ServerFnResult<()> {
    Ok(())
}

#[component]
pub(crate) fn Page() -> impl IntoView {
    let params = use_params_map();
    let resource = Resource::new(
        move || params.read().get("user").and_then(|id| id.parse().ok()),
        |id| async move {
            if let Some(id) = id {
                get_questions(id).await
            } else {
                Err(ServerFnError::new("invalid user ID"))
            }
        },
    );

    view! {
        <h2>"Fragebogen AQ"</h2>
        <form on:submit=|ev| {
            ev.prevent_default();
            ev.stop_propagation();
            match Data::from_event(&ev) {
                Ok(data) => console_log(&format!("{data:?}")),
                Err(err) => {
                    console_error(&format!("{err:?}"));
                }
            }
        }>
            <input type="hidden" name="user" value=Uuid::new_v4().to_string() />
            <ErrorBoundary fallback=|errors| {
                view! {
                    <pre class="error">
                        <p>"Errors: "</p>
                        <ul>
                            {move || {
                                errors
                                    .get()
                                    .into_iter()
                                    .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                                    .collect::<Vec<_>>()
                            }}
                        </ul>
                    </pre>
                }
            }>
                <Suspense fallback=move || {
                    view! { <p>"Loading..."</p> }
                }>
                    {Suspend::new(async move {
                        resource
                            .await
                            .map(|q: Vec<Question>| {
                                q.into_iter()
                                    .map(|q| view! { <QuestionElement question=q /> })
                                    .collect_view()
                            })
                    })}
                </Suspense>

            </ErrorBoundary>
            <button class="btn">Speichern</button>
        </form>
    }
}

#[component]
fn QuestionElement(question: Question) -> impl IntoView {
    let Question {
        prompt,
        id,
        options,
    } = question;

    let options = options
        .into_iter()
        .map(|o| {
            view! { <option value=o.value>{o.label}</option> }
        })
        .collect_view();

    view! {
        <label>
            <span>{prompt}</span>
            <select required name=format!("q[{id}]")>
                <option value=""></option>
                {options}
            </select>
        </label>
    }
}
