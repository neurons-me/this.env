# this.env
Light‑weight **environment recognition & trust middleware** for Rust.
It provides a simple, extensible way to determine the origin of requests and their trustworthiness, allowing your application to make informed decisions about how to handle them.

**Goal:** Let your application decide *where* it is running (localhost, extensions, remote web, CLI, …) and *how much it should trust* that origin – before you execute business‑logic.

*A tiny helper that lets your app know **where** a request comes from and **whether you should trust it.***

---

## 🚀 Quick start (Actix Web)

```rust
use this_env::actixMiddleware; // import
// Inside your Actix `App` builder
App::new()
    .wrap(actixMiddleware::default()) // 1‑line, sensible defaults
    .configure(routes::config);
```

### Custom rules

```rust
use this_env::{actixMiddleware, ActixMwConfig};

let cfg = ActixMwConfig {
    allow_pending: true,   // let “unknown yet” domains through
    prefer_html: true,     // serve pretty HTML pages to browsers
    ..Default::default()   // keep the rest as default
};

App::new()
    .wrap(actixMiddleware::config(cfg))
    .configure(routes::config);
```

---

## 🧐 What it does
1. **Sees every inbound request** (HTTP, WebSocket, CLI …).
2. **Figures out its “environment”** (localhost, remote site, browser extension …).
3. Checks a **tiny SQLite registry** to know if that environment is:
   * `Approved` – trusted, go ahead.
   * `PendingApproval` – first time seen, ask the user.
   * `Blocked` – explicitly forbidden.
4. Returns the decision *before* your business‑logic runs.

---

## 🛠️ Main Rust API (most apps only need the first two)
| Call | What it gives you |
|------|-------------------|
| `Env::resolve(&EnvRequest)` | Returns `EnvStatus` (`Approved`, `PendingApproval`, `Blocked`). |
| `Env::status(&self, db)`    | Re‑evaluate status for an existing `Env`. |
| `Env::add_endorsement(db, e)` | Add / change a user decision. |
| `Env::get_endorsements(db, domain)` | List all recorded decisions. |
| `Env::is_endorsed_by(db, domain, who)` | `true` / `false`. |

All data lives in a single `.db` file next to your executable. No server, no migrations.

---

## 🙋‍♀️ Why you might care
* **Security:** stop untrusted iframes/extensions from poking your localhost API.
* **UX:** show a clean “Do you allow this?” page instead of a CORS panic.
* **Zero‑setup:** drop‑in, no external service.

Maintained by [neurons.me](https://neurons.me) — crafted with ☕ by **suiGn**.