use bevy::platform::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, )]
pub struct WorldPosition {
    pub x: i32,
    pub y: i32,
}


// pub struct World {
// }

// impl World {
//     pub fn new(seed: usize) -> Self {
//         World {
//             buildings: HashMap::new(),
//             seed,
//         }
//     }
// }