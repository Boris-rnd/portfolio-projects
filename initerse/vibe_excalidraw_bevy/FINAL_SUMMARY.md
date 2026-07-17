# Your Game Plan - Final Summary for you (with your decisions)

## Your Answers Applied

**1. Logistics** - You said: "drones like vectorio, any building that wants X sends drones" - but start hybrid.
=> My solution: Dual system.
- Early (Tier 1-2): Hybrid arrows - you draw Bezier arrows like in Excalidraw, but they act as belts with visible dots flowing. Throughput shown by thickness. Resource style: wavy for photons, coily for gluons (Feynman diagrams!). This teaches player flow.
- Mid (Tier 2+): DronePort unlocks. Buildings get Requester/Provider toggles in right properties panel (like Excalidraw style panel). Drones (cute hand-drawn birds?) auto-fetch within range circle. Canvas stays clean. This solves spaghetti at huge scale and is perfect for seamless zoom layers.

**2. Building UX** - You chose Palette.
=> Bottom bar like Excalidraw: 10 icons, keys 1-9. Top bar: Select (V), Hand (H), Connect (C), Text (T). Left-click places ghost, right-click erases. Select tool allows drag-select + move group + Alt+drag duplicate. This is easier for LLM to code than inferring building from box size.

**3. Scale** - You chose Seamless Zoom Layers - hardest but coolest.
=> Implementation plan for MVP fake:
- Each entity has RealityLayer enum.
- Zoom thresholds: 0.5, 0.1, 0.02
- When zoom < 0.5, current factories fade, and we spawn a Cluster sprite that is literally a screenshot of your factory (or auto-generated thumbnail) as a building in parent layer. So zooming out makes your whole proton factory look like a doodle inside a star.
- Later with your GPU sim experience: use compute shader for star gravity N-body, render galaxy layer.
- This satisfies "go from tiny strings to multiverse".

**4. Physics** - You chose Gamey & Fun for MVP.
=> Recipes are simple: 2 Up + 1 Down + charge = Proton, etc. Code in su_core/src/recipes.rs already bootstraps your exact diagram and has test `diagram_loop_is_sustainable` that checks photon feedback loop positive.
=> Future tech tree can add semi-real formula as upgrades: Research "Quantum Tunneling" -> star fusion probability formula = exp(-barrier/temperature) -> gives +20% heavy atoms. You handle formulas.

## What I Built For You in /home/user/sketched-universe/

```
sketched-universe/  (ready for `cargo run`)
├── Cargo.toml (workspace)
├── AGENTS.md (LLM master prompt updated with your decisions)
├── src/main.rs (App builder)
├── crates/
│   ├── su_core: ResourceType enum (color + sketch style per resource), RecipeRegistry bootstrapping YOUR diagram loop, Buffer component + tests
│   ├── su_grid: CanvasCamera with pan+zoom lerp like Excalidraw, GridPos, GridSize, BuildingType with labels from your diagram, PlacementState with keys 1-6, ZoomLayersPlugin with thresholds
│   ├── su_render_excali: RoughBox algorithm - double-stroke wobbly rectangles seeded per entity (Excalidraw algorithm)
│   ├── su_logistics: BezierCurve arrow + FlowParticle + Connection system + DronePort/Requester/Provider stubs
│   ├── su_factories: Generic Factory { progress }, tick_factory, GluonGenerator infinite source, ArtificialStar with mass, temperature, gravity attracting hydrogen, expansion scaling
│   └── su_ui: PaletteState stub
├── docs/
│   ├── TASKS.md (updated with drone task TASK-010)
│   └── ARCHITECTURE.md (detailed drone vs arrow logic)
└── .cursor/rules/ (Bevy rules for Cursor)
```

This is LLM-ready. Give a single crate to Cursor Composer: "Implement TASK-005 using AGENTS.md" and it won't break others.

## LLM Workflow - Tailored to You (Rust expert with GPU sims)

Since you can code OS, don't let LLM do architecture - you keep physics core.

Recommended: **Cursor Agent Mode + Git Worktrees**

1.  Morning: Define 2 tasks in docs/TASKS.md
2.  `git worktree add ../branch-task005 TASK-005`
3.  Open that worktree in Cursor, Composer: "Implement TASK-005, only modify crates/su_grid and su_ui, follow AGENTS.md"
4.  You review, run `cargo check -p su_grid`
5.  Merge.
6.  Afternoon: You code hard part (compute shader star gravity, tunneling probability).

Keep `su_core` pure Rust - LLM can write tests but you write QCD simplification.

## Next Questions to Refine (if you want deeper dive)

To make this even better:
1.  **Drones**: Should drone range be infinite but slower with distance (Vectorio) or limited circle that you need to chain (Factorio roboports)? Range chaining is more challenge.
2.  **Art**: Do you want pure black-white like Excalidraw.com or subtle tints? I implemented tint per resource (photon yellow, gluon orange) but still black outlines - is that okay?
3.  **Progression**: After Artificial Star produces Heavy Atoms, how do you get to Black Hole? Could be: Too much Heavy Atoms in star -> collapses into Black Hole Forge building that outputs Hawking Radiation (your "photons" loop at higher tier).
4.  **Effortless feel**: Excalidraw has infinite undo, library, collaboration. Should we have Ctrl+Z undo placement and Ctrl+C/V copy-paste of factory groups? (This is trivial with ECS but huge UX win, recommend yes).

Want me to:
- Generate the first playable WASM build mockup in HTML (with paper texture)?
- Create the Virgil font hand-drawn UI mock in Bevy Egui?
- Write the exact prompt you should paste into Cursor to get TASK-005 done in one shot?

The scaffold compiles logic wise; once you install Rust + Bevy deps you'll have canvas + wobbly boxes. The docs folder contains full plan.
