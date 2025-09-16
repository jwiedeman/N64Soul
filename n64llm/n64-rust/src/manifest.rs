use crate::io::rom_reader::FlatRomReader;
use crate::model::{dims::ModelDims, meta::load_dims_from_meta};
use crate::{weights, weights_manifest};
use alloc::string::ToString;
use alloc::{string::String, vec::Vec};

#[derive(Debug, Clone)]
pub struct Layer {
    pub name: String,
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub layers: Vec<Layer>,
    pub align: u32,
    pub dims: ModelDims,
}

pub fn load() -> Manifest {
    let view = weights_manifest::manifest().expect("invalid weights manifest");
    let mut layers = Vec::new();
    let _ = view.for_each(|e| {
        layers.push(Layer {
            name: e.name.to_string(),
            offset: e.offset,
            size: e.size,
        });
        true
    });
    let mut rr = FlatRomReader::new();
    let dims = load_dims_from_meta(&mut rr, &view).unwrap_or_else(ModelDims::fallback);

    let manifest = Manifest {
        layers,
        align: view.align() as u32,
        dims,
    };
    if !validate(&manifest) {
        panic!("invalid weights manifest");
    }
    manifest
}

fn validate(m: &Manifest) -> bool {
    let mut last_end = 0u32;
    for layer in &m.layers {
        if layer.offset % m.align != 0 {
            return false;
        }
        if layer.offset < last_end {
            return false;
        }
        let end = layer.offset + layer.size;
        if end as u64 > weights::weights_rom_size() {
            return false;
        }
        last_end = end;
    }
    true
}

impl Manifest {
    pub fn find(&self, name: &str) -> Option<&Layer> {
        self.layers.iter().find(|l| l.name == name)
    }
}
