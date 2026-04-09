//! Pose: runtime animation state for an armature.

mod channel;

pub use channel::PoseChannel;

use crate::bone::Bone;
use crate::BoneIndex;
use serde::{Deserialize, Serialize};

/// The pose of an armature at a particular point in time.
///
/// Contains one `PoseChannel` per bone. Pose channels hold the animated
/// transform and constraint stack for each bone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pose {
    /// One pose channel per bone, indexed by `BoneIndex`.
    pub channels: Vec<PoseChannel>,
}

impl Pose {
    /// Create a default pose (all identity) for the given bones.
    pub fn from_bones(bones: &[Bone]) -> Self {
        let channels = bones
            .iter()
            .map(|bone| PoseChannel::new(&bone.name, bone.index))
            .collect();
        Self { channels }
    }

    /// Get a pose channel by bone index.
    pub fn channel(&self, index: BoneIndex) -> Option<&PoseChannel> {
        self.channels.get(index as usize)
    }

    /// Get a mutable pose channel by bone index.
    pub fn channel_mut(&mut self, index: BoneIndex) -> Option<&mut PoseChannel> {
        self.channels.get_mut(index as usize)
    }

    /// Find a pose channel by bone name.
    pub fn channel_by_name(&self, name: &str) -> Option<&PoseChannel> {
        self.channels.iter().find(|ch| ch.name == name)
    }

    /// Find a mutable pose channel by bone name.
    pub fn channel_by_name_mut(&mut self, name: &str) -> Option<&mut PoseChannel> {
        self.channels.iter_mut().find(|ch| ch.name == name)
    }

    /// Reset all pose channels to their rest/identity transforms.
    pub fn reset(&mut self) {
        for ch in &mut self.channels {
            ch.reset();
        }
    }
}
