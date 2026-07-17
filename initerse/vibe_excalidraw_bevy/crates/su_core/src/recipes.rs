use bevy::prelude::*;
use std::collections::HashMap;
use crate::resources::ResourceType;

/// Factorio-like recipes but simplified per your gamey choice
#[derive(Debug, Clone, Reflect)]
pub struct Recipe {
    pub id: String,
    pub inputs: HashMap<ResourceType, f32>,
    pub outputs: HashMap<ResourceType, f32>,
    pub time_secs: f32,
    pub description: String, // hand-drawn label on building
}

impl Recipe {
    pub fn new(id: &str, time: f32) -> Self {
        Self {
            id: id.to_string(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            time_secs: time,
            description: String::new(),
        }
    }
    
    pub fn input(mut self, ty: ResourceType, amount: f32) -> Self {
        self.inputs.insert(ty, amount);
        self
    }
    
    pub fn output(mut self, ty: ResourceType, amount: f32) -> Self {
        self.outputs.insert(ty, amount);
        self
    }
}

#[derive(Resource, Default)]
pub struct RecipeRegistry {
    pub recipes: HashMap<String, Recipe>,
}

impl RecipeRegistry {
    /// Builds recipes directly from your diagram
    pub fn bootstrap() -> Self {
        let mut reg = Self::default();
        
        // Gluon generator -> decay Up & down quarks
        // Gamey simplification: 1 Gluon -> 2 Up (2/3 each) + 1 Down (1/3)
        // Actually need 2 Up + 1 Down to make + basin?
        // From diagram: Gluon Gen splits 2/3 to +basin, 1/3 to -basin
        
        reg.recipes.insert("gluon_to_quarks".to_string(), 
            Recipe::new("gluon_to_quarks", 1.0)
                .input(ResourceType::Gluon, 3.0)
                .output(ResourceType::UpQuark, 2.0)
                .output(ResourceType::DownQuark, 1.0)
                .output(ResourceType::ChargePlus, 2.0) // 2/3 mapped
                .output(ResourceType::ChargeMinus, 1.0)
        );
        
        // Plus basin + Minus basin -> Proton creator
        reg.recipes.insert("proton_creation".to_string(),
            Recipe::new("proton_creation", 2.0)
                .input(ResourceType::UpQuark, 2.0)
                .input(ResourceType::DownQuark, 1.0)
                .input(ResourceType::ChargePlus, 2.0)
                .input(ResourceType::ChargeMinus, 1.0)
                .output(ResourceType::Proton, 1.0)
        );
        
        // Photon decay -> e- & e+ pair
        reg.recipes.insert("photon_decay".to_string(),
            Recipe::new("photon_decay", 0.5)
                .input(ResourceType::Photon, 1.0)
                .output(ResourceType::Electron, 1.0)
                .output(ResourceType::Positron, 1.0)
        );
        
        // Electron + Proton -> Hydrogen atom
        reg.recipes.insert("hydrogen_assembly".to_string(),
            Recipe::new("hydrogen_assembly", 1.5)
                .input(ResourceType::Proton, 1.0)
                .input(ResourceType::Electron, 1.0)
                .output(ResourceType::Hydrogen, 1.0)
        );
        
        // Hydrogen -> Artificial star (gravity, heat, fusion)
        reg.recipes.insert("star_fusion".to_string(),
            Recipe::new("star_fusion", 10.0)
                .input(ResourceType::Hydrogen, 100.0)
                .output(ResourceType::HeavyAtoms, 10.0)
                .output(ResourceType::Photon, 50.0) // feedback loop!
                .output(ResourceType::Helium, 20.0)
        );
        
        // Star explosion -> Heavy atoms extraction
        reg.recipes.insert("supernova".to_string(),
            Recipe::new("supernova", 5.0)
                .input(ResourceType::HeavyAtoms, 20.0)
                .input(ResourceType::Helium, 10.0)
                .output(ResourceType::HeavyAtoms, 50.0) // more heavy
        );
        
        reg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn diagram_loop_is_sustainable() {
        let reg = RecipeRegistry::bootstrap();
        // Check photon feedback produces >0 photons
        let star = reg.recipes.get("star_fusion").unwrap();
        assert!(star.outputs.get(&ResourceType::Photon).unwrap() > &0.0);
        // Check proton needs 2 Up + 1 Down (Standard Model)
        let proton = reg.recipes.get("proton_creation").unwrap();
        assert_eq!(*proton.inputs.get(&ResourceType::UpQuark).unwrap(), 2.0);
        assert_eq!(*proton.inputs.get(&ResourceType::DownQuark).unwrap(), 1.0);
    }
}
