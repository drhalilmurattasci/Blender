//! Bone hierarchy utilities.

use crate::bone::Bone;
use crate::{ArmatureResult, BoneIndex, NO_PARENT};

/// Provides hierarchy queries over a flat array of bones.
pub struct BoneHierarchy<'a> {
    bones: &'a [Bone],
}

impl<'a> BoneHierarchy<'a> {
    pub fn new(bones: &'a [Bone]) -> Self {
        Self { bones }
    }

    /// Get a bone by index.
    pub fn get(&self, index: BoneIndex) -> Option<&Bone> {
        self.bones.get(index as usize)
    }

    /// Get a bone by name.
    pub fn find_by_name(&self, name: &str) -> Option<&Bone> {
        self.bones.iter().find(|b| b.name == name)
    }

    /// Return the parent chain from the given bone up to the root (inclusive).
    pub fn ancestor_chain(&self, bone_index: BoneIndex) -> Vec<BoneIndex> {
        let mut chain = Vec::new();
        let mut current = bone_index;
        while current != NO_PARENT {
            if chain.contains(&current) {
                // Cycle detected, bail out.
                break;
            }
            chain.push(current);
            current = self.bones.get(current as usize).map_or(NO_PARENT, |b| b.parent);
        }
        chain
    }

    /// Iterate bones in topological order (parents before children).
    /// Returns bone indices in evaluation order.
    pub fn topological_order(&self) -> Vec<BoneIndex> {
        let mut order = Vec::with_capacity(self.bones.len());
        let mut visited = vec![false; self.bones.len()];

        // First, add all root bones.
        for bone in self.bones {
            if bone.is_root() && !visited[bone.index as usize] {
                self.visit_dfs(bone.index, &mut visited, &mut order);
            }
        }

        // Add any remaining unvisited bones (handles disconnected sub-trees).
        for bone in self.bones {
            if !visited[bone.index as usize] {
                self.visit_dfs(bone.index, &mut visited, &mut order);
            }
        }

        order
    }

    fn visit_dfs(&self, index: BoneIndex, visited: &mut [bool], order: &mut Vec<BoneIndex>) {
        let idx = index as usize;
        if idx >= visited.len() || visited[idx] {
            return;
        }
        visited[idx] = true;

        // Ensure parent is visited first.
        let parent = self.bones[idx].parent;
        if parent != NO_PARENT && !(visited.get(parent as usize).copied().unwrap_or(true)) {
            self.visit_dfs(parent, visited, order);
        }

        order.push(index);

        for &child in &self.bones[idx].children {
            self.visit_dfs(child, visited, order);
        }
    }

    /// Validate the hierarchy has no cycles.
    pub fn validate(&self) -> ArmatureResult<()> {
        for bone in self.bones {
            let chain = self.ancestor_chain(bone.index);
            // If the chain length exceeds the number of bones, there's a cycle.
            if chain.len() > self.bones.len() {
                return Err(crate::ArmatureError::CyclicHierarchy(bone.name.clone()));
            }
        }
        Ok(())
    }
}
