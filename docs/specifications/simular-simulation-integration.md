# Sub-spec: Simular First-Class Simulation Engine Integration for Ruchy

**Parent:** [trueno-aprender-stdlib-core-language-spec.md](trueno-aprender-stdlib-core-language-spec.md) (Pillar 5: Simulation)

---

## 0. Prerequisites

- **simular is not currently a Ruchy dependency** -- it must be added to `Cargo.toml` (crates.io latest: v0.3.1).
- **`src/stdlib/simular_bridge.rs` must be created** -- this file is referenced throughout the spec but does not exist yet.
- **All decorator syntax (`@anomaly_checked`, `@falsifiable`) requires parser extension** -- the Ruchy parser does not currently support decorator/attribute syntax on functions.

---

## 1. Overview

### 1.1 Current State

Simular exists as a standalone Rust crate with ZERO integration into the Ruchy language surface. It is neither re-exported through a bridge module nor accessible via any Ruchy syntax. Users who want simulation capabilities must write raw Rust against the simular API directly, bypassing the compiler entirely.

### 1.2 Target State

Native `sim.*` syntax for physics, Monte Carlo, optimization, and ML simulations as first-class language features. Users write simulation scenarios in Ruchy source code; the compiler transpiles these to simular API calls with full Jidoka guard insertion, deterministic reproducibility, and CLI lifecycle management via `ruchy sim` subcommands.

### 1.3 Simular Baseline

| Metric | Value |
|--------|-------|
| Version | 0.3.1 |
| Lines of Rust | ~66,000 |
| Tests | 1,846 |
| RNG | Deterministic PCG (bit-identical cross-platform) |
| Guard system | Jidoka (stop-on-anomaly) |
| Integrators | Euler, RK4, Verlet, Yoshida (4th/6th/8th order) |

### 1.4 Toyota TPS Alignment

| TPS Principle | Simular Implementation |
|---------------|----------------------|
| Jidoka (stop-on-error) | `JidokaGuard` halts simulation on anomaly detection (energy drift, NaN, divergence) |
| Poka-Yoke (mistake-proofing) | Type system prevents unit mismatches (time, mass, distance are distinct types) |
| Heijunka (load leveling) | Adaptive time-stepping distributes compute evenly across stiff/smooth regions |
| Genchi Genbutsu (go and see) | `sim inspect` CLI for direct observation of simulation state and diagnostics |

### 1.5 Success Criteria

| Criterion | Threshold |
|-----------|-----------|
| Simulation domains exposed | 4 (physics, Monte Carlo, optimization, ML) |
| Energy conservation (Kepler) | < 1e-9 drift over 100 orbits |
| Monte Carlo convergence | Estimate within 3-sigma of analytic solution |
| RNG reproducibility | Bit-identical RNG sequences across platforms for same seed (see Section 5 note on floating-point) |
| CLI subcommands | 4 (`run`, `inspect`, `verify`, `export`) |
| Language syntax coverage | sim.step / sim.monte_carlo / sim.optimize / sim.train_sim |

---

## 2. Language Syntax

### 2.1 Module Import

```ruchy
import sim
```

The `sim` module is a top-level standard library module. The transpiler injects the simular dependency automatically when `import sim` is detected.

### 2.2 Scenario Builders

```ruchy
let config = sim.kepler_config(
    semi_major_axis=1.0, eccentricity=0.0167, mass_central=1.0, integrator="yoshida8"
)

let config = sim.nbody_config(
    bodies=[
        sim.body(mass=1.0, pos=[0.0, 0.0, 0.0], vel=[0.0, 0.0, 0.0]),
        sim.body(mass=3e-6, pos=[1.0, 0.0, 0.0], vel=[0.0, 6.28, 0.0]),
    ],
    integrator="verlet", softening=1e-4
)
```

### 2.3 Time Constructors

```ruchy
let dt = sim.time_seconds(3600.0)
let dt = sim.time_hours(1.0)
let dt = sim.time_days(0.5)
let duration = sim.time_years(1.0)
```

Time values are strongly typed. The transpiler enforces that `sim.step()` receives a `SimTime` value, not a bare float. Mixing time units produces a compile-time error.

### 2.4 Simulation Execution

```ruchy
let engine = sim.engine(config)
let state = engine.initial_state()

for i in range(0, 8760):
    state = sim.step(state, dt)
    if state.energy_error() > 1e-9:
        print(f"Energy drift at step {i}: {state.energy_error()}")

# Bulk execution
let result = sim.run(config, duration=sim.time_years(1.0), dt=sim.time_hours(1.0))
```

### 2.5 Monte Carlo

```ruchy
let mc = sim.monte_carlo(samples=100_000, seed=42, variance_reduction="antithetic")

fun estimate_pi(rng):
    let x = rng.uniform(0.0, 1.0)
    let y = rng.uniform(0.0, 1.0)
    return 1.0 if x * x + y * y <= 1.0 else 0.0

let result = mc.run(estimate_pi)
print(f"Pi estimate: {result.mean() * 4.0}, Std error: {result.std_error()}")
```

### 2.6 Falsifiable Hypothesis Decorator

```ruchy
@falsifiable(
    null_hypothesis="Energy is conserved to 1e-9",
    test_statistic="max_energy_drift", rejection_threshold=1e-9
)
fun test_kepler_conservation(config):
    let result = sim.run(config, duration=sim.time_years(100.0), dt=sim.time_hours(1.0))
    return result.max_energy_error()
```

### 2.7 Full Orbit Simulation Example

```ruchy
import sim

let config = sim.kepler_config(
    semi_major_axis=1.0, eccentricity=0.0167, mass_central=1.0, integrator="yoshida8"
)

@anomaly_checked(energy_tol=1e-9, nan_check=true)
fun run_orbit():
    let engine = sim.engine(config, seed=12345)
    let state = engine.initial_state()
    let dt = sim.time_hours(1.0)
    for step in range(0, 876_000):  # 100 years
        state = sim.step(state, dt)
    print(f"Final energy error: {state.energy_error()}")
    return state

let final_state = run_orbit()
```

---

## 3. Transpiler Integration

### 3.1 Syntax-to-API Mapping

| Ruchy Syntax | Transpiled Rust API | Notes |
|-------------|---------------------|-------|
| `import sim` | `use simular::prelude::*;` | Auto-adds `simular` to Cargo.toml |
| `sim.kepler_config(...)` | `KeplerConfig::builder()...build()` | Builder pattern |
| `sim.nbody_config(...)` | `NBodyConfig::builder()...build()` | Builder pattern |
| `sim.body(mass, pos, vel)` | `Body::new(mass, pos, vel)` | Vector types inferred |
| `sim.engine(config)` | `SimEngine::new(config)` | Generic over config type |
| `sim.engine(config, seed=N)` | `SimEngine::with_seed(config, N)` | Deterministic PCG |
| `sim.step(state, dt)` | `engine.step(&mut state, dt)` | Borrows mutably |
| `sim.run(config, duration, dt)` | `SimEngine::run(config, duration, dt)` | Bulk execution |
| `state.energy_error()` | `state.energy_error()` | Computed property |
| `sim.time_seconds(v)` | `SimTime::from_seconds(v)` | Typed time |
| `sim.time_hours(v)` | `SimTime::from_hours(v)` | Typed time |
| `sim.time_days(v)` | `SimTime::from_days(v)` | Typed time |
| `sim.time_years(v)` | `SimTime::from_years(v)` | Typed time |
| `sim.monte_carlo(...)` | `MonteCarloEngine::new(...)` | With variance reduction |
| `mc.run(f)` | `mc_engine.run(\|rng\| f(rng))` | Closure wrapping |
| `sim.jidoka_check(state)` | `JidokaGuard::check(&state)?` | Returns Result |
| `@falsifiable(...)` | `impl FalsifiableHypothesis for ...` | Trait implementation |
| `@anomaly_checked(...)` | Jidoka guard insertion after mutations | Compile-time injection |

### 3.2 Decorator Transpilation

The `@anomaly_checked` decorator triggers compile-time guard injection. The transpiler inserts a `JidokaGuard::check()` call after every `sim.step()` within the decorated function body.

**Input (Ruchy):**
```ruchy
@anomaly_checked(energy_tol=1e-9)
fun simulate():
    state = sim.step(state, dt)
```

**Output (Rust):**
```rust
fn simulate() -> Result<(), SimulationError> {
    let guard = JidokaGuard::new(JidokaConfig {
        energy_tolerance: 1e-9,
        nan_check: true,
        ..Default::default()
    });
    state = engine.step(&mut state, dt);
    guard.check(&state)?;  // <-- injected by transpiler
    Ok(())
}
```

### 3.3 Transpiler Module Changes

| File | Change |
|------|--------|
| `src/backend/transpiler/mod.rs` | Add `sim` module detection in import resolution |
| `src/backend/transpiler/statements.rs` | Handle `sim.*` method calls |
| `src/backend/transpiler/effects.rs` | Inject Jidoka guards for `@anomaly_checked` |
| `src/backend/transpiler/program_transpiler.rs` | Add `simular` to generated Cargo.toml |

---

## 4. Simulation Domains

### 4.1 Domain Matrix

| Domain | Primitives | Methods | Key Types |
|--------|-----------|---------|-----------|
| Physics | N-body, Kepler, rigid body | Euler, RK4, Verlet, Yoshida (4/6/8) | `Body`, `KeplerState`, `RigidBody` |
| Monte Carlo | Sampling, estimation, hypothesis | Antithetic, importance, stratified, control variate | `MonteCarloEngine`, `Estimate` |
| Optimization | Bayesian optimization, parameter search | GP surrogate, acquisition (EI, UCB, PI) | `BayesianOptimizer`, `GaussianProcess` |
| ML Simulation | Training dynamics, drift detection | SGD sim, convergence, concept drift | `TrainingSim`, `DriftDetector` |

### 4.2 Physics Integrators

Symplectic integrators preserve phase-space volume (Liouville's theorem), making them mandatory for long-duration orbital simulations.

| Integrator | Order | Symplectic | Use Case |
|-----------|-------|-----------|----------|
| Euler | 1 | No | Teaching, prototyping |
| RK4 | 4 | No | General ODE, short duration |
| Verlet (Stormer) | 2 | Yes | Molecular dynamics, N-body |
| Yoshida4 | 4 | Yes | Orbital mechanics (default) |
| Yoshida6 | 6 | Yes | High-precision orbits |
| Yoshida8 | 8 | Yes | Ultra-precision, 100+ year simulations |

### 4.3 Monte Carlo Variance Reduction

| Method | Speedup (typical) | Conditions |
|--------|-------------------|------------|
| None (naive) | 1x | Baseline |
| Antithetic variates | 1.5-2x | Monotone integrands |
| Importance sampling | 2-10x | Known approximate density |
| Stratified sampling | 1.5-3x | Bounded domain |
| Control variates | 2-5x | Correlated known-mean variable available |

### 4.4 Optimization Domain

```ruchy
let optimizer = sim.bayesian_optimizer(
    bounds=[(0.0, 1.0), (0.0, 1.0)],
    acquisition="expected_improvement", kernel="matern52", n_initial=10
)

fun objective(params):
    let x, y = params[0], params[1]
    return -(x - 0.3) ** 2 - (y - 0.7) ** 2

let best = optimizer.minimize(objective, n_iterations=50)
print(f"Best params: {best.params}, value: {best.value}")
```

### 4.5 ML Simulation Domain

```ruchy
let train_sim = sim.training_sim(
    model="linear_regression", optimizer="sgd", learning_rate=0.01, epochs=100
)
let trajectory = train_sim.run(X_train, y_train)
print(f"Final loss: {trajectory.final_loss()}")
print(f"Converged at epoch: {trajectory.convergence_epoch()}")

let detector = sim.drift_detector(method="adwin", delta=0.002)
for batch in data_stream:
    if detector.detected(model.predict(batch).error()):
        print("Concept drift detected - retrain required")
```

---

## 5. Jidoka as Language Guard

### 5.1 Guard Decorator

The `@anomaly_checked` decorator converts a simulation function into a guarded execution context. The transpiler statically analyzes the function body to locate all `sim.step()` calls and injects `guard.check()` after each one.

```ruchy
@anomaly_checked(energy_tol=1e-9, nan_check=true, divergence_limit=1e6, severity="critical")
fun long_simulation():
    # guard.check() injected after every sim.step() call
    ...
```

### 5.2 Graduated Severity

| Level | Threshold | Action |
|-------|-----------|--------|
| Acceptable | error < tolerance | Continue, no log |
| Warning | tolerance <= error < 10 * tolerance | Log warning, continue |
| Critical | 10 * tolerance <= error < 100 * tolerance | Log error, optionally halt |
| Fatal | error >= 100 * tolerance OR NaN detected | Immediate halt, dump state |

The severity level in `@anomaly_checked` determines the minimum level that triggers a halt. With `severity="critical"`, Warning events log but continue; Critical and Fatal halt immediately.

### 5.3 Guard Injection Rules

1. The transpiler scans the decorated function AST for all `sim.step()` call expressions
2. After each `sim.step()`, a `guard.check(&state)?` call is inserted
3. The function return type is wrapped in `Result<T, SimulationError>`
4. If the function already returns `Result`, the error type is unified via `From` impl
5. Guard configuration is extracted from decorator arguments at compile time (not runtime)

### 5.4 Deterministic Reproducibility

All simulation engines accept an optional `seed` parameter. When provided, the PCG RNG is initialized deterministically, guaranteeing bit-identical RNG sequences across platforms (x86_64, aarch64, wasm32).

> **Note on floating-point reproducibility:** Bit-identical RNG sequences are guaranteed; however, bit-identical floating-point results depend on platform FMA (fused multiply-add) instruction availability. Platforms with hardware FMA (e.g., x86_64 with AVX2, aarch64) may produce results that differ at the ULP level from platforms without FMA. For strict cross-platform reproducibility, compile with `--cfg simular_no_fma` to disable FMA optimization.

```ruchy
let engine = sim.engine(config, seed=42)   # Reproducible
let engine = sim.engine(config)            # Non-deterministic (OS entropy)
```

The transpiler emits a compile-time warning if `@falsifiable` is used without a `seed` parameter, since reproducibility is required for hypothesis testing.

---

## 6. CLI Commands

### 6.1 Command Matrix

| Command | Purpose | Key Flags |
|---------|---------|-----------|
| `ruchy sim run` | Execute simulation scenario | `--duration`, `--dt-hours`, `--seed`, `--output` |
| `ruchy sim inspect` | Examine simulation results | `--energy`, `--trajectory`, `--summary` |
| `ruchy sim verify` | Assert simulation properties | `--criterion`, `--tolerance` |
| `ruchy sim export` | Export results to data formats | `--format` (parquet, csv, json, arrow) |

### 6.2 Usage Examples

```bash
# Run a Kepler orbit simulation for 365 days
ruchy sim run scenario.ruchy --duration 365 --dt-hours 1 --seed 42 --output results.apr

# Inspect energy conservation
ruchy sim inspect results.apr --energy
#   Steps: 8760 | Max energy error: 2.3e-12 | Runtime: 0.42s

# Verify a conservation criterion
ruchy sim verify results.apr --criterion "max_energy_error < 1e-9"
#   PASS: max_energy_error = 2.3e-12 < 1e-9

# Export to Parquet for analysis
ruchy sim export results.apr --format parquet --output orbit_data.parquet
```

Scenario files are standard `.ruchy` source files. The `ruchy sim run` command compiles and executes the scenario, capturing state transitions into the `.apr` (Archivo de Prueba Ruchy) output format.

---

## 7. Bridge Implementation

### 7.1 Module Location

New file: `src/stdlib/simular_bridge.rs` (follows the `aprender_bridge.rs` pattern).

### 7.2 Re-exports and Factory Functions

```rust
// src/stdlib/simular_bridge.rs

pub use simular::engine::SimEngine;
pub use simular::time::SimTime;
pub use simular::rng::SimRng;
pub use simular::jidoka::{JidokaGuard, JidokaConfig, Severity};
pub use simular::monte_carlo::MonteCarloEngine;
pub use simular::physics::{Body, KeplerConfig, KeplerState, NBodyConfig};
pub use simular::optimize::{BayesianOptimizer, GaussianProcess};
pub use simular::ml::{TrainingSim, DriftDetector};
pub use simular::hypothesis::FalsifiableHypothesis;

pub fn kepler_config(
    semi_major_axis: f64, eccentricity: f64, mass_central: f64, integrator: &str,
) -> KeplerConfig {
    KeplerConfig::builder()
        .semi_major_axis(semi_major_axis).eccentricity(eccentricity)
        .mass_central(mass_central)
        .integrator(integrator.parse().expect("valid integrator name"))
        .build()
}

pub fn nbody_config(bodies: Vec<Body>, integrator: &str, softening: f64) -> NBodyConfig {
    NBodyConfig::builder()
        .bodies(bodies)
        .integrator(integrator.parse().expect("valid integrator name"))
        .softening(softening)
        .build()
}

pub fn monte_carlo(samples: u64, seed: u64, variance_reduction: &str) -> MonteCarloEngine {
    MonteCarloEngine::builder()
        .samples(samples).seed(seed)
        .variance_reduction(variance_reduction.parse().expect("valid method"))
        .build()
}

pub fn time_seconds(v: f64) -> SimTime { SimTime::from_seconds(v) }
pub fn time_hours(v: f64) -> SimTime { SimTime::from_hours(v) }
pub fn time_days(v: f64) -> SimTime { SimTime::from_days(v) }
pub fn time_years(v: f64) -> SimTime { SimTime::from_years(v) }
```

### 7.3 Registration in stdlib

```rust
// src/stdlib/mod.rs (addition)
pub mod simular_bridge;
```

The transpiler's import resolver maps `import sim` to this bridge module.

---

## 8. Testing Requirements

### 8.1 Reproducibility Tests

| Test | Assertion |
|------|-----------|
| Same seed, same platform | Bit-identical final state |
| Same seed, x86_64 vs aarch64 | Bit-identical final state (PCG guarantee) |
| Same seed, native vs wasm32 | Bit-identical final state |
| Different seeds | Statistically independent trajectories (chi-squared test) |

### 8.2 Energy Conservation Tests

| Scenario | Duration | Integrator | Max Drift Allowed |
|----------|----------|-----------|-------------------|
| Circular orbit (e=0) | 100 orbits | Yoshida8 | < 1e-12 |
| Earth-like (e=0.0167) | 100 orbits | Yoshida8 | < 1e-9 |
| Eccentric (e=0.5) | 100 orbits | Yoshida8 | < 1e-7 |
| Highly eccentric (e=0.9) | 10 orbits | Yoshida8 | < 1e-5 |
| Non-symplectic baseline | 1 orbit | RK4 | < 1e-4 (expected to fail at 100) |

### 8.3 Monte Carlo Convergence Tests

| Estimand | Analytic Value | Samples | Acceptance |
|----------|---------------|---------|------------|
| Pi (circle area) | 3.14159... | 1,000,000 | Within 3-sigma |
| e (exponential) | 2.71828... | 1,000,000 | Within 3-sigma |
| Normal CDF at 1.96 | 0.975 | 1,000,000 | Within 3-sigma |
| Variance reduction ratio | Theoretical bound | 100,000 | Antithetic >= 1.3x speedup |

### 8.4 Property Tests

```rust
proptest! {
    #[test]
    fn rng_uniform_in_bounds(seed in 0u64..1_000_000, n in 1usize..10_000) {
        let mut rng = SimRng::seed(seed);
        for _ in 0..n {
            let v = rng.uniform(0.0, 1.0);
            prop_assert!(v >= 0.0 && v < 1.0);
        }
    }

    #[test]
    fn symplectic_preserves_energy(e in 0.0f64..0.5, steps in 100usize..10_000) {
        let config = kepler_config(1.0, e, 1.0, "yoshida4");
        let mut engine = SimEngine::with_seed(config, 42);
        let mut state = engine.initial_state();
        let initial_energy = state.total_energy();
        for _ in 0..steps { state = engine.step(&mut state, time_hours(1.0)); }
        let drift = (state.total_energy() - initial_energy).abs() / initial_energy.abs();
        prop_assert!(drift < 1e-6, "Energy drift {} exceeds threshold", drift);
    }
}
```

### 8.5 Jidoka Guard Tests

| Test | Input | Expected |
|------|-------|----------|
| NaN injection | Set velocity to NaN | Fatal halt, state dump |
| Energy spike | Multiply energy by 1e6 | Critical halt |
| Gradual drift | Accumulate 1e-8 per step | Warning at threshold, halt at 10x |
| Clean simulation | Normal Kepler orbit | Acceptable throughout, no halts |

### 8.6 Transpiler Integration Tests

| Test | Ruchy Input | Expected Rust Output |
|------|------------|---------------------|
| Import resolution | `import sim` | `use simular::prelude::*;` |
| Time type safety | `sim.step(state, 1.0)` | Compile error (bare float) |
| Guard injection | `@anomaly_checked` function | `guard.check()` after each `sim.step()` |
| Seed warning | `@falsifiable` without seed | Compile-time warning |
| Cargo.toml injection | Any `sim.*` usage | `simular = "0.3.1"` in dependencies |

### 8.7 Test Naming Convention

Convention: `test_simular_<section>_<feature>_<scenario>` (e.g., `test_simular_physics_kepler_energy_conservation_100_orbits`, `test_simular_jidoka_nan_injection_fatal_halt`)
