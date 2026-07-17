# Sketched Universe - Game Design Document v0.1
### Tagline: Factorio meets Excalidraw meets Quantum Field Theory

---

## 1. Core Vision from your Diagram

Your diagram is NOT just a placeholder - it's the **Tier 0-2 core loop**. Let me interpret and expand:

```
[Photon Loop - Energy]                 [Charge Loop - Matter]
Virtual Vacuum Fluctuation -> Photon -> e- + e+ ->\
                                                   -> H Atom -> Artificial Star -> Heavy Atoms -> Star Explosion / Extractor -> Higher tiers
Gluon Generator -> u/d quarks -> +basin (2/3) & -basin (1/3) -> Proton Creator -/
```

**This is brilliant.** It's a *closed matter-energy loop*. The artificial star produces photons that feedback into electron-positron creation. That's your early-game sustainable loop, like Factorio's coal -> power loop.

This suggests: **No mining. You CREATE matter.**

### Refinement: The Scale Ladder

Your idea: "very small buildings at beginning (quantum) and then go insanely big"

Proposed 6 Tiers of Reality:

**Tier 0: String Soup (Planck scale)**
- Buildings: 1x1 grid. String vibrator, foam extractor
- Resource: Strings, branes, quantum foam
- Mechanic: You don't place buildings - you *draw* potential wells (like Excalidraw lasso)
- Goal: Emerge first gluon

**Tier 1: Quark Era (From your diagram)**
- Buildings: Gluon Generator (2x2), +/- Basin (2x2), Proton Creator (3x3), Photon Generator (2x2), Electron Factory
- Resource: Gluons, Up(2/3), Down(1/3), Charge
- Mechanic: Introduction of grid + belts as *arrows*.

**Tier 2: Atomic Alchemy (Hydrogen -> Stars)**
- Buildings: Hydrogen Atom Assembler, Artificial Star (starts at 5x5, grows via gravity)
- Mechanic: Star building *attracts* nearby hydrogen (gravity simulation). Hotter -> fusion rate increases. First time you need HEAT management.
- Output: Heavier atoms up to Iron, plus PHOTONS (your feedback)

**Tier 3: Stellar Industry**
- Dyson Swarm (sketched rings), Nebula Collector, Supernova Trigger (player-induced star explosion)
- Resource: All elements, solar wind, neutrinos
- Scale: Buildings now 10x10 to 50x50. Map zoom level changes.

**Tier 4: Galactic / Relativistic**
- Black Hole Forge (uses Hawking radiation as power source), Wormhole Pipe (logistics), Quasar
- Mechanic: Time dilation as a mechanic? Belts that slow time near black holes to increase processing?

**Tier 5: Multiverse**
- Universe Creator (Big Bang machine), Brane Weaver, Entropy Reverser
- Mechanic: Each universe is a separate Factorio "surface". You automate creation of universes with different constants.

## 2. Excalidraw Aesthetic - How to make it *feel* effortless

This is your USP. Not pixel art, not sci-fi gloomy.

**Visual Pillars:**
1.  **Everything hand-drawn:** Boxes with wobbly edges, arrows with sketchy stroke. Use Excalidraw's own algorithm: 2 layers of stroke with random offsets. Bevy: custom shader or pre-bake textures.
2.  **Infinite canvas, not map:** Like Excalidraw, no border. White background with dot grid, not grass. Buildings look like they were *sketched* onto paper.
3.  **Effortless Building:** This is KEY.
    - In Factorio: click building, place, place belt, rotate.
    - In YOUR game: **Hold LMB and drag to draw a box** (like Excalidraw). Game auto-infers what building you want based on size + context? Or drag arrow to auto-connect.
    - **Selection = like Excalidraw:** Drag to select multiple, drag to move whole sub-factory. Copy-paste as in diagram tool. This is HUGE QOL.
    - **Arrows are logistics:** Instead of Factorio belts, you draw an arrow from A to B. That's it. Items flow along the arrow. Arrow thickness = throughput. Arrow style = resource type (dashed for photons, wavy for gluons etc)
    - **Text labels**: Player can write text anywhere on map (like Excalidraw). For annotations, but also maybe some buildings need labels? (Factorio combinators)

**Technical Approach for Shaders in Bevy:**
- Use `bevy_vector_shapes` or `bevy_prototype_lyon` for rough shapes.
- Custom material: Take perfect SDF shape, add vertex displacement with low-frequency noise (hand-drawn wobble). Do it in shader for cheapness.
- Font: Use Virgil (Excalidraw font) or "Caveat"/"Architects Daughter". Bevy text with MSDF.
- Paper texture: subtle paper grain full-screen overlay.
- Animation: When placed, building does a "draw-on" animation (stroke dashoffset).

## 3. Core Gameplay Systems

### Grid + Freeform Hybrid
- Logic Grid: All buildings snap to a hidden grid (e.g., 32px). Even if visual is wobbly, logic is precise.
- Buildings have sizes: 1x1, 2x2, 3x3, 5x5, etc. But you can draw a 3x2 box and game suggests "Proton Creator needs 3x3, expand?"
- Use hierarchical grid: At Quantum Tier, grid cell = 0.1fm, at Galactic tier, cell = 1 AU. Different surfaces/layers.

### Logistics: The Arrow System
Instead of belts + inserters + pipes, unified "Connections":

- `Connection { from: Entity, to: Entity, type: ResourceFilter, curve: Bezier }`
- Visual: Hand-drawn arrow. Can have multiple lanes.
- Flow simulation: Items are visual particles moving along bezier. Simple rate calculation: Throughput = min(source output, target input, arrow capacity)
- Splitter? Draw one arrow branching into two (like Excalidraw arrow branching). Game auto-creates splitter node.
- Power? Photons are energy. Your artificial star outputs photons which are both logistics AND power.

This simplifies Factorio complexity to match Excalidraw mental model.

### Factories are Recursive
Since you go from small to big, **factories can contain factories**:
- Zoom into a Proton Creator - inside it is a mini-map where you arrange quarks?
- Or Zoom out of Artificial Star to see it's now a component in a Galaxy.

Use Bevy's `bevy_ecs_tilemap` + nested worlds. Inspiration: Vectorio which has units as factories.

## 4. Progression Hook

Starting prompt: You are a doodler physicist. White void. You can summon virtual particles.

Win/Loss? Maybe no win, it's open ended like Factorio. Or final goal: Overcome Heat Death by creating a new multiverse that can sustain itself (your first loop becomes self-sustaining!)

Questions for you to loop back.

---

## 5. Tier 0 MVP - What is the minimum playable?

- Infinite dot-grid canvas camera (pan/zoom)
- Place: Gluon Generator, +/- basin, Proton Creator, Hydrogen Atom, Photon Generator, Electron Positron splitter, Artificial Star
- Arrows as connections
- Resource loop from your diagram working
- Art style: wobbly boxes with Virgil font
- Extraction: Artificial star makes heavier atoms

If this loop is fun for 30 mins, the rest will scale.
