//! Decoder unit tests against canonical M68000 encodings (no module data).
//! Rendering follows the ALN/MadMac conventions of the T2K module listing.

use emu68000::cpu::decode;
use emu68000::diss::{Diss, SliceMem};

fn diss_words(addr: usize, words: &[u16]) -> (String, usize) {
    let bytes: Vec<u8> = words
        .iter()
        .flat_map(|w| [(w >> 8) as u8, *w as u8])
        .collect();
    let mut mem = SliceMem::new(addr, &bytes);
    let d = decode(&mut mem, addr).unwrap();
    (Diss::new().render_line(&d), d.size)
}

#[test]
fn plain_instructions() {
    assert_eq!(diss_words(0x1000, &[0x4E75]), ("rts".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x4E71]), ("nop".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x4E73]), ("rte".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x4E76]), ("trapv".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x4E70]), ("reset".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x4AFC]), ("illegal".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x4E40]), ("trap #0".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x4E4F]), ("trap #15".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x4E72, 0x2000]), ("stop #$2000".into(), 4));
}

#[test]
fn move_and_ea_forms() {
    // move.b D0,D1 / move.w (A0)+,D2 / move.l #$12345678,D0
    assert_eq!(diss_words(0x1000, &[0x1200]), ("move.b D0,D1".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x3418]), ("move.w (A0)+,D2".into(), 2));
    assert_eq!(
        diss_words(0x1000, &[0x203C, 0x1234, 0x5678]),
        ("move.l #$12345678,D0".into(), 6)
    );
    // movea.w $1234(PC),A5 ; move.l -(A7),A0
    assert_eq!(
        diss_words(0x1000, &[0x3A7A, 0x1234]),
        ("movea.w $1234(PC),A5".into(), 4)
    );
    assert_eq!(diss_words(0x1000, &[0x205F]), ("movea.l (A7)+,A0".into(), 2));
    // move.l D7,-(A7)
    assert_eq!(diss_words(0x1000, &[0x2F07]), ("move.l D7,-(A7)".into(), 2));
}

#[test]
fn branches() {
    // bra.s with disp8 in the opcode word
    assert_eq!(diss_words(0x1000, &[0x6012]), ("bra $1014".into(), 2));
    // bra.w: disp8 = 0, 16-bit displacement follows
    assert_eq!(diss_words(0x1000, &[0x6000, 0x1234]), ("bra.w $2236".into(), 4));
    // bsr.w negative
    assert_eq!(diss_words(0x1000, &[0x6100, 0xFFFC]), ("bsr.w $0FFE".into(), 4));
    // bne.s
    assert_eq!(diss_words(0x1000, &[0x6604]), ("bne $1006".into(), 2));
    // dbra: displacement relative to the displacement word
    assert_eq!(
        diss_words(0x1000, &[0x51C9, 0xFFFC]),
        ("dbra D1, $0FFE".into(), 4)
    );
    // dbeq
    assert_eq!(
        diss_words(0x1000, &[0x57C9, 0x0004]),
        ("dbeq D1, $1006".into(), 4)
    );
}

#[test]
fn shifts_and_quick() {
    // asr.b #1,D0 ; asl.w #2,D0 ; lsr.l D1,D2 ; roxl.b #8,D7
    assert_eq!(diss_words(0x1000, &[0xE200]), ("asr.b #1,D0".into(), 2));
    assert_eq!(diss_words(0x1000, &[0xE548]), ("lsl.w #2,D0".into(), 2));
    assert_eq!(diss_words(0x1000, &[0xE2A9]), ("lsr.l D1,D1".into(), 2));
    assert_eq!(diss_words(0x1000, &[0xE117]), ("roxl.b #8,D7".into(), 2));
    // asr.w (A0) ; rol.w d16(A5)
    assert_eq!(diss_words(0x1000, &[0xE0D0]), ("asr.w (A0)".into(), 2));
    assert_eq!(
        diss_words(0x1000, &[0xE7ED, 0x0004]),
        ("rol.w $4(A5)".into(), 4)
    );
    // addq/subq
    assert_eq!(diss_words(0x1000, &[0x5A40]), ("addq.w #5,D0".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x5080]), ("addq.l #8,D0".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x5140]), ("subq.w #8,D0".into(), 2));
    // moveq sign-extension
    assert_eq!(diss_words(0x1000, &[0x7001]), ("moveq #1,D0".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x70FF]), ("moveq #-1,D0".into(), 2));
}

#[test]
fn movem_link_misc() {
    // movem.l D0-D7/A0-A6,-(A7): predecrement mask is bit-reversed
    assert_eq!(
        diss_words(0x1000, &[0x48E7, 0x7FFF]),
        ("movem.l D1-D7/A0-A7,-(A7)".into(), 4)
    );
    assert_eq!(
        diss_words(0x1000, &[0x4CDF, 0x7FFF]),
        ("movem.l (A7)+,D0-D7/A0-A6".into(), 4)
    );
    // movem.w D1-D3/A0,(A5)
    assert_eq!(
        diss_words(0x1000, &[0x48AD, 0x010E, 0x0004]),
        ("movem.w D1-D3/A0,$4(A5)".into(), 6)
    );
    // link/unlk
    assert_eq!(
        diss_words(0x1000, &[0x4E50, 0xFFFE]),
        ("link A0,#$FFFE".into(), 4)
    );
    assert_eq!(diss_words(0x1000, &[0x4E58]), ("unlk A0".into(), 2));
    // swap/ext
    assert_eq!(diss_words(0x1000, &[0x4840]), ("swap.w D0".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x4880]), ("ext.w D0".into(), 2));
    // jsr/jmp
    assert_eq!(diss_words(0x1000, &[0x4E90]), ("jsr (A0)".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x4ED0]), ("jmp (A0)".into(), 2));
    // lea
    assert_eq!(
        diss_words(0x1000, &[0x43FA, 0x1234]),
        ("lea.l $1234(PC),A1".into(), 4)
    );
}

#[test]
fn condition_forms() {
    // st/sf and the scc family
    assert_eq!(diss_words(0x1000, &[0x50C0]), ("st D0".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x51C0]), ("sf D0".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x56C0]), ("sne D0".into(), 2));
    // dbt / dbra
    assert_eq!(diss_words(0x1000, &[0x50C8, 0x0002]), ("dbt D0, $1004".into(), 4));
    assert_eq!(diss_words(0x1000, &[0x51C8, 0x0002]), ("dbra D0, $1004".into(), 4));
    // dbhi
    assert_eq!(diss_words(0x1000, &[0x52C8, 0x0002]), ("dbhi D0, $1004".into(), 4));
}

#[test]
fn arithmetic_forms() {
    assert_eq!(diss_words(0x1000, &[0xD240]), ("add.w D0,D1".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x9641]), ("sub.w D1,D3".into(), 2));
    assert_eq!(diss_words(0x1000, &[0xB041]), ("cmp.w D1,D0".into(), 2));
    assert_eq!(diss_words(0x1000, &[0xC350]), ("and.w D1,(A0)".into(), 2));
    assert_eq!(diss_words(0x1000, &[0xB340]), ("eor.w D1,D0".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x80C1]), ("divu.w D1,D0".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x81C1]), ("divs.w D1,D0".into(), 2));
    assert_eq!(diss_words(0x1000, &[0xC0C1]), ("mulu.w D1,D0".into(), 2));
    assert_eq!(diss_words(0x1000, &[0xC1C1]), ("muls.w D1,D0".into(), 2));
    // addx/subx/abcd/sbcd
    assert_eq!(diss_words(0x1000, &[0xD100]), ("addx.b D0,D0".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x9101]), ("subx.b D1,D0".into(), 2));
    assert_eq!(diss_words(0x1000, &[0xC101]), ("abcd.b D1,D0".into(), 2));
    assert_eq!(diss_words(0x1000, &[0x8101]), ("sbcd.b D1,D0".into(), 2));
    // exg
    assert_eq!(diss_words(0x1000, &[0xC340]), ("exg D1,D0".into(), 2));
    assert_eq!(diss_words(0x1000, &[0xC348]), ("exg A1,A0".into(), 2));
    assert_eq!(diss_words(0x1000, &[0xC388]), ("exg D1,A0".into(), 2));
}

#[test]
fn immediate_and_bit_ops() {
    assert_eq!(
        diss_words(0x1000, &[0x0647, 0x1234]),
        ("addi.w #$1234,D7".into(), 4)
    );
    assert_eq!(
        diss_words(0x1000, &[0x0C40, 0x00FF]),
        ("cmpi.w #$00FF,D0".into(), 4)
    );
    assert_eq!(
        diss_words(0x1000, &[0x0807, 0x0005]),
        ("btst #$0005,D7".into(), 4)
    );
    assert_eq!(diss_words(0x1000, &[0x0800, 0x0040]),
        ("btst #$0040,D0".into(), 4));
    assert_eq!(diss_words(0x1000, &[0x0880, 0x0080]), ("bclr #$0080,D0".into(), 4));
    // movep
    assert_eq!(
        diss_words(0x1000, &[0x0108, 0x1234]),
        ("movep.w $1234(A0),D0".into(), 4)
    );
    assert_eq!(
        diss_words(0x1000, &[0x01C8, 0x1234]),
        ("movep.l D0,$1234(A0)".into(), 4)
    );
}

#[test]
fn unknown_words_are_dc_w() {
    assert_eq!(diss_words(0x1000, &[0xA000]), ("dc.w $A000".into(), 2));
    assert_eq!(diss_words(0x1000, &[0xF000]), ("dc.w $F000".into(), 2));
    // 68020-only opcode region: illegal on the base 68000
    assert_eq!(diss_words(0x1000, &[0x00C0]), ("dc.w $00C0".into(), 2));
}
