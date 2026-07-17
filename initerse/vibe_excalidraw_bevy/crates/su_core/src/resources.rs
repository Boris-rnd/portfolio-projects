use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Core resources from your diagram + future tiers
/// Kept gamey per your choice - simple recipes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum ResourceType {
    // Tier 0 - String
    StringFragment,
    
    // Tier 1 - Quark Era (from diagram)
    Gluon,
    UpQuark,   // 2/3 charge
    DownQuark, // 1/3 charge
    ChargePlus, // + basin
    ChargeMinus, // - basin
    
    // Tier 1b - Photon loop
    Photon,
    Electron,
    Positron,
    
    // Tier 2 - Atomic
    Proton,
    Hydrogen,
    Deuterium,
    Helium,
    HeavyAtoms, // generic until we split periodic table
    
    // Tier 3+
    DarkMatter,
    HawkingRadiation,
    UniverseSeed,
}

impl ResourceType {
    pub fn color(&self) -> Color {
        // Excalidraw palette - mostly black but slight tint
        match self {
            Self::Gluon => Color::srgb(1.0, 0.5, 0.0), // orange
            Self::UpQuark => Color::srgb(0.9, 0.2, 0.2),
            Self::DownQuark => Color::srgb(0.2, 0.6, 0.9),
            Self::Photon => Color::srgb(1.0, 0.9, 0.2),
            Self::Electron => Color::srgb(0.2, 0.8, 0.9),
            Self::Proton => Color::srgb(0.9, 0.2, 0.4),
            Self::Hydrogen => Color::srgb(0.6, 0.9, 1.0),
            Self::HeavyAtoms => Color::srgb(0.7, 0.7, 0.7),
            _ => Color::BLACK,
        }
    }
    
    pub fn sketch_style(&self) -> SketchLineStyle {
        match self {
            Self::Photon => SketchLineStyle::Wavy,
            Self::Gluon => SketchLineStyle::Coily,
            Self::Electron => SketchLineStyle::Dashed,
            _ => SketchLineStyle::Solid,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SketchLineStyle {
    Solid,    // normal arrow
    Dashed,   // electrons
    Wavy,     // photons
    Coily,    // gluons like Feynman diagram
    Dotted,   // gravity
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ResourceRegistry {
    pub unlocked: Vec<ResourceType>,
}

impl ResourceRegistry {
    pub fn is_unlocked(&self, ty: ResourceType) -> bool {
        self.unlocked.contains(&ty)
    }
}
