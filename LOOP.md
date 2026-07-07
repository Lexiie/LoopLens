# LOOP.md

This is the hackathon-facing LoopLens memory artifact for a Verified Repair Experience Engine.

It shows the kind of repair context an AI coding agent should get before making the next patch. The committed `.looplens/LOOP.md` is the generated repository memory that LoopLens would carry inside any project using the CLI.

## Agent Contract

Before patching a new failure in this repository, an agent should:

1. Read the TestSprite failure bundle.
2. Run `looplens recall` with the failure text or bundle.
3. Compare recalled lessons against the current failure.
4. Patch only after choosing a repair strategy.
5. Run TestSprite again.
6. Store a new experience only after PASS with `looplens learn --verified-pass`.
7. Regenerate `.looplens/LOOP.md` with `looplens export-loop`.

LoopLens should not store failed guesses as durable knowledge. It stores verified repair decisions.

## Why This Is Not TestSprite

TestSprite produces evidence that a behavior failed or passed. LoopLens consumes that evidence after a PASS and turns the successful repair into repository-specific agent knowledge.

- TestSprite answers: what failed, what passed, and what evidence proves it.
- LoopLens answers: how did this repository successfully repair a similar problem before?
- TestSprite owns verification.
- LoopLens owns reusable repair experience.

## Current Verified Memory

This repository carries three sample experiences in `.looplens/experiences/` so a fresh clone can demonstrate recall immediately.

### EXP-001 - Missing UI after auth redirect

Problem:
Login button was missing after an auth redirect in the public demo workflow.

TestSprite hypothesis:
The browser verification could not find the expected CTA.

Failed attempts:
- Treated the failure as a selector-only issue.
- Considered changing demo copy before checking workflow state.

Successful decision:
Fix auth-state rendering before editing selectors.

Reusable lesson:
When UI is missing in a browser verification run, inspect state gating and rendering conditions before changing selectors.

Evidence:
- TestSprite status: PASS
- TestSprite project: `82a9909d-e588-4719-a9ba-53b957d12eb1`
- TestSprite test: `1d52848a-4f5a-46af-a83f-f7cb9e9c0b29`
- TestSprite run: `7e9da0ed-e9a1-4cee-9a4d-92c272bd557e`
- Target URL: `https://demo-app-pink-omega.vercel.app`
- Commit: `7545ad24c4684fb408122e770846a445edd8f8a8`
- Branch: `main`
- Agent: `code`
- Files changed: `examples/demo-app/src/App.jsx`, `examples/demo-app/src/styles.css`
- Dashboard: `https://www.testsprite.com/dashboard/tests/82a9909d-e588-4719-a9ba-53b957d12eb1/test/1d52848a-4f5a-46af-a83f-f7cb9e9c0b29`
- Confidence: `0.94`

Agent guidance for similar failures:
- Check conditional rendering and state transitions before changing selectors.
- Treat missing UI as a possible app-state bug, not automatically as a test locator bug.
- Preserve the verification trail when learning from the final PASS.

## Recall Example

Command:

```bash
cargo run -q -p looplens -- recall --problem "auth login button missing"
```

Expected repair context:

```text
Similar repair: EXP-001
Why matched:
- lexical: auth, button, login, missing
- hypothesis overlap: login
- score breakdown: lexical, patch/file, hypothesis, confidence, recency
Previous decision: Fix auth-state rendering before editing selectors.
Lesson learned: When UI is missing in a browser verification run, inspect state gating and rendering conditions before changing selectors.
Candidate strategy: inspect app state gating before modifying selectors.
```

This is the core product behavior: the agent gets repository-specific repair experience before it spends tokens rediscovering the same lesson.

Additional sample memories cover:
- `EXP-002`: preserving Git and TestSprite evidence with the repair lesson.
- `EXP-003`: fixing patch/file-only recall eligibility so file paths are first-class recall signals.

## Learning Example

After a new TestSprite PASS, the agent should store memory with explicit evidence:

```bash
looplens learn \
  --verified-pass \
  --problem "Login flow failed" \
  --testsprite-hypothesis "Missing login button" \
  --failed-attempt "Changed selector" \
  --successful-decision "Fix auth state rendering" \
  --patch app/login/page.tsx \
  --lesson "Check auth-state rendering before modifying selectors." \
  --testsprite-run-id "7e9da0ed-e9a1-4cee-9a4d-92c272bd557e" \
  --test-id "1d52848a-4f5a-46af-a83f-f7cb9e9c0b29" \
  --target-url "https://demo-app-pink-omega.vercel.app" \
  --dashboard-url "https://www.testsprite.com/dashboard/tests/82a9909d-e588-4719-a9ba-53b957d12eb1/test/1d52848a-4f5a-46af-a83f-f7cb9e9c0b29" \
  --agent "code" \
  --file-changed "app/login/page.tsx" \
  --confidence 0.94
```

Then export:

```bash
looplens export-loop
```

## Hackathon Build Loop

### Iteration 1 - Product core

Commands:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `npm --prefix examples/demo-app run build`

Patch summary:
- Added Rust workspace with `packages/core` and `packages/cli`.
- Implemented local `.looplens/` memory with YAML experiences, trajectory Markdown, recall ranking, `learn`, and `export-loop`.
- Added `examples/demo-app` as a public TestSprite verification surface.

Result:
Local CLI smoke test passed for `init -> learn -> recall -> export-loop`.

Lesson:
Keep LoopLens scoped to repair decisions and repository memory. TestSprite remains the verification layer.

### Iteration 2 - Public verification

Command:

```bash
testsprite test create --plan-from .testsprite/looplens-demo.plan.json --run --wait
```

Result:
PASS against `https://demo-app-pink-omega.vercel.app`.

Patch summary:
- Deployed the Vite demo app to Vercel.
- Added `.testsprite/looplens-demo.plan.json`.
- Captured run output in `.testsprite/looplens-demo-run.json`.
- Added playable demo video and README proof.

Lesson:
The public app should demonstrate the loop, while the CLI remains the product.

### Iteration 3 - Verified evidence semantics

Commands:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- CLI smoke test for `init -> learn --verified-pass -> recall -> export-loop`
- `npm --prefix examples/demo-app run build`

Patch summary:
- Added required `looplens learn --verified-pass` intent flag.
- Added optional TestSprite evidence fields: run ID, test ID, target URL, dashboard URL, and verified timestamp.
- Added Git repair metadata: commit SHA, branch, agent, and changed files.
- Added explainable recall scoring across lexical, hypothesis, patch/file, confidence, and recency signals.
- Preserved compatibility with older experience YAML by defaulting missing evidence fields during load.
- Updated `export-loop` output to include verification evidence.
- Expanded tests for init layout, persistence reload, export evidence, legacy YAML loading, and invalid confidence.

Result:
LoopLens now stores and exports verified repair evidence, not only lessons.

Lesson:
Repair memory is more credible when every stored lesson carries verification evidence.

### Iteration 4 - Explainable recall and final rerun

Commands:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- CLI smoke test for explainable `recall`
- `testsprite test rerun 1d52848a-4f5a-46af-a83f-f7cb9e9c0b29 --wait --timeout 240`
- `testsprite test wait 9eb5fd97-b3f0-4da6-9f30-52deb51c5247 --timeout 120`

Patch summary:
- Reframed LoopLens as a Verified Repair Experience Engine.
- Added explainable recall output with lexical, hypothesis, patch/file, confidence, and recency scoring signals.
- Added Git repair metadata to evidence: commit SHA, branch, agent, and changed files.
- Updated sample `.looplens/` memory so fresh clones can recall a verified repair immediately.

Result:
TestSprite rerun `9eb5fd97-b3f0-4da6-9f30-52deb51c5247` passed against `https://demo-app-pink-omega.vercel.app` with 28/28 steps passed.

Lesson:
Recall is more useful to agents when the engine explains why an experience matched, not only that it matched.

### Iteration 5 - Submission hardening

Patch summary:
- Added GitHub Actions CI for Rust formatting, Rust tests, demo app install, and demo app build.
- Added a manual TestSprite verification workflow powered by `TESTSPRITE_API_KEY`.
- Added `EXP-002` and `EXP-003` sample verified repair memories.
- Added Known Limitations so reviewers can distinguish MVP scope from roadmap.

Lesson:
For a hackathon developer tool, submission evidence matters as much as raw feature count: CI, sample memory, and honest limitations make the project easier to judge.
