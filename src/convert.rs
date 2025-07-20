use crate::loader::SpeckleBase;
use parry3d::{na::Point3, shape::TriMesh};

/// Convert SpeckleBase to a Parry3D Mesh
pub fn convert_to_parry_mesh(obj: &SpeckleBase) -> Option<TriMesh> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Extract vertex positions
    let vertex_data = &obj.display_value.vertices.data;
    let mut chunk_count = 0;
    for chunk in vertex_data.chunks(3) {
        chunk_count += 1;
        if chunk.len() != 3 {
            println!(
                "ERROR: chunk {} has invalid size. Obj id: {}",
                chunk_count,
                { &obj.id }
            );
            break;
        }
        vertices.push(Point3::new(
            chunk[0] as f32,
            chunk[1] as f32,
            chunk[2] as f32,
        ));
    }

    // Extract faces (assuming triangulated data)
    let face_data = &obj.display_value.faces.data;
    let mut i = 0;

    while i < face_data.len() {
        let face_size = face_data[i] as usize;

        if face_size < 3 {
            panic!("Invalid face size: {}", face_size);
        }

        if i + face_size >= face_data.len() {
            break;
        }

        let face_indices = &face_data[i + 1..i + 1 + face_size]; // Extract face indices

        // Convert face into triangles (ear-clipping or triangle fan)
        for j in 1..(face_size - 1) {
            indices.push([
                face_indices[0] as u32,
                face_indices[j] as u32,
                face_indices[j + 1] as u32,
            ]);
        }

        i += face_size + 1; // Move to the next face
    }
    if vertices.len() < 3 {
        println!(
            "ERROR: Object id {} has {} vertices.",
            obj.id,
            vertices.len()
        );
        return None;
    }

    // Create Parry TriMesh
    match TriMesh::new(vertices, indices) {
        Ok(mesh) => Some(mesh),
        Err(err) => {
            println!("Error creating mesh: {err}");
            None
        }
    }
}
