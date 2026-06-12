use std::f32::consts::PI;

fn build_wav(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut v = Vec::with_capacity(44 + data_len as usize);
    v.extend_from_slice(b"RIFF");
    v.extend_from_slice(&(36 + data_len).to_le_bytes());
    v.extend_from_slice(b"WAVE");
    v.extend_from_slice(b"fmt ");
    v.extend_from_slice(&16u32.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes()); // PCM
    v.extend_from_slice(&1u16.to_le_bytes()); // mono
    v.extend_from_slice(&sample_rate.to_le_bytes());
    v.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    v.extend_from_slice(&2u16.to_le_bytes());
    v.extend_from_slice(&16u16.to_le_bytes());
    v.extend_from_slice(b"data");
    v.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        v.extend_from_slice(&s.to_le_bytes());
    }
    v
}

fn sine_tone(freq: f32, dur: f32, sr: u32, amp: f32) -> Vec<i16> {
    let n = (sr as f32 * dur) as usize;
    let fade = ((sr as f32 * 0.008) as usize).max(1);
    (0..n)
        .map(|i| {
            let t = i as f32 / sr as f32;
            let env = if i < fade {
                i as f32 / fade as f32
            } else if i + fade > n {
                (n - i) as f32 / fade as f32
            } else {
                1.0
            };
            ((2.0 * PI * freq * t).sin() * env * amp * 32767.0) as i16
        })
        .collect()
}

/// FSK-like modem handshake noise (~2.5 s).
pub fn handshake_wav() -> Vec<u8> {
    let sr = 22050u32;
    let dur = 2.5_f32;
    let n = (sr as f32 * dur) as usize;
    let toggle_period = sr as usize / 250; // ~4 ms per symbol
    let fade = (sr as f32 * 0.015) as usize;
    let mut seed = 0xdeadbeef_cafebabe_u64;
    let samples: Vec<i16> = (0..n)
        .map(|i| {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let toggle = (i / toggle_period) % 2;
            let freq = if toggle == 0 { 1200.0_f32 } else { 2400.0_f32 };
            let t = i as f32 / sr as f32;
            let sine = (2.0 * PI * freq * t).sin();
            let noise = ((seed >> 48) as i8) as f32 / 512.0;
            let env = if i < fade {
                i as f32 / fade as f32
            } else if i + fade > n {
                (n - i) as f32 / fade as f32
            } else {
                1.0
            };
            ((sine * 0.65 + noise * 0.35) * env * 0.30 * 32767.0) as i16
        })
        .collect();
    build_wav(&samples, sr)
}

/// Two-tone "CONNECT" ascending sound.
pub fn connect_wav() -> Vec<u8> {
    let sr = 22050u32;
    let mut s = sine_tone(1200.0, 0.18, sr, 0.38);
    s.extend(sine_tone(2400.0, 0.14, sr, 0.38));
    build_wav(&s, sr)
}

/// Descending "NO ANSWER" tone.
pub fn no_answer_wav() -> Vec<u8> {
    let sr = 22050u32;
    let dur = 0.65_f32;
    let n = (sr as f32 * dur) as usize;
    let fade = (sr as f32 * 0.012) as usize;
    let samples: Vec<i16> = (0..n)
        .map(|i| {
            let progress = i as f32 / n as f32;
            let freq = 900.0 - 680.0 * progress;
            let t = i as f32 / sr as f32;
            let env = if i < fade {
                i as f32 / fade as f32
            } else if i + fade > n {
                (n - i) as f32 / fade as f32
            } else {
                1.0 - progress * 0.3
            };
            ((2.0 * PI * freq * t).sin() * env * 0.36 * 32767.0) as i16
        })
        .collect();
    build_wav(&samples, sr)
}

/// Short "carrier drop" static burst used on disconnect/logout.
pub fn carrier_drop_wav() -> Vec<u8> {
    let sr = 22050u32;
    let n = (sr as f32 * 0.09) as usize;
    let mut seed = 0xabcdef01_u64;
    let samples: Vec<i16> = (0..n)
        .map(|i| {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let noise = ((seed >> 48) as i8) as f32 / 128.0;
            let env = 1.0 - (i as f32 / n as f32);
            (noise * env * 0.38 * 32767.0) as i16
        })
        .collect();
    build_wav(&samples, sr)
}

/// Tiny mechanical key-navigation click.
pub fn nav_click_wav() -> Vec<u8> {
    let sr = 22050u32;
    let n = (sr as f32 * 0.005) as usize;
    let mut seed = 0x12345678_u64;
    let samples: Vec<i16> = (0..n)
        .map(|i| {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let noise = ((seed >> 48) as i8) as f32 / 128.0;
            let env = 1.0 - (i as f32 / n as f32).powi(2);
            (noise * env * 0.22 * 32767.0) as i16
        })
        .collect();
    build_wav(&samples, sr)
}
