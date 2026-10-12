//! Opt-in, supervised updates. A separate OS job survives daemon restarts.
use anyhow::{bail, Context, Result};

#[cfg(unix)]
mod unix {
    use super::*;
    use flate2::read::GzDecoder;
    use serde::Deserialize;
    use sha2::{Digest, Sha256};
    use std::fs::{self, File, OpenOptions};
    use std::io::{Read, Write};
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
    use std::path::Path;
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    const FEED: &str = "https://i1.is/releases/smart-tree/latest.json";
    const BINS: [&str; 4] = ["st", "std", "m8", "n8x"];
    const LIMIT: u64 = 512 * 1024 * 1024;
    #[derive(Deserialize)]
    struct Release {
        tag_name: String,
        assets: Vec<Asset>,
    }
    #[derive(Deserialize)]
    struct Asset {
        name: String,
        browser_download_url: String,
        digest: String,
    }

    fn stable(s: &str) -> Result<(u64, u64, u64)> {
        let parts: Vec<_> = s.strip_prefix('v').unwrap_or(s).split('.').collect();
        if parts.len() != 3
            || parts.iter().any(|p| {
                p.is_empty()
                    || !p.bytes().all(|b| b.is_ascii_digit())
                    || (p.len() > 1 && p.starts_with('0'))
            })
        {
            bail!("Only stable major.minor.patch releases are eligible");
        }
        Ok((parts[0].parse()?, parts[1].parse()?, parts[2].parse()?))
    }
    fn trusted(path: &Path) -> Result<()> {
        for p in path.ancestors() {
            let m = fs::symlink_metadata(p)?;
            if m.file_type().is_symlink() || m.uid() != 0 || m.mode() & 0o022 != 0 {
                bail!(
                    "Updater path must be root-owned, non-writable by others, without symlinks: {}",
                    p.display()
                );
            }
        }
        Ok(())
    }
    fn sync_dir(p: &Path) -> Result<()> {
        File::open(p)?.sync_all()?;
        Ok(())
    }
    fn copy_sync(from: &Path, to: &Path) -> Result<()> {
        let mut source = File::open(from)?;
        let mut dest = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o755)
            .open(to)?;
        std::io::copy(&mut source, &mut dest)?;
        dest.set_permissions(fs::Permissions::from_mode(0o755))?;
        dest.sync_all()?;
        Ok(())
    }
    fn lock(dir: &Path) -> Result<File> {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(dir.join("lock"))?;
        if unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            bail!("Another updater is running");
        }
        Ok(f)
    }
    async fn body(client: &reqwest::Client, url: &str, limit: u64) -> Result<Vec<u8>> {
        let mut response = client.get(url).send().await?.error_for_status()?;
        if response.content_length().is_some_and(|n| n > limit) {
            bail!("Release exceeds size limit");
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            if bytes.len() as u64 + chunk.len() as u64 > limit {
                bail!("Release exceeds size limit");
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(bytes)
    }
    fn extract(bytes: &[u8], stage: &Path) -> Result<()> {
        let mut archive = tar::Archive::new(GzDecoder::new(bytes));
        let mut found = std::collections::HashSet::new();
        let mut total = 0u64;
        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?.into_owned();
            let name = path.to_str().context("Invalid archive name")?;
            // Immutable publisher emits exactly these root-level regular files.
            if (!BINS.contains(&name) && name != "BUILD_INFO.json")
                || !entry.header().entry_type().is_file()
                || !found.insert(name.to_string())
            {
                bail!("Unexpected archive member: {}", path.display());
            }
            total = total
                .checked_add(entry.size())
                .context("Archive size overflow")?;
            if total > LIMIT {
                bail!("Expanded release exceeds size limit");
            }
            let mut dest = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o755)
                .open(stage.join(name))?;
            std::io::copy(&mut entry, &mut dest)?;
            dest.set_permissions(fs::Permissions::from_mode(0o755))?;
            dest.sync_all()?;
        }
        if BINS.iter().any(|name| !found.contains(*name)) {
            bail!("Release must contain all four binaries");
        }
        sync_dir(stage)
    }
    fn command(mut command: Command) -> Result<()> {
        // No inherited stdin; a stuck service manager/candidate is bounded.
        let mut child = command.stdin(Stdio::null()).stderr(Stdio::null()).spawn()?;
        let start = Instant::now();
        loop {
            if let Some(status) = child.try_wait()? {
                if !status.success() {
                    bail!("Update subprocess failed: {status}");
                }
                return Ok(());
            }
            if start.elapsed() > Duration::from_secs(40) {
                let _ = child.kill();
                let _ = child.wait();
                bail!("Update subprocess timed out");
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
    fn binary_version(binary: &Path, private_dir: &Path) -> Result<String> {
        let output_path = private_dir.join("version-output");
        let mut c = Command::new(binary);
        c.arg("--version").stdout(File::create(&output_path)?);
        command(c)?;
        let mut output = String::new();
        File::open(&output_path)?
            .take(65536)
            .read_to_string(&mut output)?;
        fs::remove_file(output_path)?;
        let first = output
            .lines()
            .find(|line| line.contains("Smart Tree v"))
            .context("Missing Smart Tree version")?;
        let version = first
            .split("Smart Tree v")
            .nth(1)
            .context("Missing version")?
            .split_whitespace()
            .next()
            .context("Empty version")?;
        stable(version)?;
        Ok(version.to_string())
    }
    fn restart() -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            let mut c = Command::new("/bin/launchctl");
            c.args(["kickstart", "-k", "system/is.8b.smart-tree-daemon"]);
            command(c)
        }
        #[cfg(not(target_os = "macos"))]
        {
            let mut c = Command::new("/usr/bin/systemctl");
            c.args(["restart", "smart-tree-daemon.service"]);
            command(c)
        }
    }
    async fn healthy(client: &reqwest::Client, token: &str, version: &str) -> Result<()> {
        for _ in 0..20 {
            if let Ok(r) = client
                .get("http://127.0.0.1:28428/info")
                .bearer_auth(token)
                .timeout(Duration::from_secs(2))
                .send()
                .await
            {
                if r.status().is_success() {
                    if let Ok(v) = r.json::<serde_json::Value>().await {
                        if v["name"] == "smart-tree-daemon"
                            && v["version"] == version.trim_start_matches('v')
                        {
                            return Ok(());
                        }
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        bail!("Daemon did not report the expected version after restart")
    }
    // Before pending is durable no installed file changes. With pending present,
    // recovery is idempotent, even after power loss between individual renames.
    fn prepare(dir: &Path, txn: &Path) -> Result<()> {
        fs::create_dir(txn.join("old"))?;
        for name in BINS {
            let path = dir.join(name);
            let m = fs::symlink_metadata(&path)?;
            if !m.is_file() || m.file_type().is_symlink() {
                bail!("Managed binaries must be regular files");
            }
            copy_sync(&path, &txn.join("old").join(name))?;
        }
        sync_dir(&txn.join("old"))?;
        let mut journal = File::create(txn.join("pending"))?;
        journal.write_all(b"rollback required\n")?;
        journal.sync_all()?;
        sync_dir(txn)
    }
    fn replace(dir: &Path, stage: &Path) -> Result<()> {
        for name in BINS {
            fs::rename(stage.join(name), dir.join(name))?;
            sync_dir(dir)?;
        }
        Ok(())
    }
    fn restore(dir: &Path, txn: &Path) -> Result<()> {
        for name in BINS {
            let next = txn.join(format!("restore-{name}"));
            if next.exists() {
                fs::remove_file(&next)?;
            }
            copy_sync(&txn.join("old").join(name), &next)?;
            fs::rename(&next, dir.join(name))?;
            sync_dir(dir)?;
        }
        Ok(())
    }
    pub async fn run() -> Result<()> {
        if unsafe { libc::geteuid() } != 0 {
            bail!("Automatic updates require the root-owned updater service");
        }
        let exe = std::env::current_exe()?.canonicalize()?;
        let parent = exe.parent().context("No installation directory")?;
        // The independent worker survives an interrupted or unhealthy st update.
        let dir = if parent.file_name().is_some_and(|n| n == ".auto-update") {
            parent
                .parent()
                .context("No managed installation directory")?
        } else {
            parent
        };
        trusted(dir)?;
        let state = dir.join(".auto-update");
        trusted(&state)?;
        let _lock = lock(&state)?;
        if !state.join("enabled").is_file() {
            bail!("Automatic updates are disabled");
        }
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(180))
            .build()?;
        let token_path = crate::daemon::token_path();
        trusted(&token_path)?;
        let token = fs::read_to_string(token_path)?;
        let token = token.trim();
        let txn = state.join("transaction");
        if txn.join("pending").exists() {
            restore(dir, &txn)?;
            restart()?;
            // Don't use CURRENT_VERSION: this process may have started mid-update.
            let old = fs::read_to_string(txn.join("previous-version"))?;
            healthy(&client, token, old.trim()).await?;
            fs::remove_dir_all(&txn)?;
            sync_dir(&state)?;
            bail!("Recovered interrupted update; next scheduled run will retry");
        }
        for name in BINS {
            trusted(&dir.join(name))?;
        }
        let current = binary_version(&dir.join("st"), &state)?;
        let release: Release = serde_json::from_slice(&body(&client, FEED, 1024 * 1024).await?)?;
        if stable(&release.tag_name)? <= stable(&current)? {
            println!("No newer stable release; daemon unchanged");
            return Ok(());
        }
        let os = if cfg!(target_os = "macos") {
            "apple-darwin"
        } else {
            "unknown-linux-gnu"
        };
        let name = format!("st-{}-{}.tar.gz", std::env::consts::ARCH, os);
        let asset = release
            .assets
            .iter()
            .find(|a| a.name == name)
            .context("Stable release has no binary for this platform")?;
        let url = reqwest::Url::parse(&asset.browser_download_url)?;
        if url.scheme() != "https"
            || url.host_str() != Some("i1.is")
            || !url.path().starts_with("/releases/smart-tree/")
            || !url.username().is_empty()
            || url.password().is_some()
            || url.port().is_some()
        {
            bail!("Untrusted asset URL");
        }
        let bytes = body(&client, url.as_str(), LIMIT).await?;
        let expected = asset
            .digest
            .strip_prefix("sha256:")
            .context("Release lacks SHA-256 digest")?;
        if hex::decode(expected)? != Sha256::digest(&bytes).as_slice() {
            bail!("Release checksum mismatch");
        }
        // Suppress repeated installation of a build that failed its health check.
        if fs::read_to_string(state.join("rejected-digest"))
            .ok()
            .as_deref()
            == Some(expected)
        {
            bail!("This release previously failed; awaiting a different release or operator reset");
        }
        healthy(&client, token, &current).await?;
        if txn.exists() {
            fs::remove_dir_all(&txn)?;
        }
        fs::create_dir(&txn)?;
        sync_dir(&state)?;
        fs::set_permissions(&txn, fs::Permissions::from_mode(0o700))?;
        let stage = txn.join("new");
        fs::create_dir(&stage)?;
        extract(&bytes, &stage)?;
        for name in BINS {
            let mut c = Command::new(stage.join(name));
            c.arg("--help").stdout(Stdio::null());
            command(c)?;
        }
        if binary_version(&stage.join("st"), &txn)? != release.tag_name.trim_start_matches('v') {
            bail!("Candidate version disagrees with release metadata");
        }
        let mut old = File::create(txn.join("previous-version"))?;
        old.write_all(current.as_bytes())?;
        old.sync_all()?;
        prepare(dir, &txn)?;
        let installed = async {
            replace(dir, &stage)?;
            restart()?;
            healthy(&client, token, &release.tag_name).await
        }
        .await;
        if let Err(error) = installed {
            restore(dir, &txn)?;
            restart()?;
            healthy(&client, token, &current).await?;
            fs::write(state.join("rejected-digest"), expected)?;
            fs::remove_dir_all(&txn)?;
            sync_dir(&state)?;
            bail!("Update failed and previous version restored: {error}");
        }
        // Removing pending commits the transaction. Retain old binaries for inspection.
        fs::remove_file(txn.join("pending"))?;
        sync_dir(&txn)?;
        fs::write(
            state.join("last-success"),
            format!("{} {}\n", release.tag_name, expected),
        )?;
        println!(
            "Updated all four binaries to {} and verified the restarted daemon",
            release.tag_name
        );
        Ok(())
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        fn bundle(names: &[&str]) -> Vec<u8> {
            let mut builder = tar::Builder::new(Vec::new());
            for name in names {
                let mut h = tar::Header::new_gnu();
                h.set_size(3);
                h.set_mode(0o755);
                h.set_cksum();
                builder.append_data(&mut h, name, &b"bin"[..]).unwrap();
            }
            let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
            gz.write_all(&builder.into_inner().unwrap()).unwrap();
            gz.finish().unwrap()
        }
        #[test]
        fn accepts_complete_bundle_and_rejects_duplicates_or_missing_binaries() {
            let d = tempfile::tempdir().unwrap();
            extract(&bundle(&BINS), d.path()).unwrap();
            for n in BINS {
                assert_eq!(fs::read(d.path().join(n)).unwrap(), b"bin");
                assert_eq!(
                    fs::metadata(d.path().join(n)).unwrap().mode() & 0o777,
                    0o755
                );
            }
            for names in [
                vec!["st", "std", "m8"],
                vec!["st", "st", "std", "m8", "n8x"],
                vec!["unexpected"],
            ] {
                let d = tempfile::tempdir().unwrap();
                assert!(extract(&bundle(&names), d.path()).is_err());
            }
        }
        #[test]
        fn version_probe_rejects_unknown_executables() {
            let d = tempfile::tempdir().unwrap();
            let binary = d.path().join("st");
            fs::write(&binary, b"#!/bin/sh\necho 'Smart Tree v10.1.0'\n").unwrap();
            fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
            assert_eq!(binary_version(&binary, d.path()).unwrap(), "10.1.0");
            fs::write(&binary, b"#!/bin/sh\necho 'some other program 10.1.0'\n").unwrap();
            assert!(binary_version(&binary, d.path()).is_err());
        }
        #[test]
        fn stable_only() {
            for s in ["1.2.3-rc1", "1.2", "01.2.3", "1.2.3.4", "1.2.x"] {
                assert!(stable(s).is_err());
            }
            assert!(stable("v10.0.2").unwrap() > stable("10.0.1").unwrap());
        }
        #[test]
        fn crash_recovery_restores_entire_bundle_idempotently() {
            let d = tempfile::tempdir().unwrap();
            let t = d.path().join("txn");
            fs::create_dir(&t).unwrap();
            for n in BINS {
                fs::write(d.path().join(n), format!("old {n}")).unwrap();
            }
            prepare(d.path(), &t).unwrap();
            fs::write(d.path().join("st"), "new").unwrap();
            restore(d.path(), &t).unwrap();
            restore(d.path(), &t).unwrap();
            for n in BINS {
                assert_eq!(
                    fs::read_to_string(d.path().join(n)).unwrap(),
                    format!("old {n}")
                );
            }
        }
        #[test]
        fn bundle_failure_rolls_back_partial_replacement() {
            let d = tempfile::tempdir().unwrap();
            let t = d.path().join("txn");
            fs::create_dir(&t).unwrap();
            let s = t.join("new");
            fs::create_dir(&s).unwrap();
            for n in BINS {
                fs::write(d.path().join(n), "old").unwrap();
            }
            fs::write(s.join("st"), "new").unwrap();
            prepare(d.path(), &t).unwrap();
            assert!(replace(d.path(), &s).is_err());
            restore(d.path(), &t).unwrap();
            for n in BINS {
                assert_eq!(fs::read(d.path().join(n)).unwrap(), b"old");
            }
        }
        #[test]
        fn concurrent_jobs_are_excluded() {
            let d = tempfile::tempdir().unwrap();
            let a = lock(d.path()).unwrap();
            assert!(lock(d.path()).is_err());
            drop(a);
            assert!(lock(d.path()).is_ok());
        }
        #[test]
        fn rejects_links_and_missing_members() {
            let d = tempfile::tempdir().unwrap();
            let mut b = tar::Builder::new(Vec::new());
            let mut h = tar::Header::new_gnu();
            h.set_entry_type(tar::EntryType::Symlink);
            h.set_size(0);
            h.set_mode(0o755);
            h.set_link_name("/etc/passwd").unwrap();
            h.set_cksum();
            b.append_data(&mut h, "st", &b""[..]).unwrap();
            let bytes = b.into_inner().unwrap();
            let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
            gz.write_all(&bytes).unwrap();
            assert!(extract(&gz.finish().unwrap(), d.path()).is_err());
        }
    }
}

pub async fn run() -> Result<()> {
    #[cfg(unix)]
    {
        unix::run().await
    }
    #[cfg(not(unix))]
    {
        bail!("Automatic daemon updates currently support macOS and Linux only")
    }
}
