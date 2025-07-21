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

#[pyclass]
struct Collision {
    #[pyo3(get)]
    ids: (String, String),
    #[pyo3(get)]
    dist: f32,
    #[pyo3(get)]
    point: (f32, f32, f32),
}

#[pymethods]
impl Collision {
    fn __str__(&self) -> PyResult<String> {
        PyResult::Ok(format!(
            "Collision(ids:('{}', '{}'),dist: {:.2}, point: ({:.2}, {:.2}, {:.2}))",
            self.ids.0, self.ids.1, self.dist, self.point.0, self.point.1, self.point.2
        ))
    }
    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}

#[pyfunction]
#[pyo3(signature = (a,b,min_dist=0.0))]
fn clash_detection(a: String, b: String, min_dist: Option<f32>) -> PyResult<Vec<Collision>> {
    let set_a = to_world_object(load_objs(a));
    let set_b = to_world_object(load_objs(b));

    let pos1 = Isometry::identity();
    let mut clash_pairs = Vec::new();
    let mut seen = HashSet::new();

    let dist = min_dist.unwrap_or(0.0);

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

            if let Ok(contact) =
                parry3d::query::contact(&pos1, &a.tri_mesh, &pos1, &b.tri_mesh, 0.5)
            {
                if let Some(pc) = contact {
                    if pc.dist < dist {
                        clash_pairs.push(Collision {
                            ids: (a.speckle_id.clone(), b.speckle_id.clone()),
                            dist: pc.dist,
                            point: (pc.point1.x, pc.point1.y, pc.point1.z),
                        });
                    }
                }
            }
            seen.insert(key);
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
    m.add_class::<Collision>()?;
    Ok(())
}
