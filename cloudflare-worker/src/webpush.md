# Web Push on Cloudflare Worker

Full RFC 8291 encryption in a Worker is easiest via:

```bash
npm install web-push
```

Then in a bundler build, call `webpush.sendNotification(subscription, payload, { vapidDetails })`.

Until then:
- `/v1/wake` always **queues** the mid
- App **polls** `/v1/poll` every 25s (and on resume)
- Local notification fires on poll hit

This is enough for reliable wake without Firebase when the OS allows background timers;
on strict mobile OEMs, pair with a foreground service or periodic WorkManager task.
