/// Emits characters from a queue at a rate approximating `baud` baud,
/// assuming `tick()` is called at 20 Hz (every 50 ms).
pub struct BaudTyper {
    pub baud: u32,
    buffer: Vec<char>,
    ticks_per_char: u32,
    tick_count: u32,
}

impl BaudTyper {
    pub fn new(baud: u32) -> Self {
        // chars/sec ≈ baud/10 (8N1).  ticks/char = 20 / (baud/10).
        let cps = (baud / 10).max(1);
        let ticks_per_char = (20_u32).div_ceil(cps).max(1);
        Self {
            baud,
            buffer: vec![],
            ticks_per_char,
            tick_count: 0,
        }
    }

    pub fn enqueue(&mut self, s: &str) {
        self.buffer.extend(s.chars());
    }

    /// Returns the next character to display, if any.
    pub fn tick(&mut self) -> Option<char> {
        if self.buffer.is_empty() {
            return None;
        }
        self.tick_count += 1;
        if self.tick_count >= self.ticks_per_char {
            self.tick_count = 0;
            return Some(self.buffer.remove(0));
        }
        None
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}
