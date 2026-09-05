# Full source

The complete tree (111 files) lives in the development workspace commit.

To push **everything** from a machine with GitHub auth:

```bash
cd liberty-messenger   # or clone this repo
# if you have the full local tree:
git remote add origin https://github.com/zametkikostik/Liberty-Reach-Messenger-V2.git
git push -u origin main
```

Or download the local archive from the builder and extract over this repo, then push.

GitHub Actions will build APK when `mobile/` is complete.
