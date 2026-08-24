//! Debugger-facing primitives shared by every CPU debugger (6809, 6800,
//! 6502, ...): the UI breakpoint set, the stepping contract a debugger
//! needs from a CPU or board harness, and the generic step/run and
//! disassembly walks built on top of them.
//!
//! CPU crates keep their own registers, decoders, and instruction
//! semantics; these types cover the policy that is common to all of
//! them.  Source-level breakpoint helpers take the *resolved* address
//! list, so this module stays independent of any source-map format.

/// What the last `step`/`run` call produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugEvent {
    /// The CPU executed and is now stopped at a new PC.
    Stepped,
    /// Execution stopped because the PC reached a breakpoint.
    BreakpointHit(u16),
    /// The emulator faulted (invalid memory access, etc.).
    Faulted,
}

/// A UI breakpoint: an address, optionally tagged with a symbol name so
/// the frontend can display it.  Disabled breakpoints stay listed and
/// rendered but never stop the machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Breakpoint {
    pub addr: u16,
    pub label: Option<String>,
    pub enabled: bool,
}

/// The breakpoint list plus the source-line helpers shared by all CPU
/// debuggers.
#[derive(Debug, Default, Clone)]
pub struct BreakpointSet {
    list: Vec<Breakpoint>,
}

impl BreakpointSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn as_slice(&self) -> &[Breakpoint] {
        &self.list
    }

    pub fn iter(&self) -> impl Iterator<Item = &Breakpoint> {
        self.list.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Breakpoint> {
        self.list.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// Add an enabled breakpoint; returns false when one already exists
    /// at `addr` (the existing entry is left untouched).
    pub fn add(&mut self, addr: u16, label: Option<String>) -> bool {
        if self.list.iter().any(|bp| bp.addr == addr) {
            return false;
        }
        self.list.push(Breakpoint {
            addr,
            label,
            enabled: true,
        });
        true
    }

    pub fn remove(&mut self, addr: u16) {
        self.list.retain(|bp| bp.addr != addr);
    }

    pub fn clear(&mut self) {
        self.list.clear();
    }

    /// Toggle a breakpoint's enabled flag.
    pub fn toggle_enabled(&mut self, addr: u16) {
        if let Some(bp) = self.list.iter_mut().find(|bp| bp.addr == addr) {
            bp.enabled = !bp.enabled;
        }
    }

    /// Whether an *enabled* breakpoint sits at `addr`; the run loop's
    /// stop check.
    pub fn hit_at(&self, addr: u16) -> bool {
        self.list.iter().any(|bp| bp.enabled && bp.addr == addr)
    }

    // -- source-line helpers -------------------------------------------------
    //
    // The caller resolves a (file, line) to its machine addresses via
    // its own source map; these ops only need the address list.

    /// Whether any breakpoint (in any state) sits on a source line.
    pub fn line_has(&self, line_addrs: &[u16]) -> bool {
        self.list.iter().any(|bp| line_addrs.contains(&bp.addr))
    }

    /// Whether an *enabled* breakpoint sits on a source line.
    pub fn line_has_enabled(&self, line_addrs: &[u16]) -> bool {
        self.list
            .iter()
            .any(|bp| bp.enabled && line_addrs.contains(&bp.addr))
    }

    /// Set or remove breakpoints on every address of a source line.
    pub fn toggle_line(&mut self, line_addrs: &[u16]) {
        if self.line_has(line_addrs) {
            self.list.retain(|bp| !line_addrs.contains(&bp.addr));
        } else {
            for &addr in line_addrs {
                self.add(addr, None);
            }
        }
    }

    /// Enable or disable every breakpoint on a source line.
    pub fn set_line_enabled(&mut self, line_addrs: &[u16], enabled: bool) {
        for bp in self.list.iter_mut() {
            if line_addrs.contains(&bp.addr) {
                bp.enabled = enabled;
            }
        }
    }
}

/// The stepping contract a debugger needs from a CPU or board harness:
/// single-step, cycle counter, and fault reporting.  Pausing and cycle
/// budgeting stay the frontend's job.
pub trait DebugCpu {
    fn pc(&self) -> u16;
    /// The board's cycle counter (advances with every instruction).
    fn cycles(&self) -> u64;
    /// Execute exactly one instruction; `false` means the CPU faulted.
    fn step_one(&mut self) -> bool;
}

/// Execute one instruction; reports a breakpoint hit at the new PC.
pub fn step_debug(cpu: &mut impl DebugCpu, breakpoints: &BreakpointSet) -> DebugEvent {
    if cpu.step_one() {
        if breakpoints.hit_at(cpu.pc()) {
            DebugEvent::BreakpointHit(cpu.pc())
        } else {
            DebugEvent::Stepped
        }
    } else {
        DebugEvent::Faulted
    }
}

/// Run up to `budget_cycles` cycles.  Stops early at an enabled
/// breakpoint (checked before the instruction executes) or on fault.
pub fn run_debug(
    cpu: &mut impl DebugCpu,
    breakpoints: &BreakpointSet,
    budget_cycles: u64,
) -> DebugEvent {
    let target_cycle = cpu.cycles().saturating_add(budget_cycles);
    loop {
        if breakpoints.hit_at(cpu.pc()) {
            return DebugEvent::BreakpointHit(cpu.pc());
        }
        if cpu.cycles() >= target_cycle {
            return DebugEvent::Stepped;
        }
        if !cpu.step_one() {
            return DebugEvent::Faulted;
        }
    }
}

/// Disassemble up to `before` lines before `start` and `after` lines
/// after it (the line at `start` itself included).
///
/// `decode(pc)` renders one instruction and returns its next PC, or
/// `None` when the address cannot be decoded (illegal opcode, bus read
/// outside memory).  A known instruction boundary at or below `start`
/// seeds the backwards walk; otherwise a byte-offset heuristic accepts
/// the sequence that lands exactly on `start`.  The forward walk starts
/// at `start` itself.
pub fn disassemble_around(
    mut decode: impl FnMut(u16) -> Option<(u16, String)>,
    start: u16,
    before: usize,
    after: usize,
    seed: Option<u16>,
) -> Vec<(u16, String)> {
    let mut prefix: Vec<(u16, String)> = Vec::new();

    // If we know an instruction boundary at or before the start, decode
    // backwards from it; otherwise fall back to trying nearby offsets
    // and accepting the sequence that lands exactly on the start.
    if let Some(seed) = seed.filter(|seed| *seed <= start) {
        let mut pc = seed;
        while pc != start && prefix.len() < before {
            let Some((next, line)) = decode(pc) else {
                prefix.clear();
                break;
            };
            prefix.push((pc, line));
            pc = next;
        }
        if pc != start {
            // The seed walk did not reach the start exactly (stale seed
            // or self-modifying code): fall back to the offset heuristic.
            prefix.clear();
        }
    }
    if prefix.is_empty() {
        for offset in 1..=32u16 {
            // Stay inside the 16-bit address space: an offset past the
            // start address would wrap below zero.
            let Some(mut pc) = start.checked_sub(offset) else { break };
            let mut candidate = Vec::new();
            while pc != start && candidate.len() <= before {
                let Some((next, line)) = decode(pc) else {
                    candidate.clear();
                    break;
                };
                candidate.push((pc, line));
                pc = next;
            }
            if pc == start && candidate.len() > prefix.len() {
                prefix = candidate;
                if prefix.len() == before {
                    break;
                }
            }
        }
    }

    let mut lines = prefix;
    let mut pc = start;
    for _ in 0..=after {
        let Some((next, line)) = decode(pc) else {
            break;
        };
        lines.push((pc, line));
        pc = next;
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A trivial 2-byte-instruction CPU over a byte array; decode is a
    /// fixed table so the walk tests are deterministic.
    struct FakeCpu {
        pc: u16,
        cycles: u64,
        fault: bool,
        mem: Vec<u8>,
    }

    impl FakeCpu {
        fn new(program: &[u8]) -> Self {
            Self {
                pc: 0,
                cycles: 0,
                fault: false,
                mem: program.to_vec(),
            }
        }
    }

    impl DebugCpu for FakeCpu {
        fn pc(&self) -> u16 {
            self.pc
        }
        fn cycles(&self) -> u64 {
            self.cycles
        }
        fn step_one(&mut self) -> bool {
            if self.fault {
                return false;
            }
            let size = 2u16;
            self.cycles += size as u64;
            self.pc = self.pc.wrapping_add(size);
            true
        }
    }

    fn fake_decode(cpu: &FakeCpu, pc: u16) -> Option<(u16, String)> {
        let addr = pc as usize;
        if addr + 2 > cpu.mem.len() {
            return None;
        }
        let line = format!(
            "{pc:04X}  {:02X} {:02X}",
            cpu.mem[addr],
            cpu.mem[addr + 1]
        );
        Some((pc + 2, line))
    }

    #[test]
    fn breakpoint_set_ops() {
        let mut bps = BreakpointSet::new();
        assert!(bps.add(0x100, Some("start".into())));
        // Duplicate add is a no-op.
        assert!(!bps.add(0x100, None));
        assert_eq!(bps.len(), 1);
        assert!(bps.hit_at(0x100));
        bps.toggle_enabled(0x100);
        assert!(!bps.hit_at(0x100));
        bps.remove(0x100);
        assert!(bps.is_empty());
    }

    #[test]
    fn line_helpers_operate_on_resolved_addresses() {
        let mut bps = BreakpointSet::new();
        bps.add(0x202, None);
        bps.add(0x301, None);
        assert!(bps.line_has(&[0x200, 0x202]));
        assert!(bps.line_has_enabled(&[0x202]));
        bps.set_line_enabled(&[0x202], false);
        assert!(!bps.line_has_enabled(&[0x202]));
        bps.toggle_line(&[0x202, 0x203]);
        // The line's breakpoints are removed (toggle when any exists);
        // the unrelated breakpoint survives.
        assert!(!bps.line_has(&[0x202]));
        assert!(!bps.line_has(&[0x203]));
        assert!(bps.line_has(&[0x301]));
    }

    #[test]
    fn step_reports_breakpoint_hit_and_fault() {
        let mut cpu = FakeCpu::new(&[0; 8]);
        let bps = BreakpointSet::new();
        // No breakpoint: a plain step.
        assert_eq!(step_debug(&mut cpu, &bps), DebugEvent::Stepped);
        assert_eq!(cpu.pc(), 2);
        // Breakpoint at the new PC is reported.
        let mut bps = BreakpointSet::new();
        bps.add(4, None);
        assert_eq!(step_debug(&mut cpu, &bps), DebugEvent::BreakpointHit(4));
        // A fault surfaces as Faulted.
        let mut cpu = FakeCpu::new(&[0; 8]);
        cpu.fault = true;
        assert_eq!(step_debug(&mut cpu, &bps), DebugEvent::Faulted);
    }

    #[test]
    fn run_stops_at_breakpoint_before_executing() {
        let mut cpu = FakeCpu::new(&[0; 16]);
        let mut bps = BreakpointSet::new();
        bps.add(4, None);
        // The breakpoint at 4 stops the run before that instruction
        // executes: the PC is exactly 4.
        assert_eq!(
            run_debug(&mut cpu, &bps, 1_000),
            DebugEvent::BreakpointHit(4)
        );
        assert_eq!(cpu.pc(), 4);
        // Disabled breakpoints never stop the machine: the run consumes
        // the whole budget (the first run already spent 4 cycles).
        bps.toggle_enabled(4);
        assert_eq!(run_debug(&mut cpu, &bps, 1_000), DebugEvent::Stepped);
        assert_eq!(cpu.pc(), 1004);
        assert_eq!(cpu.cycles(), 1004);
    }

    #[test]
    fn run_budget_is_cycle_based() {
        let mut cpu = FakeCpu::new(&[0; 16]);
        let bps = BreakpointSet::new();
        assert_eq!(run_debug(&mut cpu, &bps, 5), DebugEvent::Stepped);
        // 5-cycle budget: instructions of 2 cycles run while
        // cycles < 5, so 3 instructions (6 cycles) execute.
        assert_eq!(cpu.cycles(), 6);
        assert_eq!(cpu.pc(), 6);
    }

    #[test]
    fn disassemble_walk_lands_on_start() {
        let program = (0..64u8).collect::<Vec<_>>();
        let cpu = FakeCpu::new(&program);
        // start=6 (aligned to 2-byte instructions): the backwards walk
        // finds the boundary at 4 and the forward walk starts at 6.
        let lines = disassemble_around(|pc| fake_decode(&cpu, pc), 6, 2, 3, None);
        let pcs = lines.iter().map(|(pc, _)| *pc).collect::<Vec<_>>();
        assert_eq!(pcs, vec![2, 4, 6, 8, 10, 12]);
        // A seed boundary at 4 walks back only from the seed to the
        // start (the 6809 behavior), then forward from the start.
        let lines = disassemble_around(|pc| fake_decode(&cpu, pc), 6, 2, 3, Some(4));
        let pcs = lines.iter().map(|(pc, _)| *pc).collect::<Vec<_>>();
        assert_eq!(pcs, vec![4, 6, 8, 10, 12]);
    }
}
