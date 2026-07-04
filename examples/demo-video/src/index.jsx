import React from 'react';
import {
  AbsoluteFill,
  Composition,
  interpolate,
  Sequence,
  spring,
  useCurrentFrame,
  useVideoConfig,
  registerRoot,
} from 'remotion';
import {
  BrainCircuit,
  CheckCircle2,
  FileText,
  Search,
  Sparkles,
  Terminal,
  Wrench,
} from 'lucide-react';
import './style.css';

const FPS = 30;

function LoopLensDemo() {
  return (
    <AbsoluteFill className="videoRoot">
      <Sequence from={0} durationInFrames={150}>
        <Opening />
      </Sequence>
      <Sequence from={135} durationInFrames={210}>
        <Problem />
      </Sequence>
      <Sequence from={330} durationInFrames={270}>
        <Workflow />
      </Sequence>
      <Sequence from={585} durationInFrames={240}>
        <CliScene />
      </Sequence>
      <Sequence from={810} durationInFrames={210}>
        <Proof />
      </Sequence>
      <Sequence from={1005} durationInFrames={150}>
        <Closing />
      </Sequence>
    </AbsoluteFill>
  );
}

function Opening() {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const scale = spring({ frame, fps, config: { damping: 18, stiffness: 90 } });
  const fade = interpolate(frame, [0, 24, 126, 150], [0, 1, 1, 0], { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' });

  return (
    <Scene style={{ opacity: fade }}>
      <div className="heroGrid">
        <div className="heroText" style={{ transform: `scale(${0.94 + scale * 0.06})` }}>
          <Badge icon={<BrainCircuit />} label="Repair Experience Layer" />
          <h1>LoopLens</h1>
          <p>Repository memory for AI coding agents after TestSprite finds what broke.</p>
        </div>
        <TerminalCard lines={['$ looplens recall --failure-bundle failure.md', '$ looplens learn --verified PASS', '$ looplens export-loop']} />
      </div>
    </Scene>
  );
}

function Problem() {
  const frame = useCurrentFrame();
  const opacity = interpolate(frame, [0, 25, 180, 210], [0, 1, 1, 0], { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' });
  const y = interpolate(frame, [0, 35], [40, 0], { extrapolateRight: 'clamp' });

  return (
    <Scene style={{ opacity }}>
      <div className="splitScene" style={{ transform: `translateY(${y}px)` }}>
        <div>
          <Badge icon={<Sparkles />} label="The gap" />
          <h2>Agents fix bugs, then forget the repair.</h2>
          <p className="lead">Every new failure becomes fresh reasoning: repeated attempts, repeated tokens, repeated mistakes.</p>
        </div>
        <div className="compareBox">
          <div className="compareRow muted"><span>FAIL</span><p>Login CTA missing in browser verification</p></div>
          <div className="compareRow muted"><span>TRY</span><p>Changed selector and added test id</p></div>
          <div className="compareRow pass"><span>PASS</span><p>Fixed auth-state rendering</p></div>
          <div className="memoryCallout">LoopLens stores the verified decision, not noisy logs.</div>
        </div>
      </div>
    </Scene>
  );
}

function Workflow() {
  const frame = useCurrentFrame();
  const opacity = interpolate(frame, [0, 20, 238, 270], [0, 1, 1, 0], { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' });
  const active = Math.min(3, Math.floor(frame / 58));
  const learned = frame > 185;

  return (
    <Scene style={{ opacity }}>
      <div className="appScene">
        <div className="appHeader">
          <div>
            <Badge icon={<Search />} label="Live demo" />
            <h2>Recall similar verified repairs before patching.</h2>
          </div>
          <span>demo-app-pink-omega.vercel.app</span>
        </div>
        <div className="appPanels">
          <div className="panel">
            <label>TestSprite failure bundle</label>
            <div className="textarea">Login button missing after auth redirect. TestSprite says the login flow cannot find the expected CTA.</div>
            <div className="steps">
              <Step icon={<Search />} label="Recall" active={active >= 0} />
              <Step icon={<Wrench />} label="Repair" active={active >= 1} />
              <Step icon={<CheckCircle2 />} label="PASS" active={active >= 2 || learned} />
              <Step icon={<FileText />} label="LOOP.md" active={active >= 3 || learned} />
            </div>
          </div>
          <div className="panel memory">
            <h3><Sparkles size={22} /> Similar verified repairs</h3>
            <Memory selected id="EXP-001" title="Login flow failed after auth state rendering changed" meta="score 0.99" />
            <Memory id="EXP-002" title="Checkout CTA disabled after cart hydration" meta="score 0.92" />
          </div>
          <div className="panel decision">
            <h3><BrainCircuit size={22} /> Repair context</h3>
            <h4>Fix auth state rendering before editing selectors</h4>
            <p>When UI is missing, inspect state gating before changing test selectors.</p>
            <div className={learned ? 'learnButton learned' : 'learnButton'}><CheckCircle2 size={22} /> Learn PASS</div>
          </div>
        </div>
      </div>
    </Scene>
  );
}

function CliScene() {
  const frame = useCurrentFrame();
  const opacity = interpolate(frame, [0, 20, 210, 240], [0, 1, 1, 0], { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' });
  const highlight = Math.min(3, Math.floor(frame / 56));

  return (
    <Scene style={{ opacity }}>
      <div className="cliGrid">
        <div>
          <Badge icon={<Terminal />} label="CLI native" />
          <h2>Local-first memory that lives with the repo.</h2>
          <p className="lead">No dashboard, no backend, no auth. The repair experience is reviewable YAML and Markdown.</p>
        </div>
        <div className="bigTerminal">
          {[
            '$ looplens init',
            '$ looplens recall --failure-bundle .testsprite/failure.md',
            '$ looplens learn --problem "Login flow failed" --confidence 0.94',
            '$ looplens export-loop',
          ].map((line, index) => <code className={highlight === index ? 'codeActive' : ''} key={line}>{line}</code>)}
        </div>
      </div>
    </Scene>
  );
}

function Proof() {
  const frame = useCurrentFrame();
  const opacity = interpolate(frame, [0, 20, 180, 210], [0, 1, 1, 0], { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' });

  return (
    <Scene style={{ opacity }}>
      <div className="proofScene">
        <Badge icon={<CheckCircle2 />} label="Verified in public" />
        <h2>TestSprite PASS</h2>
        <div className="proofCards">
          <InfoCard label="Live URL" value="demo-app-pink-omega.vercel.app" />
          <InfoCard label="Run ID" value="7e9da0ed-e9a1-4cee-9a4d-92c272bd557e" />
          <InfoCard label="Test ID" value="1d52848a-4f5a-46af-a83f-f7cb9e9c0b29" />
        </div>
      </div>
    </Scene>
  );
}

function Closing() {
  const frame = useCurrentFrame();
  const opacity = interpolate(frame, [0, 20, 130, 150], [0, 1, 1, 0], { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' });

  return (
    <Scene style={{ opacity }}>
      <div className="closing">
        <Badge icon={<FileText />} label="Open source" />
        <h2>TestSprite teaches agents what failed.</h2>
        <h2 className="green">LoopLens teaches how this repo fixed it.</h2>
        <p>github.com/Lexiie/LoopLens</p>
      </div>
    </Scene>
  );
}

function Scene({ children, style }) {
  return <AbsoluteFill className="scene" style={style}>{children}</AbsoluteFill>;
}

function Badge({ icon, label }) {
  return <div className="badge">{React.cloneElement(icon, { size: 22 })}<span>{label}</span></div>;
}

function TerminalCard({ lines }) {
  return <div className="terminal"><div className="terminalTitle"><Terminal size={21} /> looplens</div>{lines.map((line) => <code key={line}>{line}</code>)}</div>;
}

function Step({ icon, label, active }) {
  return <div className={active ? 'step active' : 'step'}>{React.cloneElement(icon, { size: 22 })}<span>{label}</span></div>;
}

function Memory({ id, title, meta, selected }) {
  return <div className={selected ? 'memoryItem selected' : 'memoryItem'}><span>{id}</span><strong>{title}</strong><small>{meta}</small></div>;
}

function InfoCard({ label, value }) {
  return <div className="infoCard"><span>{label}</span><strong>{value}</strong></div>;
}

function RemotionRoot() {
  return <Composition id="LoopLensDemo" component={LoopLensDemo} durationInFrames={1155} fps={FPS} width={1920} height={1080} />;
}

registerRoot(RemotionRoot);
