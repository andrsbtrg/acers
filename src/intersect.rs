use std::collections::HashMap;

use parry3d::{math::Isometry, query::intersection_test, shape::TriMesh};

use crate::utils;

pub struct WorldObject {
    pub id: usize,
    pub speckle_id: String,
    pub name: String,
    pub tri_mesh: TriMesh,
}

#[allow(dead_code)]
pub fn evaluate_intersection(meshes: &[WorldObject]) {
    let mut mesh_set = HashMap::new();
    for mesh in meshes {
        mesh_set.insert(mesh.id, mesh.to_owned());
    }

    let pos1 = Isometry::identity();

    let ids: Vec<_> = meshes.iter().map(|m| m.id).collect();
    let unique = utils::unique_pairs(&ids);

    for (a, b) in unique {
        let left = mesh_set.get(&a).unwrap();
        let right = mesh_set.get(&b).unwrap();

        if let Ok(result) = intersection_test(&pos1, &left.tri_mesh, &pos1, &right.tri_mesh) {
            println!(
                "Intersection mesh {} - {}\t{}",
                left.name, right.name, result
            );
        }
    }
}
