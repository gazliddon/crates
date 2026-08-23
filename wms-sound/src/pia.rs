const DDR_SELECT: u8 = 0x04;
const IRQ_FLAG: u8 = 0x80;
const IRQ_ENABLE: u8 = 0x01;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PiaEvent {
    PortAOutput(u8),
    Irq,
}

/// The register-level portion of a Motorola 6821 PIA.
#[derive(Clone, Debug, Default)]
pub struct Pia6821 {
    ddra: u8,
    ddrb: u8,
    porta: u8,
    portb: u8,
    input_a: u8,
    input_b: u8,
    cra: u8,
    crb: u8,
    irq_a: bool,
    irq_b: bool,
    cb1: bool,
}

impl Pia6821 {
    pub fn read(&mut self, offset: u8) -> u8 {
        match offset & 3 {
            0 => {
                if self.cra & DDR_SELECT == 0 {
                    self.ddra
                } else {
                    self.read_port(self.porta, self.input_a, self.ddra)
                }
            }
            1 => self.cra | if self.irq_a { IRQ_FLAG } else { 0 },
            2 => {
                if self.crb & DDR_SELECT == 0 {
                    self.ddrb
                } else {
                    self.read_port(self.portb, self.input_b, self.ddrb)
                }
            }
            _ => self.crb | if self.irq_b { IRQ_FLAG } else { 0 },
        }
    }

    pub fn write(&mut self, offset: u8, value: u8) -> Option<PiaEvent> {
        match offset & 3 {
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

    pub fn set_port_b_input(&mut self, value: u8) {
        self.input_b = value;
    }

    /// Models the external CB1 handshake used by the game CPU to deliver a
    /// sound command. The edge latches the PIA flag even before software
    /// enables IRQs; the control register determines when that flag reaches
    /// the CPU's IRQ input.
    pub fn pulse_cb1(&mut self) -> Option<PiaEvent> {
        if self.cb1 {
            // A second edge while the flag is latched does not create a
            // second interrupt; CB1 must first return low.
            return None;
        }
        self.cb1 = true;
        if self.irq_b {
            return None;
        }
        self.irq_b = true;
        Some(PiaEvent::Irq)
    }

    pub fn set_cb1(&mut self, level: bool) {
        self.cb1 = level;
    }

    pub fn irq_pending(&self) -> bool {
        (self.irq_a && self.cra & IRQ_ENABLE != 0) || (self.irq_b && self.crb & IRQ_ENABLE != 0)
    }

    pub fn port_a_output(&self) -> u8 {
        self.porta & self.ddra
    }

    pub fn data_direction_a(&self) -> u8 {
        self.ddra
    }

    pub fn control_a(&self) -> u8 {
        self.cra
    }

    pub fn state_bytes(&self) -> [u8; 10] {
        [
            self.ddra,
            self.ddrb,
            self.porta,
            self.portb,
            self.input_a,
            self.input_b,
            self.cra,
            self.crb,
            self.irq_a as u8,
            self.irq_b as u8 | ((self.cb1 as u8) << 1),
        ]
    }

    pub fn port_b_input(&self) -> u8 {
        self.input_b
    }

    fn read_port(&mut self, output: u8, input: u8, ddr: u8) -> u8 {
        self.irq_a = false;
        self.irq_b = false;
        (output & ddr) | (input & !ddr)
    }
}

#[cfg(test)]
mod tests {
    use super::{Pia6821, PiaEvent};

    #[test]
    fn data_direction_and_port_output_follow_control_bit() {
        let mut pia = Pia6821::default();
        pia.write(1, 0x00);
        pia.write(0, 0xff);
        pia.write(1, 0x04);
        assert_eq!(pia.write(0, 0xa5), Some(PiaEvent::PortAOutput(0xa5)));
        assert_eq!(pia.port_a_output(), 0xa5);
    }

    #[test]
    fn enabled_cb1_pulse_sets_irq() {
        let mut pia = Pia6821::default();
        pia.write(3, 0x05);
        assert_eq!(pia.pulse_cb1(), Some(PiaEvent::Irq));
        assert_eq!(pia.pulse_cb1(), None);
        assert!(pia.irq_pending());
    }
}
