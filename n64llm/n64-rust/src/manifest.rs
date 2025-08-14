use crate::{weights, weights_manifest::{self, ManifestView}};
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
}

pub fn load() -> Manifest {
    let view = ManifestView::new(&weights_manifest::MODEL_MANIFEST)
        .expect("invalid weights manifest");
    let mut layers = Vec::new();
    let _ = view.for_each(|e| {
        layers.push(Layer {
            name: e.name.to_string(),
            offset: e.offset,
            size: e.size,
        });
        true
    });
    let manifest = Manifest { layers };
    if !validate(&manifest, view.align() as u32) {
        panic!("invalid weights manifest");
    }
    manifest
}

fn validate(m: &Manifest, align: u32) -> bool {
    let mut last_end = 0u32;
    for layer in &m.layers {
        if layer.offset % align != 0 {
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
