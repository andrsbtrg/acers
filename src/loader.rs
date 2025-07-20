use serde_json::Value;
use std::{collections::HashMap, fs};

#[allow(dead_code)]
pub enum SpeckleTypes {
    Base(SpeckleBase),
    Collection,
    DataChunk(DataChunk),
    Mesh(Mesh),
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct SpeckleBase {
    pub id: String,
    pub name: String,
    pub display_value: Mesh,
    pub transform: Transform,
    speckle_type: String,
}

#[derive(Debug)]
pub struct DataChunk {
    pub data: Vec<f64>,
}

#[derive(Debug)]
pub struct Transform {
    pub matrix: Vec<f64>,
}

impl Transform {
    pub fn identity() -> Self {
        Transform {
            matrix: [
                1., 0., 0., 0., //
                0., 1., 0., 0., //
                0., 0., 1., 0., //
                0., 0., 0., 1., //
            ]
            .into(),
        }
    }
}

#[derive(Debug)]
pub struct Mesh {
    pub vertices: DataChunk,
    pub faces: DataChunk, // Face
}
pub fn load_from_file(path: &str) -> Vec<SpeckleBase> {
    let content = fs::read_to_string(path).expect("Failed to read file");
    load_objs(content)
}
pub fn load_objs(content: String) -> Vec<SpeckleBase> {
    // # Step 1: Read and store all objects
    let mut object_map: HashMap<String, Value> = HashMap::new();
    for line in content.lines() {
        let Some((_, json_str)) = line.split_once('\t') else {
            continue;
        };
        let Ok(json) = serde_json::from_str::<Value>(json_str) else {
            continue;
        };
        let Some(_speckle_type) = json.get("speckle_type") else {
            continue;
        };
        let Some(id) = json.get("id") else {
            continue;
        };
        if let Some(id) = id.as_str() {
            object_map.insert(id.to_string(), json);
        }
    }

    let mut base_objs = Vec::<SpeckleBase>::new();

    // # Step 2: Resolve references and extract mesh data
    for (_id, obj) in object_map.iter() {
        if let Some(speckle_base) = get_speckle_base(obj, &object_map) {
            base_objs.push(speckle_base);
        }
    }
    // Step 3: Apply transforms
    // Step 4: Construct Mesh

    base_objs
}

fn get_speckle_base(obj: &Value, object_map: &HashMap<String, Value>) -> Option<SpeckleBase> {
    let speckle_type = obj.get("speckle_type")?.as_str()?;

    /* if speckle_type != "Base" {
        return None;
    } */
    let id = obj.get("id")?;
    let name = obj.get("name").or(Some(id))?;

    let display_value_arr = obj
        .get("@displayValue")
        .or_else(|| obj.get("displayValue"))?
        .as_array()?;

    let mut faces = Vec::new();
    let mut vertices = Vec::new();
    for display_value in display_value_arr {
        let obj_type = display_value.get("speckle_type")?.as_str()?;

        let found = match obj_type {
            "reference" => {
                let reference_id = display_value.get("referencedId")?.as_str()?;

                let Some(mesh_obj) = object_map.get(&reference_id.to_string()) else {
                    println!(
                        "ERROR: Could not find object {} in object_map",
                        reference_id
                    );
                    return None;
                };
                Some((mesh_obj, reference_id))
            }
            "Objects.Geometry.Mesh" => {
                let id = display_value.get("id")?.as_str()?;
                Some((display_value, id))
            }
            _ => None,
        };

        let Some((mesh_obj, mesh_id)) = found else {
            println!("ERROR: Display value type not supported: {}", obj_type);
            continue;
        };

        let referenced_type = mesh_obj.get("speckle_type")?.as_str()?;
        if referenced_type != "Objects.Geometry.Mesh" {
            continue;
        }

        let Some(mut face_vec) = (match obj_type {
            "reference" => extract_ref_data(object_map, mesh_obj, "faces"),
            "Objects.Geometry.Mesh" => extract_data(mesh_obj, "faces"),
            _ => None,
        }) else {
            println!("ERROR: Could not extract faces from object: {}", mesh_id);
            continue;
        };

        faces.append(&mut face_vec);

        let Some(mut vertices_vec) = (match obj_type {
            "reference" => extract_ref_data(object_map, mesh_obj, "vertices"),
            "Objects.Geometry.Mesh" => extract_data(mesh_obj, "vertices"),
            _ => None,
        }) else {
            println!("ERROR: Could not extract vertices from object: {}", mesh_id);
            continue;
        };
        vertices.append(&mut vertices_vec);
    }
    if faces.is_empty() {
        println!("WARN: object id {} has no faces.", id);
        return None;
    }

    if vertices.is_empty() {
        println!("WARN: object id {} has no vertices.", id);
        return None;
    }

    let some_data = match get_transform_matrix(obj) {
        Some(data) => data,
        None => Transform::identity().matrix,
    };

    Some(SpeckleBase {
        speckle_type: speckle_type.to_string(),
        id: String::from(id.as_str().unwrap()),
        name: name.to_string(),
        display_value: Mesh {
            vertices: DataChunk { data: vertices },
            faces: DataChunk { data: faces },
        },
        transform: Transform { matrix: some_data },
    })
}

fn get_transform_matrix(obj: &Value) -> Option<Vec<f64>> {
    let properties = obj.get("properties")?;
    let t = properties.get("transform")?;

    let matrix = t.get("matrix")?.as_array()?;
    Some(matrix.iter().map(|v| v.as_f64().unwrap_or(1.0)).collect())
}

fn extract_data(mesh_obj: &Value, arg: &str) -> Option<Vec<f64>> {
    let ref_array = mesh_obj.get(arg)?.as_array()?;
    let arr: Vec<f64> = ref_array.iter().filter_map(|v| v.as_f64()).collect();

    Some(arr)
}

fn extract_ref_data(
    object_map: &HashMap<String, Value>,
    mesh_obj: &Value,
    arg: &str,
) -> Option<Vec<f64>> {
    let ref_array = mesh_obj.get(arg)?.as_array()?;
    let mut result = Vec::new();
    for ref_obj in ref_array {
        let reference_id = ref_obj.get("referencedId")?.as_str()?;

        let Some(reference_obj) = object_map.get(&reference_id.to_string()) else {
            println!(
                "ERROR: Unable to find reference {} with id {} in object map.",
                arg, reference_id
            );
            return None;
        };

        let reference_data = reference_obj.get("data")?.as_array()?;

        let mut arr: Vec<f64> = reference_data.iter().filter_map(|v| v.as_f64()).collect();
        result.append(&mut arr);
    }
    Some(result)
}
