Create new factory building: {{NAME}}.

In crate su_factories.

Steps:
1. Create component {{NAME}} { buffer: Buffer, ... } if needed or use generic Factory.
2. Define recipe in su_core/src/recipes.rs if not exists: inputs {{INPUTS}} -> outputs {{OUTPUTS}} at rate {{RATE}} secs
3. Create plugin struct {{NAME}}Plugin implementing Plugin, adds tick_factory or custom tick system that consumes {{INPUTS}} and produces {{OUTPUTS}}
4. Size: {{SIZE}} from BuildingType enum in su_grid::placement. Add if missing.
5. Visual: RoughBox with seed = entity id hash. Label {{LABEL}}
6. Add to SuFactoriesPlugin lib.rs
7. Test: place building, check buffer fills

Reference: gluon.rs for infinite source example.

Acceptance:
- cargo check -p su_factories passes
- Building appears in bottom palette
- Factory produces when inputs present

User wants palette UX, so also update su_ui palette.

Example:
NAME=DronePort
INPUTS=None (provides logistics)
OUTPUTS=None
SIZE=3x3
LABEL="Drone port\nrange 500"
