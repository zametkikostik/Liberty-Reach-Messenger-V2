# Remaining full source upload

Local git has the complete 111-file tree (commit on builder).

Already on GitHub: README EN/RU/BG, workflows, CI, many docs, worker package, Flutter main/pubspec, core services (bridge, vault, voice, typing, keystore, network).

**To finish in one shot from your machine:**

```bash
# Get the full tree from the development environment or re-clone and merge
cd liberty-messenger
git remote add origin https://github.com/zametkikostik/Liberty-Reach-Messenger-V2.git
git push -u origin main --force
```

Or enable Actions after `mobile/lib/screens/*` and `rust-core/src/**` are present.

Status of this remote: bootstrap for APK workflow + core app entrypoints.
