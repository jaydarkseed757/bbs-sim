#[derive(Debug, Clone, PartialEq)]
pub enum DialPhase {
    PickingUpLine,
    Dialing,
    Connecting,
    Handshaking,
    Connected,
}

pub struct ModemSim {
    pub phase: DialPhase,
    phase_ticks: u32,
}

impl ModemSim {
    pub fn new() -> Self {
        Self {
            phase: DialPhase::PickingUpLine,
            phase_ticks: 0,
        }
    }

    /// Advance one tick; returns a string to emit if a modem event fires.
    pub fn tick(&mut self) -> Option<String> {
        self.phase_ticks += 1;
        match self.phase {
            DialPhase::PickingUpLine => {
                if self.phase_ticks == 1 {
                    return Some("ATZ\r\n".into());
                }
                if self.phase_ticks >= 10 {
                    self.advance(DialPhase::Dialing);
                    return Some("OK\r\n".into());
                }
            }
            DialPhase::Dialing => {
                if self.phase_ticks >= 20 {
                    self.advance(DialPhase::Connecting);
                }
            }
            DialPhase::Connecting => {
                if self.phase_ticks >= 15 {
                    self.advance(DialPhase::Handshaking);
                }
            }
            DialPhase::Handshaking => {
                if self.phase_ticks >= 20 {
                    self.advance(DialPhase::Connected);
                    return Some("CONNECT 2400\r\n".into());
                }
            }
            DialPhase::Connected => {}
        }
        None
    }

    fn advance(&mut self, next: DialPhase) {
        self.phase = next;
        self.phase_ticks = 0;
    }
}
