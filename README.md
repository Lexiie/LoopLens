# LoopLens

![OKX.AI](https://img.shields.io/badge/OKX.AI-Genesis-111111?style=for-the-badge)
![Rust](https://img.shields.io/badge/Rust-Core-b7410e?style=for-the-badge&logo=rust&logoColor=white)
![MCP](https://img.shields.io/badge/Agent--native-MCP-0c6f5b?style=for-the-badge)
![Memory](https://img.shields.io/badge/Persistent-Engineering%20Memory-28524c?style=for-the-badge)
[![LoopLens CI](https://github.com/Lexiie/LoopLens/actions/workflows/ci.yml/badge.svg)](https://github.com/Lexiie/LoopLens/actions/workflows/ci.yml)

**Persistent Engineering Memory for AI Coding Agents.**

LoopLens gives coding agents access to evidence-backed engineering experience across sessions and projects. It captures the useful reasoning behind verified engineering work, stores it as structured local memory, and recalls relevant context before the next agent starts from scratch.

```text
Coding Agent
     -> LoopLens recall_context
     -> Relevant Engineering Experience
     -> Agent Works
     -> Verification
     -> LoopLens store_experience
     -> Persistent Engineering Memory
```

LoopLens is not a coding agent, CI system, or test runner. It is a memory service consumed by coding agents through MCP/A2MCP, HTTP, or the CLI.

## Why

AI coding agents can inspect a repository and solve tasks, but the engineering experience from a session often disappears when the session ends. The next agent can repeat failed approaches, rediscover repository-specific behavior, or miss a previous architectural decision.

LoopLens stores the reusable part:

- **What task was solved**: bugfix, feature, refactor, migration, build, testing, configuration, and more.
- **What failed**: attempts future agents should avoid.
- **What worked**: the verified decision and lesson.
- **Why it is relevant**: task overlap, stack compatibility, file/path overlap, confidence, recency, and scope.
- **What verified it**: tests, build, lint, CI, human approval, browser/API verification, or a custom verifier.

## OKX / A2MCP Capability

The first OKX-facing capability is **Engineering Context Recall**.

Request:

```json
{
  "task": "Refactor authentication middleware",
  "stack": ["typescript", "nextjs"],
  "files": ["src/auth.ts"]
}
```

Response:

```json
{
  "relevant_experience": [],
  "avoid": [],
  "recommended_checks": [],
  "confidence": 0.0
}
```

Value proposition:

> Before an AI coding agent starts from scratch, ask LoopLens what previous engineering experience is relevant.

## Architecture

```text
packages/core      Engineering memory model, storage, retrieval, ranking
packages/cli       Developer CLI over the core engine
packages/mcp       Agent-native JSON-RPC/stdio adapter
packages/service   Minimal HTTP service adapter for ASP-style calls
examples/demo-app  OKX-oriented interactive demo surface
.looplens          Local project memory and sample experiences
```

One core supports two deployment modes:

```text
Local coding-agent use:      Claude / Codex -> MCP -> LoopLens Local Server -> .looplens/
OKX service use:             OKX.AI -> HTTPS -> LoopLens Service -> LoopLens Core -> Store
```

## Install

```bash
cargo install --path packages/cli
```

Or run from the workspace:

```bash
cargo run -q -p looplens -- --help
```

## CLI Workflow

Initialize repository memory:

```bash
looplens init
```

Recall relevant engineering experience:

```bash
looplens recall \
  --task "Refactor login redirect" \
  --file src/auth.ts \
  --language typescript \
  --framework nextjs
```

Store a verified engineering experience:

```bash
looplens learn \
  --verified \
  --task "Login redirect refactor" \
  --type refactor \
  --hypothesis "Redirect behavior depends on session initialization" \
  --failed-attempt "Changed route matcher before checking session state" \
  --successful-decision "Initialize session before redirect evaluation" \
  --file src/auth/session.ts \
  --lesson "Check session initialization before modifying redirect rules." \
  --verification-source test \
  --verification-command "npm run test:e2e" \
  --agent code \
  --confidence 0.94
```

Inspect project context exposed to agents:

```bash
looplens project-context
```

Export agent-readable memory:

```bash
looplens export-loop
```

Legacy v1 flags such as `--problem`, `--patch`, and `--verified-pass` remain accepted for migration, but the v2 vocabulary is task and verification oriented.

## MCP Adapter

Run the stdio adapter:

```bash
cargo run -q -p looplens-mcp -- .
```

Supported JSON-RPC methods:

- `get_project_context`
- `recall_context`
- `record_attempt`
- `store_experience`

Example call:

```json
{"jsonrpc":"2.0","id":1,"method":"recall_context","params":{"task":"login CTA disappeared","files":["examples/demo-app/src/App.jsx"],"languages":["javascript"],"frameworks":["react"]}}
```

## HTTP Service

Run the service adapter:

```bash
PORT=8787 cargo run -q -p looplens-service
```

Endpoints:

- `GET /health`
- `GET /project_context`
- `POST /recall_context`
- `POST /store_experience`

Example:

```bash
curl -s http://127.0.0.1:8787/recall_context \
  -H 'content-type: application/json' \
  -d '{"task":"login CTA disappeared","stack":["javascript","react"],"files":["examples/demo-app/src/App.jsx"]}'
```

Deploy helpers are included:

- `Dockerfile` builds the HTTP service and includes `.looplens` sample memory.
- `render.yaml` defines a Render web service with `/health` as the health check.
- `packages/worker` provides a Cloudflare Workers adapter for free HTTPS deployment when Docker hosts require payment verification.
- [docs/okx-submission.md](docs/okx-submission.md) contains the ASP registration fields and endpoint checklist.

## Storage

LoopLens keeps project memory in boring, reviewable files:

```text
.looplens/
  project.toml
  experiences/
    exp-001.yaml
  trajectories/
    exp-001.md
  LOOP.md
```

`RepairExperience` from v1 has been generalized to `EngineeringExperience`. Existing v1 YAML is loaded and migrated in memory, including old verifier evidence.

## Verification

Generic verification evidence replaces verifier-specific evidence:

```yaml
verification:
  source: test
  result: passed
  command: npm run test:e2e
  reference: optional
```

Built-in sources include `test`, `build`, `lint`, `ci`, `human`, and `custom`. Legacy verifier sources are still readable for migration. Only `verified_success` experiences are treated as high-confidence reusable strategies; failed attempts are still useful as approaches to avoid.

## Demo App

Run the OKX-oriented demo locally:

```bash
cd examples/demo-app
npm install
npm run dev
```

Build it:

```bash
npm run build
```

## Current Limitations

- Retrieval is local and explainable, but does not use embeddings.
- `record_attempt` is exposed on the MCP surface, but durable attempt logs are compacted into `store_experience` for this MVP.
- The HTTP service is intentionally minimal and does not include production auth, accounts, or multi-tenant storage yet.
- Shared cross-project memory is a roadmap item; project memory remains authoritative.
