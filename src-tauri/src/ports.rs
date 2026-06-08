use serde::Serialize;
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Serialize, Clone)]
pub struct PortInfo {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub directory: String,
}

pub fn scan_ports() -> Vec<PortInfo> {
    #[cfg(target_os = "windows")]
    return scan_windows();

    #[cfg(target_os = "macos")]
    return scan_macos();

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    return vec![];
}

#[cfg(target_os = "windows")]
fn decode_output(bytes: &[u8]) -> String {
    // tasklist/netstat bazen UTF-16 LE BOM ile çıktı verebilir
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        let u16s: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect();
        return String::from_utf16_lossy(&u16s).to_string();
    }
    String::from_utf8_lossy(bytes).to_string()
}

#[cfg(target_os = "windows")]
fn scan_windows() -> Vec<PortInfo> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    // 1. Listening TCP portlarını ve PID'lerini al — netstat çok hızlı
    let netstat_out = match Command::new("netstat")
        .args(["-ano", "-p", "TCP"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        Ok(o) => decode_output(&o.stdout),
        Err(_) => return vec![],
    };

    // port → pid map
    let mut port_pid: Vec<(u16, u32)> = Vec::new();
    let mut seen_ports = std::collections::HashSet::new();
    for line in netstat_out.lines() {
        let cols: Vec<&str> = line.split_whitespace().collect();
        // TCP  0.0.0.0:PORT  ...  LISTENING  PID
        if cols.len() < 5 { continue; }
        if !cols[3].eq_ignore_ascii_case("LISTENING") { continue; }
        let local = cols[1];
        let port: u16 = match local.rsplit(':').next().and_then(|p| p.parse().ok()) {
            Some(p) => p,
            None => continue,
        };
        let pid: u32 = match cols[4].parse() {
            Ok(p) => p,
            Err(_) => continue,
        };
        if port == 0 || seen_ports.contains(&port) { continue; }
        seen_ports.insert(port);
        port_pid.push((port, pid));
    }

    if port_pid.is_empty() { return vec![]; }

    // 2. PID → process name — tasklist CSV, çok hızlı
    let mut pid_name: HashMap<u32, String> = HashMap::new();
    if let Ok(out) = Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        for line in decode_output(&out.stdout).lines() {
            let line = line.trim().trim_matches('"');
            if line.is_empty() { continue; }
            // "chrome.exe","1234","Console","1","100,000 K"
            let parts: Vec<&str> = line.splitn(5, "\",\"").collect();
            if parts.len() < 2 { continue; }
            let raw_name = parts[0].trim_matches('"');
            let name = raw_name.trim_end_matches(".exe").trim_end_matches(".EXE").to_string();
            let pid: u32 = match parts[1].trim_matches('"').parse() {
                Ok(p) => p,
                Err(_) => continue,
            };
            pid_name.insert(pid, name);
        }
    }

    // 3. PID → exe path — wmic, sadece ihtiyaç duyulan PID'ler için
    //    wmic yoksa (Windows 11 bazı sürümleri) graceful fallback
    let unique_pids: Vec<String> = {
        let mut pids: Vec<u32> = port_pid.iter().map(|(_, pid)| *pid).collect();
        pids.sort_unstable();
        pids.dedup();
        pids.iter().map(|p| p.to_string()).collect()
    };

    let mut pid_dir: HashMap<u32, String> = HashMap::new();
    if !unique_pids.is_empty() {
        let where_clause = unique_pids
            .iter()
            .map(|p| format!("ProcessId={}", p))
            .collect::<Vec<_>>()
            .join(" OR ");
        let query = format!(
            "SELECT ProcessId,ExecutablePath FROM Win32_Process WHERE {}",
            where_clause
        );
        if let Ok(out) = Command::new("wmic")
            .args(["process", "where", &format!("({})", where_clause), "get", "ProcessId,ExecutablePath", "/FORMAT:CSV"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            let _ = query; // unused if wmic path used
            for line in decode_output(&out.stdout).lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with("Node") { continue; }
                // Node,ExecutablePath,ProcessId
                let parts: Vec<&str> = line.splitn(3, ',').collect();
                if parts.len() < 3 { continue; }
                let exe_path = parts[1].trim();
                let pid: u32 = match parts[2].trim().parse() {
                    Ok(p) => p,
                    Err(_) => continue,
                };
                if !exe_path.is_empty() {
                    let dir = std::path::Path::new(exe_path)
                        .parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();
                    pid_dir.insert(pid, dir);
                }
            }
        }
    }

    // 4. Birleştir
    let mut ports: Vec<PortInfo> = port_pid
        .into_iter()
        .map(|(port, pid)| {
            let name = pid_name.get(&pid).cloned().unwrap_or_else(|| format!("pid_{}", pid));
            let dir  = pid_dir.get(&pid).cloned().unwrap_or_default();
            PortInfo { port, pid, process_name: name, directory: dir }
        })
        .collect();

    ports.sort_by_key(|p| p.port);
    ports
}

#[cfg(target_os = "macos")]
fn scan_macos() -> Vec<PortInfo> {
    // LISTEN portlarını al
    let lsof_out = match Command::new("lsof")
        .args(["-iTCP", "-sTCP:LISTEN", "-n", "-P"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return vec![],
    };

    let stdout = String::from_utf8_lossy(&lsof_out.stdout);
    let mut port_pid: Vec<(u16, u32, String)> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 { continue; }
        let process_name = parts[0].to_string();
        let pid: u32 = parts[1].parse().unwrap_or(0);
        let port_str = parts[8].split(':').last().unwrap_or("0");
        let port: u16 = port_str.parse().unwrap_or(0);
        if port == 0 || seen.contains(&port) { continue; }
        seen.insert(port);
        port_pid.push((port, pid, process_name));
    }

    if port_pid.is_empty() { return vec![]; }

    // PID → exe path: proc_pidpath (en hızlı yol)
    use std::collections::HashMap;
    let mut pid_dir: HashMap<u32, String> = HashMap::new();
    let unique_pids: Vec<String> = {
        let mut pids: Vec<u32> = port_pid.iter().map(|(_, pid, _)| *pid).collect();
        pids.sort_unstable();
        pids.dedup();
        pids.iter().map(|p| p.to_string()).collect()
    };

    // lsof -p PID1,PID2 -Fn ile executable path al
    let pids_arg = unique_pids.join(",");
    if let Ok(out) = Command::new("lsof")
        .args(["-p", &pids_arg, "-Fn", "-a", "-d", "txt"])
        .output()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        let mut cur_pid: u32 = 0;
        for line in text.lines() {
            if let Some(pid_str) = line.strip_prefix('p') {
                cur_pid = pid_str.parse().unwrap_or(0);
            } else if let Some(path) = line.strip_prefix('n') {
                if cur_pid != 0 && !path.is_empty() {
                    let dir = std::path::Path::new(path)
                        .parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();
                    pid_dir.entry(cur_pid).or_insert(dir);
                }
            }
        }
    }

    let mut ports: Vec<PortInfo> = port_pid
        .into_iter()
        .map(|(port, pid, process_name)| {
            let directory = pid_dir.get(&pid).cloned().unwrap_or_default();
            PortInfo { port, pid, process_name, directory }
        })
        .collect();

    ports.sort_by_key(|p| p.port);
    ports
}

