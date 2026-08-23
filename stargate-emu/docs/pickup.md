# Pickup — 2026-08-23

Session handoff: emulator docs, gazm plugin + nvim config cleanup, and the
roadmap for gazm releases. Read this to resume where we left off.

## What was done

### Emulator docs (`docs/` in this repo)

- `docs/emulation_spec.md` — added "6800 ignored for now", MAME source as
  correctness reference, and the enumerated Obsidian markdown rules.
- `docs/williams_hw/` — restructured hardware docs: `README.md` (index +
  "where things live"), `base.md` (shared Williams platform), `defender.md`,
  `stargate.md`, `robotron.md`, and `blitter.md` (a software-developer
  manual for the blitter).
- Old `docs/stargate_hw.md` removed (content moved into `williams_hw/`).

### gazm repo (`~/development/gazm`, github.com/gazliddon/gazm)

- `18edbfe` — `gazm-plugin` treesitter hook supports both nvim-treesitter
  master and main
- `93b2efe` — plugin cleanup: deleted dead `text.lua`/`test.lua`,
  `.gazm`-only filetype via `vim.filetype.add`, plenary removed, README +
  .gitignore added

### dotfiles repo (`~/dotfiles`, github.com/gazliddon/dotfiles)

- `bf42a9f` — gazm LSP config in `plugins/gazm.lua`; `main = 'gazm'`;
  dropped `fmt` + plenary
- `8dd11fd` — nvim core + gaz framework updates
- `50168e4` — plugin config updates; removed `nvim-treesitter-text_objects`;
  added `conjure.lua`, `mini-ai.lua`, `prettier.json`
- `f5c3aac` — shell + ghostty tweaks
- `b75fec8` — .gitignore for tool-state/credential dirs (`.config/gh`, `op`,
  `1Password`, `weechat`, ...)

Also: `plugins/sixtyeight.lua` deleted (dead 68xx LSP experiment; lspconfig
now comes from the `lsp-config.lua` dependency). Note: `~/.config` is a
symlink to `~/dotfiles/.config` — same files.

## Next actions (in order)

### 1. ~~Migrate nvim-treesitter to the `main` branch~~ — DONE (2026-08-23)

Completed and verified: plugin on `main` (e82ef6ae), config rewritten
(dotfiles commit `7e3f9d9`), all 24 parsers in
`~/.local/share/nvim/site/parser/`, fence crash gone, gazm highlighting +
LSP working. tree-sitter-cli 0.26.12 installed via Homebrew (mason can't
provide it; the plain `tree-sitter` formula is library-only now). Details
and gotchas in `~/development/gazm/docs/AGENT_TRAIL.md`.

### 2. gazm GitHub releases (binaries for Windows/Linux/macOS)

Blocked by: `gazm/Cargo.toml` path-deps on harness crates outside the repo
(`../../crates/{emu6809,emu6800,grl-sources,grl-utils,grl-symbols,grl-eval,unraveler}`).

Decide: **A)** gazm CI checks out both `gazliddon/crates` + `gazliddon/gazm`
(path deps); **B)** publish the 7 crates to crates.io, switch gazm to
version deps, keep dev workflow via `[patch.crates-io]` at the
`~/development/crates` workspace root (recommended); **C)** vendor crates
into the gazm repo (rejected).

Then: GitHub Actions release workflow (tag-triggered), e.g.
`taiki-e/upload-rust-binary` on a matrix of `ubuntu-latest`, `windows-latest`,
`macos-latest` (arm64) + `macos-13` (x86_64). gazm has no platform-specific
deps; version is 0.9.16.

### 3. Publish gazm-plugin as its own repo + dev-mode spec

- `git subtree split -P gazm-plugin -b plugin-export` then push to a new
  repo (e.g. github.com/gazliddon/gazm.nvim). The vendored `treesitter-gazm`
  grammar travels with it.
- Rewrite `plugins/gazm.lua` to lazy dev-mode with `fallback = true`
  (local `~/development/gazm/gazm-plugin` when present, git otherwise) and
  auto-detect the LSP command (dev build path -> `gazm` on PATH).
- Document other-machine setup in the plugin README (`cargo install --path`
  for the CLI; lazy handles plugin + grammar).

## Open decisions

- crates.io vs CI dual-checkout for the gazm deps (recommend crates.io).
- Plugin repo name (`gazm.nvim`?).
- Publish `gazm` itself to crates.io for `cargo install gazm`?

## Environment notes

- DSH sandbox: `workspace-write` allows writes under the session workspace
  only (+ platform temp dirs). Workspaces are per-directory — see
  `~/.dsh/storages/workspace.json`. Created so far: `stargate-emu`,
  `gazm`. Sessions started in a workspace get that path as the sandbox
  root. A dotfiles workspace would need path `~/dotfiles` (not `~/.config`,
  which canonicalizes there anyway).
- The harness crates already form a workspace + repo
  (`~/development/crates/Cargo.toml`, github.com/gazliddon/crates).
  `docs/emulation_spec.md` still describes them as "multiple independent
  repositories" — stale, candidate for a doc fix.
- `lsp2` project (`~/development/lsp2`) is untouched; no longer wired into
  nvim.

## Small leftovers

- `queries/gazm/indents.scm` is empty — fill it and switch the `<CR>`
  indent hack in `lsp.lua` to `vim.bo.indentexpr`.
- The spec's `<CR>` format-on-Enter mapping is shadowed by the plugin's
  `<CR>` indent mapping (both buffer-local insert mappings).
- `docs/todo.md` in this repo: remaining items are "understand the crate",
  "evaluate testing tools", "launch/test instructions", "gazm symbol/map
  loading fix", "assemble stargate".
