Phase 0: Boot (Day 1-2)
- TASK-001: Project scaffolding, Bevy app with empty infinite canvas (camera movement, world struct)
- TASK-002: Dot-grid background shader (Excalidraw style, infinite, zoom-dependent opacity)
- TASK-003: Infinite Canvas Camera: pan (middle-drag + WASD) + zoom lerp + screenspace to world
- TASK-004: Rough Rendering crate: trait RoughDrawable -> Mesh2d generation with hand-drawn jitter. Implement rough_box(width,height) -> Mesh. Use rand seeded per entity for stable wobble.
Phase 1: Effortless Building (Day 3-5) - THE CORE FEEL
- TASK-005: Grid + Placement System: logical grid (64px), but visual free. Placement indicator (ghost wobbly box). Right-click to delete. Size-aware placement (Entity with GridSize(3,3))
- TASK-006: Resource System (pure Rust): Define ResourceType enum: Gluon, UpQuark, DownQuark, ChargePlus, ChargeMinus, Proton, Electron, Positron, Photon, Hydrogen, Helium... Recipe struct: inputs, outputs, time. Storage in Buffer component.
- TASK-007: Factory Plugin Pattern: Generic Factory { recipe, timer, input_buffer, output_buffer }. Create first factories from diagram: GluonGenerator, PlusBasin, MinusBasin, ProtonCreator
- TASK-008: Arrow Logistics (most important UX): User can draw arrow from output of A to input of B. Create Connection { from, to, bezier, resource_type }. System moves resources at rate. Visual: particle moving along curve. Arrows are Bezier with 2 control points draggable.
- TASK-009: Your diagram loop end-to-end: GluonGen -> +-basins -> ProtonCreator + Electron (from Photon decay) -> Hydrogen -> Artificial Star -> Photons (feedback)
- TASK-010: Selection & Manipulation like Excalidraw: Drag-select rectangles, move group, duplicate with Alt+drag, Ctrl+D. Use bevy_mod_picking
Phase 2: Excalidraw Juice (Day 6-8)
 TASK-011: Visual polish: Virgil font labels on buildings, hand-drawn animation on spawn (draw like stroke), paper texture overlay, subtle shadow
 TASK-012: Properties panel (right sidebar): When select building, show rate, buffers, pause button like Excalidraw stroke panel
 TASK-013: Text tool: press T place text anywhere, like annotations. Not gameplay, but feel.
 TASK-014: Library/Handler bar: Top bar like Excalidraw? Bottom bar with building palette. Drag from palette or press key (1-9)
 TASK-015: Save/Load: Serialize core state to JSON/RON. Save = { grids, connections, resources }
Phase 3: Scale (Day 9+)
 TASK-016: Artificial Star Expansion: Star entity's size scales with Hydrogen stored. Gravity field: pull nearby H atoms (force simulation). Heat component: hotter with more fusion.
 TASK-017: Quantum Tunneling: Star fusion probability = f(temp, quantum). Show tunneling effect visualized as wavy dashed line.
 TASK-018: Stellar Extraction & Supernova: Star can explode producing HeavyAtoms resource cloud, collected by extractor building.
 TASK-019: Zoom Layers: When zoom out > threshold, switch to Galactic view. Buildings become stars. N-body gravity using compute shader (stub for your GPU experience)
 TASK-020: Tier progression / Tech tree: sketched on paper as prerequisites drawn as arrows between technologies?
 TASK-021: Black Hole Forge
 TASK-022: String Creator (Planck scale intro building) - make it feel tiny, requires ultra zoom in.
Bonus Phase: Endless
 Multiverse layer
 Multiplayer whiteboard (like Excalidraw live collab - Bevy net?)
 WASM deploy for web demo




Main menu, to load, save, quit, keybind settings
Save/load system

Get some of the same buildings as https://vectorio.wiki.gg/wiki/Buildings
Implement a global inventory system, to have the requirements to build stuff

Add some artstyle, like neon effects to make the game look more animated, reactive and pretty

Gameplay:
Add turrets, enemy spawning, a base to defend, more enemies the more level/matter we have, 
More enemy types, more enemies, more item/ressources types




 Implement Building System
 UI or state for selecting a building type
 Mouse click to place a building on the grid
 Building component (e.g., Miner, Turret, Wall)
 Implement Entity System (Enemies)
 Simple enemy component and spawner
 Enemy movement logic basic stub