//! The Motorola 6821 peripheral interface adapter (PIA) as a
//! standalone device for any emulator.  Host-specific wiring stays
//! with the host: what drives the CA1/CB1 lines, what the port pins
//! mean, and when a latched flag asserts the host's IRQ line.  This
//! module models the chip itself: registers (with data-direction
//! registers), port read/write semantics, CA1/CB1 edge detection and
//! flags.
//!
//! Consumers: the Williams Stargate main board (`williams-emu`, edge
//! timestamps + deferred IRQ policy) and the sound board (`wms-sound`,
//! DDR + port output events).  Both use the MAME `pia6821_device`
//! register order: `0` = port A data / DDRA, `1` = control A, `2` =
//! port B data / DDRB, `3` = control B.

const DDR_SELECT: u8 = 0x04;
const IRQ_FLAG: u8 = 0x80;
const IRQ_ENABLE: u8 = 0x01;

/// Events a host may act on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PiaEvent {
    /// Port A data register written while CRA selects data; the
    /// payload is the DDR-masked output level.
    PortAOutput(u8),
    /// A CA1/CB1 active edge latched a previously clear flag.
    Irq,
}

/// The register-level portion of a Motorola 6821 PIA.
#[derive(Clone, Copy, Debug, Default)]
pub struct Pia6821 {
    ddra: u8,
    ddrb: u8,
    /// Output latches for ports A and B.
    porta: u8,
    portb: u8,
    cra: u8,
    crb: u8,
    /// CA1/CB1 interrupt flags (bit 7 of the control register).
    irq_a1: bool,
    irq_b1: bool,
    /// Host time at which each flag latched on its CA1/CB1 edge.  The
    /// host decides when a latched flag asserts its IRQ line (the
    /// Stargate main board defers by ~13 scanlines); `None` = no edge
    /// since the flag was cleared.
    latch_a1: Option<u64>,
    latch_b1: Option<u64>,
    /// Current level of the CA1/CB1 input lines (for edge detection).
    line_ca1: bool,
    line_cb1: bool,
}

impl Pia6821 {
    pub fn new() -> Self {
        Self::default()
    }

    /// Read a register.  Reading a port data register (CRA/CRB bit 2
    /// selects data) clears the corresponding CA1/CB1 flag and its
    /// latch.  `pins_a`/`pins_b` are the current pin levels, as
    /// supplied by the host; the returned data register is the
    /// DDR-masked combination of the output latch and the pins.
    pub fn read(&mut self, reg: u8, pins_a: u8, pins_b: u8) -> u8 {
        match reg & 3 {
            // Port A data / DDRA, selected by CRA bit 2.
            0 if self.cra & DDR_SELECT == 0 => self.ddra,
            0 => {
                self.irq_a1 = false;
                self.latch_a1 = None;
                self.read_port(self.porta, pins_a, self.ddra)
            }
            1 => self.cra | if self.irq_a1 { IRQ_FLAG } else { 0 },
            // Port B data / DDRB, selected by CRB bit 2.
            2 if self.crb & DDR_SELECT == 0 => self.ddrb,
            2 => {
                self.irq_b1 = false;
                self.latch_b1 = None;
                self.read_port(self.portb, pins_b, self.ddrb)
            }
            _ => self.crb | if self.irq_b1 { IRQ_FLAG } else { 0 },
        }
    }

    /// Read a register without side effects (debugger inspection).
    pub fn peek(&self, reg: u8, pins_a: u8, pins_b: u8) -> u8 {
        match reg & 3 {
            0 if self.cra & DDR_SELECT == 0 => self.ddra,
            0 => self.read_port(self.porta, pins_a, self.ddra),
            1 => self.cra | if self.irq_a1 { IRQ_FLAG } else { 0 },
            2 if self.crb & DDR_SELECT == 0 => self.ddrb,
            2 => self.read_port(self.portb, pins_b, self.ddrb),
            _ => self.crb | if self.irq_b1 { IRQ_FLAG } else { 0 },
        }
    }

    /// Write a register.  Control writes are masked to bits 0-5 (bits
    /// 6-7 are the read-only flag and CA2/CB2 state).  Writing port A
    /// data while CRA selects data emits `PortAOutput`.
    pub fn write(&mut self, reg: u8, value: u8) -> Option<PiaEvent> {
        match reg & 3 {
            0 if self.cra & DDR_SELECT == 0 => self.ddra = value,
            0 => {
                self.porta = value;
                return Some(PiaEvent::PortAOutput(self.porta & self.ddra));
            }
            1 => self.cra = value & 0x3f,
            2 if self.crb & DDR_SELECT == 0 => self.ddrb = value,
            2 => self.portb = value,
            _ => self.crb = value & 0x3f,
        }
        None
    }

    /// Set the CA1 line level without edge logic (host reset).
    pub fn set_ca1(&mut self, state: bool) {
        self.line_ca1 = state;
    }

    /// Set the CB1 line level without edge logic (host reset).
    pub fn set_cb1(&mut self, state: bool) {
        self.line_cb1 = state;
    }

    /// Drive the CA1 line, latching the IRQA1 flag on the active edge
    /// (control bit 1: 1 = low->high, 0 = high->low), timestamped at
    /// host time `now`.  Returns `Irq` when a previously clear flag
    /// latches.  A second active edge while the flag is latched
    /// refreshes the timestamp but emits nothing.
    pub fn drive_ca1(&mut self, state: bool, now: u64) -> Option<PiaEvent> {
        if self.line_ca1 != state {
            self.line_ca1 = state;
            if state == (self.cra & 0x02 != 0) {
                let fresh = !self.irq_a1;
                self.irq_a1 = true;
                self.latch_a1 = Some(now);
                return fresh.then_some(PiaEvent::Irq);
            }
        }
        None
    }

    /// Drive the CB1 line (see `drive_ca1` for semantics).
    pub fn drive_cb1(&mut self, state: bool, now: u64) -> Option<PiaEvent> {
        if self.line_cb1 != state {
            self.line_cb1 = state;
            if state == (self.crb & 0x02 != 0) {
                let fresh = !self.irq_b1;
                self.irq_b1 = true;
                self.latch_b1 = Some(now);
                return fresh.then_some(PiaEvent::Irq);
            }
        }
        None
    }

    /// Whether either flag is latched while its control enable bit
    /// (bit 0) is set — the 6821 IRQ output without any host timing
    /// policy.  Hosts with deferred assertion (the Stargate main
    /// board) compute their own level from the flags and latches.
    pub fn irq_pending(&self) -> bool {
        (self.irq_a1 && self.cra & IRQ_ENABLE != 0) || (self.irq_b1 && self.crb & IRQ_ENABLE != 0)
    }

    // -- diagnostics -------------------------------------------------------

    pub fn irq_a1(&self) -> bool {
        self.irq_a1
    }

    pub fn irq_b1(&self) -> bool {
        self.irq_b1
    }

    pub fn latch_a1(&self) -> Option<u64> {
        self.latch_a1
    }

    pub fn latch_b1(&self) -> Option<u64> {
        self.latch_b1
    }

    /// Raw register contents (control registers without the flag
    /// bits); data registers are the raw output latches.
    pub fn reg(&self, reg: usize) -> u8 {
        match reg {
            0 => self.porta,
            1 => self.cra,
            2 => self.portb,
            _ => self.crb,
        }
    }

    pub fn regs(&self) -> [u8; 4] {
        [self.reg(0), self.reg(1), self.reg(2), self.reg(3)]
    }

    pub fn port_a_output(&self) -> u8 {
        self.porta & self.ddra
    }

    pub fn port_b_output(&self) -> u8 {
        self.portb & self.ddrb
    }

    pub fn data_direction_a(&self) -> u8 {
        self.ddra
    }

    pub fn data_direction_b(&self) -> u8 {
        self.ddrb
    }

    pub fn control_a(&self) -> u8 {
        self.cra
    }

    pub fn control_b(&self) -> u8 {
        self.crb
    }

    fn read_port(&self, output: u8, input: u8, ddr: u8) -> u8 {
        (output & ddr) | (input & !ddr)
    }

    // -- save state -------------------------------------------------------

    /// Payload (20 bytes): `ddra ddrb porta portb cra crb | irq_a1
    /// irq_b1 line_ca1 line_cb1 | latch_a1 latch_b1` (present flag +
    /// u32 host time each).
    pub fn state(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(20);
        v.push(self.ddra);
        v.push(self.ddrb);
        v.push(self.porta);
        v.push(self.portb);
        v.push(self.cra);
        v.push(self.crb);
        v.push(self.irq_a1 as u8);
        v.push(self.irq_b1 as u8);
        v.push(self.line_ca1 as u8);
        v.push(self.line_cb1 as u8);
        for latch in [self.latch_a1, self.latch_b1] {
            v.push(latch.is_some() as u8);
            v.extend_from_slice(&(latch.unwrap_or(0) as u32).to_be_bytes());
        }
        v
    }

    /// Restore from a `state()` payload; `tag` names the device for
    /// error messages.
    pub fn restore(&mut self, version: u8, payload: &[u8], tag: &[u8]) -> Result<(), String> {
        if version != 1 {
            return Err(format!("unsupported device version {version} in {tag:?}"));
        }
        if payload.len() != 20 {
            return Err(format!("bad {tag:?} block length"));
        }
        self.ddra = payload[0];
        self.ddrb = payload[1];
        self.porta = payload[2];
        self.portb = payload[3];
        self.cra = payload[4];
        self.crb = payload[5];
        self.irq_a1 = payload[6] != 0;
        self.irq_b1 = payload[7] != 0;
        self.line_ca1 = payload[8] != 0;
        self.line_cb1 = payload[9] != 0;
        let mut q = 10;
        for latch in [&mut self.latch_a1, &mut self.latch_b1] {
            let has = payload[q] != 0;
            let at = u32::from_be_bytes(payload[q + 1..q + 5].try_into().unwrap()) as u64;
            *latch = if has { Some(at) } else { None };
            q += 5;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Pia6821, PiaEvent};

    #[test]
    fn data_direction_and_port_output_follow_control_bit() {
        let mut pia = Pia6821::default();
        pia.write(1, 0x00); // CRA: DDR selected
        pia.write(0, 0xff); // DDRA = all outputs
        pia.write(1, 0x04); // CRA: data selected
        assert_eq!(pia.write(0, 0xa5), Some(PiaEvent::PortAOutput(0xa5)));
        assert_eq!(pia.port_a_output(), 0xa5);
    }

    #[test]
    fn port_reads_are_ddr_masked_combinations_of_latch_and_pins() {
        let mut pia = Pia6821::default();
        pia.write(1, 0x04); // port A data selected, DDRA = 0 (inputs)
        assert_eq!(pia.read(0, 0x5a, 0), 0x5a); // all input: pins win
        pia.write(1, 0x00); // DDR selected
        pia.write(0, 0xf0); // DDRA: high nibble output
        pia.write(1, 0x04); // data selected
        pia.write(0, 0xa0); // output latch: high nibble
        // High nibble comes from the latch, low nibble from the pins.
        assert_eq!(pia.read(0, 0x05, 0), 0xa5);
    }

    #[test]
    fn port_data_read_clears_flag_and_latch() {
        let mut pia = Pia6821::default();
        pia.write(1, 0x04); // CRA: data selected
        pia.drive_ca1(true, 1);
        pia.drive_ca1(false, 42); // active-low edge: flag latches at t=42
        assert!(pia.irq_a1());
        assert_eq!(pia.latch_a1(), Some(42));
        assert_eq!(pia.read(0, 0xaa, 0xbb), 0xaa);
        assert!(!pia.irq_a1());
        assert_eq!(pia.latch_a1(), None);
    }

    #[test]
    fn ddr_reads_do_not_clear_flags() {
        let mut pia = Pia6821::default();
        pia.write(1, 0x00); // CRA: DDR selected
        pia.drive_ca1(true, 1);
        pia.drive_ca1(false, 42);
        assert!(pia.irq_a1());
        assert_eq!(pia.read(0, 0, 0), 0); // DDRA read: flag survives
        assert!(pia.irq_a1());
    }

    #[test]
    fn edges_latch_on_control_polarity() {
        let mut pia = Pia6821::default();
        pia.write(3, 0x02); // CRB: CB1 active-high
        pia.drive_cb1(true, 7); // low->high on active-high: latch
        assert!(pia.irq_b1());
        assert_eq!(pia.latch_b1(), Some(7));
        pia.drive_cb1(false, 8); // high->low: not an active edge
        assert!(pia.irq_b1());
        assert_eq!(pia.latch_b1(), Some(7));
    }

    #[test]
    fn enabled_cb1_pulse_asserts_irq_pending() {
        let mut pia = Pia6821::default();
        pia.write(3, 0x05); // CRB: IRQ enabled, CB1 active-low
        assert_eq!(pia.drive_cb1(true, 0), None); // rising: not active
        assert_eq!(pia.drive_cb1(false, 0), Some(PiaEvent::Irq)); // falling: latch
        assert!(pia.irq_pending());
        // A second pulse while latched emits nothing but stays pending.
        pia.drive_cb1(true, 0);
        assert_eq!(pia.drive_cb1(false, 0), None);
        assert!(pia.irq_pending());
        // Reading the port data clears the flag and drops the output.
        pia.write(3, 0x04); // CRB: data selected for port B reads
        pia.read(2, 0, 0);
        assert!(!pia.irq_pending());
    }

    #[test]
    fn control_writes_are_masked_to_bits_0_5() {
        let mut pia = Pia6821::default();
        pia.write(1, 0xff);
        assert_eq!(pia.control_a(), 0x3f);
        pia.write(3, 0xfe);
        assert_eq!(pia.control_b(), 0x3e);
    }

    #[test]
    fn peek_reads_without_side_effects() {
        let mut pia = Pia6821::default();
        pia.write(1, 0x04);
        pia.drive_ca1(true, 1);
        pia.drive_ca1(false, 42);
        assert_eq!(pia.peek(1, 0, 0) & 0x80, 0x80);
        assert!(pia.irq_a1()); // survives the peek
        assert_eq!(pia.peek(0, 0xaa, 0xbb), 0xaa);
        assert!(pia.irq_a1());
    }

    #[test]
    fn state_round_trips() {
        let mut pia = Pia6821::default();
        pia.write(1, 0x02);
        pia.write(3, 0x02);
        pia.drive_ca1(true, 5);
        pia.drive_cb1(true, 9);
        let payload = pia.state();
        let mut other = Pia6821::default();
        other.restore(1, &payload, b"PIA0").unwrap();
        assert_eq!(other.state(), payload);
        assert_eq!(other.control_a(), 0x02);
        assert!(other.irq_a1());
        assert_eq!(other.latch_a1(), Some(5));
        assert_eq!(other.latch_b1(), Some(9));
    }
}
