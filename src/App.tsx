import { useEffect, useRef, useState, useMemo, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface Port {
  port: number;
  pid: number;
  process_name: string;
  directory: string;
}

const INTERVALS = [
  { label: "3s",  value: 3000  },
  { label: "5s",  value: 5000  },
  { label: "10s", value: 10000 },
  { label: "30s", value: 30000 },
];

export default function App() {
  const [ports, setPorts]             = useState<Port[]>([]);
  const [initialLoad, setInitialLoad] = useState(true);
  const [refreshing, setRefreshing]   = useState(false);
  const [showOptions, setShowOptions] = useState(false);
  const [intervalMs, setIntervalMs]   = useState(5000);
  const [confirmKill, setConfirmKill] = useState<Port | null>(null);
  const [search, setSearch]           = useState("");
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const fetchPorts = useCallback(async (isManual = false) => {
    try {
      if (isManual) setRefreshing(true);
      const result = await invoke<Port[]>("get_ports");
      setPorts(result);
    } catch (e) {
      console.error(e);
    } finally {
      setInitialLoad(false);
      setRefreshing(false);
    }
  }, []);

  useEffect(() => {
    fetchPorts();
    if (intervalRef.current) clearInterval(intervalRef.current);
    intervalRef.current = setInterval(() => fetchPorts(), intervalMs);
    return () => { if (intervalRef.current) clearInterval(intervalRef.current); };
  }, [intervalMs, fetchPorts]);

  const killPort       = useCallback(async (pid: number) => { await invoke("kill_port", { pid }); fetchPorts(); }, [fetchPorts]);
  const confirmAndKill = useCallback((port: Port) => setConfirmKill(port), []);
  const handleKillConfirm = useCallback(async () => {
    if (confirmKill) {
      await killPort(confirmKill.pid);
      setConfirmKill(null);
    }
  }, [confirmKill, killPort]);
  const openTerminal = useCallback(async (dir: string) => { await invoke("open_terminal", { dir }); }, []);
  const openFolder   = useCallback(async (dir: string) => { await invoke("open_folder",   { dir }); }, []);
  const quitApp      = useCallback(async ()            => { await invoke("quit_app"); }, []);

  const filteredPorts = useMemo(() => {
    const q = search.trim().toLowerCase();
    if (!q) return ports;
    return ports.filter(
      (p) =>
        String(p.port).includes(q) ||
        p.process_name.toLowerCase().includes(q) ||
        p.directory.toLowerCase().includes(q)
    );
  }, [ports, search]);

  return (
    <div className="app">
      {/* Header — data-tauri-drag-region ile native sürükleme */}
      <div className="header" data-tauri-drag-region>
        <span className="title" data-tauri-drag-region>Active Ports</span>
        <button
          className={`refresh-btn ${refreshing ? "spinning" : ""}`}
          onClick={() => fetchPorts(true)}
          title="Refresh"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none">
            <path d="M1 4v6h6" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
            <path d="M23 20v-6h-6" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
            <path d="M20.49 9A9 9 0 0 0 5.64 5.64L1 10m22 4-4.64 4.36A9 9 0 0 1 3.51 15" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
          </svg>
        </button>
      </div>

      {/* Search bar */}
      <div className="search-bar">
        <svg className="search-icon" width="12" height="12" viewBox="0 0 16 16" fill="none">
          <circle cx="6.5" cy="6.5" r="5" stroke="currentColor" strokeWidth="1.6"/>
          <path d="M10.5 10.5L14 14" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round"/>
        </svg>
        <input
          className="search-input"
          type="text"
          placeholder="Search port, process…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
        {search && (
          <button className="search-clear" onClick={() => setSearch("")}>×</button>
        )}
      </div>

      {/* Port listesi */}
      <div className="port-list">
        {initialLoad ? (
          <div className="empty">Scanning…</div>
        ) : filteredPorts.length === 0 ? (
          <div className="empty">{search ? "No results" : "No active ports"}</div>
        ) : (
          filteredPorts.map((p) => (
            <div className="port-row" key={`${p.port}-${p.pid}`}>
              <div className="port-info">
                <div className="port-top">
                  <span className="port-num">:{p.port}</span>
                  <span className="proc-badge">{p.process_name}</span>
                </div>
                <div className="port-meta">
                  {p.directory ? (
                    <span className="meta-path" title={p.directory}>
                      {p.directory.length > 38 ? "…" + p.directory.slice(-35) : p.directory}
                    </span>
                  ) : (
                    <span className="meta-pid">PID {p.pid}</span>
                  )}
                </div>
              </div>
              <div className="port-actions">
                <button className="action-btn" onClick={() => openTerminal(p.directory)} title="Terminal">
                  <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                    <path d="M1.5 3.5l3 2.5-3 2.5" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round"/>
                    <path d="M6.5 8.5h4" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round"/>
                  </svg>
                </button>
                <button className="action-btn" onClick={() => openFolder(p.directory)} title="Folder">
                  <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                    <path d="M1 4h10v6a1 1 0 01-1 1H2a1 1 0 01-1-1V4z" stroke="currentColor" strokeWidth="1.3"/>
                    <path d="M1 4V3a1 1 0 011-1h2l1.5 2H1" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round"/>
                  </svg>
                </button>
                <button className="kill-btn" onClick={() => confirmAndKill(p)} title="Kill">
                  <svg width="9" height="9" viewBox="0 0 9 9" fill="none">
                    <path d="M1 1l7 7M8 1L1 8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
                  </svg>
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      {/* Options paneli */}
      {showOptions && (
        <div className="options-panel">
          <div className="options-row">
            <span className="options-label">Refresh interval</span>
            <div className="interval-group">
              {INTERVALS.map((i) => (
                <button
                  key={i.value}
                  className={`interval-btn ${intervalMs === i.value ? "active" : ""}`}
                  onClick={() => setIntervalMs(i.value)}
                >
                  {i.label}
                </button>
              ))}
            </div>
          </div>
          <div className="options-row">
            <span className="options-label">Version</span>
            <span className="options-value">0.1.0</span>
          </div>
          <div className="options-hint">
            ⚠ Run as Administrator for full port visibility
          </div>
        </div>
      )}

      {/* Kill onay modalı */}
      {confirmKill && (
        <div className="modal-overlay" onClick={() => setConfirmKill(null)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <p className="modal-title">Kill Process?</p>
            <p className="modal-desc">
              <span className="modal-proc">{confirmKill.process_name}</span> on port{" "}
              <span className="modal-port">:{confirmKill.port}</span> will be terminated.
            </p>
            <div className="modal-actions">
              <button className="modal-btn modal-cancel" onClick={() => setConfirmKill(null)}>Cancel</button>
              <button className="modal-btn modal-confirm" onClick={handleKillConfirm}>Kill</button>
            </div>
          </div>
        </div>
      )}

      {/* Footer */}
      <div className="footer">
        <button
          className={`footer-btn ${showOptions ? "active" : ""}`}
          onClick={() => setShowOptions(!showOptions)}
        >
          <svg width="11" height="11" viewBox="0 0 11 11" fill="none">
            <circle cx="5.5" cy="5.5" r="1.5" stroke="currentColor" strokeWidth="1.2"/>
            <path d="M5.5 1v1.2M5.5 8.8V10M1 5.5h1.2M8.8 5.5H10" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round"/>
          </svg>
          Options
        </button>
        <div className="footer-center">
          <div className="live-dot" />
          <span>{ports.length} active ports</span>
        </div>
        <button
          className="footer-btn quit-btn"
          onClick={quitApp}
        >
          Quit
        </button>
      </div>
    </div>
  );
}