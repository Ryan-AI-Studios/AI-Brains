use ai_brains_contracts::bridge::{BridgeDirection, BridgeRecord};
use ai_brains_crypto::SqlCipherKey;
use ai_brains_daemon_api::{DaemonRequest, DaemonResponse};
use ai_brains_store::connection::VaultConnection;
use ai_brains_store::event_store::SqliteEventStore;
use ai_brainsd::DaemonWriter;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(windows)]
use tokio::net::windows::named_pipe::ServerOptions;

#[cfg(windows)]
use ai_brainsd::instance_guard::{InstanceDecision, ProbeOutcome};
#[cfg(windows)]
use ai_brainsd::pipe_error::{PipeErrorKind, classify_pipe_error};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

// Must match ledgerful's IpcClient (track 0064: aibrains-sync → ledgerful-bridge).
const PIPE_NAME: &str = r"\\.\pipe\ledgerful-bridge";

#[allow(clippy::disallowed_methods)]
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();
    let run_as_service = args.iter().any(|a| a == "--service");

    if run_as_service {
        #[cfg(windows)]
        {
            return ai_brainsd::windows_service::run_service();
        }
        #[cfg(not(windows))]
        {
            eprintln!("--service flag is only supported on Windows.");
            std::process::exit(1);
        }
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async_main())
}

async fn async_main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
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

    let key = SqlCipherKey::from_raw(vault_key_str);
    let conn = VaultConnection::open(vault_path, &key)?;
    conn.migrate()?;

    let event_store = Arc::new(SqliteEventStore::new(conn));
    let writer = DaemonWriter::start(spool_dir, event_store.clone()).await?;

    let (shutdown_tx, _shutdown_rx) = tokio::sync::broadcast::channel(1);

    #[cfg(windows)]
    {
        match check_existing_instance(PIPE_NAME).await {
            InstanceDecision::AlreadyRunning => {
                println!(
                    "Daemon already running on {pipe_name}. Exiting. \
                     Use `ai-brains daemon status` to check.",
                    pipe_name = PIPE_NAME
                );
                return Ok(());
            }
            InstanceDecision::ProbeFailed => {
                println!(
                    "Probe of existing pipe failed — proceeding to create {pipe_name}.",
                    pipe_name = PIPE_NAME
                );
            }
            InstanceDecision::Proceed => {}
        }

        let pipe_sa = match ai_brainsd::pipe_security::build_pipe_security_attributes() {
            Ok(sa) => Some(sa),
            Err(e) => {
                tracing::warn!(
                    "Failed to build pipe security descriptor (continuing with default): {}",
                    e
                );
                None
            }
        };

        println!("AI-Brains Daemon started. Listening on {}", PIPE_NAME);

        let writer_clone = writer.clone();
        let shutdown_tx_clone = shutdown_tx.clone();

        let sa_ptr_usize: usize = pipe_sa
            .as_ref()
            .map(|sa| sa as *const _ as *const std::ffi::c_void as usize)
            .unwrap_or(0);

        tokio::spawn(async move {
            let mut first_instance = true;
            let mut use_sd = sa_ptr_usize != 0;
            loop {
                let server_result = if use_sd {
                    let mut opts = ServerOptions::new();
                    opts.first_pipe_instance(first_instance);
                    let sa_ptr = sa_ptr_usize as *mut std::ffi::c_void;
                    let res =
                        unsafe { opts.create_with_security_attributes_raw(PIPE_NAME, sa_ptr) };
                    if res.is_err() {
                        tracing::warn!(
                            "Pipe creation with custom SD failed, falling back to default SD"
                        );
                        use_sd = false;
                        let mut opts2 = ServerOptions::new();
                        opts2.first_pipe_instance(first_instance);
                        opts2.create(PIPE_NAME)
                    } else {
                        res
                    }
                } else {
                    let mut opts = ServerOptions::new();
                    opts.first_pipe_instance(first_instance);
                    opts.create(PIPE_NAME)
                };

                let server = match server_result {
                    Ok(s) => {
                        first_instance = false;
                        s
                    }
                    Err(e) => match classify_pipe_error(&e) {
                        PipeErrorKind::AccessDenied => {
                            eprintln!(
                                "Access denied creating pipe {pipe_name} — \
                                     another instance owns it or the security descriptor \
                                     denies access. Exiting.\n\
                                     Hint: use `ai-brains daemon stop` to stop the \
                                     existing instance, or check the service is not \
                                     already running via `sc query AI-Brains-Daemon`.",
                                pipe_name = PIPE_NAME
                            );
                            std::process::exit(1);
                        }
                        PipeErrorKind::PipeBusy => {
                            tracing::debug!("Pipe busy, retrying in 1s: {}", e);
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            continue;
                        }
                        PipeErrorKind::Other => {
                            eprintln!(
                                "Failed to create named pipe instance on {pipe_name}: {err}",
                                pipe_name = PIPE_NAME,
                                err = e
                            );
                            std::process::exit(1);
                        }
                    },
                };

                tokio::select! {
                    res = server.connect() => {
                        if let Err(e) = res {
                            tracing::warn!("Failed to connect client: {}", e);
                            continue;
                        }

                        let writer_inner = writer_clone.clone();
                        let mut shutdown_rx_inner = shutdown_tx_clone.subscribe();
                        let shutdown_tx_inner = shutdown_tx_clone.clone();
                        tokio::spawn(async move {
                            tokio::select! {
                                _ = handle_client(server, writer_inner, shutdown_tx_inner) => {}
                                _ = shutdown_rx_inner.recv() => {
                                    tracing::info!("Shutting down client connection...");
                                }
                            }
                        });
                    }
                    _ = tokio::signal::ctrl_c() => {
                        println!("\nShutdown signal received. Closing daemon...");
                        let _ = shutdown_tx_clone.send(());
                        break;
                    }
                }
            }
        });
    }

    #[cfg(not(windows))]
    {
        let socket_path = "/tmp/ledgerful-bridge.sock";
        let _ = std::fs::remove_file(socket_path);

        let listener = tokio::net::UnixListener::bind(socket_path)?;
        println!(
            "AI-Brains Daemon started. Listening on Unix socket: {}",
            socket_path
        );

        let writer_clone = writer.clone();
        let shutdown_tx_clone = shutdown_tx.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    res = listener.accept() => {
                        match res {
                            Ok((stream, _addr)) => {
                                let writer_inner = writer_clone.clone();
                                let shutdown_tx_for_client = shutdown_tx_clone.clone();
                                let mut shutdown_rx_inner = shutdown_tx_clone.subscribe();
                                    tokio::spawn(async move {
                                    tokio::select! {
                                        _ = handle_client(stream, writer_inner, shutdown_tx_for_client) => {}
                                        _ = shutdown_rx_inner.recv() => {
                                            tracing::info!("Shutting down client connection...");
                                        }
                                    }
                                });
                            }
                            Err(e) => {
                                eprintln!("Failed to accept UDS connection: {}", e);
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            }
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {
                        println!("\nShutdown signal received. Closing daemon...");
                        let _ = shutdown_tx_clone.send(());
                        break;
                    }
                }
            }
        });
    }

    // Wait for shutdown signal (Ctrl-C or internal Shutdown request)
    let mut shutdown_rx = shutdown_tx.subscribe();
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("\nCtrl-C received. Closing daemon...");
        }
        _ = shutdown_rx.recv() => {
            println!("Internal shutdown signal received. Closing daemon...");
        }
    }

    #[cfg(not(windows))]
    {
        let socket_path = "/tmp/ledgerful-bridge.sock";
        let _ = std::fs::remove_file(socket_path);
    }

    // Give some time for background tasks to finish
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    println!("Daemon exited cleanly.");
    Ok(())
}

#[cfg(windows)]
async fn check_existing_instance(pipe_name: &str) -> InstanceDecision {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::windows::named_pipe::ClientOptions;
    use tokio::time::timeout;

    let probe_timeout = std::time::Duration::from_secs(2);

    let mut stream = match ClientOptions::new().open(pipe_name) {
        Ok(s) => s,
        Err(_) => return InstanceDecision::from_probe(ProbeOutcome::ConnectFailed),
    };

    let ping = DaemonRequest::Ping;
    let json = match serde_json::to_vec(&ping) {
        Ok(j) => j,
        Err(_) => return InstanceDecision::from_probe(ProbeOutcome::NoResponse),
    };

    let mut payload = json;
    payload.push(b'\n');

    if timeout(probe_timeout, stream.write_all(&payload))
        .await
        .is_err()
    {
        return InstanceDecision::from_probe(ProbeOutcome::NoResponse);
    }

    let mut buf = vec![0u8; 1024];
    match timeout(probe_timeout, stream.read(&mut buf)).await {
        Ok(Ok(n)) if n > 0 => {
            if let Ok(resp) = serde_json::from_slice::<DaemonResponse>(&buf[..n])
                && matches!(resp, DaemonResponse::Pong)
            {
                return InstanceDecision::from_probe(ProbeOutcome::Pong);
            }
            InstanceDecision::from_probe(ProbeOutcome::NoResponse)
        }
        _ => InstanceDecision::from_probe(ProbeOutcome::NoResponse),
    }
}

#[allow(clippy::disallowed_methods)]
async fn handle_client<S>(
    mut server: S,
    writer: DaemonWriter,
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
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

        // Process newline-delimited JSON records
        while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            let line_with_nl = buffer.drain(..pos + 1).collect::<Vec<u8>>();
            let line = &line_with_nl[..line_with_nl.len() - 1];
            if line.is_empty() {
                continue;
            }

            let request = match serde_json::from_slice::<DaemonRequest>(line) {
                Ok(request) => Some(request),
                Err(_) => {
                    // Try parsing as raw BridgeRecord directly
                    match serde_json::from_slice::<ai_brains_contracts::bridge::BridgeRecord>(line)
                    {
                        Ok(record) => Some(DaemonRequest::Sync(record)),
                        Err(e) => {
                            eprintln!(
                                "Failed to parse as either DaemonRequest or BridgeRecord: {}",
                                e
                            );
                            None
                        }
                    }
                }
            };

            if let Some(request) = request {
                let result: Result<(), BoxError> = match request {
                    DaemonRequest::Ping => {
                        let mut payload = serde_json::to_vec(&DaemonResponse::Pong)?;
                        payload.push(b'\n');
                        server.write_all(&payload).await?;
                        Ok(())
                    }
                    DaemonRequest::Shutdown => {
                        tracing::info!("Shutdown request received via IPC.");
                        let _ = shutdown_tx.send(());
                        Ok(())
                    }
                    DaemonRequest::Ingest(req) => match writer.ingest(req).await {
                        Ok(resp) => {
                            let mut payload = serde_json::to_vec(&DaemonResponse::Ingest(resp))?;
                            payload.push(b'\n');
                            server.write_all(&payload).await?;
                            Ok(())
                        }
                        Err(e) => Err(e),
                    },
                    DaemonRequest::Sync(record) => {
                        if record.record_kind == "query" {
                            let payload = record.payload_value();
                            let query_text =
                                payload.get("text").and_then(|v| v.as_str()).unwrap_or("");

                            // T112: pass IDs through as Option so the daemon
                            // defaults to unscoped search.
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

                                        let resp_record = BridgeRecord {
                                            bridge_version: "0.3".to_string(),
                                            direction: BridgeDirection::Outbound,
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
                                    let mut payload = serde_json::to_vec(&DaemonResponse::Sync {
                                        success: true,
                                    })?;
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
                    let resp = DaemonResponse::Error(api_err);
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
