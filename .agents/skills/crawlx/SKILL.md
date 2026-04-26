name: crawlx
description: >
  How to operate the CrawlX scraping system in this repo — using Tandem Browser on Windows,
  reading the API token from %USERPROFILE%, running the Tandem HTTP scrape flow, using
  TandemBrowserEngine in code, running package tests, and working with the SafeLine WAF.
  Use this skill whenever the user mentions Tandem, TandemBrowserEngine, scraping with CrawlX,
  waterfall engine, Windows browser setup, SafeLine WAF, or wants to scrape a URL using the
  local browser backend.

# CrawlX — Operator Guide (Windows Native)

CrawlX is a monorepo web-scraping orchestrator. The primary external browser backend is
**Tandem Browser**, a local Electron browser that exposes a REST API on `http://127.0.0.1:8765`.

---

## 1. Starting Tandem Browser (Windows 11 Native)

Tandem is currently installed at `C:\dev\tandem`. It runs as a standalone Windows service.

**Verify it's up:**
```powershell
# In PowerShell
curl.exe -sf http://127.0.0.1:8765/status
```

Tandem is ready when `GET http://127.0.0.1:8765/status` returns `{"ready":true,...}`.

> **Note:** On Windows, Tandem addresses several memory leaks found in the WSL build and
> utilizes native WDDM graphics. It binds to `0.0.0.0:8765` to support local and Tailscale
> connectivity.

---

## 2. API Token

Tandem writes its bearer token to `%USERPROFILE%\.tandem\api-token`.

```powershell
$TOKEN = Get-Content "$env:USERPROFILE\.tandem\api-token"
```

Every request (except `GET /status`) requires the header `Authorization: Bearer $TOKEN`.

---

## 3. Tandem HTTP API — Scrape Flow

All endpoints are at `http://127.0.0.1:8765`. **Critical:** Windows native requires
bypass headers for DevTools/JS execution.

### Recommended: One-off Script Execution (No REPL)
To run multi-line scripts without dropping into the Node REPL, use the PowerShell 
**Here-String** syntax piped into `tsx`:

```powershell
@'
const token = 'YOUR_TOKEN';
const url = 'https://example.com';
const res = await fetch('http://127.0.0.1:8765/tabs/open', {
  method: 'POST',
  headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' },
  body: JSON.stringify({ url, focus: true, allowStealthCompromise: true })
});
console.log(await res.json());
'@ | npx.cmd --yes tsx
```

---

## 4. Key Packages

| Package | Path | What it does |
|---------|------|--------------|
| `@crawlx/waterfall-engine` | `packages/waterfall-engine/` | `TandemBrowserEngine` (Defaults to port 8765) |
| `@crawlx/jobs` | `packages/jobs/` | `ScrapeWorker` with Tandem + Manual Review wiring |
| `@crawlx/db` | `packages/db/` | Schemas for `watch_jobs` and `manual_reviews` |

### TandemBrowserEngine — TypeScript usage

```typescript
import { TandemBrowserEngine } from './packages/waterfall-engine/src/engines/tandem-browser';

const engine = new TandemBrowserEngine({
  apiToken: '...',                 // from %USERPROFILE%\.tandem\api-token
  baseUrl: 'http://127.0.0.1:8765', // default Windows port
  timeoutMs: 30_000,
});

const result = await engine.scrape({ url: 'https://example.com' });
```

---

## 5. SafeLine WAF Test Environment

| Thing | Value |
|-------|-------|
| Direct origin (Nginx) | `http://localhost:8095` |
| WAF-protected hostname | `http://mock.localtest.me` |
| SafeLine access log | `C:\dev\Safeline\safeline-runtime\logs\nginx\safeline\accesslog_1` |

**Check the last 10 log entries:**
```powershell
Get-Content "C:\dev\Safeline\safeline-runtime\logs\nginx\safeline\accesslog_1" -Tail 10
```

---

## 6. Networking & Docker

If the `browser-worker` is in Docker, reach the Windows Tandem via:
`http://host.docker.internal:8765`

**Warning:** If `wslrelay.exe` is running, it may capture `127.0.0.1:8765` and route to WSL. 
Ensure WSL isolation when testing the Windows-native backend.

---

## 7. Manual Review Queue

When automated engines fail, the system logs a record to the `manual_reviews` table.
Status can be `pending`, `completed`, or `rejected`.
