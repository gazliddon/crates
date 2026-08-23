# Emulator Spec

## MULTI-CRATE FILE SYSTEM PATHS

- Workspace Context: This project consists of multiple independent
  repositories/folders linked via relative file paths inside `Cargo.toml`.
- Crate Locations (Relative to the Harness Root):
    - Main App: `./stargate-emu/`
    - Core Traits: `../emucore/`
    - 6800 Engine: `../emu6800/`
    - 6809 Engine: `../emu6809/`
- Current Focus: We are ignoring the 6800 core for now. All effort is on the
  6809 engine (and `emucore` traits it needs); the 6800 crate may be revisited
  later.
- Authorization: You have full permission to traverse into any of these
  subfolders to edit files, refactor shared traits, update `Cargo.toml`
  manifests, and fix bugs.

## ARCHITECTURAL DOMAIN KNOWLEDGE

You are acting as a veteran hardware engineer specializing in late-1970s and
1980s Motorola 8-bit computer architectures. We are focusing specifically on
implementing:

1. Motorola 6809: Highly orthogonal, advanced 8-bit/16-bit hybrid. Features
   Accumulators A and B (concatenating into 16-bit Register D), Index Registers
   X and Y, Stack Pointers U (User) and S (System), and Direct Page Register
   (DP) for fast 8-bit memory addressing modes.
2. Motorola 6808: Object-code compatible with the 6800. Simple, classic 8-bit
   architecture with Accumulators A and B, Index Register X, and a Condition
   Code Register (H, I, N, Z, V, C). **Not the current focus — ignored for
   now.**

## COMPONENT INTEGRATION SCOPE

We are simulating complete systems built around these chips. This includes
modeling how the CPU interacts over the system buses with common companion
hardware components:

- Address & Data Bus line decoding (Direct Page vs Extended addressing).
- Memory Controllers & Address Decoding logic (mapping RAM, ROM, and I/O
  windows).
- Peripheral Interface Adapters (PIA like the MC6821) and Asynchronous
  Communications Interface Adapters (ACIA like the MC6850).
- System timing cycles (E and Q clock signals for the 6809).

## EMULATION REQUIREMENTS

- Exact Cycle-Counting: Every instruction must consume the exact number of clock
  cycles specified in the Motorola datasheets, accounting for variations based
  on the addressing mode used (Immediate, Direct, Indexed, Extended, Inherent,
  Relative).
- Flag Correctness: The Condition Code Register (CCR) flags must be updated with
  mathematical precision following logical and arithmetic operations (especially
  Half-Carry 'H' for BCD operations, Signed Overflow 'V', and Negative 'N').
- Inter-Emulator Differential Validation: We will ultimately validate our Rust
  implementation by comparing register state dumps cycle-by-cycle against
  reference outputs from gold-standard emulators like MAME.
- MAME Source Code as Reference: Use the MAME source code itself as a
  reference for machine architecture correctness — both the CPU cores (e.g.
  `src/devices/cpu/m6809/`) and the support chips (e.g. PIA `6821pia`, ACIA
  `6850acia`) as well as the machine drivers that wire them together. When in
  doubt about how the hardware behaves or how the system is wired, consult the
  MAME sources before writing or changing emulation logic.

## TEST SCRIPT DISCOVERY & DIFFERENTIAL TESTING

- Verification Priority: Existing test scripts that compare this emulator
  against MAME already live somewhere within the subdirectories.
- Discovery Mandate: Before writing new code, your first action must be to
  comprehensively explore the directory structure, identify these scripts (e.g.,
  Python, Bash, or Rust integration files), and read their implementation to
  understand how the MAME differential engine works.
- Execution Loop:
    1. Locate the existing MAME trace/comparison scripts.
    2. Run the scripts using your terminal tools.
    3. Analyze any failure points, register mismatches, or flag discrepancies
       reported between our logic and MAME.
    4. Edit the relevant local crate (`emu6800`, `emu6809`, or `emuCore`) to
       resolve the errors.

## LIVING DOCUMENTATION MANDATE

- Log Book Requirement: You must maintain a running progress log. Create or
  update a file named `docs/differential_testing_log.md`.
- Documentation Scope: Every time you run a comparison test or modify execution
  logic, you must log:
    1. Which component or instruction matrix was evaluated.
    2. The exact error or state mismatch found (e.g., "6809 Accumulator B
       mismatch at cycle 42001").
    3. The specific file, line, and architectural logic you edited to fix it.
    4. The verification status (Pass/Fail) after your changes.
- Persistence: Never clear or erase old log entries; append your progress
  chronologically so I can easily audit your changes via Git diffs.
- Put documentation in the crate docs directory it should be in. The project separates
  concerns and the goal is for the dependent crates to be separate from this
  emulation harness.
- All documentation should be Obsidian compatible Markdown. Look at my vim setup
  for what the rules I have in place for that is and then enumerate those rules
  here

### OBSIDIAN-COMPATIBLE MARKDOWN RULES (from `~/.config/nvim`)

These rules come from `lua/plugins/markdownlint.json`,
`lua/plugins/prettier.json`, and `lua/plugins/none-ls.lua`.

- **Line length**: wrap prose at 80 columns (`MD013`). Code blocks may run
  to 120 columns. Tables are exempt from line length.
- **List indentation**: use 4 spaces per level, starting at 4 spaces
  (`MD007` indent 4, start_indent 4). No tabs.
- **Formatting**: Prettier with `--print-width 80 --prose-wrap always`,
  `tabWidth 4` for Markdown, `useTabs false`.
- **Relaxed lint rules (explicitly disabled)**: multiple consecutive blank
  lines (`MD012`), first-heading-level requirement (`MD002`), bare URLs
  (`MD034`), and multiple top-level headings (`MD025`) are allowed.
- **Obsidian conventions** (from `obsidian.nvim`, vault `~/Documents/gazstuff`):
    - Wiki links `[[Note Name]]`, embedded files `![[image.png]]`, tags
      `#tag`.
    - Task checkboxes `- [ ]` and `- [x]` (checked boxes are the only
      completion state).
    - Images/attachments live in an `assets/` folder.
    - YAML frontmatter for note properties is fine.
- **Tooling**: `markdownlint` (with the config above) and Prettier are wired
  up as null-ls sources for Markdown, so docs should pass both cleanly.
- **British English**: docs use British spelling — `colour`, not `color`;
  `programme` only in non-computing senses. Hardware identifiers
  (`color_registers`, `Center Coin`) keep their schematic spelling.
- **Spell checking**: harper-ls checks docs. Technical terms (register
  names, game names, domain jargon) belong in the per-project workspace
  dictionary at the repo root — `.harper-dictionary.txt` (harper roots at
  the nearest `.git`, `gazm.toml`, or `.obsidian` vault marker).
  Personal/global words go in the user
  dictionary (`~/Library/Application Support/harper-ls/dictionary.txt`).
  Add words with the `HarperAddToWSDict` code action instead of editing
  prose around them.
