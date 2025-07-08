# this.env
Light‑weight **environment recognition & trust middleware** for Rust.
It provides a simple, extensible way to determine the origin of requests and their trustworthiness, allowing your application to make informed decisions about how to handle them.

**Goal:** Let your application decide and learn *where* it is running (localhost, extensions, remote web, CLI, …) and *how much it should trust* that origin – before you execute business‑logic.

---

## 🛠️ Main Rust API
| Call | What it gives you |
|------|-------------------|
| `Env::resolve(&EnvRequest)` | Returns `EnvStatus` (`Approved`, `PendingApproval`, `Blocked`). |
| `Env::status(&self, db)`    | Re‑evaluate status for an existing `Env`. |
| `Env::add_endorsement(db, e)` | Add / change a user decision. |
| `Env::get_endorsements(db, domain)` | List all recorded decisions. |
| `Env::is_endorsed_by(db, domain, who)` | `true` / `false`. |

All data lives in a single `.db` file next to your executable. No server, no migrations.

------

# 🧠 MIDDLEWARE

A smart middleware system that inspects incoming HTTP requests and evaluates their environment (origin, IP, headers, cookies, etc.) to determine whether the request should be Approved, Blocked, or Pending Approval. It is ideal for building decentralized permission systems and real-time trust evaluation.

1. Recibe EnvRequest
2. Busca identidad → log
3. Si no hay identidad → PendingApproval
4. Si hay identidad:

  a. Instancia Env → log

  b. ¿Endorsed?

   \- Sí → Approved

   \- No → PendingApproval

5. ¿Blocked por alguna regla? → Blocked

## 🚀 Quick start (Actix Web)

```rust
use this_env::actixMiddleware; // import
// Inside your Actix `App` builder
App::new()
    .wrap(actixMiddleware::default()) // 1‑line, sensible defaults
    .configure(routes::config);
```

Add Port Number to the Middleware.

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

This file defines a status handler using **this.env**. It responds to **HTTP GET requests** at the /status endpoint with the current environment status. It uses the this.env crate to read the environment status stored by the middleware. The response includes the status, port, and username in JSON format.

```rust
use actix_web::{get, HttpRequest, HttpResponse, Responder, HttpMessage};
use this_env::EnvStatus;
use serde_json::json;

#[get("/status")]
async fn status(req: HttpRequest) -> impl Responder {
    let decision = req
        .extensions()
        .get::<EnvStatus>()
        .unwrap(); // Safe in this context; middleware always sets it

    HttpResponse::Ok().json(json!({
        "status": format!("{decision:?}")
    }))
}
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

### 📦 Key Components

1. **Env**
Represents the interpreted request “environment” extracted from an incoming Actix HttpRequest.
Usage:

```rust
let env_request = Env::from_request(&req);
```

This attempts to extract identifying info (e.g., origin, IP, etc.) into an evaluable context.

2. **EnvStatus**
An enum returned by Env::resolve that represents the outcome of evaluating a request environment.
Variants:

```rust
pub enum EnvStatus {
    Approved,
    PendingApproval(PendingData),
    Blocked(String),
}
```

• **Approved:** The request is trusted and allowed to proceed.
**• PendingApproval(data):** The request is not yet decided; it awaits user confirmation or other logic.
• **Blocked(reason):** The request is explicitly denied, with an optional reason string.

###### Usage:

```rust
let status = match env_request {
    Some(env) => Env::resolve(&env).unwrap_or(EnvStatus::Blocked("not-resolved".into())),
    None => EnvStatus::Blocked("missing-env".into()),
};
```

----

### ✅ Status Response Integration

You can use this logic to return a JSON response in your Actix handler:

```rust
let status_str = match resolved_status {
    EnvStatus::Approved => "approved",
    EnvStatus::PendingApproval(_) => "pending",
    EnvStatus::Blocked(_) => "blocked",
};

HttpResponse::Ok().json(StatusResponse {
    status: status_str.to_string(),
    port: 7777,
    username: "abellae",
});
```


Maintained by [neurons.me](https://neurons.me) — crafted with ☕ by **suiGn**.