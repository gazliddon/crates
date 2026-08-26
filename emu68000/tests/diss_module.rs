//! Validation of the decoder and disassembler against the Imagitec Tempest
//! 2000 sound module.
//!
//! TEST.TXT is the ALN-assembled 68000 module loaded at `$4000` (text
//! `$4000-$70CF`, data `$70D0-$7717`, bss `$8000-$87FF`); TEST.SYM is its
//! ALN symbol table (magic `$601B` + 6 header bytes, then 14-byte records:
//! name[8] + type[1] + 5-byte big-endian value; types 0x82/0xA2 are code
//! labels, 0x84/0xA4 data, 0xC0/0xE0 equates).
//!
//! The text region embeds several data blocks (copyright strings, offset and
//! pointer tables, and the DSP program at `$6B34`); a linear walk decodes
//! through them with arbitrary lengths, so code symbols pointing into those
//! blocks cannot be expected to land on walk boundaries. [`DATA_SPANS`]
//! documents the embedded-data blocks (hand-verified against the bytes);
//! every code symbol outside them must land exactly on an instruction
//! boundary.

use emu68000::cpu::decode;
use emu68000::diss::{Diss, DissCtx, SliceMem};

const TEXT_START: usize = 0x4000;
const TEXT_END: usize = 0x70D0; // file length = $30D0

/// Embedded-data spans inside the text region (verified by hand-decoding;
/// each contains a string or table). End points are where the linear walk
/// re-emerges, which may consume a word or two past the block itself.
const DATA_SPANS: &[(usize, usize)] = &[
    (0x45DA, 0x4606), // padding + "Copyright 1993, Imagitec Design Inc\0"
    (0x4B18, 0x4B3E), // "Copyright 1993, Imagitec Design Inc\0"
    (0x5B60, 0x5B82), // dispatch offset table (asys_ntE auto-symbols)
    (0x6AD2, 0x6B34), // haltdata/sampleno/FXVOL/MUSVOL/FXENA/MUSENA data
    (0x6F9E, 0x6FC8), // padding + "Copyright 1993, Imagitec Design, Inc\0"
];

struct Sym {
    name: String,
    typ: u8,
    val: u32,
}

fn load_syms(path: &str) -> Vec<Sym> {
    let data = std::fs::read(path).expect("read TEST.SYM");
    assert_eq!(&data[0..2], &[0x60, 0x1B], "ALN symbol magic $601B");
    assert_eq!((data.len() - 8) % 14, 0, "record alignment");
    let mut out = Vec::new();
    let mut off = 8;
    while off + 14 <= data.len() {
        let name = String::from_utf8_lossy(&data[off..off + 8])
            .trim_end_matches('\0')
            .to_string();
        let typ = data[off + 8];
        let b = &data[off + 9..off + 14];
        let val = ((b[0] as u64) << 32
            | (b[1] as u64) << 24
            | (b[2] as u64) << 16
            | (b[3] as u64) << 8
            | b[4] as u64) as u32;
        out.push(Sym { name, typ, val });
        off += 14;
    }
    out
}

fn load_module(path: &str) -> Vec<u8> {
    let data = std::fs::read(path).expect("read TEST.TXT");
    assert_eq!(data.len(), 0x30D0, "module is 12496 bytes");
    data
}

/// Walk the module linearly, stopping when a fetch runs past the image (the
/// module ends with a padding word at $70CE that needs an immediate beyond
/// the file). Returns (instruction starts, listing, stop pc).
fn walk(module: &[u8]) -> (Vec<usize>, Vec<(usize, String, usize)>, usize) {
    let mut mem = SliceMem::new(TEXT_START, module);
    let mut pc = TEXT_START;
    let mut starts: Vec<usize> = Vec::new();
    let mut listing: Vec<(usize, String, usize)> = Vec::new();
    while pc < TEXT_END {
        match decode(&mut mem, pc) {
            Ok(d) => {
                starts.push(pc);
                listing.push((pc, Diss::new().render_line(&d), d.size));
                pc += d.size;
            }
            Err(_) => break, // fetch ran past the image (padding word)
        }
    }
    (starts, listing, pc)
}

fn in_data_span(a: usize) -> bool {
    DATA_SPANS.iter().any(|&(lo, hi)| a >= lo && a < hi)
}

#[test]
fn sym_file_parses() {
    let syms = load_syms("tests/data/TEST.SYM");
    // known anchors from the earlier hand-decoding session
    let get = |n: &str| syms.iter().find(|s| s.name == n).map(|s| s.val);
    assert_eq!(get("INIT_SOU"), Some(0x4E1A)); // INIT_SOUND (8-char truncation)
    assert_eq!(get("PLAYFX2"), Some(0x4FD0));
    assert_eq!(get("GPUSTART"), Some(0x6B34));
    assert_eq!(get("intmask"), Some(0x715C));
    assert_eq!(get("INITDSP"), Some(0x4D46));
}

#[test]
fn module_disassembles_with_labels() {
    let module = load_module("tests/data/TEST.TXT");
    let syms = load_syms("tests/data/TEST.SYM");
    let (starts, listing, stop) = walk(&module);

    // the module ends with `rts` at $70CC followed by a padding word at
    // $70CE; the walk must have decoded everything except that word
    assert_eq!(stop, 0x70CE, "walk must reach the trailing padding word");

    // every code symbol must land on an instruction boundary, except code
    // symbols inside the embedded-data spans
    let mut failures: Vec<(u32, String)> = Vec::new();
    for s in &syms {
        if !matches!(s.typ, 0x82 | 0xA2) {
            continue;
        }
        if !(TEXT_START as u32..TEXT_END as u32).contains(&s.val) {
            continue; // e.g. stack = $8800
        }
        if !starts.contains(&(s.val as usize)) && !in_data_span(s.val as usize) {
            failures.push((s.val, s.name.clone()));
        }
    }
    assert!(
        failures.is_empty(),
        "code symbols not on instruction boundaries: {failures:?}"
    );

    // the spans must explain exactly the misses: every span contains at
    // least one non-boundary code symbol
    let missed: Vec<u32> = syms
        .iter()
        .filter(|s| matches!(s.typ, 0x82 | 0xA2))
        .filter(|s| (TEXT_START as u32..TEXT_END as u32).contains(&s.val))
        .filter(|s| !starts.contains(&(s.val as usize)))
        .map(|s| s.val)
        .collect();
    for &(lo, hi) in DATA_SPANS {
        assert!(
            missed
                .iter()
                .any(|&m| (m as usize) >= lo && (m as usize) < hi),
            "data span {lo:04X}..{hi:04X} explains no missed symbol"
        );
    }

    // golden instructions (hand-verified from the module)
    let at = |a: usize| {
        listing
            .iter()
            .find(|(pc, _, _)| *pc == a)
            .map(|(_, t, _)| t.clone())
    };
    assert_eq!(
        at(0x4D46).unwrap(), // INITDSP
        "movem.l D0-D7/A0-A6,-(A7)"
    );
    assert_eq!(at(0x4D4A).unwrap(), "move.l #$00000000,$00F1A114.l");
    assert_eq!(
        at(0x4E1A).unwrap(), // INIT_SOUND
        "movem.l D0-D7/A0-A6,-(A7)"
    );
    assert_eq!(
        at(0x4FD0).unwrap(), // PLAYFX2
        "movem.l D1-D7/A2-A6,-(A7)"
    );

    // the module entry: lea.l $00008800,A7 (set up the stack)
    assert_eq!(at(0x4000).unwrap(), "lea.l $00008800.l,A7");

    // INITDSP ends with the rts we hand-decoded earlier; find it nearby
    let initdsp = listing
        .iter()
        .position(|(pc, _, _)| *pc == 0x4D46)
        .expect("INITDSP present");
    let tail: Vec<&str> = listing[initdsp..]
        .iter()
        .filter(|(pc, _, _)| *pc >= 0x4D46 && *pc <= 0x4DD4)
        .map(|(_, t, _)| t.as_str())
        .collect();
    assert!(tail.iter().any(|t| t == &"rts"), "INITDSP ends with rts");

    // the DSP blob is embedded at $6B34 (byte-identical to dsp.bin); the
    // walk happens to stay aligned through it and lands on the resume point
    assert!(starts.contains(&0x6B34), "GPUSTART ($6B34) on a boundary");
    assert!(starts.contains(&0x6E98), "walk re-emerges at $6E98");
    assert!(starts.contains(&0x7092), ".InitBli at $7092 on a boundary");
}

#[test]
fn label_rendering_resolves_targets() {
    let module = load_module("tests/data/TEST.TXT");
    let syms = load_syms("tests/data/TEST.SYM");
    let labels: Vec<(u32, String)> = syms
        .iter()
        .filter(|s| matches!(s.typ, 0x82 | 0xA2 | 0x84 | 0xA4))
        .map(|s| (s.val, s.name.clone()))
        .collect();
    let diss = Diss::with_labels(labels);
    let mut mem = SliceMem::new(TEXT_START, &module);

    // every branch/PC-relative target that matches a symbol must render with
    // the symbol's name
    let mut resolved = 0;
    let mut pc = TEXT_START;
    while pc < TEXT_END {
        match decode(&mut mem, pc) {
            Ok(d) => {
                let line = diss.render_line(&d);
                if let Some(t) = d.label {
                    if let Some(name) = diss.label_at(t) {
                        resolved += 1;
                        assert!(line.contains(name), "line shows label: {line}");
                    }
                }
                pc += d.size;
            }
            Err(_) => break, // trailing padding word
        }
    }
    // the module is full of bsr/jmp to exported routines; require a handful
    assert!(
        resolved >= 5,
        "expected label-resolved branches, got {resolved}"
    );
}

#[test]
fn ctx_helpers_work() {
    let module = load_module("tests/data/TEST.TXT");
    let ctx = DissCtx::from_slice(TEXT_START, "TEST", &module);
    let d = Diss::new();
    let dis = ctx.diss(&d, 0x4000).unwrap();
    assert_eq!(dis.size, 6);
    assert_eq!(dis.text, "lea.l $00008800.l,A7");
}
