use leptos::prelude::*;

#[component]
pub(crate) fn ErrorBox(children: Children) -> impl IntoView {
    view! {

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
    {children()}
            </ErrorBoundary>
        }
}
