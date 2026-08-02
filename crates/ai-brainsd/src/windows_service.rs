#![cfg(windows)]
#![allow(clippy::disallowed_methods)]

use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use windows_service::{
    Result as WsResult, define_windows_service,
    service::{
        ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, ServiceExitCode,
        ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
    service_manager::{ServiceManager, ServiceManagerAccess},
};

const SERVICE_NAME: &str = "AI-Brains-Daemon";
const SERVICE_DISPLAY_NAME: &str = "AI-Brains Daemon";
const SERVICE_DESCRIPTION: &str = "Local-first AI coding memory vault — captures conversation history without tool logs or hidden thinking.";
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

/// SCM service-specific exit codes (non-zero = failure visible to `sc` / Services MMC).
const SERVICE_EXIT_STARTUP_FAILED: u32 = 1;
const SERVICE_EXIT_RUNTIME_FAILED: u32 = 2;
const SERVICE_EXIT_THREAD_PANIC: u32 = 3;

/// How long the service control thread waits for fatal startup (vault + optional HTTP)
/// before treating the start as failed without ever reporting Running.
const STARTUP_READY_TIMEOUT: Duration = Duration::from_secs(120);

pub fn run_service() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let result = service_dispatcher::start(SERVICE_NAME, ffi_service_main);
    result.map_err(|e| format!("Failed to start service dispatcher: {e}"))?;
    Ok(())
}

define_windows_service!(ffi_service_main, my_service_main);

fn my_service_main(_arguments: Vec<OsString>) {
    if let Err(e) = run_service_inner() {
        tracing::error!("Service runtime failed: {}", e);
    }
}

fn run_service_inner() -> WsResult<()> {
    let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();

    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            ServiceControl::Stop => {
                let _ = shutdown_tx.send(());
                ServiceControlHandlerResult::NoError
            }
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::StartPending,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: STARTUP_READY_TIMEOUT,
        process_id: None,
    })?;

    // Fatal startup (vault open, optional HTTP) runs on the daemon thread and
    // signals this channel *before* we report Running to SCM (CR1-P2-01).
    // If HTTP is enabled and bind/token fails, we never mark Running and stop
    // with ServiceSpecific(STARTUP_FAILED).
    let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();

    let daemon_thread = thread::spawn(move || run_daemon_runtime(shutdown_rx, ready_tx));

    let startup_result = match ready_rx.recv_timeout(STARTUP_READY_TIMEOUT) {
        Ok(r) => r,
        Err(mpsc::RecvTimeoutError::Timeout) => Err(format!(
            "Daemon startup did not become ready within {}s",
            STARTUP_READY_TIMEOUT.as_secs()
        )),
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            Err("Daemon thread exited before signaling startup ready".to_string())
        }
    };

    match startup_result {
        Ok(()) => {
            status_handle.set_service_status(ServiceStatus {
                service_type: SERVICE_TYPE,
                current_state: ServiceState::Running,
                controls_accepted: ServiceControlAccept::STOP,
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            })?;

            let exit_code = match daemon_thread.join() {
                Ok(Ok(())) => ServiceExitCode::Win32(0),
                Ok(Err(e)) => {
                    tracing::error!("Daemon runtime failed after Running: {e}");
                    ServiceExitCode::ServiceSpecific(SERVICE_EXIT_RUNTIME_FAILED)
                }
                Err(_) => {
                    tracing::error!("Daemon thread panicked after Running");
                    ServiceExitCode::ServiceSpecific(SERVICE_EXIT_THREAD_PANIC)
                }
            };

            status_handle.set_service_status(ServiceStatus {
                service_type: SERVICE_TYPE,
                current_state: ServiceState::Stopped,
                controls_accepted: ServiceControlAccept::empty(),
                exit_code,
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            })?;

            Ok(())
        }
        Err(e) => {
            tracing::error!("Service startup failed (SCM will not see Running): {e}");
            // Drain the daemon thread (it should have already returned after
            // signaling the startup error).
            match daemon_thread.join() {
                Ok(Ok(())) => {}
                Ok(Err(runtime_err)) => {
                    tracing::error!("Daemon thread error after failed startup: {runtime_err}");
                }
                Err(_) => {
                    tracing::error!("Daemon thread panicked during failed startup");
                }
            }

            status_handle.set_service_status(ServiceStatus {
                service_type: SERVICE_TYPE,
                current_state: ServiceState::Stopped,
                controls_accepted: ServiceControlAccept::empty(),
                exit_code: ServiceExitCode::ServiceSpecific(SERVICE_EXIT_STARTUP_FAILED),
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            })?;

            // Startup never reached Running — SCM-visible failure is the
            // ServiceSpecific exit code above. Log path already covered.
            Ok(())
        }
    }
}

fn run_daemon_runtime(
    shutdown_rx: mpsc::Receiver<()>,
    ready_tx: mpsc::Sender<Result<(), String>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            let msg = format!("tokio runtime build failed: {e}");
            let _ = ready_tx.send(Err(msg.clone()));
            return Err(msg.into());
        }
    };

    // Propagate daemon errors (including HTTP hard-fail) — do not swallow.
    rt.block_on(run_daemon_async(shutdown_rx, ready_tx))
}

/// Fatal setup (env, vault, writer, optional HTTP) then long-running pipe loop.
///
/// Signals `ready_tx` with `Ok(())` only after fatal startup succeeds so the
/// service control thread can report `Running`. On any fatal startup error
/// (including HTTP enable failure), signals `Err` and returns without ever
/// starting the pipe accept loop.
async fn run_daemon_async(
    shutdown_rx: mpsc::Receiver<()>,
    ready_tx: mpsc::Sender<Result<(), String>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match run_daemon_startup().await {
        Ok(started) => {
            if ready_tx.send(Ok(())).is_err() {
                return Err("Service control channel closed during startup ready".into());
            }
            run_daemon_pipe_loop(started, shutdown_rx).await
        }
        Err(e) => {
            let msg = e.to_string();
            tracing::error!("Service fatal startup failed: {msg}");
            let _ = ready_tx.send(Err(msg));
            Err(e)
        }
    }
}

/// State produced by fatal service startup, consumed by the pipe loop.
struct ServiceDaemonStarted {
    writer: crate::DaemonWriter,
    services: crate::services::GovernedServices,
    ipc_shutdown_tx: tokio::sync::broadcast::Sender<()>,
}

/// Vault open, writer, optional HTTP. HTTP hard-fails when enabled (R1-01 / CR1-P2-01).
async fn run_daemon_startup()
-> Result<ServiceDaemonStarted, Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();

    let program_data =
        std::env::var("ProgramData").unwrap_or_else(|_| "C:\\ProgramData".to_string());
    let sidecar_env = PathBuf::from(&program_data)
        .join("AI-Brains")
        .join("daemon.env");
    if sidecar_env.exists() {
        let _ = dotenvy::from_path_override(&sidecar_env);
    }

    if std::env::var("AI_BRAINS_VAULT_PATH").is_err()
        && let Some(mut global_env) = dirs::home_dir()
    {
        global_env.push(".ai-brains");
        global_env.push(".env");
        if global_env.exists() {
            dotenvy::from_path_override(global_env).ok();
        }
    }

    let mut spool_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    spool_dir.push(".ai-brains");
    spool_dir.push("spool");

    let vault_path = std::env::var("AI_BRAINS_VAULT_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push(".ai-brains");
            path.push("vault.db");
            path
        });

    let vault_key_str = std::env::var("AI_BRAINS_VAULT_KEY").unwrap_or_else(|_| {
        "x'0000000000000000000000000000000000000000000000000000000000000000'".to_string()
    });

    let key = ai_brains_crypto::SqlCipherKey::from_raw(vault_key_str);
    let conn = ai_brains_store::connection::VaultConnection::open(vault_path, &key)?;
    conn.migrate()?;

    let event_store =
        std::sync::Arc::new(ai_brains_store::event_store::SqliteEventStore::new(conn));
    let writer = crate::DaemonWriter::start(spool_dir, event_store.clone()).await?;
    let services = crate::services::GovernedServices::new(event_store.clone());

    let (ipc_shutdown_tx, _ipc_shutdown_rx) = tokio::sync::broadcast::channel(1);

    // Optional loopback HTTP (T161) — third caller of handle_daemon_request.
    // T195 F10: service host refuses HTTP unless AI_BRAINS_HTTP_SERVICE is truthy
    // (same 1/true/yes set as AI_BRAINS_HTTP — F33). Gate lives here only — not
    // inside shared maybe_start_http (interactive main.rs path unchanged).
    // When opted in, hard-fail service start on bind/token errors so operators
    // cannot assume a listening API that never bound (R1-01 / CR1-P2-01).
    // LocalSystem residual (R-HTTP-SYS): token lands under SYSTEM profile — not
    // for interactive desktop clients; prefer interactive `ai-brainsd --http`.
    let service_args: Vec<String> = std::env::args().collect();
    let http_enabled = crate::http_adapter::http_enabled_from_env_and_args(&service_args);
    let service_opt_in = crate::http_adapter::service_http_opt_in_from_env();
    if crate::http_adapter::service_should_start_http(http_enabled, service_opt_in) {
        tracing::warn!(
            "HTTP enabled under Windows service (LocalSystem) with AI_BRAINS_HTTP_SERVICE opt-in: \
             bearer token is stored under the SYSTEM profile (%USERPROFILE%\\.ai-brains\\http.token \
             for SYSTEM) with owner-only ACL and is NOT readable by interactive Session 1 \
             CLI/desktop clients. Prefer interactive `ai-brainsd --http` (or `ai-brains daemon start` \
             with AI_BRAINS_HTTP=1) for local clients. Shared multi-session token is out of scope."
        );
        crate::http_adapter::maybe_start_http(
            &service_args,
            writer.clone(),
            services.clone(),
            &ipc_shutdown_tx,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to start HTTP API in service (hard-fail): {e}");
            e
        })?;
    } else if http_enabled {
        tracing::warn!(
            "Service HTTP refused: AI_BRAINS_HTTP (or --http) would enable HTTP, but \
             AI_BRAINS_HTTP_SERVICE is not truthy (1/true/yes). Skipping HTTP; named-pipe IPC \
             continues. Set AI_BRAINS_HTTP_SERVICE=1 to opt in (token under SYSTEM profile is \
             not for Session 1 desktop clients — residual R-HTTP-SYS)."
        );
    }

    Ok(ServiceDaemonStarted {
        writer,
        services,
        ipc_shutdown_tx,
    })
}

async fn run_daemon_pipe_loop(
    started: ServiceDaemonStarted,
    shutdown_rx: mpsc::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ServiceDaemonStarted {
        writer,
        services,
        ipc_shutdown_tx,
    } = started;

    // Must match ledgerful's IpcClient (track 0064: aibrains-sync → ledgerful-bridge).
    let pipe_name = r"\\.\pipe\ledgerful-bridge";

    // T184 F-1: fail closed — refuse default DACL if SDDL build fails.
    let pipe_sa = Box::new(
        crate::pipe_security::build_pipe_security_attributes().map_err(|e| {
            format!("Failed to build pipe security descriptor (refusing default DACL): {e}")
        })?,
    );

    let writer_clone = writer.clone();
    let services_clone = services.clone();
    let ipc_shutdown_tx_clone = ipc_shutdown_tx.clone();
    let pipe_name_owned = pipe_name.to_string();

    let sa_ptr_usize: usize = pipe_sa.as_ref() as *const _ as *const std::ffi::c_void as usize;
    std::mem::forget(pipe_sa);

    let server_handle = tokio::spawn(async move {
        use tokio::net::windows::named_pipe::ServerOptions;

        let mut first_instance = true;
        loop {
            let mut opts = ServerOptions::new();
            opts.first_pipe_instance(first_instance);
            let sa_ptr = sa_ptr_usize as *mut std::ffi::c_void;
            // Fail closed: no fallback to create() without custom SD (T184 F-1).
            let server_result =
                unsafe { opts.create_with_security_attributes_raw(&pipe_name_owned, sa_ptr) };

            let server = match server_result {
                Ok(s) => {
                    first_instance = false;
                    s
                }
                Err(e) => match crate::pipe_error::classify_pipe_error(&e) {
                    crate::pipe_error::PipeErrorKind::AccessDenied => {
                        tracing::error!(
                            "Access denied creating pipe {} — exiting service.",
                            pipe_name_owned
                        );
                        return;
                    }
                    crate::pipe_error::PipeErrorKind::PipeBusy => {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        continue;
                    }
                    crate::pipe_error::PipeErrorKind::Other => {
                        tracing::error!(
                            "Failed to create pipe {}: {} — exiting service (no default-DACL fallback).",
                            pipe_name_owned,
                            e
                        );
                        return;
                    }
                },
            };

            match server.connect().await {
                Ok(()) => {
                    let writer_inner = writer_clone.clone();
                    let services_inner = services_clone.clone();
                    let shutdown_tx_inner = ipc_shutdown_tx_clone.clone();
                    let mut shutdown_rx_inner = ipc_shutdown_tx_clone.subscribe();
                    tokio::spawn(async move {
                        tokio::select! {
                            _ = handle_service_client(server, writer_inner, services_inner, shutdown_tx_inner) => {}
                            _ = shutdown_rx_inner.recv() => {
                                tracing::info!("Shutting down client connection...");
                            }
                        }
                    });
                }
                Err(e) => {
                    tracing::warn!("Failed to connect client: {}", e);
                }
            }
        }
    });

    let ipc_shutdown_tx_for_control = ipc_shutdown_tx.clone();
    thread::spawn(move || {
        if shutdown_rx.recv().is_ok() {
            let _ = ipc_shutdown_tx_for_control.send(());
        }
    });

    let mut shutdown_rx_ipc = ipc_shutdown_tx.subscribe();
    tokio::select! {
        _ = shutdown_rx_ipc.recv() => {
            tracing::info!("Internal shutdown signal received in service.");
        }
    }

    server_handle.abort();
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    tracing::info!("Service daemon exited cleanly.");
    Ok(())
}

async fn handle_service_client<S>(
    mut server: S,
    writer: crate::DaemonWriter,
    services: crate::services::GovernedServices,
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buffer = Vec::new();
    let mut chunk = vec![0u8; 4096];

    loop {
        let n = server.read(&mut chunk).await?;
        if n == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..n]);

        if buffer.len() > 8 * 1024 * 1024 {
            return Err("Buffer exceeded 8 MiB limit. Disconnecting.".into());
        }

        while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            let line_with_nl = buffer.drain(..pos + 1).collect::<Vec<u8>>();
            let line = &line_with_nl[..line_with_nl.len() - 1];
            if line.is_empty() {
                continue;
            }

            match crate::dispatch::parse_live_request_line(line) {
                Ok(request) => {
                    let result: Result<(), Box<dyn std::error::Error + Send + Sync>> =
                        match crate::dispatch::handle_daemon_request(request, &writer, &services)
                            .await
                        {
                            Ok(outcome) => {
                                crate::dispatch::write_dispatch_result(
                                    &mut server,
                                    outcome,
                                    &shutdown_tx,
                                )
                                .await
                            }
                            Err(e) => Err(e),
                        };

                    if let Err(e) = result {
                        let api_err = ai_brains_contracts::response::ApiError::new(
                            "DAEMON_ERROR",
                            e.to_string(),
                        );
                        let resp = ai_brains_daemon_api::DaemonResponse::Error(api_err);
                        if let Ok(mut payload) = serde_json::to_vec(&resp) {
                            payload.push(b'\n');
                            let _ = server.write_all(&payload).await;
                        }
                    }
                }
                Err(api_err) => {
                    // AC3 fail-closed: always write Error — never silent drop (client hang).
                    tracing::warn!("Invalid live request: {}", api_err.message);
                    let resp = ai_brains_daemon_api::DaemonResponse::Error(api_err);
                    if let Ok(mut payload) = serde_json::to_vec(&resp) {
                        payload.push(b'\n');
                        let _ = server.write_all(&payload).await;
                    }
                }
            }
        }
    }

    server.flush().await?;
    Ok(())
}

pub fn install_service(exe_path: &str) -> WsResult<()> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: SERVICE_TYPE,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: PathBuf::from(format!("{} --service", exe_path)),
        launch_arguments: vec![],
        dependencies: vec![],
        account_name: None,
        account_password: None,
    };

    let service = service_manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;
    service.set_description(SERVICE_DESCRIPTION)?;
    Ok(())
}

pub fn uninstall_service() -> WsResult<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    if let Ok(status) = service.query_status()
        && status.current_state == ServiceState::Running
    {
        let _ = service.stop();
        thread::sleep(Duration::from_secs(2));
    }

    service.delete()?;
    Ok(())
}
