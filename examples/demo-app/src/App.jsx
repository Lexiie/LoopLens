import React, { useMemo, useState } from 'react';
import {
  BrainCircuit,
  CheckCircle2,
  Database,
  FileCode2,
  Network,
  RotateCcw,
  Search,
  ServerCog,
  ShieldCheck,
  Sparkles,
  Terminal,
} from 'lucide-react';
import { createRoot } from 'react-dom/client';
import './styles.css';

const experiences = [
  {
    id: 'EXP-001',
    task: 'Login flow failed after auth state rendering changed',
    stack: ['react', 'vite'],
    files: ['examples/demo-app/src/App.jsx'],
    avoid: ['Treating the failure as selector-only before checking render state'],
    decision: 'Fix auth state rendering before editing selectors',
    lesson: 'When UI is missing, inspect state gating before changing test selectors.',
    score: 0.94,
  },
  {
    id: 'EXP-002',
    task: 'Checkout CTA disabled after cart hydration',
    stack: ['react', 'node'],
    files: ['app/cart/page.tsx'],
    avoid: ['Adding waits before confirming hydration defaults'],
    decision: 'Repair cart hydration default state',
    lesson: 'Hydration mismatches can masquerade as selector failures.',
    score: 0.87,
  },
  {
    id: 'EXP-003',
    task: 'Settings save toast never appeared after mutation refactor',
    stack: ['typescript', 'react-query'],
    files: ['src/settings/mutation.ts'],
    avoid: ['Increasing timeout before checking mutation lifecycle'],
    decision: 'Restore async mutation success callback',
    lesson: 'Verify mutation lifecycle before adding waits.',
    score: 0.81,
  },
];

function App() {
  const [task, setTask] = useState('Refactor the login redirect flow without breaking the CTA render path.');
  const [stack, setStack] = useState('react vite');
  const [selected, setSelected] = useState(experiences[0]);
  const [stored, setStored] = useState(false);

  const ranked = useMemo(() => {
    const terms = `${task} ${stack}`.toLowerCase().split(/[^a-z0-9]+/).filter((term) => term.length > 2);
    return experiences
      .map((experience) => {
        const haystack = `${experience.task} ${experience.stack.join(' ')} ${experience.files.join(' ')} ${experience.decision} ${experience.lesson}`.toLowerCase();
        const hits = terms.filter((term) => haystack.includes(term));
        return { ...experience, liveScore: Math.min(0.99, experience.score + hits.length * 0.012), hits };
      })
      .sort((a, b) => b.liveScore - a.liveScore);
  }, [task, stack]);

  return (
    <main className="shell">
      <section className="hero">
        <div className="heroCopy">
          <div className="kicker"><Network size={18} /> OKX.AI Genesis / A2MCP</div>
          <h1>LoopLens</h1>
          <p>Persistent engineering memory that gives coding agents evidence-backed experience before they start from scratch.</p>
        </div>
        <div className="terminalPanel" aria-label="LoopLens A2MCP preview">
          <div className="terminalTop"><Terminal size={18} /> recall_context</div>
          <code>{JSON.stringify({ task, stack: stack.split(' '), files: ['src/auth.ts'] }, null, 2)}</code>
        </div>
      </section>

      <section className="workspace" aria-label="LoopLens engineering memory demo">
        <div className="inputPanel">
          <div className="panelHeader"><FileCode2 size={18} /> Agent task context</div>
          <label htmlFor="task">Task</label>
          <textarea id="task" value={task} onChange={(event) => setTask(event.target.value)} />
          <label htmlFor="stack">Stack</label>
          <input id="stack" value={stack} onChange={(event) => setStack(event.target.value)} />
          <div className="flowStrip">
            <Step icon={<Search size={18} />} label="Recall" active />
            <Step icon={<BrainCircuit size={18} />} label="Work" active={selected.id === ranked[0].id} />
            <Step icon={<ShieldCheck size={18} />} label="Verify" active={stored} />
            <Step icon={<Database size={18} />} label="Store" active={stored} />
          </div>
        </div>

        <div className="memoryList">
          <div className="panelHeader"><Sparkles size={18} /> Relevant experience</div>
          {ranked.map((experience) => (
            <button
              className={`memoryItem ${selected.id === experience.id ? 'selected' : ''}`}
              key={experience.id}
              onClick={() => setSelected(experience)}
            >
              <span>{experience.id}</span>
              <strong>{experience.task}</strong>
              <small>{experience.hits.length || 1} matching signals / confidence {experience.liveScore.toFixed(2)}</small>
            </button>
          ))}
        </div>

        <div className="decisionPanel">
          <div className="panelHeader"><ServerCog size={18} /> Returned context</div>
          <h2>{selected.decision}</h2>
          <p>{selected.lesson}</p>
          <div className="contextGrid">
            <div>
              <span>Avoid</span>
              <p>{selected.avoid[0]}</p>
            </div>
            <div>
              <span>Scope</span>
              <p>project + stack memory</p>
            </div>
          </div>
          <div className="loopPreview">
            <span>store_experience</span>
            <code>outcome: verified_success{`\n`}lesson: {selected.lesson}</code>
          </div>
          <div className="actions">
            <button className="secondary" onClick={() => setStored(false)}><RotateCcw size={17} /> Reset</button>
            <button className="primary" onClick={() => setStored(true)}><CheckCircle2 size={17} /> Store Verified</button>
          </div>
        </div>
      </section>
    </main>
  );
}

function Step({ icon, label, active }) {
  return <div className={`step ${active ? 'active' : ''}`}>{icon}<span>{label}</span></div>;
}

createRoot(document.getElementById('root')).render(<App />);

