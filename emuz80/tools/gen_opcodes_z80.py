#!/usr/bin/env python3
"""Generate resources/opcodesZ80.json for the emuz80 crate.

Source: opcode-table.json by Deep Toaster (MIT licensed, see
resources/opcodesZ80.json.NOTICE) — a compact Z80 *form* table whose
opcode bytes may be symbolic expressions over register/bit/vector tokens
(e.g. `(r<<3)+$40`, `p+$C7`). This script expands those expressions into
gazm-style Instruction rows:

- `opcode`   — prefix+opcode as a single hex value, with register/bit/vector
               fields ZEROED (the base). The assembler backend ORs in the
               register bits from `bit_fields`; the base is what this table
               can know.
- `bit_fields` — per symbolic operand byte: `{var: shift}`, e.g. LD r1,r2
               is `{"r1": 3, "r2": 0}` (0x40 | r1<<3 | r2), ADD A,r is
               `{"r": 0}`, INC r is `{"r": 3}`. Vars: r, r1, r2, dd, cc,
               b, p.
- `template` — the operand shape exactly as written in the source
               ("A,(IX+d)", "r1,r2", "b,(HL)", "d", ...); the assembler
               parser canonicalizes operands to these strings for lookup.
- `size`     — total instruction bytes (prefixes + opcode + operands).
- `operand_size` — bytes that follow the opcode (n=1, nn=2, d=1).
- `addr_mode`— coarse form-level mode (see AddrModeEnum in src/isa).
- `cycles`   — first number of the source's "a/b" strings.

Undocumented silicon quirks are dropped when a documented encoding exists
for the same (action, template): the BIT b,(IX+d)/(IY+d) rows whose base
is not the standard 01nnn110 pattern ($46 family).
"""

import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
SRC = HERE / "opcode-table.json"
OUT = HERE.parent / "resources" / "opcodesZ80.json"

VARS = ("r", "r1", "r2", "dd", "cc", "b", "p")

SYMBOLIC = re.compile(r"^(\((\w+)(?:<<\d+)?\)?\+?)+|^(\w+)\+(?:r\+)?\$([0-9A-F]{2})$|^(\w+)$")
BASE_EXPR = re.compile(r"\$([0-9A-F]{2})")  # the constant part of a symbolic byte expr


def base_of(expr: str) -> int:
    """Numeric base of a symbolic opcode byte ('r+$88' -> 0x88, 'r' -> 0x00)."""
    if expr == "r":
        return 0x00
    m = BASE_EXPR.search(expr)
    if not m:
        raise ValueError(f"no base in symbolic expr {expr!r}")
    return int(m.group(1), 16)


def is_symbolic(byte: str) -> bool:
    return not re.fullmatch(r"[0-9A-F]{2}", byte)


def bit_fields(expr: str) -> dict:
    """{var: shift} for a symbolic opcode byte, e.g. '(r<<3)+$40' -> {'r': 3}."""
    fields = {}
    for var, shift in re.findall(r"\((\w+)<<(\d+)\)", expr):
        fields[var] = int(shift)
    for var in re.findall(r"[a-z0-9]+", expr):
        if var in VARS and var not in fields:
            # Bare operands sit at bit 0 ('r+$88' -> r, 'r' -> r), except the
            # RST vector: 'p+$C7' is shorthand for p<<3 | 0xC7.
            fields[var] = 3 if var == "p" else 0
    return fields


CONDITIONS = ("C", "NC", "NZ", "Z", "M", "P", "PE", "PO")
PAIRS = ("BC", "DE", "HL", "IX", "IY", "SP", "AF")


def classify_mode(operand: str) -> str:
    if operand == "":
        return "Inherent"
    if operand in ("0", "1", "2"):
        return "ImmediateMode"
    if operand == "p":
        return "Restart"
    if operand == "d":
        return "Relative"
    if operand in ("n", "A,n", "r,n"):
        return "Immediate8"
    if operand in ("nn", "dd,nn", "IX,nn", "IY,nn"):
        return "Immediate16"
    if operand == "r1,r2":
        return "RegisterRegister"
    if operand in CONDITIONS:
        return "Condition"
    if operand.endswith(",d") and operand.partition(",")[0] in CONDITIONS:
        return "ConditionRelative"
    if operand.endswith(",nn") and operand.partition(",")[0] in CONDITIONS:
        return "ConditionImmediate"
    if operand in ("r", "A,r"):
        return "Register"
    if operand in ("dd",) or operand in PAIRS or operand in ("HL,dd", "IX,dd", "IY,dd", "DE,HL"):
        return "RegisterPair"
    if operand in ("(HL)", "A,(HL)", "r,(HL)", "(HL),r", "(HL),n"):
        return "Indirect"
    if operand in ("(IX)", "(IY)", "(IX+d)", "A,(IX+d)", "r,(IX+d)", "(IX+d),r",
                   "(IX+d),n", "(IY+d)", "A,(IY+d)", "r,(IY+d)", "(IY+d),r", "(IY+d),n"):
        return "Indexed"
    if operand == "b,(HL)":
        return "BitIndirect"
    if operand in ("b,(IX+d)", "b,(IY+d)", "b,(IX+d),r", "b,(IY+d),r"):
        return "BitIndexed"
    if operand in ("A,(nn)", "(nn),A"):
        return "AbsoluteIndirect"
    if operand in ("HL,(nn)", "IX,(nn)", "IY,(nn)", "BC,(nn)", "DE,(nn)", "SP,(nn)",
                   "(nn),HL", "(nn),IX", "(nn),IY", "(nn),BC", "(nn),DE", "(nn),SP"):
        return "AbsoluteIndirect16"
    if operand in ("A,(BC)", "A,(DE)", "(BC),A", "(DE),A"):
        return "RegisterIndirect"
    if operand in ("SP,HL", "SP,IX", "SP,IY"):
        return "StackRegister"
    if operand in ("(SP),HL", "(SP),IX", "(SP),IY"):
        return "StackIndirect"
    if operand in ("A,I", "A,R", "I,A", "R,A"):
        return "SpecialRegister"
    if operand == "AF,AF'":
        return "ExchangeAF"
    if operand in ("A,(n)", "(n),A", "(n),r", "r,(n)"):
        return "Port"
    if operand in ("r,(C)", "(C)", "(C),0", "(C),r"):
        return "PortRegister"
    if operand == "b,r":
        return "BitRegister"
    raise ValueError(f"unclassified operand {operand!r}")


def operand_size_of(byte: str) -> int:
    if byte in ("n", "d", "d-$-2"):
        return 1
    if byte == "nn":
        return 2
    return 0


def main() -> None:
    with open(SRC) as f:
        entries = json.load(f)
    # Keep rows keyed by (action, template) for dedup decisions.
    rows = {}
    order = []
    for e in entries:
        mnemonic, _, operand = e["mnemonic"].partition(" ")
        action = mnemonic
        template = operand

        opcode_bytes = []
        operand_size = 0
        fields = {}
        operand_offset = len(e["bytes"])
        for idx, b in enumerate(e["bytes"]):
            if b in ("n", "nn", "d", "d-$-2"):
                operand_size += operand_size_of(b)
                if operand_offset == len(e["bytes"]):
                    operand_offset = idx  # first operand byte's position
            else:
                opcode_bytes.append(b)
                if is_symbolic(b):
                    fields.update(bit_fields(b))

        # opcode = prefix+opcode as one number, register/bit fields zeroed.
        opcode = 0
        for b in opcode_bytes:
            opcode <<= 8
            opcode |= base_of(b) if is_symbolic(b) else int(b, 16)

        size = len(opcode_bytes) + operand_size
        cycles = int(e["cycles"].split("/")[0])
        addr_mode = classify_mode(template)

        rows.setdefault((action, template), []).append({
            "addr_mode": addr_mode,
            "cycles": cycles,
            "opcode": f"{opcode:04X}",
            "action": action,
            "size": size,
            "operand_size": operand_size,
            "operand_offset": operand_offset,
            "template": template,
            "bit_fields": fields,
        })
        if (action, template) not in order:
            order.append((action, template))

    # Dedup: for BIT b,(IX+d)/(IY+d), keep the documented 01nnn110 base
    # (low bits 110) and drop the undocumented $40-$47 silicon quirks.
    instructions = []
    for key in order:
        cands = rows[key]
        if len(cands) > 1 and key[0] == "BIT" and key[1] in ("b,(IX+d)", "b,(IY+d)"):
            documented = [c for c in cands if (int(c["opcode"], 16) & 0x07) == 0x06]
            if documented:
                cands = documented
        instructions.extend(cands)

    instructions.sort(key=lambda c: (c["action"], c["template"], c["opcode"]))
    out = {
        "unknown": {
            "addr_mode": "Inherent",
            "cycles": 1,
            "opcode": "00",
            "action": "unknown",
            "size": 1,
        },
        "instructions": instructions,
    }
    with open(OUT, "w") as f:
        json.dump(out, f, indent=1)
    print(f"wrote {OUT}: {len(instructions)} rows, {len(order)} (action, template) forms")


if __name__ == "__main__":
    main()
