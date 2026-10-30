//! Native fixed-step simulation. No CPU, BIOS or ROM execution at runtime.
//! Original data: stage tables, hit points, collision classes, spawn positions.
//! Movement families and boss scripts are reconstructed and remain approximate.
use serde::Deserialize;
#[derive(Clone, Debug, Deserialize)]
pub struct Stage {
    pub width: usize,
    pub end: u32,
    pub checkpoint: u32,
    pub events: Vec<[u32; 3]>,
    pub generators: Vec<u8>,
    pub turrets: Vec<[u32; 2]>,
    pub music: Vec<[u32; 2]>,
    pub wave: u8,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Data {
    pub stages: Vec<Stage>,
    pub enemies: Vec<Vec<u8>>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct P {
    pub x: f32,
    pub y: f32,
}
impl P {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn distance(self, q: Self) -> f32 {
        ((self.x - q.x).powi(2) + (self.y - q.y).powi(2)).sqrt()
    }
}
#[derive(Default, Clone, Copy)]
pub struct Input {
    pub x: f32,
    pub y: f32,
    pub fire: bool,
    pub power: bool,
}
#[derive(Clone, Debug)]
pub struct Enemy {
    pub p: P,
    pub origin: P,
    pub kind: usize,
    pub age: u32,
    pub hp: i32,
    pub capsule: bool,
    pub turret: bool,
}
#[derive(Clone, Debug)]
pub struct Shot {
    pub p: P,
    pub v: P,
    pub friendly: bool,
    pub kind: u8,
    pub age: u32,
}
#[derive(Clone, Debug)]
pub struct Pickup {
    pub p: P,
    pub kind: u8,
}
#[derive(Clone, Debug)]
pub struct Fx {
    pub p: P,
    pub age: u32,
}
#[derive(Clone, Debug)]
pub struct Boss {
    pub kind: usize,
    pub p: P,
    pub hp: i32,
    pub max_hp: i32,
    pub age: u32,
    pub defeated: u32,
}
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Mode {
    Title,
    Playing,
    Dead,
    GameOver,
    Clear,
    Victory,
}
pub const CAMPAIGN: [usize; 14] = [1, 2, 3, 4, 5, 6, 7, 6, 5, 4, 3, 2, 1, 0];
pub struct Game {
    pub data: Data,
    pub masks: Vec<Vec<u8>>,
    pub mode: Mode,
    pub stage: usize,
    pub campaign: usize,
    pub practice: bool,
    pub scroll: f32,
    pub frame: u32,
    pub player: P,
    pub lives: i32,
    pub score: u32,
    pub high: u32,
    pub enemies: Vec<Enemy>,
    pub shots: Vec<Shot>,
    pub pickups: Vec<Pickup>,
    pub fx: Vec<Fx>,
    pub boss: Option<Boss>,
    pub options: usize,
    pub trail: Vec<P>,
    pub speed: u8,
    pub missile: bool,
    pub double: bool,
    pub laser: bool,
    pub shield: u8,
    pub meter: usize,
    pub invul: u32,
    pub timer: u32,
    pub shot_wait: u32,
    pub music: u8,
    pub sounds: Vec<u8>,
    pub god: bool,
    pub captured: u8,
    pub interior_return: Option<usize>,
    last_col: i32,
}
impl Game {
    pub fn new(data: Data, masks: Vec<Vec<u8>>) -> Self {
        Self {
            data,
            masks,
            mode: Mode::Title,
            stage: 1,
            campaign: 0,
            practice: false,
            scroll: 0.,
            frame: 0,
            player: P::new(80., 74.),
            lives: 3,
            score: 0,
            high: 0,
            enemies: vec![],
            shots: vec![],
            pickups: vec![],
            fx: vec![],
            boss: None,
            options: 0,
            trail: vec![P::new(80., 74.); 64],
            speed: 0,
            missile: false,
            double: false,
            laser: false,
            shield: 0,
            meter: 0,
            invul: 180,
            timer: 0,
            shot_wait: 0,
            music: 0x3e,
            sounds: vec![],
            god: false,
            captured: 0,
            interior_return: None,
            last_col: -1,
        }
    }
    pub fn start(&mut self) {
        self.campaign = 0;
        self.practice = false;
        self.lives = 3;
        self.score = 0;
        self.captured = 0;
        self.reset_power();
        self.load(1, 0.);
        self.mode = Mode::Playing;
    }
    pub fn load(&mut self, stage: usize, scroll: f32) {
        self.stage = stage;
        self.scroll = scroll;
        self.last_col = (scroll / 8.) as i32 - 1;
        self.player = P::new(64., 80.);
        self.trail = vec![self.player; 64];
        self.enemies.clear();
        self.shots.clear();
        self.pickups.clear();
        self.fx.clear();
        self.boss = None;
        self.invul = 180;
        self.timer = 0;
        self.shot_wait = 0;
        self.music = 0x2b;
    }
    fn reset_power(&mut self) {
        self.speed = 0;
        self.missile = false;
        self.double = false;
        self.laser = false;
        self.options = 0;
        self.shield = 0;
        self.meter = 0;
    }
    pub fn solid(&self, x: f32, y: f32) -> bool {
        if !(0. ..176.).contains(&y) || x < 0. {
            return false;
        }
        let stage = &self.data.stages[self.stage];
        let c = (x / 8.) as usize;
        let r = (y / 8.) as usize;
        c < stage.width && self.masks[self.stage][r * stage.width + c] != 0
    }
    pub fn power(&mut self) {
        if self.meter == 0 {
            return;
        }
        match self.meter {
            1 => self.speed = (self.speed + 1).min(5),
            2 => self.missile = true,
            3 => {
                self.double = true;
                self.laser = false
            }
            4 => {
                self.laser = true;
                self.double = false
            }
            5 => self.options = (self.options + 1).min(2),
            6 => self.shield = 10,
            _ => {}
        }
        self.meter = 0;
        self.sounds.push(0x0a);
    }
    fn spawn(&mut self, kind: usize, y: f32, capsule: bool, turret: bool, x: f32) {
        if !(1..66).contains(&kind) || self.enemies.len() >= 32 {
            return;
        }
        let hp = (self.data.enemies[kind][1] & 0x7f).max(1) as i32;
        self.enemies.push(Enemy {
            p: P::new(x, y.clamp(0., 160.)),
            origin: P::new(x, y),
            kind,
            age: 0,
            hp: hp.min(12),
            capsule,
            turret,
        });
    }
    fn column(&mut self, col: u32) {
        let st = self.data.stages[self.stage].clone();
        for [pos, t, y] in st.events {
            if pos & 1023 == col {
                self.spawn((t & 63) as usize, y as f32, t & 0xc0 != 0, false, 248.);
            }
        }
        for [pos, flags] in st.turrets {
            if pos == col {
                self.spawn(1, ((flags & 31) * 8) as f32, false, true, 248.);
            }
        }
        for [pos, track] in st.music {
            if pos == col {
                self.music = track as u8;
            }
        }
    }
    fn shoot(&mut self) {
        let mut origins = vec![self.player];
        for i in 0..self.options {
            origins.push(self.trail[(i + 1) * 16]);
        }
        for p in origins {
            self.shots.push(Shot {
                p: P::new(p.x + 14., p.y + 8.),
                v: P::new(if self.laser { 7. } else { 4. }, 0.),
                friendly: true,
                kind: if self.laser { 1 } else { 0 },
                age: 0,
            });
            if self.double {
                self.shots.push(Shot {
                    p: P::new(p.x + 8., p.y + 4.),
                    v: P::new(3., -3.),
                    friendly: true,
                    kind: 0,
                    age: 0,
                });
            }
            if self.missile {
                self.shots.push(Shot {
                    p: P::new(p.x + 8., p.y + 14.),
                    v: P::new(1., 1.8),
                    friendly: true,
                    kind: 2,
                    age: 0,
                });
            }
            if self.captured > 0 {
                self.shots.push(Shot {
                    p: P::new(p.x, p.y + 8.),
                    v: P::new(-4., 0.),
                    friendly: true,
                    kind: 0,
                    age: 0,
                });
            }
        }
        self.sounds.push(if self.laser { 0x16 } else { 1 });
    }
    fn damage(&mut self) {
        if self.invul > 0 || self.god {
            return;
        }
        if self.shield > 0 {
            self.shield -= 1;
            self.invul = 30;
            return;
        }
        self.fx.push(Fx {
            p: self.player,
            age: 0,
        });
        self.sounds.push(2);
        self.lives -= 1;
        self.reset_power();
        self.mode = Mode::Dead;
        self.timer = 100;
    }
    pub fn enter_interior(&mut self) {
        if self.stage != 8 {
            self.interior_return = Some(self.stage);
            self.load(8, 0.);
            self.mode = Mode::Playing;
        }
    }
    fn finish(&mut self) {
        self.score += 5000;
        self.high = self.high.max(self.score);
        self.mode = Mode::Clear;
        self.timer = 180;
        self.music = 0x3b;
    }
    pub fn tick(&mut self, input: Input) {
        self.frame += 1;
        self.sounds.clear();
        for f in &mut self.fx {
            f.age += 1;
        }
        self.fx.retain(|f| f.age < 24);
        match self.mode {
            Mode::Dead => {
                self.timer = self.timer.saturating_sub(1);
                if self.timer == 0 {
                    if self.lives <= 0 {
                        self.mode = Mode::GameOver;
                        self.music = 0x3d;
                    } else {
                        let checkpoint = self.data.stages[self.stage].checkpoint as f32 * 8.;
                        let back = if self.scroll > checkpoint {
                            checkpoint
                        } else {
                            0.
                        };
                        self.load(self.stage, back);
                        self.mode = Mode::Playing;
                    }
                }
                return;
            }
            Mode::Clear => {
                self.timer = self.timer.saturating_sub(1);
                if self.timer == 0 {
                    if self.practice {
                        self.mode = Mode::Title;
                        self.music = 0x3e;
                        return;
                    }
                    if self.stage == 8 {
                        self.captured = (self.captured + 1).min(7);
                        self.interior_return = None;
                    }
                    self.campaign += 1;
                    if self.campaign >= CAMPAIGN.len() {
                        self.mode = Mode::Victory;
                        self.music = 0x41;
                    } else {
                        self.load(CAMPAIGN[self.campaign], 0.);
                        self.mode = Mode::Playing;
                    }
                }
                return;
            }
            Mode::Playing => {}
            _ => return,
        }
        self.invul = self.invul.saturating_sub(1);
        self.shot_wait = self.shot_wait.saturating_sub(1);
        if input.power {
            self.power();
        }
        let old = self.player;
        let step = (self.speed as f32 + 4.) * 0.5;
        self.player.x = (self.player.x + input.x * step).clamp(0., 232.);
        self.player.y = (self.player.y + input.y * step).clamp(0., 158.);
        if self.player != old {
            self.trail.insert(0, old);
            self.trail.truncate(64);
        }
        if input.fire && self.shot_wait == 0 {
            self.shoot();
            self.shot_wait = 15;
        }
        let limit = self.data.stages[self.stage].end as f32 * 8.;
        let ds = if self.scroll < limit { 0.5 } else { 0. };
        self.scroll = (self.scroll + ds).min(limit);
        let col = (self.scroll / 8.) as i32;
        if col > self.last_col {
            self.column(col as u32);
            self.last_col = col;
        }
        if self.boss.is_none() && self.scroll >= limit {
            if self.stage == 6 || self.stage >= 9 {
                self.finish();
                return;
            }
            let kind = match self.stage {
                1 => 1,
                2 | 3 => 4,
                4 | 5 => 2,
                7 => 3,
                _ => 0,
            };
            let hp = if self.stage == 8 {
                24
            } else if kind == 0 {
                100
            } else {
                [0, 18, 35, 56, 27][kind]
            };
            self.boss = Some(Boss {
                kind,
                p: P::new(172., 40.),
                hp,
                max_hp: hp,
                age: 0,
                defeated: 0,
            });
            self.music = if self.stage == 8 { 0x35 } else { 0x34 };
        }
        if self.boss.is_none() && self.frame.is_multiple_of(96) {
            let g = self.data.stages[self.stage]
                .generators
                .get(col as usize / 32)
                .copied()
                .unwrap_or(0)
                >> 4;
            let kind = [0, 4, 5, 9, 13, 15, 17, 20, 22, 25, 12, 8]
                .get(g as usize)
                .copied()
                .unwrap_or(0);
            if kind > 0 {
                self.spawn(
                    kind,
                    32. + ((self.frame / 96 * 37) % 105) as f32,
                    false,
                    false,
                    248.,
                );
            }
        }
        if self.boss.is_none() && self.frame.is_multiple_of(320) {
            let kind = self.data.stages[self.stage].wave as usize;
            for i in 0..6 {
                self.spawn(
                    kind,
                    48. + ((self.frame / 320) % 2) as f32 * 64.,
                    true,
                    false,
                    248. + i as f32 * 20.,
                );
            }
        }
        let mut incoming = vec![];
        for e in &mut self.enemies {
            e.age += 1;
            if e.turret {
                e.p.x -= ds;
            } else {
                match e.kind {
                    4 => {
                        e.p.x -= 1.5;
                    }
                    5 | 9 | 13 | 15 | 17 | 20 => {
                        e.p.x -= 1.0;
                        e.p.y = e.origin.y + (e.age as f32 * 0.06).sin() * 24.;
                    }
                    3 | 7 | 14 | 25 | 31 => {
                        e.p.x -= 1.2;
                        e.p.y += (self.player.y - e.p.y).signum() * 0.35;
                    }
                    _ => {
                        e.p.x -= 0.8;
                        e.p.y = e.origin.y + (e.age as f32 * 0.04).sin() * 12.;
                    }
                }
            }
            if e.age.is_multiple_of(if e.turret { 90 } else { 150 })
                && e.p.x > self.player.x
                && e.p.x < 252.
            {
                let d = P::new(self.player.x - e.p.x, self.player.y - e.p.y);
                let len = (d.x * d.x + d.y * d.y).sqrt().max(1.);
                incoming.push(Shot {
                    p: P::new(e.p.x + 8., e.p.y + 8.),
                    v: P::new(d.x / len * 1.5, d.y / len * 1.5),
                    friendly: false,
                    kind: 0,
                    age: 0,
                });
            }
        }
        if let Some(b) = &mut self.boss {
            b.age += 1;
            if b.hp > 0 {
                b.p.y = 40. + (b.age as f32 * 0.02).sin() * 24.;
                if b.age.is_multiple_of(55) {
                    for n in -2..=2 {
                        incoming.push(Shot {
                            p: P::new(b.p.x, b.p.y + 40.),
                            v: P::new(-1.8, n as f32 * 0.45),
                            friendly: false,
                            kind: 0,
                            age: 0,
                        });
                    }
                }
            } else {
                b.defeated += 1;
            }
        }
        self.shots.extend(incoming);
        let mut killed = vec![];
        let mut hit = false;
        let stage = self.stage;
        let width = self.data.stages[stage].width;
        let mask = &self.masks[stage];
        let scroll = self.scroll;
        for s in &mut self.shots {
            s.age += 1;
            s.p.x += s.v.x;
            s.p.y += s.v.y;
            if s.friendly {
                let tx = ((scroll + s.p.x) / 8.) as usize;
                let ty = (s.p.y / 8.) as usize;
                if s.kind != 2 && ty < 22 && tx < width && mask[ty * width + tx] != 0 {
                    s.age = 999;
                    continue;
                }
                if s.kind == 2 && s.p.y < 172. {
                    let x = ((scroll + s.p.x + 4.) / 8.) as usize;
                    let y = ((s.p.y + 3.) / 8.) as usize;
                    if y < 22 && x < width && mask[y * width + x] != 0 {
                        s.v.y = 0.;
                        s.v.x = 2.;
                    }
                }
                for (i, e) in self.enemies.iter_mut().enumerate() {
                    if e.hp > 0
                        && (s.p.x - e.p.x - 8.).abs() < 12.
                        && (s.p.y - e.p.y - 8.).abs() < 10.
                    {
                        e.hp -= if s.kind == 1 { 2 } else { 1 };
                        s.age = 999;
                        if e.hp <= 0 {
                            killed.push((i, e.p, e.capsule));
                        }
                        break;
                    }
                }
                if let Some(b) = &mut self.boss {
                    if b.hp > 0
                        && s.p.x > b.p.x - 4.
                        && s.p.x < b.p.x + 80.
                        && (s.p.y - b.p.y - 40.).abs() < 20.
                    {
                        b.hp -= 1;
                        s.age = 999;
                    }
                }
            } else if s.p.distance(P::new(self.player.x + 8., self.player.y + 8.)) < 7. {
                hit = true;
                s.age = 999;
            }
        }
        self.shots.retain(|s| {
            s.age < 300 && s.p.x > -16. && s.p.x < 300. && s.p.y > -16. && s.p.y < 184.
        });
        for (_, p, capsule) in killed {
            self.score += 100;
            self.high = self.high.max(self.score);
            self.fx.push(Fx { p, age: 0 });
            self.sounds.push(3);
            if capsule {
                self.pickups.push(Pickup { p, kind: 0 });
            }
        }
        self.enemies
            .retain(|e| e.hp > 0 && e.p.x > -24. && e.age < 1200);
        for p in &mut self.pickups {
            p.p.x -= ds + 0.2;
            if p.p.distance(self.player) < 15. {
                if p.kind == 0 {
                    self.meter = self.meter % 6 + 1;
                } else {
                    self.enemies.clear();
                }
                self.sounds.push(7);
                p.p.x = -100.;
            }
        }
        self.pickups.retain(|p| p.p.x > -16.);
        if self.enemies.iter().any(|e| e.p.distance(self.player) < 11.) {
            hit = true;
        }
        if self.solid(self.scroll + self.player.x + 6., self.player.y + 8.)
            || self.solid(self.scroll + self.player.x + 12., self.player.y + 8.)
        {
            hit = true;
        }
        if self.boss.as_ref().is_some_and(|b| {
            b.hp > 0 && self.player.x + 12. > b.p.x && (self.player.y - b.p.y - 32.).abs() < 35.
        }) {
            hit = true;
        }
        if hit {
            self.damage();
        }
        if let Some(b) = &self.boss {
            if b.hp <= 0 {
                if self.stage != 8
                    && self.stage != 0
                    && b.defeated < 300
                    && self.player.x > b.p.x - 10.
                    && (self.player.y - b.p.y - 32.).abs() < 32.
                {
                    self.enter_interior();
                } else if b.defeated >= 300 || self.stage == 8 || self.stage == 0 {
                    self.finish();
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn game() -> Game {
        let data: Data = serde_json::from_slice(include_bytes!("../assets/game.json")).unwrap();
        let masks = (0..12)
            .map(|i| crate::asset(&format!("collision-{i:02}.bin")).to_vec())
            .collect();
        Game::new(data, masks)
    }
    #[test]
    fn power_meter_and_option_limit() {
        let mut g = game();
        g.start();
        for _ in 0..4 {
            g.meter = 5;
            g.power();
        }
        assert_eq!(g.options, 2);
        g.meter = 6;
        g.power();
        assert_eq!(g.shield, 10);
        g.meter = 4;
        g.power();
        assert!(g.laser && !g.double);
    }
    #[test]
    fn terrain_dimensions_and_collision() {
        let mut g = game();
        for i in 0..12 {
            g.stage = i;
            assert_eq!(g.masks[i].len(), g.data.stages[i].width * 22);
            assert!(g.masks[i].contains(&1));
            assert!(g.masks[i].contains(&0));
        }
    }
    #[test]
    fn fixed_tick_fire_and_respawn() {
        let mut g = game();
        g.start();
        g.god = true;
        for _ in 0..30 {
            g.tick(Input {
                fire: true,
                ..Input::default()
            });
        }
        assert_eq!(g.shots.iter().filter(|s| s.friendly).count(), 2);
        g.god = false;
        g.invul = 0;
        g.damage();
        assert_eq!(g.lives, 2);
        for _ in 0..100 {
            g.tick(Input::default());
        }
        assert_eq!(g.mode, Mode::Playing);
        assert!(g.invul > 0);
    }
    #[test]
    fn campaign_has_return_trip_and_fortress() {
        assert_eq!(&CAMPAIGN[..7], &[1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(&CAMPAIGN[7..], &[6, 5, 4, 3, 2, 1, 0]);
    }
}

#[cfg(test)]
mod progression_tests {
    use super::*;
    #[test]
    fn all_campaign_transitions_reach_victory() {
        let data: Data = serde_json::from_slice(crate::asset("game.json")).unwrap();
        let masks = (0..12)
            .map(|i| crate::asset(&format!("collision-{i:02}.bin")).to_vec())
            .collect();
        let mut g = Game::new(data, masks);
        g.start();
        g.god = true;
        for expected in CAMPAIGN {
            assert_eq!(g.stage, expected);
            let end = g.data.stages[g.stage].end as f32 * 8.;
            g.scroll = end;
            g.tick(Input::default());
            if let Some(b) = &mut g.boss {
                b.hp = 0;
                b.defeated = 299;
            }
            g.player = P::new(0., 0.);
            if g.mode == Mode::Playing {
                g.tick(Input::default());
            }
            assert_eq!(g.mode, Mode::Clear);
            for _ in 0..180 {
                g.tick(Input::default());
            }
        }
        assert_eq!(g.mode, Mode::Victory);
    }
}
