import { useEffect, useState } from "react";
import {
  Home, Layers3, Puzzle, Settings, ChevronDown, Play, Plus,
  Monitor, Cpu, HardDrive, Sparkles, ShieldCheck, CircleHelp,
  Download, Search, SlidersHorizontal
} from "lucide-react";

type Page = "Home" | "Instances" | "Mods" | "Settings";

const nav: { label: Page; icon: typeof Home }[] = [
  { label: "Home", icon: Home },
  { label: "Instances", icon: Layers3 },
  { label: "Mods", icon: Puzzle },
  { label: "Settings", icon: Settings }
];

function App() {
  const [page, setPage] = useState<Page>("Home");
  const [version, setVersion] = useState("1.21.4");
  const [ram, setRam] = useState(4);
  const [notice, setNotice] = useState("Your next adventure starts here.");
  const [instances, setInstances] = useState<CrescentInstance[]>([]);
  const [instanceBusy, setInstanceBusy] = useState(false);

  useEffect(() => {
    if (!window.crescent) return;
    void Promise.all([window.crescent.listInstances(), window.crescent.getSettings(), window.crescent.getVersions()])
      .then(([savedInstances, settings, manifest]) => {
        setInstances(savedInstances);
        setRam(settings.ramGb);
        setVersion(manifest.latest.release);
      })
      .catch(() => setNotice("Some local services could not be loaded. You can still explore the interface."));
  }, []);

  const createInstance = async () => {
    if (!window.crescent || instanceBusy) return;
    const name = window.prompt("Name your new Minecraft instance:", "My Crescent World");
    if (!name?.trim()) return;
    setInstanceBusy(true);
    try {
      const created = await window.crescent.createInstance({ name, version });
      setInstances(current => [created, ...current]);
      setNotice(`Instance “${created.name}” was created locally.`);
    } catch (error) {
      setNotice(error instanceof Error ? error.message : "Could not create the instance.");
    } finally { setInstanceBusy(false); }
  };

  const announce = (message: string) => setNotice(message);

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark"><span>☾</span><i>✦</i></div>
          <div><strong>CRESCENT</strong><small>CLIENT</small></div>
        </div>

        <div className="nav-label">WORKSPACE</div>
        <nav>
          {nav.map(({ label, icon: Icon }) => (
            <button key={label} className={`nav-item ${page === label ? "active" : ""}`} onClick={() => setPage(label)}>
              <Icon size={18} strokeWidth={1.8} /><span>{label}</span>
              {page === label && <span className="active-dot" />}
            </button>
          ))}
        </nav>

        <div className="sidebar-bottom">
          <div className="profile-card">
            <div className="avatar">X</div>
            <div className="profile-copy"><strong>Guest Profile</strong><small>Offline account</small></div>
            <span className="online-dot" />
          </div>
          <div className="version-tag"><span className="tiny-dot" /> CRESCENT 0.1.0 <span>EARLY ACCESS</span></div>
        </div>
      </aside>

      <section className="main-area">
        <header className="topbar">
          <div className="breadcrumb"><span>Workspace</span><b>/</b><strong>{page}</strong></div>
          <div className="top-actions">
            <button className="icon-button" aria-label="Help" onClick={() => announce("Help and documentation are not connected yet.")}><CircleHelp size={18} /></button>
            <div className="top-divider" />
            <button className="account-button" onClick={() => announce("Account sign-in will be added in a future update.")}>
              <span className="avatar small">X</span><span>Guest</span><ChevronDown size={14} />
            </button>
          </div>
        </header>

        {page === "Home" && (
          <div className="page-content">
            <div className="hero">
              <div className="hero-orbit orbit-one" /><div className="hero-orbit orbit-two" />
              <div className="hero-copy">
                <div className="eyebrow"><Sparkles size={13} /> THE NEXT CHAPTER OF MINECRAFT</div>
                <h1>Beyond the<br /><em>Moon.</em></h1>
                <p>A calmer, cleaner way to experience your worlds.<br />Your journey begins with Crescent.</p>
                <div className="hero-actions">
                  <button className="primary-button" onClick={() => announce("Game launching is not available yet. Instance setup is now working locally.")}><Play size={16} fill="currentColor" /> Play Minecraft</button>
                  <button className="secondary-button" onClick={() => setPage("Instances")}>Manage instances <ChevronDown size={15} /></button>
                </div>
              </div>
              <div className="moon-scene" aria-hidden="true">
                <div className="moon-halo" /><div className="moon"><div className="moon-shadow" /></div>
                <div className="star star-a">✦</div><div className="star star-b">✧</div><div className="star star-c">·</div>
                <div className="moon-caption">CRESCENT <span>01 / 04</span></div>
              </div>
              <div className="hero-foot"><span>DESIGNED FOR YOUR NEXT ADVENTURE</span><span>EST. 2026</span></div>
            </div>

            <div className="section-heading"><div><span className="section-kicker">YOUR SPACE</span><h2>Quick setup</h2></div><span className="muted-label">PERSONALIZE YOUR SESSION</span></div>
            <div className="setup-grid">
              <div className="panel version-panel">
                <div className="panel-top"><div className="panel-icon"><Layers3 size={17} /></div><span>GAME VERSION</span><span className="panel-status">SELECTED</span></div>
                <label htmlFor="version">Minecraft version</label>
                <div className="select-wrap"><select id="version" value={version} onChange={e => setVersion(e.target.value)}><option>1.21.4</option><option>1.21.3</option><option>1.20.6</option><option>1.20.4</option><option>1.19.4</option></select><ChevronDown size={15} /></div>
                <div className="panel-foot"><span className="tiny-dot" /> Vanilla profile <span className="spacer" /> Change loader in Instances</div>
              </div>

              <div className="panel ram-panel">
                <div className="panel-top"><div className="panel-icon"><Cpu size={17} /></div><span>MEMORY ALLOCATION</span><span className="panel-status">TUNING</span></div>
                <div className="ram-value">{ram}<small> GB</small></div>
                <input aria-label="Memory allocation" type="range" min="2" max="8" step="1" value={ram} onChange={e => { const value = Number(e.target.value); setRam(value); void window.crescent?.saveSettings({ ramGb: value }).catch(() => setNotice("Could not save RAM settings.")); }} />
                <div className="range-labels"><span>2 GB</span><span>8 GB</span></div>
                <div className="panel-foot"><HardDrive size={13} /> Adjust based on your system memory</div>
              </div>

              <div className="panel status-panel">
                <div className="panel-top"><div className="panel-icon"><ShieldCheck size={17} /></div><span>CLIENT STATUS</span></div>
                <div className="status-title">Ready to configure</div>
                <p>Your launcher workspace is set up. Game services are not connected yet.</p>
                <div className="status-bottom"><span className="status-ring" /> LOCAL WORKSPACE <span className="spacer" /> <span className="muted-label">OFFLINE</span></div>
              </div>
            </div>
            <div className="notice"><Sparkles size={15} /><span>{notice}</span><button onClick={() => setNotice("Your next adventure starts here.")}>DISMISS</button></div>
          </div>
        )}

        {page === "Instances" && (
          <div className="page-content simple-page">
            <div className="section-kicker">GAME MANAGEMENT</div><h1 className="page-title">Your instances<span>.</span></h1>
            <p className="page-description">Keep every Minecraft setup organized, separate, and ready when you are.</p>
            {instances.length === 0 ? (
              <div className="empty-state panel">
                <div className="empty-icon"><Layers3 size={26} /></div><h2>No instances yet</h2>
                <p>Create a separate local Vanilla profile. Fabric and Forge support will come in a later milestone.</p>
                <button className="primary-button" disabled={instanceBusy} onClick={() => void createInstance()}><Plus size={16} /> {instanceBusy ? "Creating..." : "Create instance"}</button>
              </div>
            ) : (
              <div className="instance-list">
                {instances.map(instance => (
                  <div className="panel instance-card" key={instance.id}>
                    <div className="panel-icon"><Layers3 size={18} /></div>
                    <div className="instance-info"><strong>{instance.name}</strong><p>Minecraft {instance.version} · Vanilla · Separate mods and config folders</p></div>
                    <span className="panel-status">LOCAL</span>
                  </div>
                ))}
                <button className="secondary-button" disabled={instanceBusy} onClick={() => void createInstance()}><Plus size={15} /> Create another instance</button>
              </div>
            )}
          </div>
        )}

        {page === "Mods" && (
          <div className="page-content simple-page">
            <div className="section-kicker">CUSTOMIZE YOUR GAME</div><h1 className="page-title">Mod library<span>.</span></h1>
            <p className="page-description">A home for the additions that make every world your own.</p>
            <div className="empty-state panel">
              <div className="empty-icon"><Puzzle size={26} /></div><h2>Your library is waiting</h2>
              <p>Choose an instance first. Mod discovery and installation are not connected yet.</p>
              <button className="secondary-button" onClick={() => setPage("Instances")}>View instances <ChevronDown size={15} /></button>
            </div>
          </div>
        )}

        {page === "Settings" && (
          <div className="page-content simple-page">
            <div className="section-kicker">PREFERENCES</div><h1 className="page-title">Settings<span>.</span></h1>
            <p className="page-description">Fine-tune your Crescent experience.</p>
            <div className="panel settings-row"><div className="panel-icon"><Monitor size={18} /></div><div><strong>Appearance</strong><p>Dark lunar theme · Default</p></div><span className="panel-status">ACTIVE</span></div>
            <div className="panel settings-row"><div className="panel-icon"><Download size={18} /></div><div><strong>Game directory</strong><p>Not configured yet</p></div><span className="muted-label">NOT SET</span></div>
            <div className="panel settings-row"><div className="panel-icon"><SlidersHorizontal size={18} /></div><div><strong>Launcher updates</strong><p>Update delivery will be configured later</p></div><span className="muted-label">PENDING</span></div>
          </div>
        )}
      </section>
    </main>
  );
}

export default App;