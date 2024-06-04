use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use leptos::logging::log;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/demo-website-1.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
        <ExecuteTrigger />
    }
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}

#[component]
fn ExecuteTrigger() -> impl IntoView {
    let (count, set_count) = create_signal(0);

    let async_data = create_resource(count, |_ | async move {
        execute_request().await.unwrap()
    });

    view! {
        <button
            on:click=move |_| {
                // on stable, this is set_count.set(3);
                set_count.set(count.get() + 1);
            }
        >
            "Response: "
            // on stable, this is move || count.get();
            {move || count.get()}
        </button>
        <p>
            "Response: " {move || match async_data.get() {
                Some(s) => s,
                None => "Loading...".to_string(),
            }}
        </p>
    }
}

#[server(ExecuteRequest, "/execute")]
pub async fn execute_request() -> Result<String, ServerFnError> {
    use mibpf_common::*;
    use mibpf_tools::*;

    let execution_response = execute(
        "fe80::a8e8:48ff:fee0:523c",
        TargetVM::Rbpf,
        BinaryFileLayout::RawObjectFile,
        0,
        "enp82s0u2u1u2",
        ExecutionModel::ShortLived,
        HelperAccessVerification::PreFlight,
        HelperAccessListSource::ExecuteRequest,
        &vec![],
        false,
        false,
        true,
    )
    .await;
    Ok(execution_response.unwrap())
}
