use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct Layer {
    pub name: String,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub layers: Vec<Layer>,
}

pub fn load() -> Manifest {
    let json = include_str!("../../assets/weights.manifest.json");
    parse(json)
}

fn parse(json: &str) -> Manifest {
    let mut layers = Vec::new();
    if let Some(start) = json.find('[') {
        let mut rest = &json[start + 1..];
        while let Some(obj_start) = rest.find('{') {
            rest = &rest[obj_start + 1..];
            if let Some(obj_end) = rest.find('}') {
                let obj = &rest[..obj_end];
                rest = &rest[obj_end + 1..];
                let mut name = String::new();
                let mut offset = 0u32;
                let mut size = 0u32;
                for field in obj.split(',') {
                    let mut parts = field.splitn(2, ':');
                    let key = parts
                        .next()
                        .unwrap_or("")
                        .trim()
                        .trim_matches(|c| c == '"');
                    let value = parts.next().unwrap_or("").trim();
                    match key {
                        "name" => {
                            name = value.trim_matches('"').to_string();
                        }
                        "offset" => {
                            offset = value.parse().unwrap_or(0);
                        }
                        "size" => {
                            size = value.parse().unwrap_or(0);
                        }
                        _ => {}
                    }
                }
                if !name.is_empty() {
                    layers.push(Layer { name, offset, size });
                }
            } else {
                break;
            }
        }
    }
    Manifest { layers }
}
