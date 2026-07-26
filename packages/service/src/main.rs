use anyhow::{Context, Result};
use looplens_core::{
    CodeEvidence, LearnInput, LoopLensEngine, MemoryScope, RecallInput, TaskType,
    VerificationEvidence, VerificationResult, VerificationSource,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct RecallRequest {
    task: String,
    #[serde(default)]
    task_type: Option<TaskType>,
    #[serde(default)]
    files: Vec<String>,
    #[serde(default)]
    stack: Vec<String>,
    #[serde(default)]
    languages: Vec<String>,
    #[serde(default)]
    frameworks: Vec<String>,
    #[serde(default = "default_top_k")]
    top_k: usize,
}

#[derive(Debug, Deserialize)]
struct StoreRequest {
    task: String,
    #[serde(default)]
    task_type: TaskType,
    #[serde(default)]
    hypothesis: Option<String>,
    #[serde(default)]
    failed_attempts: Vec<String>,
    successful_decision: String,
    #[serde(default)]
    files: Vec<String>,
    lesson: String,
    #[serde(default)]
    verification: VerificationEvidence,
    #[serde(default)]
    evidence: CodeEvidence,
    #[serde(default)]
    scope: MemoryScope,
    #[serde(default = "default_confidence")]
    confidence: f32,
}

fn main() -> Result<()> {
    let root = std::env::var_os("LOOPLENS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let bind = service_bind_address();
    let listener = TcpListener::bind(&bind).with_context(|| format!("failed to bind {bind}"))?;
    eprintln!("LoopLens service listening on http://{bind}");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let engine = LoopLensEngine::new(root.clone());
                if let Err(error) = handle_stream(stream, &engine) {
                    eprintln!("request failed: {error:#}");
                }
            }
            Err(error) => eprintln!("connection failed: {error}"),
        }
    }

    Ok(())
}

fn handle_stream(mut stream: TcpStream, engine: &LoopLensEngine) -> Result<()> {
    let (method, path, body) = read_request(&mut stream)?;

    let response = match (method.as_str(), path.as_str()) {
        ("OPTIONS", _) => return write_response(&mut stream, 204, Value::Null),
        ("GET", "/health") => Ok(json!({ "status": "ok" })),
        ("GET", "/project_context") => serde_json::to_value(engine.project_context()?)
            .context("failed to encode project context"),
        ("POST", "/recall_context") => recall_context(engine, &body),
        ("POST", "/store_experience") => store_experience(engine, &body),
        _ => return write_response(&mut stream, 404, json!({ "error": "not found" })),
    };

    match response {
        Ok(value) => write_response(&mut stream, 200, value),
        Err(error) => write_response(&mut stream, 400, json!({ "error": error.to_string() })),
    }
}

fn recall_context(engine: &LoopLensEngine, body: &str) -> Result<Value> {
    let mut request: RecallRequest = serde_json::from_str(body).context("invalid JSON body")?;
    if request.languages.is_empty() {
        request.languages = request.stack.clone();
    }
    let result = engine.recall(RecallInput {
        task: request.task,
        task_type: request.task_type,
        files: request.files,
        languages: request.languages,
        frameworks: request.frameworks,
        top_k: request.top_k,
    })?;
    let confidence = result.matches.first().map(|item| item.score).unwrap_or(0.0);
    Ok(json!({
        "relevant_experience": result.matches,
        "avoid": result.avoid,
        "recommended_checks": result.recommended_checks,
        "confidence": confidence
    }))
}

fn store_experience(engine: &LoopLensEngine, body: &str) -> Result<Value> {
    let mut request: StoreRequest = serde_json::from_str(body).context("invalid JSON body")?;
    if request.verification.source == VerificationSource::Unspecified {
        request.verification.source = VerificationSource::Custom;
    }
    request.verification.result = VerificationResult::Passed;
    let experience = engine.learn(LearnInput {
        task: request.task,
        task_type: request.task_type,
        hypothesis: request.hypothesis,
        failed_attempts: request.failed_attempts,
        successful_decision: request.successful_decision,
        files: request.files,
        lesson: request.lesson,
        verification: request.verification,
        evidence: request.evidence,
        scope: request.scope,
        confidence: request.confidence,
    })?;
    serde_json::to_value(experience).context("failed to encode experience")
}

fn read_request(stream: &mut TcpStream) -> Result<(String, String, String)> {
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    if request_line.trim().is_empty() {
        anyhow::bail!("missing request line");
    }

    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("Content-Length:") {
            content_length = value.trim().parse().context("invalid Content-Length")?;
        }
    }

    let mut body = vec![0; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().context("missing method")?.to_string();
    let path = parts.next().context("missing path")?.to_string();
    let body = String::from_utf8(body).context("request body must be utf-8")?;
    Ok((method, path, body))
}

fn write_response(stream: &mut TcpStream, status: u16, body: Value) -> Result<()> {
    let body = if status == 204 {
        String::new()
    } else {
        serde_json::to_string_pretty(&body)?
    };
    let status_text = match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        _ => "Internal Server Error",
    };
    write!(
        stream,
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: content-type\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    )?;
    Ok(())
}

fn service_bind_address() -> String {
    if let Ok(bind) = std::env::var("LOOPLENS_BIND") {
        return bind;
    }
    if let Ok(port) = std::env::var("PORT") {
        return format!("0.0.0.0:{port}");
    }
    "127.0.0.1:8787".to_string()
}

fn default_top_k() -> usize {
    3
}

fn default_confidence() -> f32 {
    0.85
}
