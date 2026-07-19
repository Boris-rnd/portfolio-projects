The world struct holds a list of 

ECS Design:

GridPos { x, y, layer } - int grid coords
GridSize { w, h }
Building { type_id: BuildingType, rotation }
Buffer { capacity, contents: HashMap<ResourceType, f32> } - f32 to allow rates
Factory { recipe: RecipeId, progress: f32 }
Connection { from: Entity, from_port: u8, to: Entity, to_port: u8, curve: CubicBezier, throughput: f32 }
Resource: ResourceRegistry, RecipeRegistry
Event-driven logistics: Use bevy_ecs events: ResourceProduced, ResourceRequested

Performance: Logistics tick at 10hz fixed schedule, rendering at variable. Use FixedUpdate.

