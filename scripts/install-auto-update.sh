#!/bin/bash
# Opt-in automatic stable releases for an already-installed managed daemon.
# Run as root AFTER installing a build that supports --auto-update.
set -euo pipefail
export PATH=/usr/bin:/bin:/usr/sbin:/sbin
mode=${1:-enable}
case "$mode" in enable|disable|status) ;; *) echo 'Usage: sudo bash scripts/install-auto-update.sh [enable|disable|status]' >&2; exit 2;; esac
[[ $(id -u) == 0 ]] || { echo 'Run as root.' >&2; exit 1; }
case $(uname -s) in
 Darwin)
  bin=/Library/PrivilegedHelperTools/is.8b.smart-tree/st
  state=/Library/PrivilegedHelperTools/is.8b.smart-tree/.auto-update
  job=/Library/LaunchDaemons/is.8b.smart-tree-updater.plist
  label=is.8b.smart-tree-updater
  daemon=/Library/LaunchDaemons/is.8b.smart-tree-daemon.plist
  ;;
 Linux)
  bin=/usr/local/bin/st
  state=/usr/local/bin/.auto-update
  job=/etc/systemd/system/smart-tree-updater.service
  ;;
 *) echo 'Only macOS and Linux are supported.' >&2; exit 1;;
esac
if [[ $mode == status ]]; then
 if [[ -f $state/enabled ]]; then echo 'Automatic stable updates enabled (daily).'; else echo 'Automatic updates disabled.'; fi
 if [[ -f $state/last-success ]]; then cat "$state/last-success"; fi
 if [[ -f $state/transaction/pending ]]; then echo 'Interrupted update pending recovery.'; fi
 if [[ -f $state/rejected-digest ]]; then echo 'A failed release is held until its digest changes or the marker is removed.'; fi
 exit 0
fi
if [[ $mode == disable ]]; then
 rm -f "$state/enabled"
 if [[ $(uname -s) == Darwin ]]; then launchctl disable "system/$label"
 else systemctl disable --now smart-tree-updater.timer; fi
 echo 'Automatic updates disabled; daemon and data preserved.'
 exit 0
fi
[[ -x $bin && ! -L $bin ]] || { echo "Install a managed, regular executable at $bin first." >&2; exit 1; }
"$bin" --help | grep -- '--auto-update' >/dev/null || { echo 'Installed build lacks --auto-update.' >&2; exit 1; }
# The privileged worker independently validates root ownership and ancestor permissions.
umask 077
mkdir -p "$state"
chmod 700 "$state"
# Keep the controller independent from the daemon binaries it replaces.
install -m 755 "$bin" "$state/worker.new"
mv -f "$state/worker.new" "$state/worker"
if [[ $(uname -s) == Darwin ]]; then
 actual=$(/usr/libexec/PlistBuddy -c 'Print :ProgramArguments:0' "$daemon")
 [[ $actual == "$bin" ]] || { echo 'Daemon executable differs from updater target; no job installed.' >&2; exit 1; }
 # Match the existing service token; refuse unknown layouts rather than changing auth.
 token=$(/usr/libexec/PlistBuddy -c 'Print :EnvironmentVariables:ST_TOKEN_PATH' "$daemon")
 [[ $token == '/Library/Application Support/SmartTree/daemon.token' ]] || { echo 'Unsupported token layout; no job installed.' >&2; exit 1; }
 cat > "$job" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>Label</key><string>is.8b.smart-tree-updater</string>
<key>ProgramArguments</key><array><string>/Library/PrivilegedHelperTools/is.8b.smart-tree/.auto-update/worker</string><string>--auto-update</string></array>
<key>EnvironmentVariables</key><dict><key>ST_TOKEN_PATH</key><string>/Library/Application Support/SmartTree/daemon.token</string><key>TOKIO_WORKER_THREADS</key><string>2</string><key>RAYON_NUM_THREADS</key><string>2</string><key>PATH</key><string>/usr/bin:/bin:/usr/sbin:/sbin</string></dict>
<key>UserName</key><string>root</string><key>RunAtLoad</key><true/>
<key>StartInterval</key><integer>86400</integer>
<key>ProcessType</key><string>Background</string><key>Nice</key><integer>10</integer>
<key>StandardOutPath</key><string>/var/log/smart-tree-updater.log</string>
<key>StandardErrorPath</key><string>/var/log/smart-tree-updater.log</string>
</dict></plist>
PLIST
 chmod 644 "$job"; chown root:wheel "$job"; plutil -lint "$job"
 : > "$state/enabled"
 launchctl enable "system/$label"
 if launchctl print "system/$label" >/dev/null 2>&1; then
  launchctl kickstart "system/$label"
 else
  launchctl bootstrap system "$job"
 fi
 launchctl print "system/$label"
else
 systemctl cat smart-tree-daemon.service | grep -q 'ExecStart=/usr/local/bin/st --http-daemon' || { echo 'Unsupported daemon executable.' >&2; exit 1; }
 cat > "$job" <<'UNIT'
[Unit]
Description=Smart Tree verified automatic stable update
After=network-online.target smart-tree-daemon.service
Wants=network-online.target
[Service]
Type=oneshot
ExecStart=/usr/local/bin/.auto-update/worker --auto-update
Environment=ST_TOKEN_PATH=/var/lib/smart-tree/daemon.token
Environment=PATH=/usr/bin:/bin:/usr/sbin:/sbin
Environment=TOKIO_WORKER_THREADS=2
Environment=RAYON_NUM_THREADS=2
UMask=0077
Nice=10
TimeoutStartSec=15min
UNIT
 cat > /etc/systemd/system/smart-tree-updater.timer <<'UNIT'
[Unit]
Description=Check for Smart Tree stable updates daily
[Timer]
OnBootSec=5min
OnUnitActiveSec=24h
RandomizedDelaySec=15min
[Install]
WantedBy=timers.target
UNIT
 chmod 644 "$job" /etc/systemd/system/smart-tree-updater.timer
 : > "$state/enabled"
 systemctl daemon-reload
 systemctl enable --now smart-tree-updater.timer
 systemctl list-timers smart-tree-updater.timer
fi
