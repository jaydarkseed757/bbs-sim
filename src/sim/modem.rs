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
    number: String,
    noise_seed: u64,
}

impl ModemSim {
    pub fn new(number: &str) -> Self {
        Self {
            phase: DialPhase::PickingUpLine,
            phase_ticks: 0,
            number: number.to_string(),
            noise_seed: 0xdeadbeef_cafebabe,
        }
    }

    /// Advance one tick. Returns text to enqueue into the baud typer, if any.
    pub fn tick(&mut self) -> Option<String> {
        self.phase_ticks += 1;
        match self.phase {
            DialPhase::PickingUpLine => {
                if self.phase_ticks == 1 {
                    return Some("ATZ\r\n".into());
                }
                if self.phase_ticks >= 8 {
                    self.advance(DialPhase::Dialing);
                    return Some("OK\r\n".into());
                }
            }
            DialPhase::Dialing => {
                if self.phase_ticks == 1 {
                    return Some(format!("ATDT{}\r\n", self.number));
                }
                if self.phase_ticks >= 25 {
                    self.advance(DialPhase::Connecting);
                }
            }
            DialPhase::Connecting => {
                let noise = self.noise_burst(12);
                if self.phase_ticks >= 20 {
                    self.advance(DialPhase::Handshaking);
                }
                return Some(noise);
            }
            DialPhase::Handshaking => {
                if self.phase_ticks >= 35 {
                    self.advance(DialPhase::Connected);
                    // Flush remaining noise then the clean connect string.
                    return Some(format!("{}\r\nCONNECT 2400\r\n", self.noise_burst(8)));
                }
                return Some(self.noise_burst(10));
            }
            DialPhase::Connected => {}
        }
        None
    }

    fn advance(&mut self, next: DialPhase) {
        self.phase = next;
        self.phase_ticks = 0;
    }

    fn noise_char(&mut self) -> char {
        // Deterministic LCG — no external deps required.
        self.noise_seed = self.noise_seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let byte = ((self.noise_seed >> 33) as u8) % 95 + 32;
        byte as char
    }

    fn noise_burst(&mut self, n: usize) -> String {
        (0..n)
            .map(|i| {
                // Insert a line break every ~40 chars so the buffer scrolls visibly.
                if i > 0 && i % 40 == 0 {
                    '\n'
                } else {
                    self.noise_char()
                }
            })
            .collect()
    }
}
