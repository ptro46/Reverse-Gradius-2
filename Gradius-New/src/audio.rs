//! Original score and synthesizer written for ASTRA FRACTURE.
//! No samples, note data, driver or melodies from Gradius are used.
use crate::core::SoundEvent;
use macroquad::audio::{
    load_sound_from_bytes, play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound,
};
use std::{collections::HashMap, io::Cursor, path::Path};
const RATE: u32 = 44100;
const TAU: f32 = std::f32::consts::TAU;
pub const TRACKS: [&str; 5] = ["menu", "sector1", "sector2", "sector3", "boss"];
pub const EFFECTS: [&str; 13] = [
    "shot",
    "beam",
    "missile",
    "hit",
    "explosion",
    "pickup",
    "upgrade",
    "dash",
    "overdrive",
    "boss",
    "clear",
    "death",
    "select",
];
fn freq(note: f32) -> f32 {
    440. * 2f32.powf((note - 69.) / 12.)
}
fn envelope(t: f32, duration: f32, attack: f32) -> f32 {
    (t / attack).min(1.) * ((duration - t) / 0.07).clamp(0., 1.)
}
fn noise(seed: &mut u32) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    (*seed as f32 / u32::MAX as f32) * 2. - 1.
}
fn note(out: &mut [[f32; 2]], at: f32, duration: f32, midi: f32, gain: f32, voice: u8, pan: f32) {
    let first = (at * RATE as f32) as usize;
    let count = (duration * RATE as f32) as usize;
    let f = freq(midi);
    for i in 0..count.min(out.len().saturating_sub(first)) {
        let t = i as f32 / RATE as f32;
        let p = t * f * TAU;
        let value = match voice {
            0 => (p.sin() + 0.22 * (p * 2.).sin() + 0.1 * (p * 3.).sin()) * (-t * 3.).exp(),
            1 => {
                (p.sin() * 0.75 + (p * 1.003).sin() * 0.18 + (p * 2.).sin() * 0.1)
                    * (1. - (-t * 3.).exp())
            }
            2 => (p + (p * 2.).sin() * 2.4 * (-t * 7.).exp()).sin() * (-t * 8.).exp(),
            _ => {
                (p.sin() * 0.75 + (p * 2.).sin() * 0.15 + (p * 3.).sin() * 0.08) * (-t * 0.7).exp()
            }
        };
        let e = envelope(t, duration, if voice == 1 { 0.18 } else { 0.008 });
        let s = value * gain * e;
        out[first + i][0] += s * (0.75 - pan * 0.25);
        out[first + i][1] += s * (0.75 + pan * 0.25);
    }
}
fn drum(out: &mut [[f32; 2]], at: f32, kind: u8, gain: f32, seed: &mut u32) {
    let first = (at * RATE as f32) as usize;
    let duration = if kind == 0 {
        0.32
    } else if kind == 1 {
        0.17
    } else {
        0.055
    };
    let mut phase = 0.;
    let mut previous = 0.;
    for i in 0..((duration * RATE as f32) as usize).min(out.len().saturating_sub(first)) {
        let t = i as f32 / RATE as f32;
        let n = noise(seed);
        let v = match kind {
            0 => {
                phase += (45. + 110. * (-t * 34.).exp()) * TAU / RATE as f32;
                phase.sin() * (-t * 14.).exp() + n * (-t * 180.).exp() * 0.18
            }
            1 => (n * 0.7 + (t * 180. * TAU).sin() * 0.3) * (-t * 24.).exp(),
            _ => {
                let h = n - previous;
                previous = n;
                h * (-t * 65.).exp() * 0.3
            }
        } * gain;
        out[first + i][0] += v;
        out[first + i][1] += v;
    }
}
pub fn music(track: usize, bars: usize) -> Vec<[i16; 2]> {
    let bpm = [96., 124., 132., 140., 148.][track];
    let beat = 60. / bpm;
    let total = bars as f32 * 4. * beat;
    let mut out = vec![[0.; 2]; (total * RATE as f32) as usize];
    let roots: [f32; 4] = match track {
        0 => [50., 46., 53., 48.],
        1 => [50., 46., 53., 48.],
        2 => [45., 41., 48., 43.],
        3 => [52., 48., 55., 50.],
        _ => [47., 43., 50., 45.],
    };
    let motifs: [[f32; 8]; 5] = [
        [0., 7., 10., 14., 12., 7., 3., 5.],
        [12., 7., 10., 14., 12., 3., 7., 5.],
        [0., 3., 7., 12., 10., 7., 5., 3.],
        [12., 19., 15., 14., 10., 14., 7., 10.],
        [0., 7., 12., 10., 14., 7., 3., 10.],
    ];
    let mut seed = 0xA51732u32 + track as u32;
    for bar in 0..bars {
        let root = roots[(bar / 2) % 4];
        let at = bar as f32 * 4. * beat;
        for (i, interval) in [0., 3., 7., 10.].iter().enumerate() {
            note(
                &mut out,
                at,
                beat * 4.,
                root + 12. + interval,
                0.048,
                1,
                (i as f32 - 1.5) * 0.45,
            );
        }
        for step in 0..8 {
            let when = at + step as f32 * beat * 0.5;
            let bass = if step % 4 == 3 { root - 5. } else { root - 12. };
            note(
                &mut out,
                when,
                beat * 0.42,
                bass,
                if track == 0 { 0.075 } else { 0.12 },
                0,
                0.,
            );
            if (bar % 4 >= 1 || track == 0) && step % 2 == 0 {
                let melody = root + motifs[track][(step + (bar % 2) * 3) % 8];
                note(
                    &mut out,
                    when,
                    beat * 0.8,
                    melody,
                    0.075,
                    2,
                    if step % 4 == 0 { -0.3 } else { 0.3 },
                );
            }
            if track != 0 {
                drum(&mut out, when, 2, 0.08, &mut seed);
                if step % 2 == 0 {
                    drum(&mut out, when, 0, 0.27, &mut seed);
                }
                if step == 2 || step == 6 {
                    drum(&mut out, when, 1, 0.15, &mut seed);
                }
            }
        }
        if bar % 4 >= 2 {
            for step in 0..16 {
                let interval = [0., 7., 12., 15.][step % 4];
                note(
                    &mut out,
                    at + step as f32 * beat / 4.,
                    beat * 0.23,
                    root + 24. + interval,
                    0.025,
                    2,
                    if step % 2 == 0 { -0.65 } else { 0.65 },
                );
            }
        }
    }
    // Stereo cross-feed delay, then soft limiting and short boundary fades.
    let delay = (beat * 0.75 * RATE as f32) as usize;
    for i in delay..out.len() {
        out[i][0] += out[i - delay][1] * 0.15;
        out[i][1] += out[i - delay][0] * 0.14;
    }
    let len = out.len();
    out.into_iter()
        .enumerate()
        .map(|(i, s)| {
            let fade = (i as f32 / 200.).min(1.) * ((len - i) as f32 / 400.).min(1.);
            [
                (s[0] * 1.4).tanh().mul_add(27000. * fade, 0.) as i16,
                (s[1] * 1.4).tanh().mul_add(27000. * fade, 0.) as i16,
            ]
        })
        .collect()
}
pub fn effect(index: usize) -> Vec<[i16; 2]> {
    let duration = [
        0.13, 0.19, 0.38, 0.22, 0.8, 0.34, 0.9, 0.45, 1.5, 1.8, 2.8, 1.6, 0.18,
    ][index];
    let mut out = vec![[0.; 2]; (duration * RATE as f32) as usize];
    let mut seed = 0xEFFF01 + index as u32;
    let mut phase = 0.;
    let mut low = 0.;
    for (i, s) in out.iter_mut().enumerate() {
        let t = i as f32 / RATE as f32;
        let n = noise(&mut seed);
        low += 0.06 * (n - low);
        let (frequency, amp) = match index {
            0 => (1250. * (-t * 12.).exp() + 220., (-t * 24.).exp() * 0.23),
            1 => (1100. + 700. * (t * 90.).sin(), (-t * 18.).exp() * 0.11),
            2 => (140. + t * 800., (-t * 7.).exp() * 0.2),
            3 => (130. + 200. * (-t * 30.).exp(), (-t * 14.).exp() * 0.35),
            4 | 11 => (38. + 70. * (-t * 8.).exp(), (-t * 4.).exp() * 0.5),
            5 | 6 | 12 => (
                freq(72. + ((t * 18.).floor() % 4.) * 3.),
                (-t * 4.).exp() * 0.22,
            ),
            7 => (250. + t * 2600., (-t * 10.).exp() * 0.24),
            8 | 9 => (60. + t * 180., (-t * 2.).exp() * 0.3),
            _ => (
                freq(62. + ((t * 4.).floor() % 4.) * 3.),
                (-t * 1.2).exp() * 0.25,
            ),
        };
        phase += frequency * TAU / RATE as f32;
        let tonal = phase.sin();
        let v = if matches!(index, 3 | 4 | 11) {
            (low * 3. + tonal * 0.3) * amp
        } else if index == 7 {
            (n * 0.4 + tonal * 0.6) * amp
        } else {
            (tonal + 0.2 * (phase * 2.).sin()) * amp
        };
        let e = ((duration - t) / 0.02).clamp(0., 1.) * (t / 0.001).min(1.);
        *s = [v * e, v * e];
    }
    out.into_iter()
        .map(|s| {
            [
                (s[0] * 30000.).clamp(-32760., 32760.) as i16,
                (s[1] * 30000.).clamp(-32760., 32760.) as i16,
            ]
        })
        .collect()
}
pub fn wav(samples: &[[i16; 2]]) -> Vec<u8> {
    let mut cursor = Cursor::new(Vec::new());
    {
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: RATE,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::new(&mut cursor, spec).unwrap();
        for frame in samples {
            writer.write_sample(frame[0]).unwrap();
            writer.write_sample(frame[1]).unwrap();
        }
        writer.finalize().unwrap();
    }
    cursor.into_inner()
}
pub fn compose(directory: &Path) {
    std::fs::create_dir_all(directory).unwrap();
    for (id, name) in TRACKS.iter().enumerate() {
        std::fs::write(directory.join(format!("{name}.wav")), wav(&music(id, 16))).unwrap();
        println!("Composed: {name}");
    }
    for (id, name) in EFFECTS.iter().enumerate() {
        std::fs::write(directory.join(format!("sfx-{name}.wav")), wav(&effect(id))).unwrap();
    }
    println!("13 original sound effects exported");
}
pub struct Audio {
    tracks: HashMap<usize, Sound>,
    effects: Vec<Sound>,
    pub enabled: bool,
    current: usize,
    previous: Option<usize>,
    fade: f32,
    cooldown: [f32; 13],
}
impl Audio {
    pub async fn new() -> Self {
        let mut effects = vec![];
        for name in EFFECTS {
            effects.push(
                load_sound_from_bytes(crate::asset(&format!("audio/sfx-{name}.wav")))
                    .await
                    .unwrap(),
            );
        }
        Self {
            tracks: HashMap::new(),
            effects,
            enabled: true,
            current: 99,
            previous: None,
            fade: 1.,
            cooldown: [0.; 13],
        }
    }
    pub async fn update(&mut self, track: usize, events: &[SoundEvent], dt: f32) {
        for c in &mut self.cooldown {
            *c = (*c - dt).max(0.);
        }
        if !self.enabled {
            for s in self.tracks.values() {
                stop_sound(s);
            }
            for sound in &self.effects {
                stop_sound(sound);
            }
            self.current = 99;
            self.previous = None;
            return;
        }
        if track != self.current {
            if let Some(old) = self.previous.take() {
                if let Some(s) = self.tracks.get(&old) {
                    stop_sound(s);
                }
            }
            if let std::collections::hash_map::Entry::Vacant(entry) = self.tracks.entry(track) {
                let s =
                    load_sound_from_bytes(crate::asset(&format!("audio/{}.wav", TRACKS[track])))
                        .await
                        .unwrap();
                entry.insert(s);
            }
            if self.current != 99 {
                self.previous = Some(self.current);
            }
            self.current = track;
            self.fade = 0.;
            play_sound(
                &self.tracks[&track],
                PlaySoundParams {
                    looped: true,
                    volume: 0.,
                },
            );
        }
        self.fade = (self.fade + dt * 1.5).min(1.);
        set_sound_volume(&self.tracks[&track], self.fade * 0.64);
        if let Some(old) = self.previous {
            set_sound_volume(&self.tracks[&old], (1. - self.fade) * 0.64);
            if self.fade >= 1. {
                stop_sound(&self.tracks[&old]);
                self.previous = None;
            }
        }
        for event in events {
            let id = *event as usize;
            if self.cooldown[id] > 0. {
                continue;
            }
            self.cooldown[id] = if id <= 2 { 0.075 } else { 0.08 };
            play_sound(
                &self.effects[id],
                PlaySoundParams {
                    looped: false,
                    volume: if id == 0 {
                        0.28
                    } else if id == 1 {
                        0.2
                    } else {
                        0.65
                    },
                },
            );
        }
    }
    pub fn stop(&self) {
        for s in self.tracks.values() {
            stop_sound(s);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_tracks_and_effects_are_valid_pcm() {
        for id in 0..5 {
            let s = music(id, 1);
            assert!(s.len() > 60000);
            assert!(s.iter().any(|p| p[0].abs() > 1000));
            assert!(s.iter().any(|p| p[0] != p[1]));
            let w = wav(&s);
            assert_eq!(&w[..4], b"RIFF");
        }
        for id in 0..13 {
            assert!(effect(id).iter().any(|p| p[0].abs() > 100));
        }
    }
}
