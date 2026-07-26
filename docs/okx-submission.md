# OKX.AI ASP Submission Notes

## Service

Name:

```text
LoopLens Engineering Context Recall
```

Category:

```text
A2MCP / developer tooling / AI coding agents
```

Description:

```text
LoopLens returns evidence-backed engineering experience for AI coding-agent tasks, including relevant prior decisions, failed approaches to avoid, recommended checks, and confidence.
```

Price:

```text
0
```

Use the free direct-response endpoint for the first submission. x402 can be added later after the product surface is accepted.

## Public Endpoint

Deploy `packages/service` and submit this endpoint:

```text
POST https://YOUR_DOMAIN/recall_context
```

Health check:

```text
GET https://YOUR_DOMAIN/health
```

## Request

```json
{
  "task": "Refactor authentication middleware without breaking redirects",
  "stack": ["typescript", "nextjs"],
  "files": ["src/auth.ts"],
  "top_k": 3
}
```

## Response

```json
{
  "relevant_experience": [],
  "avoid": [],
  "recommended_checks": [],
  "confidence": 0.0
}
```

## Deploy Options

### Render

The repository includes `render.yaml` and `Dockerfile`.

1. Create a new Render Blueprint from this repository.
2. Render will build the Docker image and expose the service over HTTPS.
3. Use Render's generated domain as the ASP endpoint.

### Generic Docker Host

```bash
docker build -t looplens-service .
docker run --rm -p 8787:8787 -e PORT=8787 looplens-service
```

Then front it with HTTPS before registering with OKX.AI.

## Local Verification

```bash
PORT=8787 cargo run -q -p looplens-service
```

```bash
curl -i http://127.0.0.1:8787/health
```

```bash
curl -i http://127.0.0.1:8787/recall_context \
  -H 'content-type: application/json' \
  -d '{"task":"login CTA disappeared","stack":["javascript","react"],"files":["examples/demo-app/src/App.jsx"]}'
```

## Submission Checklist

- Public HTTPS endpoint responds with `HTTP 200` for valid `POST /recall_context` requests.
- Endpoint is reachable from outside the local network.
- `GET /health` returns `{"status":"ok"}`.
- ASP registration uses price `0` for MVP.
- README points reviewers to the endpoint, request shape, and demo flow.
- Demo video shows two sessions: first store verified experience, second recall related experience.

