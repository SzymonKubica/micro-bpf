use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/demo-website-1.css"/>

        // sets the document title
        <Title text="µBPF Admin Tools"/>

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
    view! {
        <h1>"µBPF Admin Tools"</h1>
        <DeployForm/>
        <ExecuteForm/>
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

    view! { <h1>"Not Found"</h1> }
}

#[component]
fn ExecuteForm() -> impl IntoView {
    let (slot, set_slot) = create_signal(0);
    let (response, set_response) = create_signal("Loading".to_string());


    let send_execution_request = create_action(|input: &usize|{
        let storage_slot = input.to_owned();
        async move {
            let response = execute(storage_slot).await;
            response.unwrap()
        }
    });


    view! {
        <p>"Execute Form"</p>
        <p>"Enter the SUIT storage slot from where the program should be loaded: "</p>
        <input
            type="text"
            on:input=move |ev| {
                set_slot(event_target_value(&ev).parse::<i32>().unwrap());
            }

            prop:value=slot
        />

        <button on:click=move |_| {
            let _ = send_execution_request.dispatch(slot.get() as usize);
            set_response(send_execution_request.value().get().unwrap());
        }>
            "Send deploy request"
        </button>
        <p>"Response:"</p>
        <p>{response}</p>
    }
}

#[component]
fn DeployForm() -> impl IntoView {
    let (name, set_name) = create_signal("file_name.c".to_string());
    // Directory with all of the ebpf sources
    let (directory, set_directory) = create_signal("bpf/".to_string());
    let (slot, set_slot) = create_signal(0);

    let send_deploy_request = create_action(|input: &(String, usize)|{
        let (source_file, storage_slot) = input.to_owned();
        async move {
            let _ = deploy(source_file, storage_slot).await;
        }
    });

    view! {
        <p>"Deploy Form"</p>
        <p>"Enter the directory where the file is located: "</p>
        <input
            type="text"
            on:input=move |ev| {
                set_directory(event_target_value(&ev));
            }

            // the `prop:` syntax lets you update a DOM property,
            // rather than an attribute.
            prop:value=directory
        />
        <p>"Enter the file to be deployed: "</p>
        <input
            type="text"
            on:input=move |ev| {
                set_name(event_target_value(&ev));
            }

            // the `prop:` syntax lets you update a DOM property,
            // rather than an attribute.
            prop:value=name
        />
        <p>"Enter the target SUIT storage slot: "</p>
        <input
            type="text"
            on:input=move |ev| {
                set_slot(event_target_value(&ev).parse::<i32>().unwrap());
            }

            prop:value=slot
        />

        <button on:click=move |_| {
            send_deploy_request.dispatch((format!("{}/{}", directory.get(), name.get()), slot.get() as usize));
        }>
            Send deploy request
        </button>
    }
}


#[server(DeployRequest, "/deploy")]
pub async fn deploy(source_file: String, storage_slot: usize) -> Result<(), ServerFnError> {
    use mibpf_common::*;
    use mibpf_tools::*;
    let environment: Environment = load_env();

    let deploy_response = deploy(
     &source_file,
     &environment.out_dir,
     TargetVM::Rbpf,
     BinaryFileLayout::RawObjectFile,
     &environment.coap_root_dir,
     storage_slot,
     &environment.riot_instance_net_if,
     &environment.riot_instance_ip,
     &environment.host_net_if,
     &environment.host_ip,
     &environment.board_name,
     Some(&environment.mibpf_root_dir),
     vec![],
     HelperAccessVerification::PreFlight,
     HelperAccessListSource::ExecuteRequest,
     true
    ).await;

    Ok(deploy_response.unwrap())
}

#[server(ExecuteRequest, "/execute")]
pub async fn execute(storage_slot: usize) -> Result<String, ServerFnError> {
    use mibpf_common::*;
    use mibpf_tools::*;
    let environment: Environment = load_env();

    let execution_response = execute(
        &environment.riot_instance_ip,
        TargetVM::Rbpf,
        BinaryFileLayout::RawObjectFile,
        storage_slot,
        &environment.host_net_if,
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
