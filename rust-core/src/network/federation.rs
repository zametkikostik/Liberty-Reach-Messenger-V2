//! Federated mode + mesh discovery
use serde::{Serialize, Deserialize};
use std::sync::Mutex;
use once_cell::sync::Lazy;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FederationPeer {
    pub name: String,
    pub multiaddrs: Vec<String>,
    pub wake_url: Option<String>,
    pub privacy_url: Option<String>,
    pub priority: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FederationConfig {
    pub mode: FederationMode,
    pub local_name: String,
    pub peers: Vec<FederationPeer>,
    pub open_discovery: bool,
    pub max_bootstrap: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum FederationMode {
    LocalMesh,
    Federated,
    Private,
}

impl Default for FederationConfig {
    fn default() -> Self {
        Self {
            mode: FederationMode::Federated,
            local_name: "liberty-node".into(),
            peers: vec![],
            open_discovery: true,
            max_bootstrap: 8,
        }
    }
}

static CONFIG: Lazy<Mutex<FederationConfig>> = Lazy::new(|| Mutex::new(FederationConfig::default()));

pub fn get_config() -> FederationConfig { CONFIG.lock().unwrap().clone() }
pub fn set_config(cfg: FederationConfig) { *CONFIG.lock().unwrap() = cfg; }

pub fn add_peer(peer: FederationPeer) {
    let mut cfg = CONFIG.lock().unwrap();
    if let Some(i) = cfg.peers.iter().position(|p| p.name == peer.name) {
        cfg.peers[i] = peer;
    } else {
        cfg.peers.push(peer);
    }
}

pub fn bootstrap_addrs() -> Vec<String> {
    let cfg = CONFIG.lock().unwrap();
    match cfg.mode {
        FederationMode::LocalMesh => vec![],
        FederationMode::Private | FederationMode::Federated => {
            let mut peers = cfg.peers.clone();
            peers.sort_by_key(|p| p.priority);
            peers.into_iter().flat_map(|p| p.multiaddrs).filter(|a| !a.is_empty()).take(cfg.max_bootstrap as usize).collect()
        }
    }
}

pub fn mesh_enabled() -> bool {
    matches!(CONFIG.lock().unwrap().mode, FederationMode::LocalMesh | FederationMode::Federated)
}

pub fn export_json() -> String {
    serde_json::to_string_pretty(&*CONFIG.lock().unwrap()).unwrap_or_else(|_| "{}".into())
}

pub fn import_json(json: &str) -> Result<(), String> {
    let cfg: FederationConfig = serde_json::from_str(json).map_err(|e| e.to_string())?;
    set_config(cfg);
    Ok(())
}
