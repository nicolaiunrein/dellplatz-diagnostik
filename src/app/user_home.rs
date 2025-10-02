use crate::app::error_box::ErrorBox;
use crate::app::ServerFnResult;
use crate::types::*;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[server]
async fn get_assigned_tests(user: String) -> ServerFnResult<Vec<Test>> {
    crate::db::Db::get()
        .get_assigned_tests(user)
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
                get_assigned_tests(id).await
            } else {
                Err(ServerFnError::new("User ID needed"))
            }
        },
    );

    view! {
        Welcome...
        {user_id}
        <ErrorBox>
            <div class="user-test-tabs">
                {move || {
                    resource
                        .get()
                        .transpose()
                        .map(|list| {
                            list.into_iter()
                                .flatten()
                                .filter_map(move |Test { id, name }| {
                                    let user = user_id()?;
                                    let href = format!("/tests/{user}/{id}");
                                    Some(view! { <a href=href>{name}</a> })
                                })
                                .collect_view()
                        })
                }}
            </div>
        </ErrorBox>
    }
}
