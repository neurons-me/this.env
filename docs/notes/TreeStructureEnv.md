#  **Estructura jerárquica**

Cada Env representa **un dominio** (domain), e.g. neurons.me.

Tiene:

- **parent:** apunta a su dominio raíz si es un subdominio (e.g. admin.neurons.me → neurons.me)
- **children:** se pueden obtener con get_children(...) desde SQLite.
- **routes:** es un mapa interno (HashMap<String, RouteInfo>) que representa rutas específicas dentro del dominio (como /status, /login, etc.).

### **Entrada del sistema: EnvRequest**

- Puede venir de:

  - Http
  - Ws
  - CLI

- Se normaliza y se extrae el host (ej. dev.neurons.me), que luego se parte (split_host) en:

  - root (dominio principal)
  - subdomain o nombre completo (útil para decidir si es hijo, padre, etc.)

  

