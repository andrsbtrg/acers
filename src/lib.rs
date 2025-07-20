pub mod convert;
pub mod intersect;
pub mod loader;
pub mod utils;

use std::collections::HashSet;

use convert::convert_to_parry_mesh;
use intersect::WorldObject;
use loader::{load_objs, SpeckleBase};
use parry3d::math::Isometry;
use pyo3::prelude::*;

#[pyfunction]
fn clash_detection(a: String, b: String) -> PyResult<Vec<(String, String)>> {
    let set_a = to_world_object(load_objs(a));
    let set_b = to_world_object(load_objs(b));

    let pos1 = Isometry::identity();
    let mut clash_pairs = Vec::new();
    let mut seen = HashSet::new();

    for a in &set_a {
        for b in &set_b {
            if a.speckle_id == b.speckle_id {
                continue; // Skip self-pairs
            }

            // Ensure consistent ordering for uniqueness
            let key = if a.id < b.id {
                (a.id, b.id)
            } else {
                (b.id, a.id)
            };

            if seen.contains(&key) {
                continue;
            }

            if let Ok(result) =
                parry3d::query::intersection_test(&pos1, &a.tri_mesh, &pos1, &b.tri_mesh)
            {
                if result {
                    seen.insert(key);
                    clash_pairs.push((a.speckle_id.clone(), b.speckle_id.clone()));
                }
            }
        }
    }

    Ok(clash_pairs)
}

fn to_world_object(objs: Vec<SpeckleBase>) -> Vec<WorldObject> {
    objs.iter()
        .enumerate()
        .filter_map(|(i, obj)| {
            convert_to_parry_mesh(obj).map(|mesh| WorldObject {
                name: obj.name.to_owned(),
                speckle_id: obj.id.to_owned(),
                tri_mesh: mesh,
                id: i,
            })
        })
        .collect::<Vec<WorldObject>>()
}

/// A Python module implemented in Rust.
#[pymodule]
fn acers(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(clash_detection, m)?)?;
    Ok(())
}
