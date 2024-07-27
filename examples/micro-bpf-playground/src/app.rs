use std::{process::Command, str::FromStr, time::Duration};

use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::Deserialize;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/admin-tools-website.css"/>

        // sets the document title
        <Title text="µBPF Admin Tools"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=PlaygroundPage/>
                    <Route path="/*" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn PlaygroundPage() -> impl IntoView {
    view! {
        <h1>"µBPF Playground"</h1>
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
    let (target_vm, set_target_vm) = create_signal("rBPF".to_string());
    let (binary_layout, set_binary_layout) = create_signal("RawObjectFile".to_string());
    let (execution_model, set_execution_model) = create_signal("LongRunning".to_string());
    let (use_jit, set_use_jit) = create_signal(false);
    let (jit_compile, set_jit_compile) = create_signal(false);
    let (benchmark, set_benchmark) = create_signal(false);

    let send_execution_request = create_action(|input: &(String, String, usize, String, bool, bool, bool)| {
        let (target_vm, binary_layout, storage_slot, execution_model, use_jit, jit_compile, benchmark) = input.to_owned();
        async move {
            let response = execute(target_vm, binary_layout, storage_slot, execution_model, use_jit, jit_compile, benchmark).await;
            response.unwrap()
        }
    });

    view! {
        <p>"Execution request form"</p>
        <div>
            <input
                type="text"
                on:input=move |ev| {
                    set_slot(event_target_value(&ev).parse::<i32>().unwrap());
                }

                prop:value=slot
            />
            <text>"< SUIT storage slot"</text>
        </div>
        <div>
            <TargetVMSelector target_vm set_target_vm/>
            <text>"< Target VM"</text>
        </div>
        <div>
            <BinaryLayoutSelector binary_layout set_binary_layout/>
            <text>"< Binary format"</text>
        </div>
        <div>
            <ExecutionModelSelector execution_model set_execution_model/>
            <text>"< Binary format"</text>
        </div>
        <div>
            <input
                type="checkbox"
                on:input=move |_| { set_use_jit(!use_jit.get()) }

                prop:checked=use_jit
            />
            <text>"Use JIT"</text>
        </div>
        <div>
            <input
                type="checkbox"
                on:input=move |_| { set_jit_compile(!jit_compile.get()) }

                prop:checked=jit_compile
            />
            <text>"JIT Recompile"</text>
        </div>
        <div>
            <input
                type="checkbox"
                on:input=move |_| { set_benchmark(!benchmark.get()) }

                prop:checked=benchmark
            />
            <text>"Benchmark"</text>
        </div>

        <button on:click=move |_| {
            let _ = send_execution_request
                .dispatch((
                    target_vm.get(),
                    binary_layout.get(),
                    slot.get() as usize,
                    execution_model.get(),
                    use_jit.get(),
                    jit_compile.get(),
                    benchmark.get(),
                ));
            set_response(send_execution_request.value().get().unwrap());
        }>"Execute"</button>
        <p>"Response:"</p>
        <p>
            {move || match send_execution_request.value().get() {
                Some(v) => v,
                None => "Loading".to_string(),
            }}

        </p>
    }
}

#[component]
fn DeployForm() -> impl IntoView {
    let (name, set_name) = create_signal("display-update-thread.c".to_string());
    let (slot, set_slot) = create_signal(0);
    let (target_vm, set_target_vm) = create_signal("rBPF".to_string());
    let (binary_layout, set_binary_layout) = create_signal("RawObjectFile".to_string());


    let send_deploy_request = create_action(|input: &(String, String, String, usize)| {
        let (source_file, target_vm, binary_layout, storage_slot) = input.to_owned();
        async move {
            let _ = deploy(source_file, target_vm, binary_layout, storage_slot).await;
        }
    });

    view! {
        <p>"Please specify the program that you want to deploy."</p>
        <div>
            <input
                type="text"
                on:input=move |ev| {
                    set_name(event_target_value(&ev));
                }

                // the `prop:` syntax lets you update a DOM property,
                // rather than an attribute.
                prop:value=name
            />
            <text>"< File name (ensure that the .env file correctly specifies the sources directory)"</text>
        </div>
        <div>
            <input
                type="text"
                on:input=move |ev| {
                    set_slot(event_target_value(&ev).parse::<i32>().unwrap());
                }

                prop:value=slot
            />
            <text>"< SUIT storage slot"</text>
        </div>
        <div>
            <TargetVMSelector target_vm set_target_vm/>
            <text>"< Target VM implementation"</text>
        </div>
        <div>
            <BinaryLayoutSelector binary_layout set_binary_layout/>
            <text>"< Binary format"</text>
        </div>

        <button on:click=move |_| {
            send_deploy_request
                .dispatch((name.get(), target_vm.get(), binary_layout.get(), slot.get() as usize));
        }>"Deploy"</button>
    }
}

#[component]
pub fn TargetVMSelector(target_vm: ReadSignal<String>, set_target_vm: WriteSignal<String>) -> impl IntoView {
    view! {
        <select on:change=move |ev| {
            let new_value = event_target_value(&ev);
            set_target_vm(new_value);
        }>
            <SelectOption value=target_vm is="rBPF"/>
            <SelectOption value=target_vm is="FemtoContainer"/>
        </select>
    }
}


#[component]
pub fn BinaryLayoutSelector(binary_layout: ReadSignal<String>, set_binary_layout: WriteSignal<String>) -> impl IntoView {
    view! {
        <select on:change=move |ev| {
            let new_value = event_target_value(&ev);
            set_binary_layout(new_value);
        }>
            <SelectOption value=binary_layout is="OnlyTextSection"/>
            <SelectOption value=binary_layout is="FemtoContainersHeader"/>
            <SelectOption value=binary_layout is="ExtendedHeader"/>
            <SelectOption value=binary_layout is="RawObjectFile"/>
        </select>
    }
}

#[component]
pub fn ExecutionModelSelector(execution_model: ReadSignal<String>, set_execution_model: WriteSignal<String>) -> impl IntoView {
    view! {
        <select on:change=move |ev| {
            let new_value = event_target_value(&ev);
            set_execution_model(new_value);
        }>
            <SelectOption value=execution_model is="ShortLived"/>
            <SelectOption value=execution_model is="WithAccessToCoapPacket"/>
            <SelectOption value=execution_model is="LongRunning"/>
        </select>
    }
}

#[component]
pub fn SelectOption(is: &'static str, value: ReadSignal<String>) -> impl IntoView {
    view! {
        <option value=is selected=move || value() == is>
            {is}
        </option>
    }
}

#[server(DeployRequest, "/deploy")]
pub async fn deploy(source_file: String, target_vm: String, binary_layout: String, storage_slot: usize) -> Result<(), ServerFnError> {
    use micro_bpf_common::{BinaryFileLayout, TargetVM};
    use micro_bpf_common::*;
    use micro_bpf_tools::*;
    let environment: Environment = load_env();

    println!("Env: {:?}", environment);
    println!("Source file: {}", source_file);
    println!("Target VM: {}", target_vm);
    println!("Binary file layout: {}", binary_layout);
    println!("Storage slot: {}", storage_slot);
    let deploy_response = deploy(
        &format!("{}/{}", &environment.src_dir, source_file),
        &environment.out_dir,
        TargetVM::from_str(&target_vm).unwrap(),
        BinaryFileLayout::from_str(&binary_layout).unwrap(),
        &environment.coap_root_dir,
        storage_slot,
        &environment.riot_instance_net_if,
        &environment.riot_instance_ip,
        &environment.host_net_if,
        &environment.host_ip,
        &environment.board_name,
        Some(&environment.micro_bpf_root_dir),
        vec![],
        HelperAccessVerification::PreFlight,
        HelperAccessListSource::ExecuteRequest,
        true,
    )
    .await;

    Ok(deploy_response.unwrap())
}

#[server(ExecuteRequest, "/execute")]
pub async fn execute(target_vm: String, binary_layout: String, storage_slot: usize, execution_model: String, use_jit: bool, jit_compile: bool, benchmark: bool) -> Result<String, ServerFnError> {
    use micro_bpf_common::*;
    use micro_bpf_tools::*;
    let environment: Environment = load_env();

    println!("Env: {:?}", environment);
    println!("Target VM: {}", target_vm);
    println!("Binary file layout: {}", binary_layout);
    println!("Storage slot: {}", storage_slot);
    println!("Execution model: {}", execution_model);
    println!("Use JIT: {}", use_jit);
    println!("JIT recompile: {}", jit_compile);
    println!("Benchmark: {}", benchmark);

    let execution_response = execute(
        &environment.riot_instance_ip,
        TargetVM::from_str(&target_vm).unwrap(),
        BinaryFileLayout::from_str(&binary_layout).unwrap(),
        storage_slot,
        &environment.host_net_if,
        ExecutionModel::from_str(&execution_model).unwrap(),
        HelperAccessVerification::PreFlight,
        HelperAccessListSource::ExecuteRequest,
        &vec![],
        use_jit,
        jit_compile,
        benchmark
    )
    .await;
    Ok(execution_response.unwrap())
}

