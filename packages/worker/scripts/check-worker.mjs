import { recallContext } from '../src/index.js';

const result = recallContext({
  task: 'login CTA disappeared',
  stack: ['javascript', 'react'],
  files: ['examples/demo-app/src/App.jsx'],
});

if (!Array.isArray(result.relevant_experience) || result.relevant_experience.length === 0) {
  throw new Error('expected at least one relevant experience');
}

if (!result.avoid.length || !result.recommended_checks.length || result.confidence <= 0) {
  throw new Error('expected avoid, recommended_checks, and positive confidence');
}

console.log(JSON.stringify(result, null, 2));

