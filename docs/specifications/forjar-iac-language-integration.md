# Sub-spec: Forjar Infrastructure-as-Code Language Integration

**Parent:** [SPECIFICATION.md](../SPECIFICATION.md)
**Version:** 1.0.0
**Status:** DRAFT
**Date:** 2026-04-03
**Ticket:** INFRA-001

---

## 0. Prerequisites

- **forjar is not currently a Ruchy dependency** -- it must be added to `Cargo.toml` before any integration work begins.
- **No existing parser support for `infra` blocks** -- this is greenfield parser work. The lexer, parser, and AST must be extended to recognize and represent `infra` syntax.
- **All referenced files must be created**: `src/backend/transpiler/infra.rs` (transpiler backend), `src/infra/dag.rs` (DAG resolution), and associated test modules do not exist yet.

---

## 1. Overview

### 1.1 Current State

Ruchy has zero infrastructure-as-code integration. Users who want to manage servers,
deploy services, or configure machines must leave the Ruchy ecosystem and use
external tools (Ansible, Terraform, shell scripts).

### 1.2 Vision

The `infra {}` block becomes native Ruchy syntax. Infrastructure declarations are
first-class language constructs that the transpiler converts into type-checked Rust
code using the forjar library.

### 1.3 Forjar v1.2.1 Capabilities

| Property | Detail |
|----------|--------|
| State tracking | BLAKE3 content-addressed lock files |
| Transport | SSH (via russh), local execution |
| Execution model | DAG-ordered with dependency resolution |
| Resource types | 11 built-in resource providers |
| Delta sync | Copia integration for large file transfers |
| Idempotency | All operations are convergent |

### 1.4 Design Constraints

1. **Zero unsafe code** -- generated Rust follows the zero-unsafe policy (Issue #132)
2. **Deterministic output** -- same `.ruchy` input always produces same Rust code
3. **Type-checked parameters** -- template variables typed at transpile time
4. **DAG correctness** -- cycle detection at transpile time, not runtime
5. **No implicit network access** -- SSH requires explicit `machine` declarations

---

## 2. Language Syntax: `infra` Block

### 2.1 Full Example

```ruchy
infra webstack {
    machine prod {
        host: "10.0.1.50"
        user: "deploy"
        key:  "~/.ssh/id_ed25519"
    }

    resource package "nginx" on prod {
        state: "present"
        provider: "apt"
    }

    resource file "/etc/nginx/nginx.conf" on prod {
        source: "./configs/nginx.conf"
        owner: "root"
        mode: "0644"
        depends_on: [package("nginx")]
    }

    resource service "nginx" on prod {
        state: "running"
        enabled: true
        depends_on: [file("/etc/nginx/nginx.conf")]
    }

    policy {
        max_parallel: 4
        retry_count: 2
        timeout_seconds: 300
        on_failure: "stop"
    }
}
```

### 2.2 Syntax Grammar

```
infra_block     := "infra" IDENT "{" infra_item* "}"
infra_item      := machine_decl | resource_decl | policy_decl
machine_decl    := "machine" IDENT "{" kv_pair* "}"
resource_decl   := "resource" RESOURCE_TYPE STRING "on" IDENT "{" kv_pair* "}"
policy_decl     := "policy" "{" kv_pair* "}"
kv_pair         := IDENT ":" expr
RESOURCE_TYPE   := "package" | "file" | "service" | "mount" | "user"
                 | "docker" | "cron" | "network" | "pepita" | "model" | "gpu"
```

### 2.3 Parameter Templates and DAG Ordering

Template parameters (e.g., `f"app:{version}"`) are type-checked at transpile
time -- a type mismatch produces a compile error, not a runtime failure.

The `depends_on` field accepts resource references. The transpiler constructs
a DAG and rejects cycles at compile time:

```
package("nginx") --> file("/etc/nginx/nginx.conf") --> service("nginx")
```

### 2.4 Transpilation Target

The `infra` block transpiles to Rust code constructing a `ForjarConfig`:

```rust
use forjar::core::{ForjarConfig, Resource, Machine, Transport};
use forjar::core::resolver::DagResolver;
use forjar::core::planner::Planner;
use forjar::core::executor::Executor;

fn webstack() -> Result<(), forjar::Error> {
    let machine_prod = Machine::new("prod")
        .host("10.0.1.50")
        .user("deploy")
        .key("~/.ssh/id_ed25519")
        .transport(Transport::Ssh);

    let mut config = ForjarConfig::new("webstack");
    config.add_resource(
        Resource::package("nginx")
            .state("present").provider("apt").machine(&machine_prod),
    );
    config.add_resource(
        Resource::file("/etc/nginx/nginx.conf")
            .source("./configs/nginx.conf").owner("root").mode("0644")
            .machine(&machine_prod).depends_on(&["package:nginx"]),
    );
    config.add_resource(
        Resource::service("nginx")
            .state("running").enabled(true)
            .machine(&machine_prod).depends_on(&["file:/etc/nginx/nginx.conf"]),
    );

    let dag = DagResolver::resolve(&config)?;
    let plan = Planner::compute(&dag, &config)?;
    Executor::run(&plan, &config)?;
    Ok(())
}
```

---

## 3. Resource Types

All 11 forjar resource types are available as `infra` block resource declarations.

| Resource | States | Providers | Description |
|----------|--------|-----------|-------------|
| `package` | present, absent, latest | apt, dnf, brew, pacman | System package management |
| `file` | present, absent | local, copia | File creation, templating, permissions |
| `service` | running, stopped, restarted | systemd, openrc, launchd | Service lifecycle management |
| `mount` | mounted, unmounted | fstab, zfs, btrfs | Filesystem mount points |
| `user` | present, absent, locked | useradd, dscl | User account management |
| `docker` | running, stopped, absent | docker, podman | Container lifecycle |
| `cron` | present, absent | crontab, systemd-timer | Scheduled task management |
| `network` | up, down, configured | networkd, nmcli | Network interface config |
| `pepita` | trained, deployed, removed | pepita-cli | ML model lifecycle (pepita) |
| `model` | downloaded, cached, removed | huggingface, ollama | Model artifact management |
| `gpu` | allocated, released, monitored | nvidia-smi, rocm-smi | GPU resource management |

### 3.1 Resource Syntax Examples

```ruchy
# Package
resource package "postgresql-16" on db_server {
    state: "present"
    provider: "apt"
}

# File with Copia delta sync (for files >1MB)
resource file "/opt/models/weights.bin" on ml_server {
    source: "./models/weights.bin"
    provider: "copia"
    owner: "ml"
    mode: "0644"
}

# Docker container
resource docker "redis" on cache_server {
    image: "redis:7-alpine"
    ports: ["6379:6379"]
    volumes: ["/data/redis:/data"]
    state: "running"
    restart: "unless-stopped"
    depends_on: [mount("/data/redis")]
}

# GPU allocation
resource gpu "training-gpu" on ml_server {
    device: 0
    state: "allocated"
    provider: "nvidia-smi"
    memory_limit: "16GB"
}

# Pepita ML model
resource pepita "fraud-detector" on ml_server {
    state: "deployed"
    model_path: "./models/fraud_v3.pepita"
    endpoint_port: 8501
    depends_on: [gpu("training-gpu")]
}
```

---

## 4. Transpiler Integration

### 4.1 AST Representation

The parser produces an `InfraBlock` AST node:

```rust
pub enum Statement {
    // ... existing variants ...
    InfraBlock(InfraBlock),
}

pub struct InfraBlock {
    pub name: String,
    pub machines: Vec<MachineDecl>,
    pub resources: Vec<ResourceDecl>,
    pub policy: Option<PolicyDecl>,
    pub span: Span,
}

pub struct ResourceDecl {
    pub resource_type: ResourceType,
    pub name: Expr,
    pub machine_ref: String,
    pub properties: Vec<(String, Expr)>,
    pub depends_on: Vec<ResourceRef>,
    pub span: Span,
}
```

### 4.2 Transpiler Pipeline

```
Ruchy source
  --> Parser: InfraBlock AST node
  --> Type checker: validate parameter types, resolve template expansions
  --> DAG checker: detect cycles in depends_on graph (compile error if cycle)
  --> Transpiler: emit Rust code using forjar API
  --> Cargo build: compile with forjar = "1.2" as dependency
```

### 4.3 Forjar Core Operations

**DAG Resolution** -- `DagResolver::resolve(&config)` performs topological sort.
Parallel execution respects the partial order defined by `depends_on` edges.

**Plan Computation** -- `Planner::compute(&dag, &config)` diffs desired state
against current state, producing additions, changes, removals, and no-change sets.

**Execution** -- `Executor::run(&plan, &config)` applies the plan via the
configured transport (SSH or local), respecting `max_parallel`, `retry_count`,
and `timeout_seconds` from the policy block.

### 4.4 State Persistence via BLAKE3

After execution, forjar writes a `.forjar.lock` file:

```json
{
  "version": 2,
  "blake3_hash": "a1b2c3d4...",
  "resources": {
    "package:nginx": { "state": "present", "hash": "e5f6a7b8..." },
    "file:/etc/nginx/nginx.conf": { "state": "present", "hash": "c9d0e1f2..." },
    "service:nginx": { "state": "running", "hash": "34567890..." }
  },
  "last_applied": "2026-04-03T12:00:00Z"
}
```

BLAKE3 hashes enable fast drift detection, deterministic git-friendly lock files,
and content-addressed deduplication.

---

## 5. CLI Commands

| Command | Description | Forjar Operation |
|---------|-------------|------------------|
| `ruchy infra plan` | Show desired state diff without applying | `Planner::compute` |
| `ruchy infra apply` | Converge infrastructure to desired state | `Executor::run` |
| `ruchy infra drift` | Detect configuration drift (tripwire) | `DriftDetector::scan` |
| `ruchy infra status` | Show current resource states | `StateReader::current` |
| `ruchy infra destroy` | Remove all resources (reverse order) | `Executor::destroy` |

> **Performance consideration:** The transpile-then-compile-then-execute cycle adds ~10-30s latency compared to direct execution. For interactive use (`plan`, `status`, `drift`), consider caching compiled infra binaries so that only the first invocation incurs the full compilation cost.

### 5.1 Output Examples

```
$ ruchy infra plan deploy.ruchy
  + package:nginx (absent -> present) | ~ file:nginx.conf (hash mismatch) | = service:nginx
  Plan: 1 to add, 1 to change, 0 to remove, 1 unchanged.

$ ruchy infra apply deploy.ruchy --yes
  [1/2] package:nginx .. OK (1.2s) | [2/2] file:nginx.conf .. OK (0.3s)
  Applied: 2 succeeded, 0 failed. Lock file updated.

$ ruchy infra drift deploy.ruchy
  ! file:nginx.conf (modified outside forjar) | ! service:nginx (stopped, expected running)
  2 resources drifted. Run 'ruchy infra apply' to reconverge.
```

---

## 6. Recipes as Ruchy Functions

### 6.1 Parameterized Recipe

Forjar recipes map directly to Ruchy functions with typed parameters:

```ruchy
fun recipe_postgres(host: str, port: int, version: str) {
    infra postgres_setup {
        machine db { host: host; user: "deploy"; key: env("SSH_KEY") }
        resource package f"postgresql-{version}" on db { state: "present"; provider: "apt" }
        resource file "/etc/postgresql/postgresql.conf" on db {
            content: f"port = {port}\n"
            owner: "postgres"
            mode: "0600"
            depends_on: [package(f"postgresql-{version}")]
        }
        resource service "postgresql" on db {
            state: "running"
            enabled: true
            depends_on: [file("/etc/postgresql/postgresql.conf")]
        }
    }
}
```

### 6.2 Composable Modules and Library Convention

Recipes compose by calling other recipes. Standard recipes live in `infra/`
directories and are imported via Ruchy modules:

```ruchy
import infra.recipes.postgres
import infra.recipes.nginx

fun recipe_web_app(app_host: str, db_host: str) {
    recipe_postgres(db_host, 5432, "16")
    recipe_nginx(app_host, 443)
}
```

---

## 7. Safety and Determinism

### 7.1 Jidoka: Stop on First Failure

The executor stops immediately on the first resource failure (unless
`on_failure: "continue"` is explicitly set). The lock file records which
resources succeeded. No partial state ambiguity.

### 7.2 BLAKE3 Content-Addressed State

- **Deterministic:** Same input always produces same hash
- **Git-friendly:** Lock files are plain JSON, diffable, mergeable
- **Fast:** BLAKE3 is 3-14x faster than SHA-256

### 7.3 Atomic Lock File Writes

Lock file updates use temp-file-and-rename (single POSIX syscall) to prevent
corruption even if the process is killed mid-write.

### 7.4 Copia Delta Sync

Files larger than 1MB use Copia's rolling-checksum delta sync, transferring
only changed blocks to minimize bandwidth.

### 7.5 Transpiler Safety Guarantees

| Invariant | Enforcement |
|-----------|-------------|
| No `unsafe` blocks in generated code | AST visitor rejects `unsafe` nodes |
| No `static mut` in generated code | Uses `LazyLock<Mutex<T>>` per Issue #132 |
| No cycles in dependency graph | Topological sort at transpile time |
| All template variables are typed | Type checker validates before codegen |
| SSH keys never embedded in source | Only file paths and `env()` references allowed |

> **Note on SSH key path disclosure:** SSH key paths in transpiled code are string literals -- runtime resolution prevents path disclosure in compiled binaries. Add `.gitignore` rules for generated infra state (e.g., `.forjar.lock`, `target/`) to prevent accidental commits of sensitive infrastructure state.

---

## 8. Testing Requirements

### 8.1 Unit Tests

| Test Category | Count | Naming Convention |
|---------------|-------|-------------------|
| Infra block parsing | 15+ | `test_infra_001_xx_parse_*` |
| Resource type validation | 11+ | `test_infra_002_xx_resource_*` |
| Template expansion | 8+ | `test_infra_003_xx_template_*` |
| DAG cycle detection | 6+ | `test_infra_004_xx_dag_*` |
| Transpiler output | 10+ | `test_infra_005_xx_transpile_*` |

### 8.2 Integration Tests with Local Transport

Integration tests use `Transport::Local` to avoid SSH dependencies in CI:

```rust
#[test]
fn test_infra_integration_local_file() {
    let config = ForjarConfig::new("test").transport(Transport::Local);
    config.add_resource(Resource::file("/tmp/test_forjar.txt").content("hello"));
    let dag = DagResolver::resolve(&config).unwrap();
    let plan = Planner::compute(&dag, &config).unwrap();
    assert!(Executor::run(&plan, &config).is_ok());
    assert!(Path::new("/tmp/test_forjar.txt").exists());
}
```

### 8.3 Property Tests for DAG Resolution

```rust
proptest! {
    #[test]
    fn test_dag_resolution_never_panics(
        resources in prop::collection::vec(arb_resource(), 1..50),
        deps in prop::collection::vec(arb_dep_pair(), 0..20),
    ) {
        let config = build_config_from(resources, deps);
        match DagResolver::resolve(&config) {
            Ok(dag) => assert!(dag.is_topologically_sorted()),
            Err(e) => assert!(matches!(e, ForjarError::CycleDetected { .. })),
        }
    }
}
```

### 8.4 Mutation Tests for State Management

```bash
cargo mutants --file src/backend/transpiler/infra.rs --timeout 300
cargo mutants --file src/infra/dag.rs --timeout 300
# Acceptance: >= 75% CAUGHT/MISSED ratio
```

### 8.5 CLI and E2E Tests

```rust
#[test]
fn test_infra_cli_plan_shows_diff() {
    ruchy_cmd().arg("infra").arg("plan")
        .arg("tests/fixtures/infra/simple.ruchy")
        .assert().success().stdout(contains("Plan:"));
}

#[test]
fn test_infra_cli_destroy_requires_confirmation() {
    ruchy_cmd().arg("infra").arg("destroy")
        .arg("tests/fixtures/infra/simple.ruchy")
        .assert().failure().stderr(contains("--yes"));
}

#[test]
fn test_infra_e2e_transpile_compile_execute() {
    let src = fixtures_dir().join("infra/e2e_local.ruchy");
    ruchy_cmd().arg("transpile").arg(&src).assert().success();
    // Compile generated Rust, execute, verify .forjar.lock exists
    assert!(Path::new(".forjar.lock").exists());
}
```
