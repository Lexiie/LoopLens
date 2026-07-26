const experiences = [
  {
    id: 'EXP-001',
    task: 'Login button missing after auth redirect in the public demo workflow',
    hypothesis: 'Browser verification could not find the expected CTA during the login workflow.',
    stack: ['javascript', 'react', 'vite'],
    files: ['examples/demo-app/src/App.jsx', 'examples/demo-app/src/styles.css'],
    failed_attempts: [
      'Treated the failure as a selector-only issue.',
      'Considered changing demo copy before checking workflow state.',
    ],
    successful_decision: 'Fix auth-state rendering before editing selectors.',
    lesson: 'When UI is missing in a browser verification run, inspect state gating and rendering conditions before changing selectors.',
    confidence: 0.94,
    verified_at: '2026-07-04T17:25:45Z',
  },
  {
    id: 'EXP-002',
    task: 'Engineering memory lacked reproducible verification evidence after a successful rerun',
    hypothesis: 'The repository memory did not yet carry Git and agent metadata for verified engineering work.',
    stack: ['rust', 'cli'],
    files: ['packages/core/src/lib.rs', 'packages/cli/src/main.rs', 'README.md', 'LOOP.md'],
    failed_attempts: [
      'Stored only external run and target URL evidence.',
      'Left commit, branch, agent, and changed files outside the engineering experience.',
    ],
    successful_decision: 'Add Git repair metadata to verification evidence and export it into LOOP.md.',
    lesson: 'Verified engineering memory is more reusable when verification evidence is paired with the Git commit, branch, agent, and files changed.',
    confidence: 0.96,
    verified_at: '2026-07-07T17:42:31Z',
  },
  {
    id: 'EXP-003',
    task: 'File-path-only recall queries did not return matching engineering experiences',
    hypothesis: 'Existing tests passed, but review found that patch/file recall metadata was not eligible when it was the only matching signal.',
    stack: ['rust', 'cli'],
    files: ['packages/core/src/lib.rs'],
    failed_attempts: [
      'Added patch/file score breakdown but kept lexical-only eligibility.',
      'Verified recall with text queries but not file-only queries.',
    ],
    successful_decision: 'Allow recall matches when lexical, hypothesis, or patch/file overlap is present.',
    lesson: 'Explainable recall must treat file overlap as a first-class matching signal, not just as a score decoration.',
    confidence: 0.91,
    verified_at: '2026-07-08T00:00:00Z',
  },
];

const corsHeaders = {
  'access-control-allow-origin': '*',
  'access-control-allow-methods': 'GET, POST, OPTIONS',
  'access-control-allow-headers': 'content-type',
};

export default {
  async fetch(request) {
    const url = new URL(request.url);

    if (request.method === 'OPTIONS') {
      return new Response(null, { status: 204, headers: corsHeaders });
    }

    if (request.method === 'GET' && url.pathname === '/health') {
      return json({ status: 'ok' });
    }

    if (request.method === 'GET' && url.pathname === '/project_context') {
      return json({
        name: 'LoopLens',
        languages: ['rust', 'javascript'],
        frameworks: ['mcp', 'react', 'vite', 'cloudflare-workers'],
        runtime: 'edge',
        package_manager: 'npm',
        test_frameworks: ['cargo test', 'vite build', 'worker smoke check'],
      });
    }

    if (request.method === 'POST' && url.pathname === '/recall_context') {
      let body;
      try {
        body = await request.json();
      } catch {
        return json({ error: 'invalid JSON body' }, 400);
      }
      return json(recallContext(body));
    }

    return json({ error: 'not found' }, 404);
  },
};

export function recallContext(input) {
  const task = String(input.task || '');
  const stack = [...asArray(input.stack), ...asArray(input.languages), ...asArray(input.frameworks)];
  const files = asArray(input.files);
  const topK = Number.isInteger(input.top_k) && input.top_k > 0 ? input.top_k : 3;

  const taskTokens = tokenize(task);
  const stackTokens = tokenize(stack.join(' '));
  const fileTokens = tokenize(files.join(' '));

  const matches = experiences
    .map((experience) => {
      const textTokens = tokenize(`${experience.task} ${experience.hypothesis} ${experience.successful_decision} ${experience.lesson}`);
      const experienceStackTokens = tokenize(experience.stack.join(' '));
      const experienceFileTokens = tokenize(experience.files.join(' '));

      const matchedTerms = overlap(taskTokens, textTokens);
      const matchedStack = overlap(stackTokens, experienceStackTokens);
      const matchedFiles = overlap(fileTokens, experienceFileTokens);
      const taskSimilarity = ratio(matchedTerms.length, Math.max(taskTokens.size, 1));
      const stackMatch = ratio(matchedStack.length, Math.max(stackTokens.size, 1));
      const fileMatch = ratio(matchedFiles.length, Math.max(fileTokens.size, 1));
      const recency = recencyScore(experience.verified_at);
      const score = taskSimilarity * 0.35 + stackMatch * 0.2 + fileMatch * 0.2 + experience.confidence * 0.15 + recency * 0.1;

      return {
        lesson: experience.lesson,
        successful_decision: experience.successful_decision,
        failed_attempts: experience.failed_attempts,
        confidence: experience.confidence,
        reason: reasons(matchedTerms, matchedStack, matchedFiles),
        score,
        experience: {
          id: experience.id,
          task: experience.task,
          stack: experience.stack,
          files: experience.files,
        },
      };
    })
    .filter((match) => match.score > 0.15 || match.reason.length > 0)
    .sort((left, right) => right.score - left.score)
    .slice(0, topK);

  return {
    relevant_experience: matches,
    avoid: unique(matches.flatMap((match) => match.failed_attempts)),
    recommended_checks: unique(matches.map((match) => match.lesson)),
    confidence: matches[0]?.score || 0,
  };
}

function json(body, status = 200) {
  return new Response(JSON.stringify(body, null, 2), {
    status,
    headers: {
      ...corsHeaders,
      'content-type': 'application/json',
    },
  });
}

function asArray(value) {
  if (Array.isArray(value)) {
    return value.map(String);
  }
  if (typeof value === 'string' && value.trim()) {
    return [value];
  }
  return [];
}

function tokenize(text) {
  return new Set(
    String(text)
      .toLowerCase()
      .split(/[^a-z0-9]+/)
      .filter((token) => token.length > 2),
  );
}

function overlap(left, right) {
  return [...left].filter((token) => right.has(token)).sort();
}

function ratio(numerator, denominator) {
  return denominator === 0 ? 0 : numerator / denominator;
}

function recencyScore(verifiedAt) {
  const ageMs = Math.max(0, Date.now() - Date.parse(verifiedAt));
  const ageDays = ageMs / 86_400_000;
  return Math.min(1, 1 / (1 + ageDays / 90));
}

function reasons(task, stack, files) {
  const reason = [];
  if (task.length) reason.push(`task overlap: ${task.join(', ')}`);
  if (stack.length) reason.push(`stack overlap: ${stack.join(', ')}`);
  if (files.length) reason.push(`file/path overlap: ${files.join(', ')}`);
  if (reason.length) reason.push('verified successful outcome');
  return reason;
}

function unique(values) {
  return [...new Set(values)];
}

