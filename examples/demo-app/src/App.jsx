import React, { useMemo, useState } from 'react';
import { BrainCircuit, CheckCircle2, FileText, RotateCcw, Search, Sparkles, Terminal, Wrench } from 'lucide-react';
import { createRoot } from 'react-dom/client';
import './styles.css';

const memories = [
  {
    id: 'EXP-001',
    problem: 'Login flow failed after auth state rendering changed',
    hypothesis: 'Missing login button in browser verification',
    decision: 'Fix auth state rendering before editing selectors',
    lesson: 'When UI is missing, inspect state gating before changing test selectors.',
    score: 0.94,
  },
  {
    id: 'EXP-002',
    problem: 'Checkout CTA disabled after cart hydration',
    hypothesis: 'Button selector was stale',
    decision: 'Repair cart hydration default state',
    lesson: 'Hydration mismatches can masquerade as selector failures.',
    score: 0.87,
  },
  {
    id: 'EXP-003',
    problem: 'Settings save toast never appeared in regression run',
    hypothesis: 'Toast assertion timed out',
    decision: 'Restore async mutation success callback',
    lesson: 'Verify mutation lifecycle before adding waits.',
    score: 0.81,
  },
];

function App() {
  const [failure, setFailure] = useState('Login button missing after auth redirect. TestSprite says the login flow cannot find the expected CTA.');
  const [selected, setSelected] = useState(memories[0]);
  const [learned, setLearned] = useState(false);

  const ranked = useMemo(() => {
    const terms = failure.toLowerCase().split(/[^a-z0-9]+/).filter(Boolean);
    return memories
      .map((memory) => {
        const haystack = `${memory.problem} ${memory.hypothesis} ${memory.decision} ${memory.lesson}`.toLowerCase();
        const hits = terms.filter((term) => term.length > 2 && haystack.includes(term));
        return { ...memory, liveScore: Math.min(0.99, memory.score + hits.length * 0.015), hits };
      })
      .sort((a, b) => b.liveScore - a.liveScore);
  }, [failure]);

  return (
    <main className="shell">
      <section className="hero">
        <div className="heroCopy">
          <div className="kicker"><BrainCircuit size={18} /> Repair Experience Layer</div>
          <h1>LoopLens</h1>
          <p>Repository-scoped memory that helps AI coding agents reuse verified repair decisions after TestSprite finds a failure.</p>
        </div>
        <div className="terminalPanel" aria-label="LoopLens CLI preview">
          <div className="terminalTop"><Terminal size={18} /> looplens</div>
          <code>$ looplens recall --failure-bundle testsprite-failure.md</code>
          <code>$ looplens learn --verified PASS --lesson "Fix state before selectors"</code>
          <code>$ looplens export-loop</code>
        </div>
      </section>

      <section className="workspace">
        <div className="inputPanel">
          <label htmlFor="failure">TestSprite failure bundle</label>
          <textarea id="failure" value={failure} onChange={(event) => setFailure(event.target.value)} />
          <div className="flowStrip">
            <Step icon={<Search size={18} />} label="Recall" active />
            <Step icon={<Wrench size={18} />} label="Repair" active={selected.id === ranked[0].id} />
            <Step icon={<CheckCircle2 size={18} />} label="PASS" active={learned} />
            <Step icon={<FileText size={18} />} label="LOOP.md" active={learned} />
          </div>
        </div>

        <div className="memoryList">
          <div className="panelHeader"><Sparkles size={18} /> Similar verified repairs</div>
          {ranked.map((memory) => (
            <button
              className={`memoryItem ${selected.id === memory.id ? 'selected' : ''}`}
              key={memory.id}
              onClick={() => setSelected(memory)}
            >
              <span>{memory.id}</span>
              <strong>{memory.problem}</strong>
              <small>{memory.hits.length || 1} matched terms · score {memory.liveScore.toFixed(2)}</small>
            </button>
          ))}
        </div>

        <div className="decisionPanel">
          <div className="panelHeader"><BrainCircuit size={18} /> Repair context</div>
          <h2>{selected.decision}</h2>
          <p>{selected.lesson}</p>
          <div className="loopPreview">
            <span>LOOP.md entry</span>
            <code>Lesson: {selected.lesson}</code>
          </div>
          <div className="actions">
            <button className="secondary" onClick={() => setLearned(false)}><RotateCcw size={17} /> Reset</button>
            <button className="primary" onClick={() => setLearned(true)}><CheckCircle2 size={17} /> Learn PASS</button>
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

