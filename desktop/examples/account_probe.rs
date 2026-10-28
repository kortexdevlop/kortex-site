// Staging gate: exercises the production PKCE client and Windows vault. No credentials on stdout/disk.
use kortex_desktop::{account, config::Config, secure_store};
use std::sync::{Arc, atomic::AtomicBool};
fn main() {
    if let Err(error) = run() {
        eprintln!("Account probe failed: {error}");
        std::process::exit(1)
    }
}
fn run() -> Result<(), String> {
    let c = Config::load()?;
    if c.channel != "staging" || !c.data_dir.to_string_lossy().contains("device-ai-staging") {
        return Err("Isolated staging directory required".into());
    }
    let pending = account::start(&c)?;
    println!("AUTHORIZATION_URL={}", pending.authorization_url);
    let result = (|| {
        let view = account::complete(&c, pending, Arc::new(AtomicBool::new(false)))?;
        if !view.connected {
            return Err("Not connected".into());
        }
        println!("VAULT_CONNECTED=true");
        let view = account::synchronize(&c, Some(true))?;
        if !view.enabled {
            return Err("Sync not enabled".into());
        }
        let view = account::favorite(&c, "calculatrice", true)?;
        if !view.favorites.iter().any(|id| id == "calculatrice") {
            return Err("Favorite not synchronized".into());
        }
        println!("SYNC_FAVORITE=true");
        let mut offline = c.clone();
        offline.offline = true;
        if !account::view(&offline)?.connected {
            return Err("Offline account unavailable".into());
        }
        println!("OFFLINE_CONNECTED=true");
        let marker = c.data_dir.join("private-tool-marker.txt");
        std::fs::write(&marker, "private content kept locally").map_err(|e| e.to_string())?;
        // Allow the browser to observe the shared favorite and test AI before revocation.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(90);
        while !c.data_dir.join("finish").exists() && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(100))
        }
        account::logout(&c)?;
        if secure_store::load(&c)?.is_some() || !marker.is_file() {
            return Err("Logout invariant failed".into());
        }
        println!("LOGOUT_PRESERVES_FILES=true");
        Ok(())
    })();
    if result.is_err() {
        let _ = account::logout(&c);
    }
    result
}
