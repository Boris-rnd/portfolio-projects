AGENTS.md - Instructions for all LLM agents on this project
Project: Sketched Universe - Excalidraw Factorio
Game is 2D grid automation from quarks to multiverse. Hand-drawn aesthetic like excalidraw.com. Bevy 0.19

User decisions (FINAL):

Logistics: Hybrid bezier arrows (early) + Vectorio-style drone ports (range-based auto-delivery)
Building UX: Bottom palette + select tool like Excalidraw, keys 1-9
Physics: Gamey simple recipes (2 Up + 1 Down = Proton)
Absolute Rules
Make folders and separate the game logic from the rendering logic
Try to avoid unwrap() in systems - use if let Ok()
Docs/TASKS.md is source of truth - pick task there
Crate Responsibilities
core: resources enum, recipes, buffers, save serde
grid: camera pan/zoom lerp to cursor, grid snapping, placement ghost, zoom layer visibility
render_excali: RoughBox random seed stable, gizmos for now then Mesh2d, Virgil font placeholder
logistics: Connection component + curve + particle, DronePort + Requester/Provider + matching
factories: Factory component { recipe_id, progress } + each building type tick
ui: Or simple bevy UI bottom bar. Palette state

TODO (but not important in beginning of development): All shapes must be wobbly via RoughBox - never perfect rect
Rough algorithm reference
Generate wobbly rect by jittering edge midpoints with seeded rng.

Workflow
Read docs/TASKS.md, pick TODO lowest number
Implement
Update TASKS.md mark DONE
