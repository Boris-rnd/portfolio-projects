# ARCHITECTURE - Post User Decisions

## Logistics Decision: Hybrid + Drones

We will have TWO logistics systems that coexist.

### Phase 1: Explicit Arrows (Tutorial -> Early Game)

User action:
1. Select connection tool (C key like Excalidraw arrow tool)
2. Click output port (small dot on right side of building)
3. Drag bezier, click input port of target
4. Arrow created, filters auto-set to what target needs.

Visuals:
- Arrow = cubic bezier, control points draggable
- Items = small hand-drawn circles moving along t
- Color = ResourceType::color()
- Thickness = log(throughput) * 2 px
- Line style: solid proton, dashed electron, wavy photon (Feynman diagram style), coily gluon, dotted gravity

Implementation:
- `Connection { from, to, curve, resource_filter, throughput, particles }`
- FixedUpdate 10hz: source buffer -> try insert into target buffer, limited by throughput
- Copy source: 0.5 sec rate
- If source empty, particle stops

This is intuitive, mirrors Excalidraw arrow drawing.

### Phase 2: Drone Ports (Mid Game - Vectorio Inspired)

Unlock after Artificial Star tech.

User action:
- Place DronePort building. Shows range circle (hand-drawn).
- Any building in range can be set to Request or Provide mode (toggle in properties panel).
- Example: Set +basin to Provide UpQuark, ProtonCreator to Request UpQuark.
- Drones spawn automatically.

Drone Entity:
```
Drone {
  carrying: Option<ResourceType>,
  path: Vec<Vec2>,
  speed: 50 px/sec,
  state: ToPickup / ToDeliver / Idle
}
```

Benefits:
- Keeps canvas clean when factories grow to 100+ buildings - no spaghetti.
- Matches "area that says it wants X" idea you described.
- Excalidraw feel: less lines, more empty white space.

Implementation:
- Spatial query: for each Requester without fulfillment, find nearest Provider in range offering resource.
- Use bevy's `bevy_spatial` or simple distance check (small number entities early).
- Drone visual: small hand-drawn helicopter/bird, tilts in direction.

You can mix: some critical paths use arrows for high throughput, drones for low-volume heavy atoms.

## Zoom Layers - Seamless Illusion

Your choice: seamless zoom layers.

Concept:
```
Layer 0 (String) zoom 5x-20x cell_size = 0.1
Layer 1 (Quark) zoom 1x-5x cell_size = 64px (default)
Layer 2 (Atomic) zoom 0.2x-1x cell_size = 64px but buildings are stars
Layer 3 (Galactic) zoom 0.02x-0.2x
```

Implementation Plan:

- Each entity has `RealityLayer` component.
- CameraState knows current_zoom, computes current_layer = f(zoom).
- Visibility: only current_layer + neighbors visible.
- Transition: when zoom crosses threshold, we spawn "Cluster" entity that represents entire previous layer's bounding box as a single building with Buffer containing aggregated resources.

Better: Use render-to-texture:
1. Take screenshot of Layer N factory cluster.
2. Create Sprite with that texture that is placed in Layer N+1 as building thumbnail (hand-drawn border).
3. When zoom out, you see your factory as a doodle inside galaxy.

This is similar to Factorio's map view where factories become icons.

For MVP: fake with despawn/spawn.

Later: compute shader for star gravity N-body.

## Building UX: Palette

Your choice: Palette + click to place (Factorio style but with Excalidraw tools bar).

UI Layout Mock (designed for LLM to build in egui):

```
[Top Bar: Hand (H) | Select (V) | Arrow Connect (C) | Text (T) | Eraser (E)]
-----------------------------------------------------------------
                       Canvas (infinite dot grid)
                        Camera pan/zoom here

[ Bottom Bar: Palette ]
[ (1) GluonGen | (2) +Basin | (3) -Basin | (4) Proton | (5) H Atom | (6) Star | (7) PhotonGen | (8) e-e+ | (9) Extractor | (0) DronePort ]
```

Interactions:
- Click palette item, ghost follows mouse.
- Right-click cancel.
- Press Esc clears selection -> back to Select tool.
- Select tool (V): drag-select multiple, move group, Alt+drag duplicate (like Excalidraw).
- Hand tool (H): middle-drag also pans.

This is easy for LLM: egui bottom panel with buttons.

## Physics: Gamey Simple

Per your choice: 2 Up + 1 Down = Proton etc.

Keep formulas in `su_core::recipes` for easy balancing.

Future: add tech upgrades that make recipes more efficient based on semi-real physics - e.g., "Quantum Tunneling Research" increases star fusion photon output by 20% (exp(-barrier)).

