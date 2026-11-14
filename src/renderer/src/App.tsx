import { useEffect, useMemo, useState } from "react";
import {
  Home, Layers3, Puzzle, Settings, ChevronDown, Play, Plus, Cpu,
  HardDrive, Sparkles, Download, Search, X, Monitor,
  MoonStar, FolderOpen, LoaderCircle, UserRound
} from "lucide-react";

type Page = "Home" | "Instances" | "Mods" | "Settings";
type Modal = "create" | "launch" | "account" | null;
const nav: { label: Page; icon: typeof Home }[] = [
  { label: "Home", icon: Home }, { label: "Instances", icon: Layers3 },
  { label: "Mods", icon: Puzzle }, { label: "Settings", icon: Settings }
];

function App() {
  const [page, setPage] = useState<Page>("Home");
  const [version, setVersion] = useState("26.3");
  const [ram, setRam] = useState(4);
  const [notice, setNotice] = useState("Your next adventure starts here.");
  const [instances, setInstances] = useState<CrescentInstance[]>([]);
  const [versionGroups, setVersionGroups] = useState<{ releases: string[]; snapshots: string[]; oldBetas: string[]; oldAlphas: string[] }>({ releases: [], snapshots: [], oldBetas: [], oldAlphas: [] });
  const [javaInfo, setJavaInfo] = useState<CrescentJavaInfo | null>(null);
  const [installingId, setInstallingId] = useState<string | null>(null);
  const [instanceBusy, setInstanceBusy] = useState(false);
  const [modal, setModal] = useState<Modal>(null);
  const [newName, setNewName] = useState("My Crescent World");
  const [playerName, setPlayerName] = useState("Player");
  const [selectedInstance, setSelectedInstance] = useState<CrescentInstance | null>(null);
  const [search, setSearch] = useState("");

  useEffect(() => {
    if (!window.crescent) { setNotice("Crescent native services are unavailable. Reinstall the desktop app."); return; }
    void Promise.all([window.crescent.listInstances(), window.crescent.getSettings(), window.crescent.getVersions(), window.crescent.getJavaInfo()])
      .then(([saved, settings, manifest, java]) => {
        setInstances(saved); setRam(settings.ramGb);
        setVersionGroups({ releases: manifest.releases, snapshots: manifest.snapshots, oldBetas: manifest.oldBetas, oldAlphas: manifest.oldAlphas });
        setVersion("26.3");
        setJavaInfo(java);
      })
      .catch(error => setNotice(error instanceof Error ? error.message : "Could not load local launcher services."));
  }, []);

  const availableVersions = useMemo(() => [...new Set([...versionGroups.releases, ...versionGroups.snapshots, ...versionGroups.oldBetas, ...versionGroups.oldAlphas])], [versionGroups]);
  const filteredInstances = instances.filter(item => item.name.toLowerCase().includes(search.toLowerCase()) || item.version.includes(search));

  const createInstance = async () => {
    if (!window.crescent || instanceBusy || !newName.trim()) return;
    setInstanceBusy(true);
    try {
      const created = await window.crescent.createInstance({ name: newName.trim(), version });
      setInstances(current => [created, ...current]); setModal(null);
      setNotice(`Created ${created.name}. Install its game files to get started.`);
    } catch (error) { setNotice(error instanceof Error ? error.message : "Could not create the instance."); }
    finally { setInstanceBusy(false); }
  };

  const launchInstance = async () => {
    if (!window.crescent || !selectedInstance || !playerName.trim()) return;
    setNotice(`Starting Minecraft ${selectedInstance.version}…`);
    try {
      const result = await window.crescent.launchInstance(selectedInstance.id, playerName.trim());
      setNotice(`Minecraft started for ${result.playerName} (process ${result.processId}). ${result.authentication}`);
      setModal(null); setSelectedInstance(null);
    } catch (error) { setNotice(error instanceof Error ? error.message : "Could not launch Minecraft."); }
  };

  const installInstance = async (instance: CrescentInstance) => {
    if (!window.crescent || installingId) return;
    setInstallingId(instance.id); setNotice(`Preparing Minecraft ${instance.version}. Downloads can take a while on first install.`);
    try {
      const result = await window.crescent.installVersion(instance.id);
      setNotice(`Installed Minecraft ${result.version}: ${result.librariesDownloaded} libraries and ${result.assetsDownloaded} assets. ${result.modInstalled ? `Fabric ${result.fabricLoaderVersion} and Crescent mod installed.` : "Vanilla files installed."}`);
    } catch (error) { setNotice(error instanceof Error ? error.message : "Minecraft installation failed."); }
    finally { setInstallingId(null); }
  };

  const openLaunch = (instance: CrescentInstance) => { setSelectedInstance(instance); setPlayerName("Player"); setModal("launch"); };
  const announce = (message: string) => setNotice(message);

  return <main className="app-shell">
    <aside className="sidebar">
      <button className="brand" onClick={() => setPage("Home")} aria-label="Crescent home">
        <span className="brand-mark"><MoonStar size={23}/></span><span className="brand-word"><strong>CRESCENT</strong><small>GAME CLIENT</small></span>
      </button>
      <div className="nav-label">LIBRARY</div>
      <nav>{nav.map(({label,icon:Icon}) => <button key={label} className={`nav-item ${page===label?"active":""}`} onClick={() => setPage(label)}><Icon size={18}/><span>{label}</span>{page===label && <i/>}</button>)}</nav>
      <div className="sidebar-bottom">
        <button className="profile-card" onClick={() => setModal("account")}><span className="avatar"><UserRound size={17}/></span><span className="profile-copy"><strong>Local profile</strong><small>Offline account</small></span><ChevronDown size={15}/></button>
        <div className="version-tag"><span className="tiny-dot"/> CRESCENT <span>EARLY ACCESS</span></div>
      </div>
    </aside>

    <section className="main-area">
      <header className="topbar"><div className="breadcrumb"><span>YOUR LIBRARY</span><b>/</b><strong>{page}</strong></div><div className="top-actions"><span className="top-status"><i/> LOCAL SERVICES</span><button className="account-button" onClick={() => setModal("account")}><span className="avatar mini"><UserRound size={14}/></span> Guest <ChevronDown size={14}/></button></div></header>

      {page==="Home" && <div className="page-content">
        <section className="hero">
          <div className="hero-text"><div className="eyebrow"><Sparkles size={13}/> A NEW WAY TO PLAY</div><h1>Your worlds.<br/><em>Your Crescent.</em></h1><p>A focused home for your Minecraft instances, mods, and adventures.</p><div className="hero-actions"><button className="primary-button" onClick={() => setPage("Instances")}><Play size={16} fill="currentColor"/> Open library</button><button className="secondary-button" onClick={() => {setNewName("My Crescent World");setModal("create");}}><Plus size={16}/> New instance</button></div></div>
          <div className="hero-art" aria-hidden="true"><div className="art-glow"/><div className="planet"><span/></div><div className="orbit orbit-a"/><div className="orbit orbit-b"/><span className="art-star star-1">✦</span><span className="art-star star-2">✧</span><div className="art-label">CRESCENT / 001</div></div>
          <div className="hero-bottom"><span>BUILT FOR YOUR NEXT ADVENTURE</span><span>VERSION 0.2</span></div>
        </section>
        <div className="section-heading"><div><span className="section-kicker">OVERVIEW</span><h2>Quick setup</h2></div><span className="muted-label">{instances.length} INSTANCE{instances.length===1?"":"S"}</span></div>
        <div className="setup-grid">
          <div className="panel version-panel"><div className="panel-top"><span className="panel-icon"><Layers3 size={17}/></span><span>GAME VERSION</span></div><label htmlFor="version">Choose a version</label><div className="select-wrap"><select id="version" value={version} onChange={e=>setVersion(e.target.value)}><option value="26.3">Minecraft 26.3 · Fabric</option>{availableVersions.filter(v=>v!=="26.3").map(v=><option key={v} value={v}>{v}</option>)}</select><ChevronDown size={15}/></div><div className="panel-foot"><span className="tiny-dot"/> 26.3 installs with Fabric + Crescent</div></div>
          <div className="panel ram-panel"><div className="panel-top"><span className="panel-icon"><Cpu size={17}/></span><span>MEMORY</span></div><div className="ram-value">{ram}<small> GB</small></div><input aria-label="Memory allocation" type="range" min="2" max="6" value={ram} onChange={e=>{const n=Number(e.target.value);setRam(n);void window.crescent?.saveSettings({ramGb:n}).catch(()=>setNotice("Could not save RAM settings."));}}/><div className="range-labels"><span>2 GB</span><span>6 GB</span></div><div className="panel-foot"><HardDrive size={13}/> Allocated game memory</div></div>
          <div className="panel status-panel"><div className="panel-top"><span className="panel-icon"><Monitor size={17}/></span><span>SYSTEM CHECK</span></div><div className="status-title">{javaInfo?.available ? "Java detected" : "Java required"}</div><p>{javaInfo?.available ? javaInfo.version : javaInfo?.message ?? "Checking Java runtime…"}</p><div className="status-bottom"><span className={`status-ring ${javaInfo?.available?"ready":""}`}/>{javaInfo?.available?"READY TO PLAY":"CHECK JAVA RUNTIME"}</div></div>
        </div>
        <div className="notice"><Sparkles size={15}/><span>{notice}</span><button onClick={()=>setNotice("Your next adventure starts here.")}>DISMISS</button></div>
      </div>}

      {page==="Instances" && <div className="page-content simple-page"><div className="page-heading"><div><div className="section-kicker">GAME MANAGEMENT</div><h1 className="page-title">Your instances<span>.</span></h1><p className="page-description">Separate game versions, mods, and settings in one place.</p></div><button className="primary-button" onClick={()=>{setNewName("My Crescent World");setModal("create");}}><Plus size={16}/> New instance</button></div>
        <div className="toolbar"><div className="search-box"><Search size={16}/><input value={search} onChange={e=>setSearch(e.target.value)} placeholder="Search instances…"/></div><span>{filteredInstances.length} total</span></div>
        {filteredInstances.length===0?<div className="empty-state panel"><span className="empty-icon"><Layers3 size={25}/></span><h2>{instances.length?"No matching instances":"Your library is empty"}</h2><p>Create an instance to install Minecraft and keep its files organized.</p><button className="primary-button" onClick={()=>setModal("create")}><Plus size={16}/> Create instance</button></div>:<div className="instance-list">{filteredInstances.map(instance=><div className="panel instance-card" key={instance.id}><span className="instance-cover"><MoonStar size={25}/></span><div className="instance-info"><strong>{instance.name}</strong><p>Minecraft {instance.version} <span>·</span> {instance.loader==="fabric"?"Fabric":"Vanilla"} <span>·</span> Separate instance</p></div><span className="panel-status">{instance.loader==="fabric"?"FABRIC":"VANILLA"}</span><div className="instance-actions"><button className="secondary-button" disabled={installingId!==null} onClick={()=>void installInstance(instance)}>{installingId===instance.id?<LoaderCircle className="spin" size={14}/>:<Download size={14}/>} {installingId===instance.id?"Installing…":"Install files"}</button><button className="primary-button" disabled={installingId!==null} onClick={()=>openLaunch(instance)}><Play size={14} fill="currentColor"/> Play</button></div></div>)}</div>}
        {installingId && <div className="download-strip"><LoaderCircle className="spin" size={16}/><div><strong>Installing game files</strong><span>Downloading and verifying required files. This may take a while.</span></div><div className="indeterminate"><i/></div></div>}
      </div>}

      {page==="Mods" && <div className="page-content simple-page"><div className="section-kicker">CUSTOMIZE YOUR GAME</div><h1 className="page-title">Mod library<span>.</span></h1><p className="page-description">Your installed mods and game add-ons will live here.</p><div className="empty-state panel"><span className="empty-icon"><Puzzle size={25}/></span><h2>Mod browser is not connected yet</h2><p>The current build does not include Modrinth search or mod installation. We won't pretend this feature is ready.</p><button className="secondary-button" onClick={()=>setPage("Instances")}>Manage instances <ChevronDown size={15}/></button></div></div>}

      {page==="Settings" && <div className="page-content simple-page"><div className="section-kicker">PREFERENCES</div><h1 className="page-title">Settings<span>.</span></h1><p className="page-description">Configure your local Crescent environment.</p><div className="panel settings-row"><span className="panel-icon"><Cpu size={18}/></span><div><strong>Java runtime</strong><p>{javaInfo?(javaInfo.available?javaInfo.version:javaInfo.message):"Checking Java availability…"}</p></div><span className="panel-status">{javaInfo?.available?"DETECTED":"CHECK"}</span></div><div className="panel settings-row"><span className="panel-icon"><HardDrive size={18}/></span><div><strong>Memory allocation</strong><p>{ram} GB assigned to Minecraft</p></div><span className="panel-status">LOCAL</span></div><div className="panel settings-row"><span className="panel-icon"><FolderOpen size={18}/></span><div><strong>Game directory</strong><p>Managed by Crescent</p></div><span className="panel-status">LOCAL</span></div></div>}
    </section>

    {modal && <div className="modal-backdrop" onMouseDown={e=>{if(e.target===e.currentTarget)setModal(null);}}><section className="modal panel" role="dialog" aria-modal="true">
      <button className="modal-close" onClick={()=>setModal(null)} aria-label="Close"><X size={18}/></button>
      {modal==="create" && <><span className="modal-icon"><Plus size={20}/></span><div className="section-kicker">NEW GAME PROFILE</div><h2>Create an instance</h2><p>Give this Minecraft setup a name and choose its game version.</p><label htmlFor="instance-name">Instance name</label><input id="instance-name" className="modal-input" value={newName} maxLength={32} onChange={e=>setNewName(e.target.value)} placeholder="My Minecraft world"/><label htmlFor="instance-version">Minecraft version</label><select id="instance-version" className="modal-input" value={version} onChange={e=>setVersion(e.target.value)}><option value="26.3">26.3 · Fabric + Crescent</option>{availableVersions.filter(v=>v!=="26.3").map(v=><option key={v} value={v}>{v}</option>)}</select><div className="modal-actions"><button className="secondary-button" onClick={()=>setModal(null)}>Cancel</button><button className="primary-button" disabled={!newName.trim()||instanceBusy} onClick={()=>void createInstance()}>{instanceBusy?<LoaderCircle className="spin" size={15}/>:<Plus size={15}/>} Create instance</button></div></>}
      {modal==="launch" && <><span className="modal-icon"><Play size={20}/></span><div className="section-kicker">PLAYER PROFILE</div><h2>Ready to play?</h2><p>Enter a local offline profile name for <strong>{selectedInstance?.name}</strong>.</p><label htmlFor="player-name">Player name</label><input id="player-name" className="modal-input" value={playerName} maxLength={16} onChange={e=>setPlayerName(e.target.value.replace(/[^a-zA-Z0-9_]/g,""))} placeholder="Player"/><div className="modal-note">Offline profile only. This does not sign in to Microsoft or Ely.by.</div><div className="modal-actions"><button className="secondary-button" onClick={()=>setModal(null)}>Cancel</button><button className="primary-button" disabled={!playerName.trim()} onClick={()=>void launchInstance()}><Play size={15} fill="currentColor"/> Launch game</button></div></>}
      {modal==="account" && <><span className="modal-icon"><UserRound size={20}/></span><div className="section-kicker">ACCOUNT</div><h2>Local profile</h2><p>This build currently supports local offline profiles. Ely.by sign-in is not implemented yet.</p><div className="modal-note">Your offline player name is requested when you launch an instance. No account credentials are collected here.</div><div className="modal-actions"><button className="primary-button" onClick={()=>setModal(null)}>Got it</button></div></>}
    </section></div>}
  </main>;
}
export default App;
