use crate::weights_manifest::{ManifestView, Entry};

pub fn find<'a>(view: &'a ManifestView<'a>, name: &str) -> Option<Entry<'a>> {
    let mut found: Option<Entry<'a>> = None;
    let _ = view.for_each(|e| { if e.name == name { found = Some(e); false } else { true } });
    found
}

