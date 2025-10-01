use leptos_router::hooks::use_params_map;

use super::ServerFnResult;
use crate::types::*;
use leptos::{
    leptos_dom::logging::{console_error, console_log},
    prelude::*,
    task::spawn_local,
};

#[server]
async fn get_questions(user: String) -> ServerFnResult<Vec<Question>> {
    crate::db::Db::get()
        .get_questions()
        .await
        .map_err(ServerFnError::new)
}

#[server]
async fn submit(data: Data) -> ServerFnResult<()> {
    crate::db::Db::get()
        .submit_test(data)
        .await
        .map_err(ServerFnError::new)
}

#[server]
async fn eval_test(user_id: String) -> ServerFnResult<Vec<TestResultRecord>> {
    crate::db::Db::get()
        .evaluate_test(user_id)
        .await
        .map_err(ServerFnError::new)
}

#[component]
pub(crate) fn Page() -> impl IntoView {
    let params = use_params_map();
    let user_id = Signal::derive(move || params.read().get("user").and_then(|id| id.parse().ok()));
    let resource = Resource::new(
        move || user_id(),
        |id| async move {
            if let Some(id) = id {
                get_questions(id).await
            } else {
                Err(ServerFnError::new("invalid user ID"))
            }
        },
    );

    let submit_action = Action::new(move |data: &Data| {
        let data = data.clone();
        async move { submit(data).await }
    });

    view! {
        <h2>"Fragebogen AQ"</h2>
        <form on:submit=move|ev| {
            ev.prevent_default();
            ev.stop_propagation();
            match Data::from_event(&ev) {
                Ok(data) => {
                    submit_action.dispatch(data);
                }
                Err(err) => {
                    console_error(&format!("{err:?}"));
                }
            }
        }>
            <input type="hidden" name="user[id]" value=user_id() />
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
            // <button class="btn" on:click=move |_| {
            //     if let Some(id) = user_id.get_untracked() {
            //         spawn_local(async move {
            //     let res = eval_test(id).await;
            //             console_log(&format!("{:?}", res));
            //         });
            //     }
            // }>Eval</button>
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
        .enumerate()
        .map(|(index, o)| {
            view! { <option value=index>{o.label}</option> }
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
