//! CPU-independent lifecycle and instrumentation primitives.
//!
//! CPU crates keep their own registers, decoder, addressing modes, and
//! instruction semantics. These types cover the small amount of policy that
//! is common to 6800-family, 6809, 6502, and board-level harnesses.

/// External lines shared by most 8-bit processors.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CpuPins {
    pub irq: bool,
    pub nmi: bool,
    pub reset: bool,
}

/// Counters maintained by an emulator core or board harness.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExecutionStats {
    pub cycles: u64,
    pub instructions: u64,
}

impl ExecutionStats {
    pub fn record(&mut self, cycles: u64) {
        self.cycles += cycles;
        self.instructions += 1;
    }
}

/// Result metadata common to instruction-level tracing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CpuStep {
    pub cycles: u64,
    pub instruction: u64,
}

/// Optional instruction observer. Production emulation can use
/// [`NoopObserver`], while debuggers and differential tests can collect state
/// without coupling the CPU core to a particular board or logger.
pub trait CpuObserver<State> {
    fn before_instruction(&mut self, _state: &State) {}
    fn after_instruction(&mut self, _state: &State, _step: CpuStep) {}
}

#[derive(Default)]
pub struct NoopObserver;

impl<State> CpuObserver<State> for NoopObserver {}

/// Minimal lifecycle contract for CPU implementations.
pub trait Cpu {
    type Error;

    fn reset(&mut self) -> Result<(), Self::Error>;
    fn step(&mut self) -> Result<CpuStep, Self::Error>;
    fn stats(&self) -> ExecutionStats;
}

#[cfg(test)]
mod tests {
    use super::ExecutionStats;

    #[test]
    fn stats_record_cycles_and_instructions() {
        let mut stats = ExecutionStats::default();
        stats.record(4);
        stats.record(2);
        assert_eq!(stats.cycles, 6);
        assert_eq!(stats.instructions, 2);
    }
}
