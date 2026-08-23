/// Minimal model of the board's 8-bit multiplying DAC input.
#[derive(Clone, Debug, Default)]
pub struct Dac8 {
    value: u8,
    samples: Vec<u8>,
    events: Vec<DacEvent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DacEvent {
    pub cycle: u64,
    pub value: u8,
}

impl Dac8 {
    pub fn value(&self) -> u8 {
        self.value
    }

    pub fn write(&mut self, value: u8) {
        self.write_at(0, value);
    }

    pub fn write_at(&mut self, cycle: u64, value: u8) {
        self.value = value;
        self.samples.push(value);
        self.events.push(DacEvent { cycle, value });
    }

    /// Returns and clears the values written since the previous call.
    pub fn take_samples(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.samples)
    }

    pub fn take_events(&mut self) -> Vec<DacEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn pending_events(&self) -> usize {
        self.events.len()
    }
}
