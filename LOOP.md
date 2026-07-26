# LOOP.md

LoopLens is persistent engineering memory for AI coding agents.

Use this repository memory to recall previous engineering decisions before starting a related task. Verification can come from tests, builds, lint, CI, local browser/API checks, human approval, or a custom verifier.

## Agent Workflow

1. Call `recall_context` with the task, stack, and files you expect to touch.
2. Review relevant lessons, successful decisions, and failed attempts to avoid.
3. Do the engineering work.
4. Verify the outcome with the appropriate verifier.
5. Call `store_experience` after verified success.

## Current Product Positioning

LoopLens is not a coding agent and not a verification platform.

LoopLens answers:

> What engineering experience from previous work is relevant to what I am doing now?

## Memory Shape

```yaml
task:
  summary: Refactor authentication middleware
  type: refactor
trajectory:
  failed_attempts:
    - Changed route matcher before checking session state
  successful_decision: Initialize session before redirect evaluation
lesson: Check session initialization before modifying redirect rules.
verification:
  source: test
  result: passed
  command: npm run test:e2e
```

## Useful Commands

```bash
cargo run -q -p looplens -- recall \
  --task "login CTA disappeared" \
  --file examples/demo-app/src/App.jsx \
  --language javascript \
  --framework react
```

```bash
cargo run -q -p looplens-mcp -- .
```

```bash
PORT=8787 cargo run -q -p looplens-service
```

