# Pumpkin Architecture Documentation

Pumpkin is an open-source Minecraft server implementation written from scratch in Rust. It is engineered to deliver high performance, multithreaded scalability, and strict vanilla protocol parity while offering modern native and WebAssembly (Wasm) plugin APIs.

---

## 1. High-Level Design Principles

Pumpkin follows a hybrid concurrency architecture designed to balance deterministic game logic with high-throughput network and I/O handling:

1. **Async Network & I/O (Tokio)**: All network socket handling, packet encoding/decoding, disk storage access, RCON/Query protocols, and asynchronous plugin events operate on the Tokio async runtime.
2. **Synchronous Tick Loop**: Core game state updates—such as entity movement processing, block tick updates, inventory operations, and combat math—run synchronously inside the main tick loop (20 ticks per second, 50ms interval). This guarantees deterministic execution order and eliminates lock contention during game state updates.
3. **Parallel Computation (Rayon)**: CPU-intensive tasks that do not depend on real-time synchronous game state (e.g., chunk terrain generation, light propagation calculations, batch serialization) are delegated to a Rayon thread pool.
4. **Client-Authoritative Movement**: Unlike vanilla Java servers where player movement is strictly server-simulated, Pumpkin relies on client-authoritative player movement with server-side validation to reduce movement latency and stutter.

---

## 2. Workspace Crate Overview

The codebase is organized as a Cargo workspace with dedicated crates targeting distinct domain responsibilities:

```
crates/
├── pumpkin/                 # Executable entry point and server CLI harness
├── pumpkin-core/            # Server engine, game state, tick loop, entity systems
├── pumpkin-protocol/        # Minecraft network protocol codecs, packet definitions & framing
├── pumpkin-world/           # Chunk storage, world generation, lighting engine, region files
├── pumpkin-data/            # Generated static game data (blocks, items, entities, packets)
├── pumpkin-command/         # Command dispatcher, AST node parsing, argument resolvers
├── pumpkin-inventory/       # Container windows, player inventory, item stacks, recipe logic
├── pumpkin-nbt/             # Binary Named Binary Tag (NBT) parser and serializer
├── pumpkin-config/          # TOML configuration file definitions and loader
├── pumpkin-auth/            # Mojang session authentication & online-mode crypto
├── pumpkin-scheduler/       # Asynchronous and synchronous tick task scheduler
├── pumpkin-gametest/        # In-engine integration and game mechanic test suite
├── pumpkin-plugin-api/      # Public Rust SDK for writing plugins
├── pumpkin-plugin-runtime/  # Shared plugin lifecycle management
├── pumpkin-plugin-utils/    # Utility macros and helpers for plugin authors
├── pumpkin-plugin-wit/      # WebAssembly Interface Type (WIT) definitions
├── pumpkin-wasm-host/       # Wasmtime runtime host integration
├── pumpkin-wasm-host-v0_1/  # Wasm host implementation for v0.1 API bindings
├── pumpkin-wasm-host-v0_2/  # Wasm host implementation for v0.2 API bindings
├── pumpkin-wasm-host-common/# Common trait abstractions for Wasm runtime hosts
├── pumpkin-util/            # Shared math, spatial structures, and utility helpers
├── pumpkin-codecs/          # Binary serialization utilities
├── pumpkin-macros/          # Procedural macros for internal server registration
└── pumpkin-api-macros/      # Procedural macros for plugin API generation
```

---

## 3. Core Architecture Details

### 3.1 Network & Protocol Stack (`pumpkin-protocol` & `pumpkin-core`)

The networking subsystem converts raw TCP/UDP stream bytes into structured Rust data types:

- **State Machine**: Connections transition through discrete states:
  `Handshake` ➔ `Status` / `Login` ➔ `Configuration` ➔ `Play`.
- **Packet Wire Format**: Packets consist of a VarInt-encoded length prefix, VarInt packet ID, and payload bytes.
- **Compression & Encryption**:
  - Optional AES-CFB8 stream encryption is established during the `Login` phase via RSA key exchange (`pumpkin-auth`).
  - zlib/flate2 compression is enabled dynamically once packet size exceeds the configured threshold.
- **Player Session & Networking**: Each connected player is assigned a network handler task in Tokio that processes incoming packets and forwards decoded commands to the player session instance.

### 3.2 World Management & Chunk Engine (`pumpkin-world`)

The world engine is responsible for spatial tracking, chunk loading, terrain generation, and persistence:

- **Chunk Formats**:
  - **Anvil (`.mca`)**: Standard Minecraft MCA region file format compatibility.
  - **Linear Format**: High-performance compressed region storage.
  - **Pump Format**: Custom optimized format for native performance.
- **Lighting Engine**: Sky light and block light calculation algorithms execute concurrently across section boundaries using Rayon threads.
- **Palette Section Encoding**: Block states in chunk sections are stored using compact, bit-packed indirect palettes to minimize memory consumption.

### 3.3 Concurrency & Tick Execution Loop

The server operates on a 50 millisecond fixed tick rate (20 TPS):

1. **Pre-Tick Phase**: Fetch and process queued network packets and cross-thread events.
2. **Scheduled Tasks**: Execute sync/async tasks managed by `pumpkin-scheduler`.
3. **World & Entity Ticks**: Update block ticks, tick liquid flow, execute mob AI goals, and compute entity position updates.
4. **Post-Tick Phase**: Broadcast state deltas (entity movements, block updates, tab-list updates) to clients within view distance.
5. **Tick Synchronization**: Sleep for the remaining duration of the 50ms window. If execution exceeds 50ms, a tick warning is logged.

### 3.4 Entity & AI Architecture (`pumpkin-core`)

- **Entity Taxonomy**: Base entity properties (`EntityBase`) are shared across all entities. Specialized behaviors (mobs, items, projectiles, non-living entities) are layered using specialized structs and traits.
- **Goal-Based AI System**: Mobs execute AI behaviors through a prioritized goal dispatcher (`Goal` trait), matching goal logic to vanilla behaviors (e.g., pathfinding, target acquisition, melee attacks).
- **View Distance Spatial Tracking**: Entity visibility, metadata changes, and animations are tracked spatially so packets are sent only to players within relevant chunk tracking ranges.

### 3.5 Plugin Subsystem (`pumpkin-plugin-*` & `pumpkin-wasm-host*`)

Pumpkin supports two plugin execution paradigms:

- **Native Rust Plugins**: Compiled directly against the server, providing zero-overhead access to engine events and APIs.
- **WebAssembly (Wasm) Plugins**:
  - Sandbox isolation powered by `wasmtime` component model.
  - Interoperability defined via WebAssembly Interface Types (`pumpkin-plugin-wit`).
  - Strict host-guest boundaries ensure plugins cannot crash the host server process.

### 3.6 Data Pipeline & Codegen (`pumpkin-data` & `tools/pumpkin-codegen`)

Static Minecraft game data (block IDs, block state mappings, item properties, registry entries, tags) is not hardcoded manually:

1. Data is extracted from official vanilla client/server releases using the Fabric-based [Extractor](https://github.com/Pumpkin-MC/Extractor) tool and stored as JSON in `assets/`.
2. `tools/pumpkin-codegen` processes these assets during build time and generates strongly typed Rust code inside `crates/pumpkin-data/src/generated/`.
3. Hand-written code references `pumpkin-data` constants, maintaining clean isolation from raw data files.

---

## 4. Performance & Safety Directives

- **Locking Safety**: Async Tokio contexts must never hold blocking locks or `DashMap` guards across `.await` points to avoid deadlocking worker threads.
- **Vanilla Parity Validation**: Gameplay, math formulas, packet sequences, and event orderings are checked against decompiled vanilla source code.
- **Clippy & Code Quality**: Strict linting rules deny `unwrap`, `expect`, `panic`, and unhandled errors in core runtime paths, enforcing structured error handling (`Result`/`Option`) and tracing (`tracing` crate).
