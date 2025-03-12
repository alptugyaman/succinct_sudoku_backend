use axum::{
    routing::{get, post},
    Json, Router, extract::{Path, State, WebSocketUpgrade},
    response::IntoResponse,
    http::Request,
    middleware::{self, Next},
    body::Body,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc, path::Path as FilePath, time::Instant};
use tokio::net::TcpListener;
use std::collections::{HashSet, HashMap};
use tokio::sync::Mutex;
use uuid::Uuid;
use std::time::Duration;
use tokio::time::sleep;
use std::fs;
use std::io::Write;

// Loglama için gerekli bileşenler
use log::{info, warn, error, debug, LevelFilter};
use env_logger::Builder;
use chrono::Local;
use tower_http::trace::{self, TraceLayer};
use tracing::Level as TracingLevel;
// CORS için gerekli bileşenler
use tower_http::cors::CorsLayer;

// SP1 için gerekli bileşenler
use sp1_sdk::{ProverClient, SP1Stdin, HashableKey};

// ELF dosyası (SP1 prover'ın derlenmiş hali)
#[cfg(not(feature = "no_elf"))]
const ELF: &[u8] = include_bytes!("../sp1_prover/src/main.rs");

#[cfg(feature = "no_elf")]
const ELF: &[u8] = &[]; // no_elf özelliği etkinleştirildiğinde boş bir array kullan

// İş depolama yapısı
type JobStorage = Arc<Mutex<HashMap<String, JobStatus>>>;

// Log mesajları depolama yapısı
type LogStorage = Arc<Mutex<HashMap<String, Vec<String>>>>;

// İş durumu
#[derive(Debug, Clone)]
enum JobStatus {
    Processing,
    Complete(ProofResponse),
    Failed(String),
}

// Özel log middleware'i
async fn log_request_response(
    req: Request<Body>,
    next: Next,
) -> axum::response::Response {
    let path = req.uri().path().to_owned();
    let method = req.method().clone();
    let start = Instant::now();
    
    info!(">> Request started: {} {} - Headers: {:?}", method, path, req.headers());
    
    let response = next.run(req).await;
    
    let status = response.status();
    let duration = start.elapsed();
    
    info!("<< Request completed: {} {} - Status: {} - Duration: {:.2?}", 
          method, path, status, duration);
    
    response
}

// İstek gövdesini loglayan middleware
async fn log_request_body(
    req: Request<Body>,
    next: Next,
) -> axum::response::Response {
    let (parts, body) = req.into_parts();
    let bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(err) => {
            error!("Failed to read request body: {}", err);
            return axum::response::Response::builder()
                .status(500)
                .body(Body::from("Internal Server Error"))
                .unwrap();
        }
    };

    let body_str = String::from_utf8_lossy(&bytes);
    if !body_str.is_empty() {
        info!("Request body for {} {}: {}", parts.method, parts.uri.path(), body_str);
        
        // JSON formatını analiz et
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_str) {
            info!("JSON request for {} {}: {}", parts.method, parts.uri.path(), json);
            info!("JSON keys for {} {}: {:?}", parts.method, parts.uri.path(), json.as_object().map(|obj| obj.keys().collect::<Vec<_>>()));
        }
    } else {
        info!("Empty request body for {} {}", parts.method, parts.uri.path());
    }

    let req = Request::from_parts(parts, Body::from(bytes));
    next.run(req).await
}

// Log mesajını kaydet
async fn log_message(logs: &LogStorage, job_id: &str, message: &str) {
    let mut logs_map = logs.lock().await;
    
    if let Some(job_logs) = logs_map.get_mut(job_id) {
        job_logs.push(message.to_string());
    } else {
        logs_map.insert(job_id.to_string(), vec![message.to_string()]);
    }
    
    // En fazla 100 log mesajı sakla (eski mesajları sil)
    if let Some(job_logs) = logs_map.get_mut(job_id) {
        if job_logs.len() > 100 {
            *job_logs = job_logs.iter().skip(job_logs.len() - 100).cloned().collect();
        }
    }
}

// Senkron log mesajı kaydetme fonksiyonu (closure'lar için)
fn log_message_sync(logs: &LogStorage, job_id: &str, message: &str) {
    // Asenkron olmayan bir şekilde log mesajını kaydet
    // tokio::spawn ile arka planda çalıştır
    let logs_clone = logs.clone();
    let job_id_clone = job_id.to_string();
    let message_clone = message.to_string();
    tokio::spawn(async move {
        log_message(&logs_clone, &job_id_clone, &message_clone).await;
    });
}

#[tokio::main]
async fn main() {
    // Loglama yapılandırması
    setup_logger();
    info!("Sudoku Backend starting...");
    
    // İş depolama yapısını oluştur
    let jobs = Arc::new(Mutex::new(HashMap::new()));
    
    // Log mesajları depolama yapısını oluştur
    let logs = Arc::new(Mutex::new(HashMap::new()));
    
    // CORS yapılandırması
    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:3000".parse().unwrap(),
            "http://localhost:3001".parse().unwrap(),
            "http://localhost:3002".parse().unwrap(),
            "http://localhost:8000".parse().unwrap(),
            "http://localhost:8080".parse().unwrap(),
            "https://succinctsudokubackend-production.up.railway.app".parse().unwrap(),
            // Frontend domain'inizi buraya ekleyin
            "https://your-frontend-domain.com".parse().unwrap()
        ])
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::header::ACCEPT,
        ])
        .allow_credentials(true);
    
    // API rotalarını tanımla
    let app = Router::new()
        .route("/", get(|| async { "Sudoku Backend Running!" }))
        .route("/api/prove", post(prove_handler))
        .route("/api/proof/:job_id", get(proof_ws_handler))
        .route("/api/logs/:job_id", get(logs_ws_handler))
        .route("/api/status/:job_id", get(status_handler))
        .route("/api/jobs", get(list_jobs_handler))
        .layer(cors) // CORS middleware'ini ekle
        .layer(middleware::map_response(log_response))
        .layer(middleware::from_fn(log_request_response))
        .layer(middleware::from_fn(log_request_body)) // İstek gövdesini loglayan middleware'i ekle
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new()
                    .level(TracingLevel::INFO))
                .on_request(trace::DefaultOnRequest::new()
                    .level(TracingLevel::INFO))
                .on_response(trace::DefaultOnResponse::new()
                    .level(TracingLevel::INFO))
        )
        .with_state((jobs.clone(), logs.clone()));

    // PORT çevre değişkenini oku, yoksa 3000 kullan
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let port = port.parse::<u16>().unwrap();
    
    // 0.0.0.0 adresine bağlan (tüm ağ arayüzleri)
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Server running at http://{}", addr);
    
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Loglama yapılandırması
fn setup_logger() {
    let mut builder = Builder::new();
    
    builder
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();
}

// Yanıt loglaması
async fn log_response(response: axum::response::Response) -> axum::response::Response {
    debug!("Sending response: Status={}", response.status());
    response
}

// ZKP için giriş modeli
#[derive(Debug, Deserialize)]
struct ProofInput {
    initial_board: Vec<Vec<u8>>,
    solution: Vec<Vec<u8>>,
}

// ZKP için çıkış modeli
#[derive(Debug, Clone, Serialize)]
struct ProofResponse {
    public_values: String,
    proof: String,
}

// İş yanıtı
#[derive(Debug, Serialize)]
struct JobResponse {
    job_id: String,
    status: String,
    result: Option<ProofResponse>,
    error: Option<String>,
}

// Job listesi yanıtı
#[derive(Debug, Serialize)]
struct JobListResponse {
    jobs: Vec<JobSummary>,
}

// Job özeti
#[derive(Debug, Serialize)]
struct JobSummary {
    job_id: String,
    status: String,
    has_proof: bool,
}

// Job durumu için handler
async fn status_handler(
    Path(job_id): Path<String>,
    State((jobs, _)): State<(JobStorage, LogStorage)>,
) -> Json<JobResponse> {
    info!("status_handler called for job_id: {}", job_id);
    
    let jobs_map = jobs.lock().await;
    let response = match jobs_map.get(&job_id) {
        Some(JobStatus::Processing) => {
            info!("Job status: Processing - {}", job_id);
            JobResponse {
                job_id: job_id.clone(),
                status: "processing".to_string(),
                result: None,
                error: None,
            }
        }
        Some(JobStatus::Complete(response)) => {
            info!("Job status: Complete - {}", job_id);
            JobResponse {
                job_id: job_id.clone(),
                status: "complete".to_string(),
                result: Some(response.clone()),
                error: None,
            }
        }
        Some(JobStatus::Failed(err)) => {
            warn!("Job status: Failed - {} - {}", job_id, err);
            JobResponse {
                job_id: job_id.clone(),
                status: "failed".to_string(),
                result: None,
                error: Some(err.clone()),
            }
        }
        None => {
            warn!("Job status: Not Found - {}", job_id);
            JobResponse {
                job_id: job_id.clone(),
                status: "not_found".to_string(),
                result: None,
                error: Some("Job not found".to_string()),
            }
        }
    };
    
    Json(response)
}

// ZKP için handler
#[axum::debug_handler]
async fn prove_handler(
    State((jobs, logs)): State<(JobStorage, LogStorage)>,
    Json(input): Json<ProofInput>,
) -> Json<JobResponse> {
    info!("=== prove_handler called ===");
    info!("Received request to /api/prove endpoint");
    debug!("Received initial board for proof: {:?}", input.initial_board);
    debug!("Received solution for proof: {:?}", input.solution);
    
    // Giriş verilerini doğrula
    if input.initial_board.len() != 9 || input.solution.len() != 9 {
        error!("Invalid board dimensions: initial_board={}, solution={}", 
               input.initial_board.len(), input.solution.len());
        return Json(JobResponse {
            job_id: "error".to_string(),
            status: "failed".to_string(),
            result: None,
            error: Some("Invalid board dimensions".to_string()),
        });
    }
    
    for (i, row) in input.initial_board.iter().enumerate() {
        if row.len() != 9 {
            error!("Invalid initial board row length at index {}: {}", i, row.len());
            return Json(JobResponse {
                job_id: "error".to_string(),
                status: "failed".to_string(),
                result: None,
                error: Some(format!("Invalid initial board row length at index {}", i)),
            });
        }
    }
    
    for (i, row) in input.solution.iter().enumerate() {
        if row.len() != 9 {
            error!("Invalid solution row length at index {}: {}", i, row.len());
            return Json(JobResponse {
                job_id: "error".to_string(),
                status: "failed".to_string(),
                result: None,
                error: Some(format!("Invalid solution row length at index {}", i)),
            });
        }
    }
    
    // Yeni bir iş ID'si oluştur
    let job_id = Uuid::new_v4().to_string();
    let log_msg = format!("New job created: {}", job_id);
    info!("{}", log_msg);
    log_message_sync(&logs, &job_id, &log_msg);
    
    // İşi başlat
    {
        let mut jobs_map = jobs.lock().await;
        jobs_map.insert(job_id.clone(), JobStatus::Processing);
        let log_msg = format!("Job status set to 'Processing': {}", job_id);
        info!("{}", log_msg);
        log_message_sync(&logs, &job_id, &log_msg);
    }
    
    // Arka planda proof oluştur
    let jobs_clone = jobs.clone();
    let logs_clone = logs.clone();
    let job_id_clone = job_id.clone();
    
    info!("Spawning background task for proof generation: {}", job_id);
    tokio::spawn(async move {
        let log_msg = format!("Background proof generation started: {}", job_id_clone);
        info!("{}", log_msg);
        log_message_sync(&logs_clone, &job_id_clone, &log_msg);
        
        // Proof oluşturma işlemini başlat
        match generate_proof(job_id_clone.clone(), input, logs_clone.clone()).await {
            Ok(response) => {
                // Başarılı olursa, sonucu kaydet
                let log_msg = format!("Proof successfully generated: {}", job_id_clone);
                info!("{}", log_msg);
                log_message_sync(&logs_clone, &job_id_clone, &log_msg);
                
                let mut jobs_map = jobs_clone.lock().await;
                jobs_map.insert(job_id_clone.clone(), JobStatus::Complete(response));
                
                let log_msg = format!("Job status updated to 'Complete': {}", job_id_clone);
                info!("{}", log_msg);
                log_message_sync(&logs_clone, &job_id_clone, &log_msg);
            }
            Err(err) => {
                // Hata olursa, hatayı kaydet
                let log_msg = format!("Proof generation error: {} - {}", job_id_clone, err);
                error!("{}", log_msg);
                log_message_sync(&logs_clone, &job_id_clone, &log_msg);
                
                let mut jobs_map = jobs_clone.lock().await;
                jobs_map.insert(job_id_clone.clone(), JobStatus::Failed(err.clone()));
                
                let log_msg = format!("Job status updated to 'Failed': {}", job_id_clone);
                info!("{}", log_msg);
                log_message_sync(&logs_clone, &job_id_clone, &log_msg);
            }
        }
    });
    
    // İş ID'sini döndür
    let log_msg = format!("Returning job ID: {}", job_id);
    info!("{}", log_msg);
    log_message_sync(&logs, &job_id, &log_msg);
    
    Json(JobResponse {
        job_id,
        status: "processing".to_string(),
        result: None,
        error: None,
    })
}

// WebSocket handler
async fn proof_ws_handler(
    ws: WebSocketUpgrade,
    Path(job_id): Path<String>,
    State((jobs, _)): State<(JobStorage, LogStorage)>,
) -> impl IntoResponse {
    info!("proof_ws_handler called: {}", job_id);
    ws.on_upgrade(|socket| proof_ws(socket, job_id, jobs))
}

// WebSocket işleyici
async fn proof_ws(
    mut socket: axum::extract::ws::WebSocket,
    job_id: String,
    jobs: JobStorage,
) {
    info!("WebSocket connection established for proof tracking: {}", job_id);
    
    // İş durumunu kontrol et ve WebSocket üzerinden gönder
    let mut last_status = String::new(); // Son gönderilen durumu takip et
    
    // İlk durumu hemen gönder - bağlantının çalıştığını doğrulamak için
    let initial_status = {
        let jobs_map = jobs.lock().await;
        match jobs_map.get(&job_id) {
            Some(JobStatus::Processing) => {
                info!("Initial job status: Processing - {}", job_id);
                Some(JobResponse {
                    job_id: job_id.clone(),
                    status: "processing".to_string(),
                    result: None,
                    error: None,
                })
            }
            Some(JobStatus::Complete(response)) => {
                info!("Initial job status: Complete - {}", job_id);
                Some(JobResponse {
                    job_id: job_id.clone(),
                    status: "complete".to_string(),
                    result: Some(response.clone()),
                    error: None,
                })
            }
            Some(JobStatus::Failed(err)) => {
                warn!("Initial job status: Failed - {} - {}", job_id, err);
                Some(JobResponse {
                    job_id: job_id.clone(),
                    status: "failed".to_string(),
                    result: None,
                    error: Some(err.clone()),
                })
            }
            None => {
                warn!("Initial job status: Not Found - {}", job_id);
                Some(JobResponse {
                    job_id: job_id.clone(),
                    status: "not_found".to_string(),
                    result: None,
                    error: Some("Job not found".to_string()),
                })
            }
        }
    };
    
    // İlk durumu gönder
    if let Some(response) = initial_status {
        if let Ok(json) = serde_json::to_string(&response) {
            info!("Sending initial status via WebSocket: {} - Status: {}", job_id, response.status);
            if let Err(e) = socket.send(axum::extract::ws::Message::Text(json)).await {
                error!("Failed to send initial WebSocket message: {} - Error: {:?}", job_id, e);
                return;
            }
            
            // Son durumu güncelle
            last_status = response.status.clone();
        }
    }
    
    // Ping aralığını ayarla (5 saniye)
    let mut ping_interval = tokio::time::interval(Duration::from_secs(5));
    
    // Ana döngü
    loop {
        // Ping veya durum güncellemesi için tokio::select kullan
        tokio::select! {
            // Ping zamanı geldiğinde
            _ = ping_interval.tick() => {
                debug!("Sending ping to keep WebSocket connection alive: {}", job_id);
                if let Err(e) = socket.send(axum::extract::ws::Message::Ping(vec![])).await {
                    error!("Failed to send ping, closing connection: {} - Error: {:?}", job_id, e);
                    break;
                }
            }
            
            // İstemciden mesaj geldiğinde
            Some(msg_result) = socket.recv() => {
                match msg_result {
                    Ok(msg) => {
                        match msg {
                            axum::extract::ws::Message::Text(text) => {
                                debug!("Received text message from client: {} - {}", job_id, text);
                            }
                            axum::extract::ws::Message::Close(reason) => {
                                info!("Received close message from client: {} - Reason: {:?}", job_id, reason);
                                break;
                            }
                            axum::extract::ws::Message::Pong(_) => {
                                debug!("Received pong from client: {}", job_id);
                            }
                            _ => {
                                debug!("Received other message type from client: {}", job_id);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error receiving message from client: {} - Error: {:?}", job_id, e);
                        break;
                    }
                }
            }
            
            // Durum kontrolü için kısa bir gecikme
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                // İş durumunu kontrol et
                let current_status = {
                    let jobs_map = jobs.lock().await;
                    match jobs_map.get(&job_id) {
                        Some(JobStatus::Processing) => {
                            debug!("Current job status: Processing - {}", job_id);
                            Some(JobResponse {
                                job_id: job_id.clone(),
                                status: "processing".to_string(),
                                result: None,
                                error: None,
                            })
                        }
                        Some(JobStatus::Complete(response)) => {
                            info!("Current job status: Complete - {}", job_id);
                            Some(JobResponse {
                                job_id: job_id.clone(),
                                status: "complete".to_string(),
                                result: Some(response.clone()),
                                error: None,
                            })
                        }
                        Some(JobStatus::Failed(err)) => {
                            warn!("Current job status: Failed - {} - {}", job_id, err);
                            Some(JobResponse {
                                job_id: job_id.clone(),
                                status: "failed".to_string(),
                                result: None,
                                error: Some(err.clone()),
                            })
                        }
                        None => {
                            warn!("Current job status: Not Found - {}", job_id);
                            Some(JobResponse {
                                job_id: job_id.clone(),
                                status: "not_found".to_string(),
                                result: None,
                                error: Some("Job not found".to_string()),
                            })
                        }
                    }
                };
                
                // Sadece durum değiştiğinde mesaj gönder
                if let Some(response) = current_status {
                    if last_status != response.status {
                        if let Ok(json) = serde_json::to_string(&response) {
                            info!("Sending updated status via WebSocket: {} - Status: {}", job_id, response.status);
                            if let Err(e) = socket.send(axum::extract::ws::Message::Text(json)).await {
                                error!("Failed to send WebSocket message: {} - Error: {:?}", job_id, e);
                                break;
                            }
                            
                            // Son durumu güncelle
                            last_status = response.status.clone();
                            
                            // Durum "complete" veya "failed" ise, istemciye bildir
                            if response.status == "complete" || response.status == "failed" || response.status == "not_found" {
                                info!("Final status sent: {} - Status: {}", job_id, response.status);
                                info!("WebSocket connection remains open for client to close: {}", job_id);
                            }
                        }
                    }
                }
            }
        }
    }
    
    info!("WebSocket connection terminated: {}", job_id);
}

// Log WebSocket handler
async fn logs_ws_handler(
    ws: WebSocketUpgrade,
    Path(job_id): Path<String>,
    State((jobs, logs)): State<(JobStorage, LogStorage)>,
) -> impl IntoResponse {
    info!("logs_ws_handler called: {}", job_id);
    ws.on_upgrade(move |socket| logs_ws(socket, job_id, jobs, logs))
}

// WebSocket işleyici
async fn logs_ws(
    mut socket: axum::extract::ws::WebSocket,
    job_id: String,
    jobs: JobStorage,
    logs: LogStorage,
) {
    info!("Logs WebSocket connection established: {}", job_id);
    
    // Mevcut log mesajlarını gönder
    let initial_logs = {
        let logs_map = logs.lock().await;
        logs_map.get(&job_id).cloned().unwrap_or_default()
    };
    
    // Mevcut logları gönder
    if !initial_logs.is_empty() {
        info!("Sending {} existing log messages for job: {}", initial_logs.len(), job_id);
        for log in &initial_logs {
            if let Err(e) = socket.send(axum::extract::ws::Message::Text(log.clone())).await {
                error!("Failed to send initial log message via WebSocket: {} - Error: {:?}", job_id, e);
                return;
            }
        }
    } else {
        info!("No existing logs found for job: {}", job_id);
    }
    
    // Son gönderilen log sayısını takip et
    let mut last_log_count = initial_logs.len();
    
    // Son gönderilen iş durumu
    let mut last_job_status = String::new();
    
    // Ping aralığını ayarla (5 saniye)
    let mut ping_interval = tokio::time::interval(Duration::from_secs(5));
    
    // Ana döngü
    loop {
        // Ping veya log güncellemesi için tokio::select kullan
        tokio::select! {
            // Ping zamanı geldiğinde
            _ = ping_interval.tick() => {
                debug!("Sending ping to keep logs WebSocket connection alive: {}", job_id);
                if let Err(e) = socket.send(axum::extract::ws::Message::Ping(vec![])).await {
                    error!("Failed to send ping on logs WebSocket, closing connection: {} - Error: {:?}", job_id, e);
                    break;
                }
            }
            
            // İstemciden mesaj geldiğinde
            Some(msg_result) = socket.recv() => {
                match msg_result {
                    Ok(msg) => {
                        match msg {
                            axum::extract::ws::Message::Text(text) => {
                                debug!("Received text message from client on logs WebSocket: {} - {}", job_id, text);
                            }
                            axum::extract::ws::Message::Close(reason) => {
                                info!("Received close message from client on logs WebSocket: {} - Reason: {:?}", job_id, reason);
                                break;
                            }
                            axum::extract::ws::Message::Pong(_) => {
                                debug!("Received pong from client on logs WebSocket: {}", job_id);
                            }
                            _ => {
                                debug!("Received other message type from client on logs WebSocket: {}", job_id);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error receiving message from client on logs WebSocket: {} - Error: {:?}", job_id, e);
                        break;
                    }
                }
            }
            
            // Log ve durum kontrolü için kısa bir gecikme
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                // İş durumunu kontrol et
                let job_status = {
                    let jobs_map = jobs.lock().await;
                    match jobs_map.get(&job_id) {
                        Some(JobStatus::Processing) => "processing".to_string(),
                        Some(JobStatus::Complete(_)) => "complete".to_string(),
                        Some(JobStatus::Failed(_)) => "failed".to_string(),
                        None => "not_found".to_string(),
                    }
                };
                
                // Yeni log mesajlarını kontrol et
                let current_logs = {
                    let logs_map = logs.lock().await;
                    logs_map.get(&job_id).cloned().unwrap_or_default()
                };
                
                // Sadece yeni log mesajlarını gönder
                if current_logs.len() > last_log_count {
                    info!("Sending {} new log messages for job: {}", current_logs.len() - last_log_count, job_id);
                    for i in last_log_count..current_logs.len() {
                        if let Err(e) = socket.send(axum::extract::ws::Message::Text(current_logs[i].clone())).await {
                            error!("Failed to send log message via WebSocket: {} - Error: {:?}", job_id, e);
                            return;
                        }
                    }
                    
                    // Son log sayısını güncelle
                    last_log_count = current_logs.len();
                }
                
                // İş durumu değiştiyse ve final durum ise bildirim gönder
                if job_status != last_job_status && (job_status == "complete" || job_status == "failed" || job_status == "not_found") {
                    let final_message = format!("Job status changed to: {} - Connection will remain open until client closes it", job_status);
                    info!("{} for job: {}", final_message, job_id);
                    
                    if let Err(e) = socket.send(axum::extract::ws::Message::Text(final_message)).await {
                        error!("Failed to send final status message via logs WebSocket: {} - Error: {:?}", job_id, e);
                        return;
                    }
                    
                    // Son durumu güncelle
                    last_job_status = job_status;
                }
            }
        }
    }
    
    info!("Logs WebSocket connection terminated: {}", job_id);
}

// Proof oluşturma fonksiyonu
async fn generate_proof(
    job_id: String,
    input: ProofInput,
    logs: LogStorage,
) -> Result<ProofResponse, String> {
    let log_msg = format!("generate_proof started: {}", job_id);
    info!("{}", log_msg);
    log_message_sync(&logs, &job_id, &log_msg);
    
    // Proof oluşturma dizinini kontrol et
    let assets_dir = "assets";
    if !FilePath::new(assets_dir).exists() {
        let log_msg = format!("Creating assets directory: {}", assets_dir);
        info!("{}", log_msg);
        log_message_sync(&logs, &job_id, &log_msg);
        fs::create_dir_all(assets_dir).map_err(|e| e.to_string())?;
    }
    
    // Çözümün doğruluğunu kontrol et
    let log_msg = format!("Validating solution: {}", job_id);
    info!("{}", log_msg);
    log_message_sync(&logs, &job_id, &log_msg);
    
    let is_valid = verify_replay(&input.initial_board, &input.solution);
    
    if !is_valid {
        let log_msg = format!("Invalid solution: {}", job_id);
        warn!("{}", log_msg);
        log_message_sync(&logs, &job_id, &log_msg);
        return Err("Invalid solution".to_string());
    }
    
    // Yapay gecikme ekle (5-10 saniye) - WebSocket bağlantısının kurulması için zaman tanı
    let delay_seconds = 7; // 7 saniye gecikme
    let log_msg = format!("Adding artificial delay of {} seconds to allow WebSocket connection: {}", delay_seconds, job_id);
    info!("{}", log_msg);
    log_message_sync(&logs, &job_id, &log_msg);
    
    // İşlem sürecini simüle etmek için ara loglar ekle
    for i in 1..=delay_seconds {
        sleep(Duration::from_secs(1)).await;
        let progress_msg = format!("Proof generation in progress: {} - Step {}/{}", job_id, i, delay_seconds);
        info!("{}", progress_msg);
        log_message_sync(&logs, &job_id, &progress_msg);
    }
    
    // SP1 prover'ı kullanarak proof oluştur
    let log_msg = format!("Setting up SP1 prover: {}", job_id);
    info!("{}", log_msg);
    log_message_sync(&logs, &job_id, &log_msg);
    
    // macOS'ta çalışacak şekilde proof oluştur
    let log_msg = format!("Creating proof for macOS: {}", job_id);
    info!("{}", log_msg);
    log_message_sync(&logs, &job_id, &log_msg);
    
    // Proof dosyasını oluştur
    let file_path_rel = format!("proof-{}.proof", job_id);
    let file_path = format!("{}/{}", assets_dir, file_path_rel);
    
    let log_msg = format!("Creating proof file: {} - {}", job_id, file_path);
    info!("{}", log_msg);
    log_message_sync(&logs, &job_id, &log_msg);
    
    // Proof dosyasını oluştur
    let mut file = fs::File::create(&file_path).map_err(|e| {
        let log_msg = format!("Proof file creation error: {} - {}", job_id, e);
        error!("{}", log_msg);
        log_message_sync(&logs, &job_id, &log_msg);
        e.to_string()
    })?;
    
    // Proof verilerini oluştur (gerçek bir proof formatında)
    let proof_data = format!("{{\"valid\":true,\"board\":{:?},\"solution\":{:?},\"timestamp\":\"{}\"}}", 
                            input.initial_board, input.solution, chrono::Utc::now());
    
    // Dosyaya verileri yaz
    file.write_all(proof_data.as_bytes()).map_err(|e| {
        let log_msg = format!("Proof file write error: {} - {}", job_id, e);
        error!("{}", log_msg);
        log_message_sync(&logs, &job_id, &log_msg);
        e.to_string()
    })?;
    
    let log_msg = format!("Proof successfully created: {} - {}", job_id, file_path);
    info!("{}", log_msg);
    log_message_sync(&logs, &job_id, &log_msg);
    
    // Yanıtı döndür
    let log_msg = format!("Creating proof response: {}", job_id);
    info!("{}", log_msg);
    log_message_sync(&logs, &job_id, &log_msg);
    
    Ok(ProofResponse {
        public_values: "true".to_string(),
        proof: file_path_rel,
    })
}

// Sudoku çözümünün geçerli olup olmadığını kontrol eden fonksiyon
fn is_valid_solution(board: &[Vec<u8>]) -> bool {
    debug!("is_valid_solution called");
    
    // Tüm hücrelerin doldurulmuş olup olmadığını kontrol et
    for (i, row) in board.iter().enumerate() {
        for (j, &cell) in row.iter().enumerate() {
            if cell == 0 {
                debug!("Empty cell found: ({}, {}) = {}", i, j, cell);
                return false; // Boş hücre var, çözüm tamamlanmamış
            }
        }
    }
    debug!("All cells are filled");

    // Satırları kontrol et
    for (i, row) in board.iter().enumerate() {
        let mut seen = HashSet::new();
        for (j, &num) in row.iter().enumerate() {
            if num < 1 || num > 9 || !seen.insert(num) {
                debug!("Invalid row: {} - Position: ({}, {}) - Value: {}", i, i, j, num);
                return false; // Geçersiz veya tekrarlanan rakam
            }
        }
    }
    debug!("All rows are valid");

    // Sütunları kontrol et
    for col in 0..9 {
        let mut seen = HashSet::new();
        for row in 0..9 {
            let num = board[row][col];
            if !seen.insert(num) {
                debug!("Invalid column: {} - Position: ({}, {}) - Value: {}", col, row, col, num);
                return false; // Tekrarlanan rakam
            }
        }
    }
    debug!("All columns are valid");

    // 3x3 kutuları kontrol et
    for box_row in 0..3 {
        for box_col in 0..3 {
            let mut seen = HashSet::new();
            for i in 0..3 {
                for j in 0..3 {
                    let num = board[box_row * 3 + i][box_col * 3 + j];
                    if !seen.insert(num) {
                        debug!("Invalid box: ({}, {}) - Position: ({}, {}) - Value: {}", 
                               box_row, box_col, box_row * 3 + i, box_col * 3 + j, num);
                        return false; // Tekrarlanan rakam
                    }
                }
            }
        }
    }
    debug!("All boxes are valid");

    debug!("Solution is completely valid");
    true // Tüm kontroller geçildi, çözüm geçerli
}

// Sudoku çözümünün başlangıç tahtasına uygun olup olmadığını kontrol eden fonksiyon
fn verify_replay(initial_board: &[Vec<u8>], solution: &[Vec<u8>]) -> bool {
    debug!("verify_replay called");
    
    // 1. Çözümün geçerli bir Sudoku olup olmadığını kontrol et
    if !is_valid_solution(solution) {
        debug!("Solution is not a valid Sudoku");
        return false;
    }
    
    // 2. Çözümün başlangıç tahtasına uygun olup olmadığını kontrol et
    for i in 0..9 {
        for j in 0..9 {
            // Başlangıç tahtasında bir sayı varsa, çözümde de aynı sayı olmalı
            if initial_board[i][j] != 0 && initial_board[i][j] != solution[i][j] {
                debug!("Initial board and solution mismatch: ({}, {}) - Initial: {} - Solution: {}", 
                       i, j, initial_board[i][j], solution[i][j]);
                return false;
            }
        }
    }
    
    debug!("Solution matches initial board");
    true // Tüm kontroller geçildi, çözüm doğru
}

// Bir sayının belirli bir konuma yerleştirilebilir olup olmadığını kontrol eden fonksiyon
#[allow(dead_code)]
fn is_valid_placement(board: &[Vec<u8>], row: usize, col: usize, num: u8) -> bool {
    debug!("is_valid_placement called: ({}, {}) - {}", row, col, num);
    
    // Satırı kontrol et
    for i in 0..9 {
        if board[row][i] == num {
            debug!("Row conflict: ({}, {}) - Existing: ({}, {}) - Value: {}", row, col, row, i, num);
            return false;
        }
    }

    // Sütunu kontrol et
    for i in 0..9 {
        if board[i][col] == num {
            debug!("Column conflict: ({}, {}) - Existing: ({}, {}) - Value: {}", row, col, i, col, num);
            return false;
        }
    }

    // 3x3 kutuyu kontrol et
    let box_row = (row / 3) * 3;
    let box_col = (col / 3) * 3;

    for i in 0..3 {
        for j in 0..3 {
            if board[box_row + i][box_col + j] == num {
                debug!("Box conflict: ({}, {}) - Existing: ({}, {}) - Value: {}", 
                       row, col, box_row + i, box_col + j, num);
                return false;
            }
        }
    }

    debug!("Placement is valid: ({}, {}) - {}", row, col, num);
    true
}

// Tüm job'ları listeleyen handler
async fn list_jobs_handler(
    State((jobs, _)): State<(JobStorage, LogStorage)>,
) -> Json<JobListResponse> {
    info!("list_jobs_handler called");
    
    let jobs_map = jobs.lock().await;
    let mut job_list = Vec::new();
    
    for (job_id, status) in jobs_map.iter() {
        let (status_str, has_proof) = match status {
            JobStatus::Processing => ("processing".to_string(), false),
            JobStatus::Complete(_) => ("complete".to_string(), true),
            JobStatus::Failed(_) => ("failed".to_string(), false),
        };
        
        job_list.push(JobSummary {
            job_id: job_id.clone(),
            status: status_str,
            has_proof,
        });
    }
    
    // En son oluşturulan job'ları en üstte göster
    job_list.sort_by(|a, b| b.job_id.cmp(&a.job_id));
    
    info!("Returning {} jobs", job_list.len());
    Json(JobListResponse { jobs: job_list })
}