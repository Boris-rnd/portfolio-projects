# AGENTS.md - Instructions for all LLM agents on this project

## Project: Sketched Universe - Excalidraw Factorio

Game is 2D grid automation from quarks to multiverse. Hand-drawn aesthetic like excalidraw.com. Bevy 0.15.

User decisions (FINAL):
- Logistics: Hybrid bezier arrows (early) + Vectorio-style drone ports (range-based auto-delivery)
- Building UX: Bottom palette + select tool like Excalidraw, keys 1-9
- Zoom: Seamless zoom layers illusion (fake for MVP)
- Physics: Gamey simple recipes (2 Up + 1 Down = Proton)

## Absolute Rules
1. Plugins only. Every feature = Plugin in correct crate.
2. su_core must stay pure - no bevy rendering imports. Only bevy_reflect, serde.
3. No unwrap() in systems - use if let Ok()
4. All shapes must be wobbly via RoughBox - never perfect rect
5. Use Cargo check per crate, not full build
6. Docs/TASKS.md is source of truth - pick task there

## Crate Responsibilities
- su_core: resources enum, recipes, buffers, save serde
- su_grid: camera pan/zoom lerp to cursor, grid snapping, placement ghost, zoom layer visibility
- su_render_excali: RoughBox random seed stable, gizmos for now then Mesh2d, Virgil font placeholder
- su_logistics: Connection component + curve + particle, DronePort + Requester/Provider + matching
- su_factories: Factory component { recipe_id, progress } + each building type tick
- su_ui: egui? Or simple bevy UI bottom bar. Palette state

## Bevy 0.15 Patterns - MUST follow
```rust
impl Plugin for XPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Update, system).init_resource::<Res>();
  }
}
// Queries:
fn sys(q: Query<(&A, &mut B)>) { for (a, mut b) in q.iter() {} }
// No add_system, no Stage
```

## Rough algorithm reference
Generate wobbly rect by jittering edge midpoints with seeded rng.

## Current MVP Goal
Make diagram loop playable: GluonGen infinite -> + / - basins -> Proton -> H atom (needs electron from photon loop) -> Star -> photons loopback + heavy atoms.

Test: cargo test -p su_core should pass diagram_loop_is_sustainable

## Prompts
See prompts/new_factory.md for template to create factory.

## Workflow
1. Read docs/TASKS.md, pick TODO lowest number
2. Implement in smallest crate
3. cargo check -p that crate
4. Update TASKS.md mark DONE
