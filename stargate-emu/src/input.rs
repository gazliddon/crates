/// Inputs presented to the Stargate main-board PIAs.
///
/// The fields are deliberately logical (rather than raw port bits), so front
/// ends can map keyboards, joysticks, or network controls without knowing the
/// Williams wiring.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StargateInput {
    pub fire: bool,
    pub thrust: bool,
    pub smart_bomb: bool,
    pub hyperspace: bool,
    pub reverse: bool,
    pub inviso: bool,
    pub up: bool,
    pub down: bool,
    pub start1: bool,
    pub start2: bool,
    pub coin1: bool,
    pub service: bool,
    pub advance: bool,
}

impl StargateInput {
    /// Value read from main PIA 0 port A (IN0).
    pub fn port0(self) -> u8 {
        self.fire as u8
            | (self.thrust as u8) << 1
            | (self.smart_bomb as u8) << 2
            | (self.hyperspace as u8) << 3
            | (self.start2 as u8) << 4
            | (self.start1 as u8) << 5
            | (self.reverse as u8) << 6
            | (self.down as u8) << 7
    }

    /// Value read from main PIA 0 port B (IN1).
    pub fn port1(self) -> u8 {
        self.up as u8 | (self.inviso as u8) << 1
    }

    /// Value read from main PIA 1 port A (IN2).
    pub fn port2(self) -> u8 {
        (self.service as u8) | (self.advance as u8) << 1 | (self.coin1 as u8) << 4
    }
}
