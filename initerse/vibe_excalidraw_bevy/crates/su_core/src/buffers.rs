use bevy::prelude::*;
use std::collections::HashMap;
use crate::resources::ResourceType;

#[derive(Component, Default, Debug, Reflect)]
pub struct Buffer {
    pub capacity: f32,
    pub contents: HashMap<ResourceType, f32>,
}

impl Buffer {
    pub fn new(capacity: f32) -> Self {
        Self { capacity, contents: HashMap::new() }
    }
    
    pub fn amount(&self, ty: ResourceType) -> f32 {
        *self.contents.get(&ty).unwrap_or(&0.0)
    }
    
    pub fn can_insert(&self, ty: ResourceType, amount: f32) -> bool {
        let total: f32 = self.contents.values().sum();
        total + amount <= self.capacity
    }
    
    pub fn insert(&mut self, ty: ResourceType, amount: f32) -> f32 {
        if !self.can_insert(ty, amount) {
            return 0.0;
        }
        *self.contents.entry(ty).or_insert(0.0) += amount;
        amount
    }
    
    pub fn remove(&mut self, ty: ResourceType, amount: f32) -> f32 {
        let entry = self.contents.entry(ty).or_insert(0.0);
        let removed = amount.min(*entry);
        *entry -= removed;
        removed
    }
}

pub fn tick_buffers() {
    // Placeholder for buffer decay / validation
}
