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
        wait_hint: Duration::from_secs(10),
        process_id: None,
    })?;

    let daemon_thread = thread::spawn(move || {
        if let Err(e) = run_daemon_runtime(shutdown_rx) {
            tracing::error!("Daemon thread error: {}", e);
        }
    });

    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    let _ = daemon_thread.join();

    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

fn run_daemon_runtime(
    shutdown_rx: mpsc::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(async {
        if let Err(e) = run_daemon_async(shutdown_rx).await {
            tracing::error!("run_daemon_async error: {}", e);
        }
    });

    Ok(())
}

async fn run_daemon_async(
    shutdown_rx: mpsc::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    let (ipc_shutdown_tx, _ipc_shutdown_rx) = tokio::sync::broadcast::channel(1);

    // Must match ledgerful's IpcClient (track 0064: aibrains-sync → ledgerful-bridge).
    let pipe_name = r"\\.\pipe\ledgerful-bridge";

    let pipe_sa = crate::pipe_security::build_pipe_security_attributes().ok();

    let writer_clone = writer.clone();
    let ipc_shutdown_tx_clone = ipc_shutdown_tx.clone();
    let pipe_name_owned = pipe_name.to_string();

    let sa_ptr_usize: usize = pipe_sa
        .as_ref()
        .map(|sa| sa as *const _ as *const std::ffi::c_void as usize)
        .unwrap_or(0);

    let server_handle = tokio::spawn(async move {
        use tokio::net::windows::named_pipe::ServerOptions;

        let mut first_instance = true;
        let mut use_sd = sa_ptr_usize != 0;
        loop {
            let server_result = if use_sd {
                let mut opts = ServerOptions::new();
                opts.first_pipe_instance(first_instance);
                let sa_ptr = sa_ptr_usize as *mut std::ffi::c_void;
                let res =
                    unsafe { opts.create_with_security_attributes_raw(&pipe_name_owned, sa_ptr) };
                if res.is_err() {
                    use_sd = false;
                    let mut opts2 = ServerOptions::new();
                    opts2.first_pipe_instance(first_instance);
                    opts2.create(&pipe_name_owned)
                } else {
                    res
                }
            } else {
                let mut opts = ServerOptions::new();
                opts.first_pipe_instance(first_instance);
                opts.create(&pipe_name_owned)
            };

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
                            "Failed to create pipe {}: {} — exiting service.",
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
                    let shutdown_tx_inner = ipc_shutdown_tx_clone.clone();
                    let mut shutdown_rx_inner = ipc_shutdown_tx_clone.subscribe();
                    tokio::spawn(async move {
                        tokio::select! {
                            _ = handle_service_client(server, writer_inner, shutdown_tx_inner) => {}
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

            let request = match serde_json::from_slice::<ai_brains_daemon_api::DaemonRequest>(line)
            {
                Ok(request) => Some(request),
                Err(_) => {
                    match serde_json::from_slice::<ai_brains_contracts::bridge::BridgeRecord>(line)
                    {
                        Ok(record) => Some(ai_brains_daemon_api::DaemonRequest::Sync(record)),
                        Err(e) => {
                            tracing::warn!(
                                "Failed to parse as DaemonRequest or BridgeRecord: {}",
                                e
                            );
                            None
                        }
                    }
                }
            };

            if let Some(request) = request {
                let result: Result<(), Box<dyn std::error::Error + Send + Sync>> = match request {
                    ai_brains_daemon_api::DaemonRequest::Ping => {
                        let mut payload =
                            serde_json::to_vec(&ai_brains_daemon_api::DaemonResponse::Pong)?;
                        payload.push(b'\n');
                        server.write_all(&payload).await?;
                        Ok(())
                    }
                    ai_brains_daemon_api::DaemonRequest::Shutdown => {
                        tracing::info!("Shutdown request received via IPC.");
                        let _ = shutdown_tx.send(());
                        Ok(())
                    }
                    ai_brains_daemon_api::DaemonRequest::Ingest(req) => {
                        match writer.ingest(req).await {
                            Ok(resp) => {
                                let mut payload = serde_json::to_vec(
                                    &ai_brains_daemon_api::DaemonResponse::Ingest(resp),
                                )?;
                                payload.push(b'\n');
                                server.write_all(&payload).await?;
                                Ok(())
                            }
                            Err(e) => Err(e),
                        }
                    }
                    ai_brains_daemon_api::DaemonRequest::Sync(record) => {
                        if record.record_kind == "query" {
                            let payload = record.payload_value();
                            let query_text =
                                payload.get("text").and_then(|v| v.as_str()).unwrap_or("");

                            use std::str::FromStr;
                            let project_id =
                                ai_brains_core::ids::ProjectId::from_str(&record.project_id).ok();
                            let session_id = record
                                .session_id
                                .as_ref()
                                .and_then(|s| ai_brains_core::ids::SessionId::from_str(s).ok());

                            match writer
                                .query_memories(query_text, project_id, session_id)
                                .await
                            {
                                Ok(hits) => {
                                    let timestamp = chrono::Utc::now();
                                    for h in hits {
                                        let payload =
                                            ai_brains_contracts::bridge::BridgePayload::Insight {
                                                type_field: "Insight".to_string(),
                                                memory_id: h.memory_id,
                                                relevance: h.score.unwrap_or(1.0),
                                                content: h.content,
                                            };
                                        let resp_record =
                                            ai_brains_contracts::bridge::BridgeRecord {
                                                bridge_version: "0.3".to_string(),
                                                direction:
                                                    ai_brains_contracts::bridge::BridgeDirection::Outbound,
                                                timestamp,
                                                parent_hash: None,
                                                project_id: record.project_id.clone(),
                                                session_id: record.session_id.clone(),
                                                tx_id: None,
                                                record_kind: "insight".to_string(),
                                                payload,
                                                privacy: ai_brains_core::privacy::Privacy::LocalOnly,
                                            };
                                        let mut payload = serde_json::to_vec(&resp_record)?;
                                        payload.push(b'\n');
                                        server.write_all(&payload).await?;
                                    }
                                    server.write_all(b"\n").await?;
                                    Ok(())
                                }
                                Err(e) => Err(e),
                            }
                        } else {
                            match writer.sync(record).await {
                                Ok(_) => {
                                    let mut payload = serde_json::to_vec(
                                        &ai_brains_daemon_api::DaemonResponse::Sync {
                                            success: true,
                                        },
                                    )?;
                                    payload.push(b'\n');
                                    server.write_all(&payload).await?;
                                    Ok(())
                                }
                                Err(e) => Err(e),
                            }
                        }
                    }
                };

                if let Err(e) = result {
                    let api_err =
                        ai_brains_contracts::response::ApiError::new("DAEMON_ERROR", e.to_string());
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
