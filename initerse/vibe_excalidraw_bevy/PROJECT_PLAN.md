# PROJECT PLAN - Sketched Universe
## For LLM execution - copy this to AGENTS.md / TASKS.md

### Stack
- Rust stable, Bevy 0.15.2
- Workspace crates

### Run commands LLM must know
```
cargo check -p su_core
cargo test -p su_core
cargo run --bin sketched-universe
cargo run --target wasm32-unknown-unknown --bin sketched-universe
```

### TASK LIST - Prioritized

#### Phase 0: Boot (Day 1-2)
- [ ] TASK-001: Project scaffolding, workspace crates, Bevy app with empty infinite canvas
- [ ] TASK-002: Dot-grid background shader (Excalidraw style, infinite, zoom-dependent opacity)
- [ ] TASK-003: Infinite Canvas Camera: pan (middle-drag + WASD) + zoom lerp + screenspace to world
- [ ] TASK-004: Rough Rendering crate: trait RoughDrawable -> Mesh2d generation with hand-drawn jitter. Implement `rough_box(width,height) -> Mesh`. Use `rand` seeded per entity for stable wobble.

#### Phase 1: Effortless Building (Day 3-5) - THE CORE FEEL
- [ ] TASK-005: Grid + Placement System: logical grid (64px), but visual free. Placement indicator (ghost wobbly box). Right-click to delete. Size-aware placement (Entity with `GridSize(3,3)`)
- [ ] TASK-006: Resource System (pure Rust): Define `ResourceType` enum: Gluon, UpQuark, DownQuark, ChargePlus, ChargeMinus, Proton, Electron, Positron, Photon, Hydrogen, Helium...  Recipe struct: inputs, outputs, time. Storage in `Buffer` component.
- [ ] TASK-007: Factory Plugin Pattern: Generic `Factory { recipe, timer, input_buffer, output_buffer }`. Create first factories from diagram: GluonGenerator, PlusBasin, MinusBasin, ProtonCreator
- [ ] TASK-008: Arrow Logistics (most important UX): User can draw arrow from output of A to input of B. Create `Connection { from, to, bezier, resource_type }`. System moves resources at rate. Visual: particle moving along curve. Arrows are Bezier with 2 control points draggable.
- [ ] TASK-009: Your diagram loop end-to-end: GluonGen -> +-basins -> ProtonCreator + Electron (from Photon decay) -> Hydrogen -> Artificial Star -> Photons (feedback)
- [ ] TASK-010: Selection & Manipulation like Excalidraw: Drag-select rectangles, move group, duplicate with Alt+drag, Ctrl+D. Use `bevy_mod_picking`

#### Phase 2: Excalidraw Juice (Day 6-8)
- [ ] TASK-011: Visual polish: Virgil font labels on buildings, hand-drawn animation on spawn (draw like stroke), paper texture overlay, subtle shadow
- [ ] TASK-012: Properties panel (right sidebar): When select building, show rate, buffers, pause button like Excalidraw stroke panel
- [ ] TASK-013: Text tool: press T place text anywhere, like annotations. Not gameplay, but feel.
- [ ] TASK-014: Library/Handler bar: Top bar like Excalidraw? Bottom bar with building palette. Drag from palette or press key (1-9)
- [ ] TASK-015: Save/Load: Serialize core state to JSON/RON. `Save = { grids, connections, resources }`

#### Phase 3: Scale (Day 9+)
- [ ] TASK-016: Artificial Star Expansion: Star entity's size scales with Hydrogen stored. Gravity field: pull nearby H atoms (force simulation). Heat component: hotter with more fusion.
- [ ] TASK-017: Quantum Tunneling: Star fusion probability = f(temp, quantum). Show tunneling effect visualized as wavy dashed line.
- [ ] TASK-018: Stellar Extraction & Supernova: Star can explode producing HeavyAtoms resource cloud, collected by extractor building.
- [ ] TASK-019: Zoom Layers: When zoom out > threshold, switch to Galactic view. Buildings become stars. N-body gravity using compute shader (stub for your GPU experience)
- [ ] TASK-020: Tier progression / Tech tree: sketched on paper as prerequisites drawn as arrows between technologies?
- [ ] TASK-021: Black Hole Forge
- [ ] TASK-022: String Creator (Planck scale intro building) - make it feel tiny, requires ultra zoom in.

#### Bonus Phase: Endless
- [ ] Multiverse layer
- [ ] Multiplayer whiteboard (like Excalidraw live collab - Bevy net?)
- [ ] WASM deploy for web demo

---

### ACCEPTANCE CRITERIA For Each TASK

Each task PR must contain:
1. Cargo check passes for its crate
2. At least screenshot / gif (if rendering)
3. Update to TASKS.md marking done + note.

---

### KEY BEVY ARCHITECTURE DECISIONS

**App Structure:**

```rust
fn main() {
  App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin{..}))
    .add_plugins(SuCorePlugin) // pure logic
    .add_plugins(SuGridPlugin) // camera, grid, placement
    .add_plugins(SuRenderExcaliPlugin) // hand-drawn materials
    .add_plugins(SuLogisticsPlugin) // arrows
    .add_plugins(SuFactoriesPlugin) // all factories registers own plugin inside
    .add_plugins(SuUiPlugin) // egui UI
    .run();
}
```

**ECS Design:**

- `GridPos { x, y, layer }` - int grid coords
- `GridSize { w, h }`
- `Building { type_id: BuildingType, rotation }`
- `Buffer { capacity, contents: HashMap<ResourceType, f32> }` - f32 to allow rates
- `Factory { recipe: RecipeId, progress: f32 }`
- `Connection { from: Entity, from_port: u8, to: Entity, to_port: u8, curve: CubicBezier, throughput: f32 }`
- Resource: `ResourceRegistry`, `RecipeRegistry`

**Event-driven logistics:** Use `bevy_ecs` events: `ResourceProduced`, `ResourceRequested`

**Performance:** Logistics tick at 10hz fixed schedule, rendering at variable. Use `FixedUpdate`.

---

### HAND-DRAWN ALGORITHM REFERENCE FOR LLM

Implement this in su_render_excali:

```rust
// Pseudocode for rough box
pub fn rough_box(w: f32, h: f32, seed: u64, roughness: f32) -> Vec<Vec2> {
  let mut rng = SeededRng::from_seed(seed);
  let segments = 8;
  // for each edge, add random displacement perpendicular to edge
  // edge from (-w/2,-h/2) to (w/2,-h/2) = top edge
  // split into segments, midpoint displaced by rng.gen_range(-roughness, roughness)
  // second pass: draw two overlapping strokes with slight opacity difference
}
```

Excalidraw does: double stroke with slight offset. So generate 2 lines slightly offset.

Use the `roughr` crate: https://crates.io/crates/roughr - but adapt to Mesh2d.

Fonts: Use `https://github.com/excalidraw/virgil` . Include .ttf in assets.

---

### FOR YOU (Owner) - What to do manually vs delegate to LLM?

You do:
- GDD tweaks, feel tuning, overall art direction, quantum physics formulas (tunneling probability, gluon distribution)
- Hard problems: infinite canvas camera lerp math, bezier hit-testing, compute shaders for star gravity.

LLM does:
- Boilerplate plugin setup, UI panels, save/load, repetitive factory definitions.
- Rendering shaders under clear spec.
- Unit tests for recipe balancing.
- Docs

---

### Immediate Next Steps for YOU today

1. `cargo new sketched-universe --bin` + make workspace crates
2. Create AGENTS.md (copy from LLM_WORKFLOW)
3. Start TASK-001 yourself 30min, then hand next tasks to LLM.
4. Use `cargo add bevy@0.15` etc.

Want me to scaffold the actual Cargo project code? I can generate the initial main.rs + crates + rough rendering stub.
```

