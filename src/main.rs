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

// SP1 için gerekli bileşenler
use sp1_sdk::{ProverClient, SP1Stdin, HashableKey};

// ELF dosyası (SP1 prover'ın derlenmiş hali)
const ELF: &[u8] = include_bytes!("../target/elf-compilation/riscv32im-succinct-zkvm-elf/release/sp1_prover");

// İş depolama yapısı
type JobStorage = Arc<Mutex<HashMap<String, JobStatus>>>;

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
    
    info!(">> Request started: {} {}", method, path);
    
    let response = next.run(req).await;
    
    let status = response.status();
    let duration = start.elapsed();
    
    info!("<< Request completed: {} {} - Status: {} - Duration: {:.2?}", 
          method, path, status, duration);
    
    response
}

#[tokio::main]
async fn main() {
    // Loglama yapılandırması
    setup_logger();
    info!("Sudoku Backend starting...");
    
    // İş depolama yapısını oluştur
    let jobs = Arc::new(Mutex::new(HashMap::new()));
    
    // API rotalarını tanımla
    let app = Router::new()
        .route("/", get(|| async { "Sudoku Backend Running!" }))
        .route("/validate", post(validate_sudoku))
        .route("/verify", post(verify_sudoku))
        .route("/zkp", post(zkp_sudoku))
        .route("/prove", post(prove_handler))
        .route("/proof/:job_id", get(proof_ws_handler))
        .layer(middleware::map_response(log_response))
        .layer(middleware::from_fn(log_request_response))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new()
                    .level(TracingLevel::INFO))
                .on_request(trace::DefaultOnRequest::new()
                    .level(TracingLevel::INFO))
                .on_response(trace::DefaultOnResponse::new()
                    .level(TracingLevel::INFO))
        )
        .with_state(jobs);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
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

// Sudoku çözümünü almak için veri modeli
#[derive(Debug, Deserialize)]
struct SudokuRequest {
    board: Vec<Vec<u8>>,
}

// Doğrulama yanıtı
#[derive(Debug, Serialize)]
struct SudokuResponse {
    valid: bool,
    proof: String,
}

// Sudoku çözümünü doğrulamak için veri modeli
#[derive(Debug, Deserialize)]
struct VerifyRequest {
    initial_board: Vec<Vec<u8>>,
    solution: Vec<Vec<u8>>,
}

// ZKP yanıtı
#[derive(Debug, Serialize)]
struct ZkpResponse {
    valid: bool,
    proof: String,
    message: String,
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

// Sudoku'yu doğrulayan fonksiyon
async fn validate_sudoku(Json(payload): Json<SudokuRequest>) -> Json<SudokuResponse> {
    info!("validate_sudoku called");
    debug!("Received board: {:?}", payload.board);
    
    let is_valid = is_valid_solution(&payload.board);
    info!("Is board valid: {}", is_valid);

    // Basit bir proof üretme (SHA256 hash ile)
    let board_str = format!("{:?}", payload.board);
    let proof = sha256::digest(board_str);
    debug!("Generated proof: {}", proof);

    Json(SudokuResponse {
        valid: is_valid,
        proof,
    })
}

// Sudoku çözümünü doğrulayan fonksiyon
async fn verify_sudoku(Json(payload): Json<VerifyRequest>) -> Json<SudokuResponse> {
    info!("verify_sudoku called");
    debug!("Received initial board: {:?}", payload.initial_board);
    debug!("Received solution: {:?}", payload.solution);
    
    let is_valid = verify_replay(&payload.initial_board, &payload.solution);
    info!("Is solution valid: {}", is_valid);

    // Basit bir proof üretme (SHA256 hash ile)
    let board_str = format!("{:?}{:?}", payload.initial_board, payload.solution);
    let proof = sha256::digest(board_str);
    debug!("Generated proof: {}", proof);

    Json(SudokuResponse {
        valid: is_valid,
        proof,
    })
}

// ZKP ile Sudoku çözümünü doğrulayan fonksiyon
async fn zkp_sudoku(Json(payload): Json<VerifyRequest>) -> Json<ZkpResponse> {
    info!("zkp_sudoku called");
    debug!("Received initial board for ZKP: {:?}", payload.initial_board);
    debug!("Received solution for ZKP: {:?}", payload.solution);
    
    let is_valid = verify_replay(&payload.initial_board, &payload.solution);
    info!("Is solution valid for ZKP: {}", is_valid);

    // Basit bir proof üretme (SHA256 hash ile)
    // Gerçek bir ZKP implementasyonunda, burada SP1 kullanılacak
    let board_str = format!("{:?}{:?}", payload.initial_board, payload.solution);
    let proof = sha256::digest(board_str);
    debug!("Generated proof for ZKP: {}", proof);

    Json(ZkpResponse {
        valid: is_valid,
        proof,
        message: "This is a simulated ZKP proof. In a real implementation, SP1 would be used.".to_string(),
    })
}

// ZKP için handler
#[axum::debug_handler]
async fn prove_handler(
    State(jobs): State<JobStorage>,
    Json(input): Json<ProofInput>,
) -> Json<JobResponse> {
    info!("prove_handler called");
    debug!("Received initial board for proof: {:?}", input.initial_board);
    debug!("Received solution for proof: {:?}", input.solution);
    
    // Yeni bir iş ID'si oluştur
    let job_id = Uuid::new_v4().to_string();
    info!("New job created: {}", job_id);
    
    // İşi başlat
    {
        let mut jobs_map = jobs.lock().await;
        jobs_map.insert(job_id.clone(), JobStatus::Processing);
        info!("Job status set to 'Processing': {}", job_id);
    }
    
    // Arka planda proof oluştur
    let jobs_clone = jobs.clone();
    let job_id_clone = job_id.clone();
    
    tokio::spawn(async move {
        info!("Background proof generation started: {}", job_id_clone);
        // Proof oluşturma işlemini başlat
        match generate_proof(job_id_clone.clone(), input).await {
            Ok(response) => {
                // Başarılı olursa, sonucu kaydet
                info!("Proof successfully generated: {}", job_id_clone);
                let mut jobs_map = jobs_clone.lock().await;
                jobs_map.insert(job_id_clone.clone(), JobStatus::Complete(response));
                info!("Job status updated to 'Complete': {}", job_id_clone);
            }
            Err(err) => {
                // Hata olursa, hatayı kaydet
                error!("Proof generation error: {} - {}", job_id_clone, err);
                let mut jobs_map = jobs_clone.lock().await;
                jobs_map.insert(job_id_clone.clone(), JobStatus::Failed(err));
                info!("Job status updated to 'Failed': {}", job_id_clone);
            }
        }
    });
    
    // İş ID'sini döndür
    info!("Returning job ID: {}", job_id);
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
    State(jobs): State<JobStorage>,
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
    info!("WebSocket connection established: {}", job_id);
    
    // İş durumunu kontrol et ve WebSocket üzerinden gönder
    loop {
        let status = {
            let jobs_map = jobs.lock().await;
            match jobs_map.get(&job_id) {
                Some(JobStatus::Processing) => {
                    debug!("Job status: Processing - {}", job_id);
                    Some(JobResponse {
                        job_id: job_id.clone(),
                        status: "processing".to_string(),
                        result: None,
                        error: None,
                    })
                }
                Some(JobStatus::Complete(response)) => {
                    info!("Job completed: {}", job_id);
                    Some(JobResponse {
                        job_id: job_id.clone(),
                        status: "complete".to_string(),
                        result: Some(response.clone()),
                        error: None,
                    })
                }
                Some(JobStatus::Failed(err)) => {
                    warn!("Job failed: {} - {}", job_id, err);
                    Some(JobResponse {
                        job_id: job_id.clone(),
                        status: "failed".to_string(),
                        result: None,
                        error: Some(err.clone()),
                    })
                }
                None => {
                    warn!("Job not found: {}", job_id);
                    Some(JobResponse {
                        job_id: job_id.clone(),
                        status: "not_found".to_string(),
                        result: None,
                        error: Some("Job not found".to_string()),
                    })
                }
            }
        };
        
        if let Some(response) = status {
            // Yanıtı JSON olarak gönder
            if let Ok(json) = serde_json::to_string(&response) {
                debug!("Sending response via WebSocket: {}", job_id);
                if socket.send(axum::extract::ws::Message::Text(json)).await.is_err() {
                    error!("Failed to send WebSocket message: {}", job_id);
                    break;
                }
            }
            
            // İş tamamlandıysa veya hata olduysa döngüden çık
            match response.status.as_str() {
                "complete" | "failed" | "not_found" => {
                    info!("Closing WebSocket connection (job completed/failed/not found): {}", job_id);
                    break;
                }
                _ => {}
            }
        }
        
        // Bir süre bekle
        sleep(Duration::from_secs(1)).await;
    }
    
    info!("WebSocket connection terminated: {}", job_id);
}

// Proof oluşturma fonksiyonu
async fn generate_proof(
    job_id: String,
    input: ProofInput,
) -> Result<ProofResponse, String> {
    info!("generate_proof started: {}", job_id);
    
    // Proof oluşturma dizinini kontrol et
    let assets_dir = "assets";
    if !FilePath::new(assets_dir).exists() {
        info!("Creating assets directory: {}", assets_dir);
        fs::create_dir_all(assets_dir).map_err(|e| e.to_string())?;
    }
    
    // Çözümün doğruluğunu kontrol et
    info!("Validating solution: {}", job_id);
    let is_valid = verify_replay(&input.initial_board, &input.solution);
    
    if !is_valid {
        warn!("Invalid solution: {}", job_id);
        return Err("Invalid solution".to_string());
    }
    
    // ProverClient oluştur
    info!("Creating ProverClient: {}", job_id);
    let client = ProverClient::from_env();
    
    // Girdileri hazırla
    debug!("Preparing SP1 inputs: {}", job_id);
    let mut stdin = SP1Stdin::new();
    stdin.write(&input.initial_board);
    stdin.write(&input.solution);
    
    // Önce execute et (proof oluşturmadan önce doğrula)
    info!("Running SP1 program: {}", job_id);
    let (mut pub_values, _) = client.execute(ELF, &stdin).run().map_err(|e| {
        error!("SP1 execution error: {} - {}", job_id, e);
        e.to_string()
    })?;
    
    let is_valid = pub_values.read::<bool>();
    info!("SP1 execution result: {} - {}", job_id, is_valid);
    
    if !is_valid {
        warn!("Invalid solution according to SP1: {}", job_id);
        return Err("Invalid solution according to SP1".to_string());
    }
    
    // Prover ve Verifier anahtarlarını oluştur
    info!("Setting up prover and verifier keys: {}", job_id);
    let (pk, vk) = client.setup(ELF);
    debug!("Verification key: {} - {:?}", job_id, vk.bytes32_raw());
    
    // Proof oluştur
    info!("Generating proof: {}", job_id);
    let mut proof = client.prove(&pk, &stdin).compressed().run().map_err(|e| {
        error!("Proof generation error: {} - {}", job_id, e);
        e.to_string()
    })?;
    info!("Proof successfully generated: {}", job_id);
    
    // Proof'u kaydet
    let file_path_rel = format!("proof-{}.proof", job_id);
    let file_path = format!("{}/{}", assets_dir, file_path_rel);
    
    info!("Saving proof: {} - {}", job_id, file_path);
    proof.save(&file_path).map_err(|e| {
        error!("Proof saving error: {} - {}", job_id, e);
        e.to_string()
    })?;
    info!("Proof successfully saved: {} - {}", job_id, file_path);
    
    // Public değerleri al
    let public_valid = proof.public_values.read::<bool>();
    let final_public_values = format!("{}", public_valid);
    debug!("Public values: {} - {}", job_id, final_public_values);
    
    // Yanıtı döndür
    info!("Creating proof response: {}", job_id);
    Ok(ProofResponse {
        public_values: final_public_values,
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