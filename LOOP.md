# LOOP.md

Agent-written repair loop notes for the LoopLens hackathon submission.

## Iteration 1 - Build the product core

- Goal: implement the PRD as a CLI-native repair experience layer, not a dashboard or TestSprite replacement.
- Commands run:
  - `cargo fmt --all -- --check`
  - `cargo test --workspace`
  - `npm --prefix examples/demo-app run build`
- Patch summary:
  - Added Rust workspace with `packages/core` and `packages/cli`.
  - Implemented `.looplens/` local memory with YAML experiences, trajectory Markdown, recall ranking, `learn`, and `export-loop`.
  - Added `examples/demo-app` as a public verification surface for TestSprite.
- Result: local CLI smoke test passed for `init -> learn -> recall -> export-loop`.
- Lesson: keep LoopLens scoped to repair decisions and repository memory; TestSprite remains the verification layer.

## Iteration 2 - Deploy and verify the public workflow

- Goal: provide a live URL that TestSprite can verify from the cloud.
- Live URL: `https://demo-app-pink-omega.vercel.app`
- TestSprite command:
  - `testsprite test create --plan-from .testsprite/looplens-demo.plan.json --run --wait`
- TestSprite result: PASS
- TestSprite project: `82a9909d-e588-4719-a9ba-53b957d12eb1`
- TestSprite test: `1d52848a-4f5a-46af-a83f-f7cb9e9c0b29`
- TestSprite run: `7e9da0ed-e9a1-4cee-9a4d-92c272bd557e`
- Dashboard: `https://www.testsprite.com/dashboard/tests/82a9909d-e588-4719-a9ba-53b957d12eb1/test/1d52848a-4f5a-46af-a83f-f7cb9e9c0b29`
- Patch summary:
  - Deployed the Vite demo app to Vercel.
  - Added `.testsprite/looplens-demo.plan.json` and captured the run output in `.testsprite/looplens-demo-run.json`.
  - Updated README with live URL, TestSprite proof, and demo video.
- Lesson: the public app should demonstrate the loop clearly, while README must state that the product itself is the CLI.

## Iteration 3 - Strengthen verified-memory semantics

- Goal: make verified knowledge explicit in the CLI and exported memory.
- Patch summary:
  - Added required `looplens learn --verified-pass` intent flag.
  - Added optional TestSprite evidence fields: run ID, test ID, target URL, dashboard URL, and verified timestamp.
  - Preserved compatibility with older experience YAML by defaulting missing evidence fields during load.
  - Updated `export-loop` output to include verification evidence.
  - Expanded tests for init layout, persistence reload, export evidence, legacy YAML loading, and invalid confidence.
- Verification:
  - `cargo fmt --all -- --check` passed.
  - `cargo test --workspace` passed with 6 tests.
  - CLI smoke test passed for `init -> learn --verified-pass -> recall -> export-loop` with TestSprite evidence fields.
  - `npm --prefix examples/demo-app run build` passed.
- Result: LoopLens now stores and exports verified repair evidence, not only lessons.
- Lesson: repair memory is more credible when every stored lesson carries verification evidence.
