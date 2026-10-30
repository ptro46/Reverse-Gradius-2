//! Render extracted original PSG/SCC register streams to PCM. No Z80 at runtime.
use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};
use std::collections::HashMap;
pub struct Rendered {
    pub pcm: Vec<i16>,
    pub loop_sample: Option<usize>,
}
fn word(b: &[u8], p: usize) -> u16 {
    u16::from_le_bytes([b[p], b[p + 1]])
}
pub fn render(bytes: &[u8]) -> Rendered {
    let frames = u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize;
    let loop_frame = i32::from_le_bytes(bytes[4..8].try_into().unwrap());
    let mut pos = 8;
    let mut r = [0u8; 16];
    r[7] = 0xbf;
    let mut scc = [0u8; 144];
    let mut phase = [0f64; 8];
    let mut noise = 0x1ffffu32;
    let mut np = 0.;
    let mut env = 0.;
    let mut pcm = Vec::new();
    let mut loop_sample = None;
    let mut dc = 0.;
    let vol = [
        0., 0.0137, 0.0205, 0.0291, 0.0423, 0.0618, 0.0847, 0.1369, 0.1691, 0.2647, 0.3527, 0.4499,
        0.5704, 0.6873, 0.8482, 1.,
    ];
    for frame in 0..frames {
        if loop_frame == frame as i32 {
            loop_sample = Some(pcm.len());
        }
        let count = word(bytes, pos) as usize;
        pos += 2;
        for _ in 0..count {
            let reg = word(bytes, pos) as usize;
            let val = bytes[pos + 2];
            pos += 3;
            if reg >= 256 {
                if reg < 400 {
                    scc[reg - 256] = val;
                }
            } else {
                r[reg & 15] = val;
                if reg & 15 == 13 {
                    env = 0.;
                }
            }
        }
        let n = (frame + 1) * 22050 / 60 - frame * 22050 / 60;
        for _ in 0..n {
            np += 1789772.5 / (16. * (r[6] & 31).max(1) as f64) / 22050.;
            while np >= 1. {
                np -= 1.;
                noise = (noise >> 1) | (((noise ^ (noise >> 3)) & 1) << 16);
            }
            env += 1789772.5 / (16. * u16::from_le_bytes([r[11], r[12]]).max(1) as f64) / 22050.;
            let idx = env as usize;
            let cycle = idx / 16;
            let within = idx % 16;
            let shape = r[13];
            let up = shape & 4 != 0;
            let level = if shape < 8 {
                if cycle > 0 {
                    0
                } else if up {
                    within
                } else {
                    15 - within
                }
            } else if shape & 1 != 0 && cycle > 0 {
                if up ^ (shape & 2 != 0) {
                    15
                } else {
                    0
                }
            } else if up ^ ((shape & 2 != 0) && !cycle.is_multiple_of(2)) {
                within
            } else {
                15 - within
            };
            let mut value = 0.;
            for c in 0..3 {
                let period = (r[2 * c] as u16 + ((r[2 * c + 1] & 15) as u16) * 256).max(1);
                phase[c] = (phase[c] + 1789772.5 / (16. * period as f64) / 22050.) % 1.;
                let gate = (r[7] & (1 << c) != 0 || phase[c] < 0.5)
                    && (r[7] & (8 << c) != 0 || noise & 1 != 0);
                if gate {
                    value += vol[if r[8 + c] & 16 != 0 {
                        level
                    } else {
                        (r[8 + c] & 15) as usize
                    }] * 0.20;
                }
            }
            for c in 0..5 {
                let p = (scc[128 + 2 * c] as u16) + ((scc[129 + 2 * c] & 15) as u16) * 256;
                phase[c + 3] = (phase[c + 3] + 3579545. / (32. * (p as f64 + 1.)) / 22050.) % 1.;
                if scc[143] & (1 << c) != 0 {
                    let sample =
                        scc[c.min(3) * 32 + (phase[c + 3] * 32.) as usize] as i8 as f64 / 128.;
                    value += sample * (scc[138 + c] & 15) as f64 / 15. * 0.12;
                }
            }
            dc += 0.002 * (value - dc);
            pcm.push(((value - dc) * 26000.).clamp(-32767., 32767.) as i16);
        }
    }
    assert_eq!(pos, bytes.len());
    Rendered { pcm, loop_sample }
}
pub fn wav(pcm: &[i16]) -> Vec<u8> {
    let len = pcm.len() as u32 * 2;
    let mut b = vec![];
    b.extend(b"RIFF");
    b.extend((36 + len).to_le_bytes());
    b.extend(b"WAVEfmt ");
    b.extend(16u32.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    b.extend(22050u32.to_le_bytes());
    b.extend(44100u32.to_le_bytes());
    b.extend(2u16.to_le_bytes());
    b.extend(16u16.to_le_bytes());
    b.extend(b"data");
    b.extend(len.to_le_bytes());
    for s in pcm {
        b.extend(s.to_le_bytes());
    }
    b
}
struct Track {
    intro: Option<Sound>,
    body: Sound,
    intro_seconds: f64,
    looping: bool,
}
pub struct Audio {
    cache: HashMap<u8, Track>,
    pub enabled: bool,
    current: u8,
    started: f64,
    body_started: bool,
}
impl Audio {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            enabled: true,
            current: 0,
            started: 0.,
            body_started: false,
        }
    }
    async fn load(&mut self, id: u8) {
        if self.cache.contains_key(&id) {
            return;
        }
        let r = render(crate::asset(&format!("audio/{id:02x}.bin")));
        let split = r.loop_sample.filter(|&n| n > 0 && n < r.pcm.len());
        let intro = if let Some(n) = split {
            Some(load_sound_from_bytes(&wav(&r.pcm[..n])).await.unwrap())
        } else {
            None
        };
        let body = load_sound_from_bytes(&wav(&r.pcm[split.unwrap_or(0)..]))
            .await
            .unwrap();
        self.cache.insert(
            id,
            Track {
                intro,
                body,
                intro_seconds: split.unwrap_or(0) as f64 / 22050.,
                looping: r.loop_sample.is_some(),
            },
        );
    }
    pub async fn update(&mut self, id: u8, sounds: &[u8], time: f64) {
        if !self.enabled {
            self.stop();
            return;
        }
        if id != self.current {
            self.stop();
            self.load(id).await;
            self.current = id;
            self.started = time;
            self.body_started = false;
            if let Some(t) = self.cache.get(&id) {
                if let Some(s) = &t.intro {
                    play_sound(
                        s,
                        PlaySoundParams {
                            looped: false,
                            volume: 0.65,
                        },
                    );
                } else {
                    play_sound(
                        &t.body,
                        PlaySoundParams {
                            looped: t.looping,
                            volume: 0.65,
                        },
                    );
                    self.body_started = true;
                }
            }
        }
        if let Some(t) = self.cache.get(&self.current) {
            if !self.body_started && time - self.started >= t.intro_seconds {
                play_sound(
                    &t.body,
                    PlaySoundParams {
                        looped: t.looping,
                        volume: 0.65,
                    },
                );
                self.body_started = true;
            }
        }
        for &id in sounds {
            if id >= 0x28 {
                continue;
            }
            self.load(id).await;
            if let Some(t) = self.cache.get(&id) {
                play_sound(
                    &t.body,
                    PlaySoundParams {
                        looped: false,
                        volume: 0.45,
                    },
                );
            }
        }
    }
    pub fn stop(&mut self) {
        if let Some(t) = self.cache.get(&self.current) {
            if let Some(s) = &t.intro {
                stop_sound(s);
            }
            stop_sound(&t.body);
        }
        self.current = 0;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_audio_is_non_silent_and_bounded() {
        for id in [1, 2, 7, 0x29, 0x3e] {
            let r = render(crate::asset(&format!("audio/{id:02x}.bin")));
            assert!(r.pcm.iter().any(|v| v.abs() > 500));
            assert!(r.pcm.len() > 1000);
            let w = wav(&r.pcm);
            assert_eq!(&w[..4], b"RIFF");
            assert_eq!(w.len(), r.pcm.len() * 2 + 44);
        }
    }
}
