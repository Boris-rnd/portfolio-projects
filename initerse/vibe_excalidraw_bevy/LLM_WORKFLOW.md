# LLM Coding Workflow for Bevy + Rust Automation Game

You have Rust experience (OS, GPU simulations). You want LLMs to code *with* you. Don't treat them as autocomplete - treat them as junior devs you manage.

## 1. Philosophy: Spec-Driven, File-per-Agent, Bevy-Manifest

LLMs fail on big Bevy projects because Bevy ECS dependencies get messy. Fix: Extreme Modularity.

### Folder Structure (LLM-Optimized)

```
sketched-universe/
├── .rules/                     # For Cursor / Windsurf / Copilot
│   └── bevy-rules.md           # Coding conventions
├── docs/
│   ├── GDD.md                  # Game Design Doc (what we made)
│   ├── ARCHITECTURE.md         # ECS design, crate structure
│   └── TASKS.md                # Task list for agents (LLM readable)
├── AGENTS.md                   # Master instruction for all LLMs
├── crates/
│   ├── su_core/                # Pure logic, no Bevy dep: resources, recipes, save format
│   ├── su_render_excali/       # Only hand-drawn rendering
│   ├── su_grid/                # Grid, placement, collision
│   ├── su_logistics/           # Arrow system
│   ├── su_factories/           # Buildings: GluonGen etc - each factory is a plugin
│   └── su_ui/                  # Excalidraw-inspired UI
├── src/
│   ├── main.rs                 # Just app builder + plugin add
│   └── lib.rs
├── assets/
│   ├── fonts/Virgil.woff2
│   └── shaders/wobbly.wgsl
└── prompts/
    ├── new_factory.md          # Template prompt to create new building
    └── new_resource.md
```

Why workspace crates? So you can give ONE crate to an LLM with full context and it won't break other things.

### AGENTS.md - Your Master Prompt

Create this file at root. Every LLM tool (Cursor, Claude Code, Codex, factory.ai) reads it.

```markdown
# Agent Instructions for Sketched Universe

## Project: Automation game, Bevy 0.15+, Rust, Excalidraw aesthetic

## Absolute Rules
1.  **Always use Bevy Plugins**: Never put logic in main.rs. New feature = new Plugin in its own crate.
2.  **ECS Pattern**: Components = data only (no logic). Systems = pure functions. Resources = singletons for game state.
3.  **No unwrap() in systems**. Use query.get() and continue if fails.
4.  **Rendering separation**: Logic in su_core, rendering in su_render_excali. Never import rendering in core.
5.  **Hand-drawn**: Any shape must use RoughGenerator::generate_box() not perfect rectangles.
6.  **Run `cargo check -p <crate>`**, not full build, for speed.
7.  **Tests**: Logic crates must have at least one `#[test]` for recipes.

## Workflow
- Read docs/ARCHITECTURE.md before coding
- Pick task from docs/TASKS.md
- Implement in smallest crate possible
- Document with `///` comments for LLM next pass

## Bevy Specifics
- Use bevy_ecs_titan for save? Use Moon saving patterns.
- Camera: use PanOrbit? No. Custom Infinite Canvas Camera.
- Inputs: Use bevy::input + leafwing-input-manager
```

## 2. Task Breakdown (LLM-readable)

In `docs/TASKS.md`, use this format - LLMs parse this perfectly:

```markdown
### TASK-001: Infinite Canvas Camera
Status: TODO
Crate: su_grid
Depends: none
Acceptance:
  - WASD + drag middle mouse to pan
  - Scroll to zoom 0.1x to 10x with lerp
  - Exposes resource `CanvasCamera { zoom, position }`
  - Dot grid shader scales with zoom
Test: `cargo run -p su_grid --example camera`
Prompt: "Implement infinite canvas camera as Bevy plugin..."
```

List 20-30 small tasks. LLMs love small tasks.

Initial task list:

TASK-001 to 005: Core scaffolding, canvas camera, grid, placement, rough rendering
TASK-006 to 010: Resource system, arrow logistics, your diagram loop (Gluon -> Proton -> H -> Star)
TASK-011 to 015: Factories UI, Excalidraw selection box, copy-paste
TASK-016 to 020: Scaling system (tier transitions), Star gravity, Explorer view
TASK-021+: Black hole, Multiverse layer

## 3. LLM Workflow Setup - Practical

### Option A: You + Cursor / Windsurf (Recommended for your skill level)

You are senior, LLM is middle. Workflow:

1.  **You write GDD + ARCHITECTURE.md** (manual)
2.  **You prompt Cursor Composer**: "/ Implement TASK-001 based on docs/TASKS.md and AGENTS.md. Only modify crates/su_grid/"
3.  **You run check**, fix logical errors yourself (don't let LLM loop)
4.  **Commit**, then next task.

Use **Cursor Rules**:
- `.cursor/rules/bevy.mdc` -> always use Plugin, no globals
- `.cursor/rules/excalidraw.mdc` -> all UI must use wobbly shader, etc.

### Option B: Multi-Agent (Codex / Claude Code / Factory.ai)

If you want near-full auto:

- Use **Git worktrees**: `git worktree add ../task-001 TASK-001`
- Launch 3 agents in parallel, each on different crate. Because crates are independent, no merge conflict.
- Merge with `git merge` when `cargo test -p su_core` passes.

### Option C: Spec with Raspberry (Advanced)

Write crate `su_core` entirely pure Rust with NO Bevy. Perfect for LLM to generate property-based tests. Then Bevy wrapping is thin.

Your background in GPU sims: Use `su_core` to hold simulation that can later be moved to compute shader (wgpu). LLMs can write compute shader easier when logic is separate.

## 4. Bevy Specific Patterns for LLMs

LLMs often hallucinate old Bevy 0.12 API. Force 0.15+.

Include in AGENTS.md:

```rust
// CORRECT Bevy 0.15 pattern (LLM MUST follow)
pub struct GluonPlugin;

impl Plugin for GluonPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<GluonBasins>()
            .add_systems(Update, (produce_gluons, distribute_quarks).chain());
    }
}

// WRONG - Do not do .add_system, no StartupStage etc.
```

**Give LLMs a reference file**: `docs/BEVY_SNIPPET.rs` with up to date patterns for plugin, query, gizmos, etc.

## 5. Hand-drawn Rendering - De-risk Early

This is biggest tech risk. Assign as TASK-002.

Approach LLMs can implement:

**Step 1: Rough.js port**
- Port Excalidraw's rough algorithm (or use `roughr` Rust crate).
- Create `RoughRect` component that on spawn generates a Mesh2d with wobbly points.

**Step 2: Shader for wobble**
- WGSL shader: vertex offset with `sin(uv * 10.0) * 0.01` * `rand_per_instance`.

**Step 3: Dots**
- Grid background using bevy's `Material2d` shader drawing dots.

Let one agent only focus on this.

## 6. Your Quantum Physics Simulation - How to integrate

You know quantum. LLMs don't need to.

- Keep your QFT logic in `su_core::physics::qcd.rs` - gluon generator probability.
- Use LLMs for glue: visualization + ECS wiring.
- For performance: Later you can use `bevy_compute` to run your GPU sims for star gravity (N-body). You already know GPU sims - let LLM write boilerplate.

## 7. Suggested Prompts Library

In `prompts/` folder save good prompts:

`prompts/new_factory.md`:
```
Create new factory building: {{NAME}}.
In crate su_factories.
- Component: {{NAME}} { progress: Timer, input: Buffer, output: Buffer }
- System: tick_factory that consumes {{INPUT}} and produces {{OUTPUT}} at rate {{RATE}}
- Plugin to register
- Visual: 3x3 rough box with label.
Use TASK-006 as reference.
```

Then you just replace NAME.

## 8. Daily Loop

1. Morning: You define 2-3 tasks in TASKS.md
2. Agent Session 1h: Agents implement crates
3. You review: `cargo run` - does Excalidraw feel good? Is building effortless?
4. Afternoon: You hardcode physics fun (quantum tunneling probability for star fusion etc)
5. Push to git, agents rebase from your fix.

## 9. Tools Stack

- Bevy 0.15 (latest stable) + bevy_ecs_tilemap + bevy_mod_picking + bevy_vector_shapes or vello?
- `leafwing-input-manager` for Excalidraw-like shortcuts (Ctrl+D duplicate, Alt+Drag)
- `roughr` crate for wobbly boxes
- `bevy_persistent` for saves
- `egui` or `bevy_egui` for quick Excalidraw properties panel (on right), later custom.
- Cargo `nextest` for faster tests

## 10. MVP Milestone Plan (LLM Executable)

Week 1: TASKS 001-006 (canvas + place wobbly boxes)
Week 2: TASK 007-010 (your diagram loop works, arrows flow)
Week 3: Polish: selection, copy-paste, paper texture, sound (hand-drawn pop)
Week 4: Tech tree + scaling v1 (Star grows)

Goal: Share a web build (bevy wasm) that feels like Excalidraw.

---

This workflow prevents common failure: LLMs generating massive main.rs with spaghetti and old Bevy API.
```

