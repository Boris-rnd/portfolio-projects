# Initerse Game Feature Implementation Plan

## Goal Description
Implement requested features for the Initerse Bevy game:
1. Fix grid alignment so building tiles align with grid squares instead of corners.
2. Upgrade UI hotbar to 10 square slots, allowing toggling selections, adding a "Destroy" option, and mapping slots to keyboard '1'-'0'.
3. Add a transparent "ghost" building preview that follows the mouse when placing buildings.
4. Improve camera zoom to support touchpad smoothly and add connection visuals.
5. Make storage change color when it has stored items.
6. Display visual connection lines when buildings are connected.

## Proposed Changes

### Grid System
#### [MODIFY] [src/grid.rs](file:///home/boris/Documents/initerse/src/grid.rs)
- Update [world_to_grid](file:///home/boris/Documents/initerse/src/grid.rs#42-47) to use `floor()` instead of `round()` so coordinates map cleanly to squares.
- Update [grid_to_world](file:///home/boris/Documents/initerse/src/grid.rs#48-51) to offset the world position by `+ TILE_SIZE / 2.0` on X and Y, centering the grid cell coordinates between the drawn lines.

### User Interface
#### [MODIFY] [src/ui.rs](file:///home/boris/Documents/initerse/src/ui.rs)
- Change `SelectedBuilding` enum to include `Destroy` and map it across 10 Hotbar slots.
- Update [setup_ui](file:///home/boris/Documents/initerse/src/ui.rs#28-96) to draw 10 square slots at the bottom, resembling a Minecraft hotbar, removing text and relying on colors/icons.
- Implement toggle logic: if selecting an already selected slot, revert to `None`.
- Add a new system `hotbar_keyboard_system` to listen to keyboard keys `1` to `0` and set `SelectedBuilding` accordingly.

### Interactivity & Placement
#### [MODIFY] [src/interaction.rs](file:///home/boris/Documents/initerse/src/interaction.rs)
- **Hover Preview:** Add a `GhostPreview` entity/component. Update its visibility, position, and color in a system based on `SelectedBuilding` and mouse position over the grid.
- **Placement & Destruction:** Update [building_placement_system](file:///home/boris/Documents/initerse/src/interaction.rs#27-85). If `Destroy` is selected, despawn the clicked building.
- **Connection Visuals:** In [drag_and_drop_connection_system](file:///home/boris/Documents/initerse/src/interaction.rs#86-129), when creating a [Connection](file:///home/boris/Documents/initerse/src/connection.rs#15-18), also spawn a sprite scaled and rotated to act as a line between the two buildings. Keep a marker component like `ConnectionLine` to update its position or render it.

### Camera Controls
#### [MODIFY] [src/camera.rs](file:///home/boris/Documents/initerse/src/camera.rs)
- Listen to Bevy's `TouchpadMagnify` event (if available) or adjust `MouseWheel` scaling to smoothly support touchpads. Add `+/-` keys as an alternative zoom method.

### Building Logic
#### [MODIFY] [src/building.rs](file:///home/boris/Documents/initerse/src/building.rs)
- Add a `storage_visuals_system` that queries [(&Storage, &mut Sprite)](file:///home/boris/Documents/initerse/src/main.rs#12-35) and alters the color. For instance, interpolating from empty (blue) to full (bright cyan or white) based on `current_amount / max_capacity`.
- Update the item transfer visual logic if necessary to match the grid offsets and make it look clean over connection lines.

## Verification Plan
### Automated Tests
- Run `cargo check` to ensure code compiles.
- Run `cargo clippy` to pass linting.

### Manual Verification
- Run the game visually (`cargo run`).
- Verify grid lines align perfectly around buildings.
- Press `1...0` to change selection, verifying hotbar selects/deselects visually.
- Verify building destruction works.
- Check hover preview follows the mouse perfectly on the grid.
- Drag right-click between Collector and Storage and verify a line appears and items transfer.
- Ensure camera zooms properly with smooth scroll and mouse.
