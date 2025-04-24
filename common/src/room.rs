use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Component)]
pub struct Face {
    pub id: usize,
    pub normal: Vec3,
}

impl Face {
    pub fn normal(&self) -> Vec3 {
        self.normal
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Component)]
pub struct Room {
    pub position: Vec3,
    pub face: Face,
    pub id: u64,
}

impl Room {
    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn face(&self) -> Face {
        self.face.clone()
    }

    pub fn project_other_to_face(&self, other: &Self) -> Vec3 {
        other.position()
            - self.face().normal().dot(other.position() - self.position()) * self.face().normal()
    }
}

impl Ord for Room {
    fn cmp(&self, other: &Room) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for Room {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Room {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Room {
    fn eq(&self, other: &Self) -> bool {
        self.position.distance(other.position) < 0.01
    }
}

impl Eq for Room {}

#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Edge;
