/// Emits characters from a queue at a rate approximating `baud` baud,
/// assuming `tick()` is called at 20 Hz (every 50 ms).
///
/// Slow baud (< ~200 baud): multiple ticks per character.
/// Fast baud (≥ 200 baud):  multiple characters per tick.
pub struct BaudTyper {
    buffer: Vec<char>,
    ticks_per_char: u32,
    chars_per_tick: usize,
    tick_count: u32,
}

impl BaudTyper {
    pub fn new(baud: u32) -> Self {
        let cps = (baud as f32 / 10.0).max(1.0); // chars per second (8N1)
        let fps = 20.0_f32;                        // tick rate
        let ratio = cps / fps;                     // chars per tick (float)

        let (ticks_per_char, chars_per_tick) = if ratio >= 1.0 {
            (1_u32, ratio as usize) // fast: flush N chars every tick
        } else {
            ((1.0 / ratio).round() as u32, 1_usize) // slow: one char every N ticks
        };

        Self {
            buffer: vec![],
            ticks_per_char,
            chars_per_tick,
            tick_count: 0,
        }
    }

    pub fn enqueue(&mut self, s: &str) {
        self.buffer.extend(s.chars());
    }

    /// Returns the batch of characters to display this tick (may be empty).
    pub fn tick(&mut self) -> Vec<char> {
        if self.buffer.is_empty() {
            return vec![];
        }
        self.tick_count += 1;
        if self.tick_count < self.ticks_per_char {
            return vec![];
        }
        self.tick_count = 0;
        let n = self.chars_per_tick.min(self.buffer.len());
        self.buffer.drain(..n).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}
