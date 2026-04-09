//! Relation builder: high-level API for constructing common dependency patterns.

use crate::builder::Depsgraph;
use crate::node::{ComponentType, DepNodeType, IdType};
use crate::NodeId;

/// High-level builder for creating common dependency graph patterns.
pub struct RelationBuilder<'a> {
    graph: &'a mut Depsgraph,
}

impl<'a> RelationBuilder<'a> {
    pub fn new(graph: &'a mut Depsgraph) -> Self {
        Self { graph }
    }

    /// Add a time source node (root of the graph).
    pub fn add_time_source(&mut self) -> NodeId {
        self.graph.add_node("TimeSource", DepNodeType::TimeSource)
    }

    /// Add an object with standard components.
    ///
    /// Blender's evaluation order for an object is:
    ///   1. Animation (F-Curves, NLA)
    ///   2. Drivers (depend on animation, may depend on time source)
    ///   3. Transform (depends on animation + drivers)
    ///   4. Constraints (depends on transform; result feeds back into final transform)
    ///   5. Geometry / Modifiers (depends on final constrained transform)
    pub fn add_object(&mut self, name: &str, has_geometry: bool) -> ObjectNodes {
        let id_node = self.graph.add_id_node(name, IdType::Object);

        let animation = self.graph.add_node(
            format!("{name}/Animation"),
            DepNodeType::Component(ComponentType::Animation),
        );

        let drivers = self.graph.add_node(
            format!("{name}/Parameters"),
            DepNodeType::Component(ComponentType::Parameters),
        );

        let transform = self.graph.add_node(
            format!("{name}/Transform"),
            DepNodeType::Component(ComponentType::Transform),
        );

        let constraints = self.graph.add_node(
            format!("{name}/Constraints"),
            DepNodeType::Component(ComponentType::Constraints),
        );

        // Animation depends on the ID node.
        self.graph.add_dependency(animation, id_node);
        // Drivers depend on animation (animation must evaluate before drivers).
        self.graph.add_dependency(drivers, animation);
        // Transform depends on animation and drivers.
        self.graph.add_dependency(transform, animation);
        self.graph.add_dependency(transform, drivers);
        // Constraints depend on transform.
        self.graph.add_dependency(constraints, transform);

        let geometry = if has_geometry {
            let geo = self.graph.add_node(
                format!("{name}/Geometry"),
                DepNodeType::Component(ComponentType::Geometry),
            );
            // Geometry depends on constraints (final transform) not just raw transform.
            self.graph.add_dependency(geo, constraints);
            Some(geo)
        } else {
            None
        };

        ObjectNodes {
            id: id_node,
            transform,
            animation,
            drivers,
            constraints,
            geometry,
        }
    }

    /// Add an armature object with pose evaluation.
    pub fn add_armature_object(&mut self, name: &str, bone_names: &[&str]) -> ArmatureNodes {
        let obj = self.add_object(name, false);

        let pose = self.graph.add_node(
            format!("{name}/Pose"),
            DepNodeType::Component(ComponentType::Pose),
        );
        // Pose depends on the constrained transform and animation.
        self.graph.add_dependency(pose, obj.constraints);
        self.graph.add_dependency(pose, obj.animation);

        let mut bone_nodes = Vec::new();
        for bone_name in bone_names {
            let bone = self.graph.add_node(
                format!("{name}/{bone_name}/Bone"),
                DepNodeType::Component(ComponentType::Bone),
            );
            self.graph.add_dependency(bone, pose);
            bone_nodes.push((*bone_name, bone));
        }

        ArmatureNodes {
            object: obj,
            pose,
            bones: bone_nodes.into_iter().map(|(n, id)| (n.to_string(), id)).collect(),
        }
    }

    /// Add a constraint dependency: the constrained node depends on the target.
    pub fn add_constraint_relation(&mut self, constrained: NodeId, target: NodeId) {
        self.graph.add_dependency(constrained, target);
    }

    /// Add a parent-child relationship between objects.
    pub fn add_parent_relation(&mut self, child_transform: NodeId, parent_transform: NodeId) {
        self.graph.add_dependency(child_transform, parent_transform);
    }
}

/// Node IDs for a standard object.
pub struct ObjectNodes {
    pub id: NodeId,
    pub transform: NodeId,
    pub animation: NodeId,
    /// Drivers / Parameters node (evaluated after animation, before transform).
    pub drivers: NodeId,
    /// Constraints node (evaluated after transform, before geometry).
    pub constraints: NodeId,
    pub geometry: Option<NodeId>,
}

/// Node IDs for an armature object.
pub struct ArmatureNodes {
    pub object: ObjectNodes,
    pub pose: NodeId,
    pub bones: Vec<(String, NodeId)>,
}
