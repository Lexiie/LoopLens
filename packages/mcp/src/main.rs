use anyhow::{Context, Result};
use looplens_core::{
    CodeEvidence, LearnInput, LoopLensEngine, MemoryScope, RecallInput, TaskType,
    VerificationEvidence, VerificationResult, VerificationSource,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct RpcRequest {
    #[serde(default = "jsonrpc_version")]
    jsonrpc: String,
    id: Value,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct RpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i32,
    message: String,
}

#[derive(Debug, Deserialize)]
struct RecallParams {
    task: String,
    #[serde(default)]
    task_type: Option<TaskType>,
    #[serde(default)]
    files: Vec<String>,
    #[serde(default)]
    languages: Vec<String>,
    #[serde(default)]
    frameworks: Vec<String>,
    #[serde(default = "default_top_k")]
    top_k: usize,
}

#[derive(Debug, Deserialize)]
struct StoreParams {
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
    let root = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let engine = LoopLensEngine::new(root);
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<RpcRequest>(&line) {
            Ok(request) => handle_request(&engine, request),
            Err(error) => RpcResponse {
                jsonrpc: "2.0",
                id: Value::Null,
                result: None,
                error: Some(RpcError {
                    code: -32700,
                    message: error.to_string(),
                }),
            },
        };
        serde_json::to_writer(&mut stdout, &response)?;
        writeln!(stdout)?;
        stdout.flush()?;
    }

    Ok(())
}

fn handle_request(engine: &LoopLensEngine, request: RpcRequest) -> RpcResponse {
    if request.jsonrpc != "2.0" {
        return rpc_error(request.id, -32600, "jsonrpc must be 2.0");
    }

    let result = match request.method.as_str() {
        "get_project_context" => engine.project_context().and_then(to_value),
        "recall_context" => recall_context(engine, request.params),
        "store_experience" => store_experience(engine, request.params),
        "record_attempt" => Ok(json!({
            "recorded": false,
            "message": "record_attempt is accepted by the MCP surface; durable attempt logs are stored when store_experience is called"
        })),
        _ => return rpc_error(request.id, -32601, "unknown method"),
    };

    match result {
        Ok(value) => RpcResponse {
            jsonrpc: "2.0",
            id: request.id,
            result: Some(value),
            error: None,
        },
        Err(error) => rpc_error(request.id, -32000, &error.to_string()),
    }
}

fn recall_context(engine: &LoopLensEngine, params: Value) -> Result<Value> {
    let params: RecallParams =
        serde_json::from_value(params).context("invalid recall_context params")?;
    let result = engine.recall(RecallInput {
        task: params.task,
        task_type: params.task_type,
        files: params.files,
        languages: params.languages,
        frameworks: params.frameworks,
        top_k: params.top_k,
    })?;
    to_value(result)
}

fn store_experience(engine: &LoopLensEngine, params: Value) -> Result<Value> {
    let mut params: StoreParams =
        serde_json::from_value(params).context("invalid store_experience params")?;
    if params.verification.source == VerificationSource::Unspecified {
        params.verification.source = VerificationSource::Custom;
    }
    params.verification.result = VerificationResult::Passed;
    let experience = engine.learn(LearnInput {
        task: params.task,
        task_type: params.task_type,
        hypothesis: params.hypothesis,
        failed_attempts: params.failed_attempts,
        successful_decision: params.successful_decision,
        files: params.files,
        lesson: params.lesson,
        verification: params.verification,
        evidence: params.evidence,
        scope: params.scope,
        confidence: params.confidence,
    })?;
    to_value(experience)
}

fn rpc_error(id: Value, code: i32, message: &str) -> RpcResponse {
    RpcResponse {
        jsonrpc: "2.0",
        id,
        result: None,
        error: Some(RpcError {
            code,
            message: message.to_string(),
        }),
    }
}

fn to_value<T: Serialize>(value: T) -> Result<Value> {
    serde_json::to_value(value).context("failed to encode response")
}

fn jsonrpc_version() -> String {
    "2.0".to_string()
}

fn default_top_k() -> usize {
    3
}

fn default_confidence() -> f32 {
    0.85
}
