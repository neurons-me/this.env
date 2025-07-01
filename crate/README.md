# this.env
Light‑weight **environment recognition & trust middleware** for Rust.


**Goal:** Let your application decide *where* it is running (localhost, extensions, remote web, CLI, …) and *how much it should trust* that origin – before you execute business‑logic.

`this.env` constructs a persistent, inspectable model of the environment where each request originates. It tracks not only the domain and protocol but also metadata, route-level context, and endorsements (approvals or rejections). This enables a trust system where the app can determine whether to allow, challenge, or deny a request, and how to treat its origin in future interactions.

---

## 📦 Core Data‑model
```
┌──────────────────────────────────────────────┐
│                   Env                        │
│ ─ domain     : String                        │
│ ─ id         : String ▸ deterministic hash   │
│ ─ env_type   : EnvType                       │
│ ─ trust      : TrustLevel                    │
│ ─ routes     : HashMap<String, RouteInfo>    │
│ ─ parent     : Option<String>                │
└──────────────────────────────────────────────┘
                     ↓
              ┌────────────────┐
              │  Endorsement   │
              └────────────────┘
```

---

## 🔀 Request / Response flow - Middleware.
```
Incoming HTTP / WS / CLI
            │
            ▼
┌───────────────┐           ┌───────────────────┐
│ Middleware.   │──────────▶│ EnvRequest        │
└───────────────┘           └───────────────────┘
                                   │
                                   ▼
                          Env::resolve(&req)
                                   │
     ┌─────────────────────────────┼──────────────────────────────┐
     │                             │                              │
     ▼                             ▼                              ▼
Approved (pass‑through)   PendingApproval (401)         Blocked (403)
```

The Middleware turns its native request into **`EnvRequest`** and call `Env::resolve`.  
The returned **`EnvStatus`** drives your policy (continue, show modal, deny).

---

## 🗄️ Public API (most used)
| Function / type                      | Purpose |
|--------------------------------------|---------|
| `EnvRequest` (`Http`, `Ws`, `Cli`)   | Framework‑agnostic wrapper for inbound traffic. |
| `Env::new(domain, typ, trust)`       | Create an `Env` in memory. |
| `Env::init_sqlite(path)`             | Boot / open the SQLite registry. |
| `Env::resolve(&EnvRequest)`          | **One‑liner** to get `EnvStatus` for a request. |
| `Env::status(&self, &conn)`          | Evaluate endorsements for an existing env. |
| `Env::add_endorsement(...)`          | Store/overwrite an endorsement (approve/block). |
| `Env::get_endorsements(...)`         | Fetch all endorsements. |
| `Env::is_endorsed_by(...)`           | Convenience helper. |
| `Env::get_parent / get_children`     | Walk the hierarchy (`admin.foo.com` ➝ `foo.com`). |

See the inline docs (`cargo doc --open`) for the full surface.

---

## 🛠 Adapters out‑of‑the‑box
| Adapter crate               | Status |
|-----------------------------|--------|
| **`this_env::ActixMiddleware`** | ✅ stable |
|       |  |
|              |  |

---

## 🚀 Quick‑start (Actix)

```rust
use this_env::ActixMiddleware;

HttpServer::new(|| {
    App::new()
        .wrap(ActixMiddleware)             // ← drop‑in
        .configure(routes)
})
.bind(("127.0.0.1", 7777))?
.run()
.await?;
```

When an unknown domain hits your API, `this.env` intercepts **before**
your handlers and returns the **pending‑approval** HTML (or JSON).  
Approve/Block once and it is persisted in `.this/env/.env.db`.

---

## ✨ Why this.env?
* **Framework‑agnostic** – adapters are <100 lines.
* **Hierarchical trust** – sub‑domains inherit unless overridden.
* **Single‑file storage** – no extra service.
* **Extensible** – add Ws, Desktop, P2P flows next.

Maintained by [neurons.me](https://neurons.me) • Authored by **suiGn**