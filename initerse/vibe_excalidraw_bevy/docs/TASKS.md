# TASKS - Updated with User Decisions

User choices:
- Logistics: Hybrid arrows (early) + Drone ports (Vectorio-style) later. Drones = area requests.
- Building UX: Palette (bottom bar like Excalidraw) + click to place ghost. Keys 1-9.
- Scale: Seamless zoom layers (illusion of nested universe). Hardest tech.
- Physics: Gamey simple recipes for MVP. 2 Up + 1 Down = Proton.

### TASK-001: Scaffold ✅ Done (partial)
Status: DONE - structure created
Files: Cargo.toml + crates

### TASK-002: Dot-grid shader
Status: TODO
Crate: su_render_excali
Accept: Infinite dot grid that fades with zoom, like Excalidraw background

### TASK-003: Infinite Canvas Camera ✅ Stub done
Status: DONE stub
Need polishing: smooth zoom to mouse cursor, bounds

### TASK-004: Rough Renderer ✅ Stub done
Status: DONE stub
Needs: Convert gizmo lines to Mesh2d with Virgil font labels

### TASK-005: Grid + Palette Placement
Status: TODO NEXT
- Bottom UI bar showing 10 building types from diagram
- Ghost wobbly box follows mouse snaps to grid
- Left click places, right click deletes
- Test: can place whole diagram loop

### TASK-006: Resources & Recipes ✅ Done
Status: DONE - see su_core/src/resources.rs, recipes.rs
- Tests pass? Need run cargo test

### TASK-007: Factory tick
Status: TODO
- Integrate Buffer + Factory + RecipeRegistry
- Star needs gravity

### TASK-008: Hybrid Arrow Connections
Status: TODO - structures done in su_logistics
- Draw arrow: click output port of A, click input of B
- Visual flow particles
- Excalidraw-style: arrow thickness = throughput
- Style per resource: wavy photon, coily gluon

### TASK-009: Full Diagram Loop Playable
Status: TODO MILLSTONE
Must be able to place: GluonGen (infinite source) -> splits to +basin/-basin -> Proton Creator -> combine with Electron from Photon loop -> Hydrogen -> Artificial Star -> produces photons back + heavy atoms.
Win if can sustain.

### TASK-010: Drone Logistics (Vectorio-style)
Status: TODO - after arrows
- DronePort building: 3x3, range visual circle (hand-drawn)
- Requester: building sets what it wants
- Provider: building says what it offers
- Drones auto-fetch within range
- Clear canvas: no belts spaghetti at large scale
- Transition: early game uses arrows, mid unlocks drones (like logistic bots in Factorio)

### TASK-011: Seamless Zoom Layer Illusion
Status: TODO HARD
- For now fake it: when zoom < threshold, despawn current layer entities and spawn parent representing factory as one entity
- Later: render nested world to texture

### TASK-012+: Polish etc.

Priority order:
005 -> 008 -> 007 -> 009 -> 010 -> 002 -> 011
