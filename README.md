# LoopLens

![Hackathon](https://img.shields.io/badge/TestSprite-Hackathon%20S3-19C379?style=for-the-badge)
![Rust](https://img.shields.io/badge/Rust-CLI-b7410e?style=for-the-badge&logo=rust&logoColor=white)
![Local First](https://img.shields.io/badge/Local--first-Repository%20Memory-2d6a4f?style=for-the-badge)
![Verified](https://img.shields.io/badge/TestSprite-PASS-19C379?style=for-the-badge)

**Repair Experience Layer for AI Coding Agents.**

LoopLens is a local-first CLI that helps AI coding agents remember how a repository successfully repaired similar failures before. TestSprite tells the agent **what broke**. LoopLens preserves **how this repository got back to PASS**.

```text
TestSprite failure bundle
        -> looplens recall
        -> agent repair
        -> TestSprite PASS
        -> looplens learn
        -> reusable LOOP.md memory
```

## Why LoopLens

AI coding agents are good at debugging, but most repair loops are stateless. A failure appears, the agent reasons from scratch, patches the code, and then forgets the path that worked.

LoopLens turns verified repairs into repository memory:

- **Decision history**: what the agent tried and what finally worked.
- **Verified knowledge only**: experiences are stored after FAIL -> patch -> PASS.
- **AI-friendly recall**: future failures get relevant lessons instead of the whole history.
- **Git-native storage**: YAML and Markdown files that can be reviewed, diffed, committed, and rolled back.

## Hackathon Proof

- **Live demo**: https://demo-app-pink-omega.vercel.app
- **TestSprite status**: PASS
- **Run ID**: `7e9da0ed-e9a1-4cee-9a4d-92c272bd557e`
- **Test ID**: `1d52848a-4f5a-46af-a83f-f7cb9e9c0b29`
- **TestSprite dashboard**: https://www.testsprite.com/dashboard/tests/82a9909d-e588-4719-a9ba-53b957d12eb1/test/1d52848a-4f5a-46af-a83f-f7cb9e9c0b29

The demo app is a public surface for verification. The actual product is the CLI in `packages/cli`, powered by the core engine in `packages/core`.

## Product Boundary

| TestSprite | LoopLens |
| --- | --- |
| Verification layer | Repair experience layer |
| Failure bundle | Decision history |
| Browser/API testing | Repository repair memory |
| Answers "what failed?" | Answers "how did we fix this before?" |
| Produces PASS/FAIL evidence | Stores verified repair lessons |

LoopLens does not replace TestSprite. It completes the loop after verification by making successful repairs reusable.

## Install

```bash
cargo install --path packages/cli
```

Or run directly from the workspace:

```bash
cargo run -q -p looplens -- --help
```

## CLI Workflow

Initialize repository memory:

```bash
looplens init
```

Recall similar verified repairs from a TestSprite failure bundle:

```bash
looplens recall --failure-bundle .testsprite/failure-bundle.md
```

Store a new repair experience only after the final verification is PASS:

```bash
looplens learn \
  --problem "Login flow failed" \
  --testsprite-hypothesis "Missing login button" \
  --failed-attempt "Changed selector" \
  --successful-decision "Fix auth state rendering" \
  --patch app/login/page.tsx \
  --lesson "Check auth-state rendering before modifying selectors." \
  --confidence 0.94
```

Export the repository memory for agents and reviewers:

```bash
looplens export-loop
```

## Repository Memory

`looplens init` creates a local, repo-scoped memory store:

```text
.looplens/
  config.toml
  experiences/
    exp-001.yaml
  trajectories/
    exp-001.md
  LOOP.md
```

Experience files are intentionally boring: readable YAML, stable Markdown, no cloud account, no backend, no dashboard.

```yaml
id: EXP-001
problem: Login flow failed
testsprite_hypothesis: Missing login button
trajectory_summary:
  failed_attempts:
    - Added data-testid
    - Updated selector
  successful_decision: Fix auth state rendering
patches:
  - app/login/page.tsx
lesson: Check auth-state rendering before modifying selectors.
verified: PASS
confidence: 0.94
```

## Architecture

```text
packages/core      Repair Experience Engine
packages/cli       CLI adapter over the core engine
examples/demo-app  Public hackathon demo surface
.testsprite        TestSprite plan and run artifact
```

The core engine owns storage, retrieval, ranking, and LOOP export. The CLI is deliberately thin so the same engine can later power an MCP adapter.

## Demo App

Run the public demo locally:

```bash
cd examples/demo-app
npm install
npm run dev
```

Build it:

```bash
npm run build
```

## Verification

Commands already run for this submission:

```bash
cargo fmt --all -- --check
cargo test --workspace
npm --prefix examples/demo-app run build
testsprite test create --plan-from .testsprite/looplens-demo.plan.json --run --wait
```

The TestSprite plan lives at `.testsprite/looplens-demo.plan.json`, and the captured run output lives at `.testsprite/looplens-demo-run.json`.

## Roadmap

- MCP adapter for native agent access.
- Stronger local retrieval with embeddings.
- Repair trajectory compaction for long-running repositories.
- Cross-repository memory with provenance and confidence scoring.

