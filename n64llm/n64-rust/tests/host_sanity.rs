#![cfg(feature = "host")]

use n64_gpt::weights_manifest::manifest;
use n64_gpt::stream::streamer::stream_entry;
use n64_gpt::io::rom_reader::RomReader;

#[test]
fn manifest_crc_alignment_math_is_stable() {
    let man = manifest().expect("manifest");
    assert_eq!(man.align(), 64);
    assert_eq!(man.count(), 2);

    let mut entries = Vec::new();
    man.for_each(|e| { entries.push(e); true }).unwrap();

    assert_eq!(entries[0].name, "tok");
    assert_eq!(entries[0].offset, 64);
    assert_eq!(entries[0].size, 16);
    assert_eq!(entries[0].crc32, Some(0x4D6F28D3));

    assert_eq!(entries[1].name, "ffn");
    assert_eq!(entries[1].offset, 128);
    assert_eq!(entries[1].size, 4);
    assert_eq!(entries[1].crc32, Some(0xD202EF8D));
}

struct DummyRom;

impl RomReader for DummyRom {
    fn read(&mut self, _off: u64, _dst: &mut [u8]) -> bool { true }
}

#[test]
fn cart_streamer_handles_empty_segments() {
    let mut rr = DummyRom;
    let stats = stream_entry(&mut rr, 0, 0, |_| {});
    let s = stats.expect("stats");
    assert_eq!(s.bytes, 0);
    assert_eq!(s.bursts, 0);
}

