# this.env
Light‑weight **environment recognition & trust middleware** for Rust.
**Goal:** Let your application decide and learn *where* it is running (localhost, extensions, remote web, CLI, …) and *how much it should trust* that origin – before you execute business‑logic.

------

# 🧠 MIDDLEWARE

A smart middleware system that inspects incoming HTTP requests and evaluates their environment (origin, IP, headers, cookies, etc.) to determine whether the request should be Approved, Blocked, or Pending Approval. It is ideal for building decentralized permission systems and real-time trust evaluation.

##### 🚀 Quick start (Actix Web)

```rust
use this_env::actixMiddleware; // import
// Inside your Actix `App` builder
App::new()
    .wrap(actixMiddleware::default()) // 1‑line, sensible defaults
    .configure(routes::config);
```

###### Add Port Number and Instance to the Middleware.

We pass the **port** to the ActixMiddleware configuration so that this.env can **differentiate its database and runtime context per running instance/port**.

##### Custom rules

```rust
use this_env::{ThisEnvMiddleware, ThisEnvMiddlewareConfig};

let mw_config = ThisEnvMiddlewareConfig {
    port: port_clone.clone(),
    instance: instance_clone.clone(),
    ..Default::default()
};

HttpServer::new(move || {
    let mw_config_clone = mw_config.clone();
    App::new()
        .app_data(state.clone())
        .wrap(cors_permissive())
        .wrap(ThisEnvMiddleware::new(mw_config_clone))
        .configure(router::config)
        .service(Files::new("/", "./static").index_file("html/index.html"))
})
```

## EndPoints

Middleware endpoints like **/__env/logs** are **special internal routes exposed by this.env middleware** for introspection and management.

#### **/__env/**:

This.Env Middleware Home Dashboard.

#### **/__env/logs**:

- **Purpose**: Returns the logs collected by this.env, useful for debugging and monitoring requests or data structure analysis.
- **Who uses it**: Typically **developers or admin tools**, not public clients.
- **Behavior**: The middleware intercepts and stores request/response metadata, and this endpoint serves that stored information as JSON or plain text.

---


Maintained by [neurons.me](https://neurons.me) — crafted with ☕ by **suiGn**.