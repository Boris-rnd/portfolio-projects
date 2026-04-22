The building tiles are not aligned corretly compared to the background tiles (they are in the middle of the corners)
When selecting collector in hotbar, then storage, both get "selected" in green, which doesn't make sense, we should be able to select/deselect them, and also a menu to destroy
Also it would be great to preview the creation of the building in "hover" mode when selected
It would be great to also be able to zoom in/out with the mouse or with the touchpad
Also I need to be able to drag from the collector to the storage to initiate a connection (represented by a line for example) which would start transfering the items from the collector to the storage
Make the storage change color when has items in it
Be able to select from hotbar using the keyboard (1 for collector, 2 for storage, etc, and make the hotbar size be of 10 items (which would be represented by squares, not text, like a minecraft hotbar for example)



Matter Collector and Storage Implementation
We will implement a modular component-based system in Bevy to handle resources, collectors, and storage.

Game Content Ideas
To expand the game, we should eventually add:

Conveyor Belts: To transport resources over long distances.
Refineries: To process basic matter into advanced materials used for towers.
Defensive Turrets: To defend the factory against enemy waves.
Enemy Spawning: Enemies that pathfind toward your core or active buildings.
Tech Tree: Research labs that consume matter to unlock new buildings.
User Review Required
Are there specific resource kinds you want immediately (like Iron, Copper, Plasma), or just a generic "Matter" type for now? For this implementation, I will assume a generic "Matter" type is sufficient for the MVP.

Proposed Changes
src/building.rs & src/connection.rs
Introduce basic components for our buildings and connections.

GridPosition(IVec2) component to track coordinates.
Collector component to track collection_rate.
Storage component to track current_amount and max_capacity.
Connection component or resource to store directed links between buildings (e.g., Collector -> Storage).
ItemMovement component for spawned visual items (circles) moving from source to target.
src/ui.rs (New Module)
Implement a Bevy UI Hotbar system to select which building to place.
Track a SelectedBuildingType resource to know whether the user wants to place a Collector or Storage.
src/interaction.rs (New Module)
Handle left-clicks to place the currently selected building on the grid.
Implement a drag-and-drop linking system. For example, if the user starts dragging from a Collector and drops onto a Storage, a formal Connection is created.
Update collection_system: Instead of instantly dropping matter, spawn a tiny item entity (a circle) that moves towards the linked Storage over time. Once it arrives, increment the Storage's amount.
Verification Plan
Automated Tests
N/A for initial prototype visualization.
Manual Verification
Run the game with cargo run.
Click on the grid to place a Storage block (e.g., right-click).
Click to place a Collector block next to it (e.g., left-click).
Use the already integrated bevy-inspector-egui to inspect the Storage component in real time and verify its current_amount increases periodically while the Collector is adjacent to it.