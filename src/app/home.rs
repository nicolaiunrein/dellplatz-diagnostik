use crate::types::*;
use leptos::prelude::*;
use uuid::Uuid;

use super::ServerFnResult;

#[server]
async fn create_user() -> ServerFnResult<User> {
    let user = crate::db::Db::get()
        .create_user()
        .await
        .map_err(ServerFnError::new)?;
    Ok(user)
}

#[component]
pub(crate) fn Page() -> impl IntoView {
    let create_user_action = Action::new(|_| async move { create_user().await });
    view! {
        <button type="button" class="btn" on:click=move |_| {
            create_user_action.dispatch(());
        }>
        Generate User
        </button>
        <div>
        {move || create_user_action.value().get().map(|res| match res {
            Ok(user) => user.id.to_string().into_any(),
            Err(err) => view!{<pre class="error">{move || err.to_string()}</pre>}.into_any()
        })}
        </div>

    }
}
