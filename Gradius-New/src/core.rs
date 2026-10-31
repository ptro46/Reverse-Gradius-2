//! Original deterministic 120 Hz simulation. No ROM, emulator or legacy asset dependency.
use std::ops::{Add, AddAssign, Mul, Sub};
pub const W: f32 = 1600.;
pub const H: f32 = 900.;
pub const DT: f32 = 1. / 120.;
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V {
    pub x: f32,
    pub y: f32,
}
impl V {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn len(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn norm(self) -> Self {
        self * (1. / self.len().max(0.0001))
    }
    pub fn dist(self, b: Self) -> f32 {
        (self - b).len()
    }
    pub fn angle(a: f32) -> Self {
        Self::new(a.cos(), a.sin())
    }
}
impl Add for V {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        V::new(self.x + b.x, self.y + b.y)
    }
}
impl Sub for V {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        V::new(self.x - b.x, self.y - b.y)
    }
}
impl Mul<f32> for V {
    type Output = Self;
    fn mul(self, s: f32) -> Self {
        V::new(self.x * s, self.y * s)
    }
}
impl AddAssign for V {
    fn add_assign(&mut self, b: Self) {
        *self = *self + b;
    }
}
pub fn segment_distance(a: V, b: V, p: V) -> f32 {
    let d = b - a;
    let t = (((p.x - a.x) * d.x + (p.y - a.y) * d.y) / (d.x * d.x + d.y * d.y).max(0.0001))
        .clamp(0., 1.);
    (a + d * t).dist(p)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Title,
    Flight,
    BossIntro,
    Boss,
    Dying,
    Clear,
    GameOver,
    Victory,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weapon {
    Pulse,
    Spread,
    Beam,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Scout,
    Weaver,
    Lancer,
    Carrier,
    Turret,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundEvent {
    Shot,
    Beam,
    Missile,
    Hit,
    Explosion,
    Pickup,
    Upgrade,
    Dash,
    Overdrive,
    Boss,
    Clear,
    Death,
    Select,
}
#[derive(Clone, Copy, Default)]
pub struct Input {
    pub axis: V,
    pub fire: bool,
    pub focus: bool,
    pub dash: bool,
    pub overdrive: bool,
    pub upgrade: bool,
    pub weapon: Option<Weapon>,
}
#[derive(Clone, Copy, Debug)]
pub struct Sector {
    pub name: &'static str,
    pub subtitle: &'static str,
    pub boss: &'static str,
    pub duration: f32,
}
pub const SECTORS: [Sector; 3] = [
    Sector {
        name: "LUNE BRISEE",
        subtitle: "Les debris d'un monde silencieux",
        boss: "LE VEILLEUR",
        duration: 84.,
    },
    Sector {
        name: "COURONNE DE CENDRES",
        subtitle: "Au coeur des forges heliques",
        boss: "SOLSTICE",
        duration: 90.,
    },
    Sector {
        name: "LA NEF DE VERRE",
        subtitle: "Le dernier signal de l'Architecte",
        boss: "LE CHOEUR",
        duration: 96.,
    },
];
#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub p: V,
    pub vel: V,
    pub hull: i32,
    pub lives: i32,
    pub shield: i32,
    pub speed: u8,
    pub spread: u8,
    pub beam: u8,
    pub missiles: u8,
    pub drones: u8,
    pub weapon: Weapon,
    pub energy: f32,
    pub invul: f32,
    pub dash: f32,
    pub dash_cooldown: f32,
    pub overload: f32,
    pub shot_wait: f32,
    pub missile_wait: f32,
    pub roll: f32,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            p: V::new(255., 450.),
            vel: V::default(),
            hull: 3,
            lives: 3,
            shield: 0,
            speed: 0,
            spread: 0,
            beam: 0,
            missiles: 0,
            drones: 0,
            weapon: Weapon::Pulse,
            energy: 25.,
            invul: 2.,
            dash: 0.,
            dash_cooldown: 0.,
            overload: 0.,
            shot_wait: 0.,
            missile_wait: 0.,
            roll: 0.,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Enemy {
    pub p: V,
    pub origin: V,
    pub vel: V,
    pub kind: Kind,
    pub hp: f32,
    pub radius: f32,
    pub age: f32,
    pub fire: f32,
    pub seed: u32,
    pub elite: bool,
}
#[derive(Clone, Debug)]
pub struct Bullet {
    pub p: V,
    pub old: V,
    pub vel: V,
    pub friendly: bool,
    pub kind: u8,
    pub radius: f32,
    pub damage: f32,
    pub age: f32,
    pub grazed: bool,
}
#[derive(Clone, Debug)]
pub struct Pickup {
    pub p: V,
    pub age: f32,
    pub kind: u8,
    pub phase: f32,
}
#[derive(Clone, Debug)]
pub struct Rock {
    pub p: V,
    pub radius: f32,
    pub hp: f32,
    pub angle: f32,
    pub seed: u32,
}
#[derive(Clone, Debug)]
pub struct Gate {
    pub x: f32,
    pub gap: f32,
    pub age: f32,
    pub fired: bool,
}
#[derive(Clone, Debug)]
pub struct Fx {
    pub p: V,
    pub age: f32,
    pub size: f32,
    pub kind: u8,
    pub seed: u32,
}
#[derive(Clone, Debug)]
pub struct Boss {
    pub p: V,
    pub hp: f32,
    pub max: f32,
    pub age: f32,
    pub shoot: f32,
    pub volley: u32,
    pub phase: u8,
    pub hit: f32,
}
impl Boss {
    pub fn exposed(&self) -> bool {
        self.age % 7. > 1.4
    }
    pub fn core_radius(&self) -> f32 {
        if self.phase == 3 {
            48.
        } else {
            40.
        }
    }
}
#[derive(Clone, Debug)]
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }
    pub fn next_u32(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 16) as u32
    }
    pub fn f(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
}
pub struct Game {
    pub mode: Mode,
    pub sector: usize,
    pub elapsed: f32,
    pub time: f32,
    pub mode_time: f32,
    pub player: Player,
    pub enemies: Vec<Enemy>,
    pub bullets: Vec<Bullet>,
    pub pickups: Vec<Pickup>,
    pub rocks: Vec<Rock>,
    pub gates: Vec<Gate>,
    pub fx: Vec<Fx>,
    pub boss: Option<Boss>,
    pub score: u64,
    pub record: u64,
    pub kills: u32,
    pub grazes: u32,
    pub combo: u32,
    pub combo_time: f32,
    pub meter: usize,
    pub wave: u32,
    pub spawn_wait: f32,
    pub hazard_wait: f32,
    pub trail: Vec<V>,
    pub events: Vec<SoundEvent>,
    pub shake: f32,
    pub flash: f32,
    pub god: bool,
    pub seed: u64,
    rng: Rng,
}
impl Default for Game {
    fn default() -> Self {
        Self::new(0xA57A_F12A)
    }
}
impl Game {
    pub fn new(seed: u64) -> Self {
        Self {
            mode: Mode::Title,
            sector: 0,
            elapsed: 0.,
            time: 0.,
            mode_time: 0.,
            player: Player::default(),
            enemies: vec![],
            bullets: vec![],
            pickups: vec![],
            rocks: vec![],
            gates: vec![],
            fx: vec![],
            boss: None,
            score: 0,
            record: 0,
            kills: 0,
            grazes: 0,
            combo: 0,
            combo_time: 0.,
            meter: 0,
            wave: 0,
            spawn_wait: 1.,
            hazard_wait: 7.,
            trail: vec![V::new(255., 450.); 90],
            events: vec![],
            shake: 0.,
            flash: 0.,
            god: false,
            seed,
            rng: Rng::new(seed),
        }
    }
    pub fn start(&mut self, sector: usize) {
        let record = self.record;
        let god = self.god;
        *self = Self::new(self.seed);
        self.record = record;
        self.god = god;
        self.sector = sector.min(2);
        self.mode = Mode::Flight;
        self.events.push(SoundEvent::Select);
    }
    pub fn walls(&self, x: f32) -> (f32, f32) {
        walls(self.sector, x, self.elapsed)
    }
    pub fn drone(&self, index: usize) -> V {
        self.trail[(index + 1) * 25].lerp(self.player.p, 0.04)
    }
    pub fn add_fx(&mut self, p: V, size: f32, kind: u8) {
        if self.fx.len() < 180 {
            let seed = self.rng.next_u32();
            self.fx.push(Fx {
                p,
                age: 0.,
                size,
                kind,
                seed,
            });
        }
    }
    fn bullet(&mut self, p: V, vel: V, friendly: bool, kind: u8, damage: f32) {
        if self.bullets.len() < 1400 {
            self.bullets.push(Bullet {
                p,
                old: p,
                vel,
                friendly,
                kind,
                radius: if friendly { 4. } else { 6. },
                damage,
                age: 0.,
                grazed: false,
            });
        }
    }
    fn hostile(&mut self, p: V, angle: f32, speed: f32, kind: u8) {
        self.bullet(p, V::angle(angle) * speed, false, kind, 1.);
    }
    pub fn spawn(&mut self, kind: Kind, p: V, elite: bool) {
        if self.enemies.len() >= 60 {
            return;
        }
        let (hp, radius, speed) = match kind {
            Kind::Scout => (3., 21., 210.),
            Kind::Weaver => (5., 25., 155.),
            Kind::Lancer => (10., 30., 105.),
            Kind::Carrier => (24., 48., 85.),
            Kind::Turret => (8., 25., 80.),
        };
        let seed = self.rng.next_u32();
        self.enemies.push(Enemy {
            p,
            origin: p,
            vel: V::new(-speed, 0.),
            kind,
            hp: hp * (if elite { 1.5 } else { 1. }),
            radius,
            age: 0.,
            fire: 1.3,
            seed,
            elite,
        });
    }
    fn wave(&mut self) {
        let n = self.wave;
        let y = self.rng.range(240., 650.);
        let kind = match (n + self.sector as u32) % 7 {
            0 | 1 => Kind::Scout,
            2 | 5 => Kind::Weaver,
            3 => Kind::Lancer,
            4 => Kind::Carrier,
            _ => Kind::Turret,
        };
        let count = match kind {
            Kind::Scout => 5,
            Kind::Weaver => 4,
            Kind::Lancer => 2,
            Kind::Carrier => 1,
            Kind::Turret => 3,
        };
        for i in 0..count {
            let py = if kind == Kind::Turret {
                if i % 2 == 0 {
                    180.
                } else {
                    715.
                }
            } else {
                (y + (i as f32 - 2.) * 31.).clamp(190., 715.)
            };
            self.spawn(kind, V::new(W + 70. + i as f32 * 94., py), i == count - 1);
        }
        self.wave += 1;
        self.spawn_wait = if kind == Kind::Carrier {
            4.2
        } else {
            3.1 - (self.sector as f32 * 0.24)
        };
    }
    pub fn upgrade(&mut self) -> bool {
        if self.meter == 0 {
            return false;
        }
        let p = &mut self.player;
        let success = match self.meter {
            1 => {
                if p.speed < 2 {
                    p.speed += 1;
                    true
                } else {
                    false
                }
            }
            2 => {
                if p.spread < 2 {
                    p.spread += 1;
                    p.weapon = Weapon::Spread;
                    true
                } else {
                    false
                }
            }
            3 => {
                if p.missiles < 2 {
                    p.missiles += 1;
                    true
                } else {
                    false
                }
            }
            4 => {
                if p.beam < 2 {
                    p.beam += 1;
                    p.weapon = Weapon::Beam;
                    true
                } else {
                    false
                }
            }
            5 => {
                if p.drones < 2 {
                    p.drones += 1;
                    true
                } else {
                    false
                }
            }
            6 if p.shield < 4 => {
                p.shield = 4;
                true
            }
            _ => false,
        };
        if success {
            self.meter = 0;
            self.events.push(SoundEvent::Upgrade);
            self.add_fx(self.player.p, 65., 2);
        }
        success
    }
    fn shoot(&mut self) {
        let p = self.player;
        let mut origins = vec![p.p];
        for i in 0..p.drones as usize {
            origins.push(self.drone(i));
        }
        let boosted = p.overload > 0.;
        for origin in origins {
            let origin = origin + V::new(43., 0.);
            match p.weapon {
                Weapon::Pulse => {
                    self.bullet(
                        origin,
                        V::new(1050., 0.),
                        true,
                        0,
                        if boosted { 2.5 } else { 1.6 },
                    );
                    if boosted {
                        for y in [-10., 10.] {
                            self.bullet(origin + V::new(-8., y), V::new(1040., 0.), true, 0, 1.);
                        }
                    }
                }
                Weapon::Spread => {
                    let n = if p.spread == 2 { 2 } else { 1 };
                    for i in -n..=n {
                        self.bullet(origin, V::angle(i as f32 * 0.12) * 960., true, 0, 1.25);
                    }
                }
                Weapon::Beam => {
                    self.bullet(
                        origin,
                        V::new(1750., 0.),
                        true,
                        1,
                        if boosted {
                            3.8
                        } else {
                            2.4 + p.beam as f32 * 0.5
                        },
                    );
                }
            }
        }
        self.events.push(if p.weapon == Weapon::Beam {
            SoundEvent::Beam
        } else {
            SoundEvent::Shot
        });
        self.player.shot_wait = if boosted { 0.065 } else { 0.12 };
    }
    fn damage(&mut self) {
        let p = &mut self.player;
        if p.invul > 0. || p.dash > 0. || self.god {
            return;
        }
        p.invul = 1.25;
        self.shake = 13.;
        self.flash = 0.15;
        if p.shield > 0 {
            p.shield -= 1;
            self.events.push(SoundEvent::Hit);
        } else {
            p.hull -= 1;
            self.events.push(SoundEvent::Hit);
            if p.hull <= 0 {
                p.lives -= 1;
                self.mode = Mode::Dying;
                self.mode_time = 0.;
                self.events.push(SoundEvent::Death);
            }
        }
        self.combo = 0;
        self.add_fx(self.player.p, 80., 1);
    }
    fn begin_boss(&mut self) {
        self.mode = Mode::BossIntro;
        self.mode_time = 0.;
        self.enemies.clear();
        self.bullets.retain(|b| b.friendly);
        self.gates.clear();
        self.rocks.clear();
        let max = [420., 620., 840.][self.sector];
        self.boss = Some(Boss {
            p: V::new(W + 250., 450.),
            hp: max,
            max,
            age: 0.,
            shoot: 2.,
            volley: 0,
            phase: 1,
            hit: 0.,
        });
        self.events.push(SoundEvent::Boss);
    }
    pub fn tick(&mut self, input: Input) {
        self.time += DT;
        self.mode_time += DT;
        self.shake = (self.shake - DT * 26.).max(0.);
        self.flash = (self.flash - DT).max(0.);
        for fx in &mut self.fx {
            fx.age += DT;
        }
        self.fx.retain(|fx| fx.age < 1.25);
        if self.mode == Mode::Title || self.mode == Mode::GameOver || self.mode == Mode::Victory {
            return;
        }
        if self.mode == Mode::Clear {
            if self.mode_time > 4.2 {
                if self.sector == 2 {
                    self.mode = Mode::Victory;
                    self.record = self.record.max(self.score);
                } else {
                    self.sector += 1;
                    self.elapsed = 0.;
                    self.mode_time = 0.;
                    self.wave = 0;
                    self.spawn_wait = 1.;
                    self.hazard_wait = 6.;
                    self.mode = Mode::Flight;
                    self.boss = None;
                    self.bullets.clear();
                    self.pickups.clear();
                    self.player.hull = 3;
                    self.player.energy = (self.player.energy + 25.).min(100.);
                    self.player.invul = 2.;
                }
            }
            return;
        }
        if self.mode == Mode::Dying {
            if self.mode_time > 1.4 {
                if self.player.lives <= 0 {
                    self.mode = Mode::GameOver;
                    self.record = self.record.max(self.score);
                } else {
                    self.player.hull = 3;
                    self.player.p = V::new(255., 450.);
                    self.player.vel = V::default();
                    self.player.invul = 3.;
                    self.player.shield = 0;
                    self.bullets.clear();
                    self.enemies.clear();
                    self.rocks.clear();
                    self.gates.clear();
                    self.trail = vec![self.player.p; 90];
                    self.mode = if self.boss.is_some() {
                        Mode::Boss
                    } else {
                        self.elapsed = (self.elapsed / 20.).floor() * 20.;
                        self.spawn_wait = 1.;
                        Mode::Flight
                    };
                    self.mode_time = 0.;
                }
            }
            return;
        }
        let p = &mut self.player;
        p.invul = (p.invul - DT).max(0.);
        p.dash = (p.dash - DT).max(0.);
        p.dash_cooldown = (p.dash_cooldown - DT).max(0.);
        p.overload = (p.overload - DT).max(0.);
        p.shot_wait = (p.shot_wait - DT).max(0.);
        p.missile_wait = (p.missile_wait - DT).max(0.);
        p.energy = (p.energy + DT * 1.4).min(100.);
        let mut axis = input.axis;
        if axis.len() > 1. {
            axis = axis.norm();
        }
        let speed = if input.focus {
            180.
        } else {
            340. + p.speed as f32 * 45.
        };
        let desired = axis * speed;
        p.vel = p.vel.lerp(desired, DT * 18.);
        if input.dash && p.dash_cooldown == 0. && p.energy >= 22. {
            p.energy -= 22.;
            p.dash = 0.25;
            p.dash_cooldown = 0.9;
            p.vel = if axis.len() > 0.1 {
                axis.norm() * 1100.
            } else {
                V::new(1000., 0.)
            };
            self.events.push(SoundEvent::Dash);
        }
        p.p += p.vel * DT * (if p.dash > 0. { 2.2 } else { 1. });
        p.p.x = p.p.x.clamp(70., W - 140.);
        p.p.y = p.p.y.clamp(125., 770.);
        p.roll += (axis.y * 0.12 - p.roll) * DT * 9.;
        if input.overdrive && p.energy >= 100. && p.overload == 0. {
            p.energy = 0.;
            p.overload = 6.;
            self.events.push(SoundEvent::Overdrive);
            self.shake = 6.;
            self.flash = 0.08;
            self.bullets.retain(|b| b.friendly);
        }
        if let Some(w) = input.weapon {
            if w == Weapon::Pulse
                || (w == Weapon::Spread && p.spread > 0)
                || (w == Weapon::Beam && p.beam > 0)
            {
                p.weapon = w;
            }
        }
        if input.upgrade {
            self.upgrade();
        }
        self.trail.insert(0, self.player.p);
        self.trail.truncate(90);
        if input.fire && self.player.shot_wait == 0. {
            self.shoot();
        }
        if input.fire && self.player.missiles > 0 && self.player.missile_wait == 0. {
            let p = self.player;
            for i in 0..p.missiles {
                self.bullet(
                    p.p + V::new(-10., 15. + i as f32 * 10.),
                    V::new(420., 80.),
                    true,
                    2,
                    5.,
                );
            }
            self.player.missile_wait = 0.6;
            self.events.push(SoundEvent::Missile);
        }
        if self.mode == Mode::Flight {
            self.elapsed += DT;
            self.spawn_wait -= DT;
            self.hazard_wait -= DT;
            if self.spawn_wait <= 0. {
                self.wave();
            }
            if self.hazard_wait <= 0. {
                if self.sector == 1 {
                    let gap = self.player.p.y;
                    self.gates.push(Gate {
                        x: W + 40.,
                        gap,
                        age: 0.,
                        fired: false,
                    });
                    self.hazard_wait = 8.;
                } else {
                    let y = self.rng.range(240., 690.);
                    let radius = self.rng.range(32., 58.);
                    let seed = self.rng.next_u32();
                    self.rocks.push(Rock {
                        p: V::new(W + 80., y),
                        radius,
                        hp: 10.,
                        angle: 0.,
                        seed,
                    });
                    self.hazard_wait = 5.5;
                }
            }
            if self.elapsed >= SECTORS[self.sector].duration {
                self.begin_boss();
            }
        }
        if self.mode == Mode::BossIntro {
            if let Some(b) = &mut self.boss {
                b.p.x += (1225. - b.p.x) * DT * 2.;
            }
            if self.mode_time > 2.6 {
                self.mode = Mode::Boss;
                self.mode_time = 0.;
            }
        }
        let mut fire = vec![];
        for e in &mut self.enemies {
            e.age += DT;
            e.fire -= DT;
            e.p += e.vel * DT;
            match e.kind {
                Kind::Scout => {
                    e.p.y = e.origin.y + (e.age * 3.1).sin() * 38.;
                }
                Kind::Weaver => {
                    e.p.y = e.origin.y + (e.age * 2.0 + e.seed as f32).sin() * 65.;
                }
                Kind::Lancer => {
                    e.p.x += (1190. - e.p.x).max(0.) * DT * 0.12;
                }
                _ => {}
            }
            if e.fire <= 0. && e.p.x < W - 10. && e.p.x > 90. {
                let d = self.player.p - e.p;
                let angle = d.y.atan2(d.x);
                let n = if e.kind == Kind::Carrier {
                    2
                } else if e.kind == Kind::Lancer {
                    1
                } else {
                    0
                };
                for i in -n..=n {
                    fire.push((
                        e.p,
                        angle + i as f32 * 0.15,
                        235. + self.sector as f32 * 25.,
                        self.sector as u8,
                    ));
                }
                e.fire = if e.kind == Kind::Scout { 2.5 } else { 1.7 };
            }
        }
        if self.mode == Mode::Boss {
            let b = self.boss.as_mut().unwrap();
            b.age += DT;
            b.hit = (b.hit - DT).max(0.);
            b.p.y = 450. + (b.age * 0.47).sin() * 135.;
            b.phase = if b.hp / b.max < 0.33 {
                3
            } else if b.hp / b.max < 0.67 {
                2
            } else {
                1
            };
            b.shoot -= DT;
            if b.shoot <= 0. {
                let d = self.player.p - b.p;
                let aim = d.y.atan2(d.x);
                let phase = b.phase as i32;
                match self.sector {
                    0 => {
                        for i in -(2 + phase)..=(2 + phase) {
                            fire.push((
                                b.p + V::new(-40., 0.),
                                aim + i as f32 * 0.105,
                                240. + phase as f32 * 20.,
                                0,
                            ));
                        }
                    }
                    1 => {
                        for i in 0..(12 + phase * 4) {
                            let a = i as f32 * std::f32::consts::TAU / (12 + phase * 4) as f32
                                + b.volley as f32 * 0.18;
                            fire.push((b.p + V::angle(a) * 95., a, 180. + phase as f32 * 22., 1));
                        }
                    }
                    _ => {
                        for i in 0..11 {
                            if i % 5 != b.volley as i32 % 5 {
                                let y = 160. + i as f32 * 57.;
                                fire.push((
                                    V::new(b.p.x - 50., y),
                                    std::f32::consts::PI,
                                    220. + phase as f32 * 15.,
                                    2,
                                ));
                            }
                        }
                        for i in -1..=1 {
                            fire.push((b.p, aim + i as f32 * 0.14, 310., 2));
                        }
                    }
                }
                b.volley += 1;
                b.shoot = match self.sector {
                    0 => 0.9 - phase as f32 * 0.1,
                    1 => 1.0 - phase as f32 * 0.08,
                    _ => 0.85 - phase as f32 * 0.09,
                };
            }
        }
        for (p, a, s, k) in fire {
            self.hostile(p, a, s, k);
        }
        for r in &mut self.rocks {
            r.p.x -= DT * (120. + self.sector as f32 * 22.);
            r.angle += DT * 0.3;
        }
        for g in &mut self.gates {
            g.x -= DT * 140.;
            g.age += DT;
            if g.age > 1.1 && !g.fired {
                g.fired = true;
                self.events.push(SoundEvent::Beam);
            }
        }
        self.gates.retain(|g| g.x > -80. && g.age < 12.);
        let mut damaged = false;
        let mut impacts = vec![];
        let mut near = 0;
        let mut dead = vec![];
        for bullet in &mut self.bullets {
            bullet.old = bullet.p;
            bullet.age += DT;
            if bullet.friendly && bullet.kind == 2 {
                let target = self
                    .enemies
                    .iter()
                    .filter(|e| e.hp > 0.)
                    .min_by(|a, b| bullet.p.dist(a.p).total_cmp(&bullet.p.dist(b.p)))
                    .map(|e| e.p)
                    .or_else(|| self.boss.as_ref().map(|b| b.p));
                if let Some(t) = target {
                    bullet.vel = bullet.vel.lerp((t - bullet.p).norm() * 620., DT * 4.);
                }
            }
            bullet.p += bullet.vel * DT;
            if bullet.friendly {
                for (i, e) in self.enemies.iter_mut().enumerate() {
                    if e.hp > 0.
                        && segment_distance(bullet.old, bullet.p, e.p) < e.radius + bullet.radius
                    {
                        e.hp -= bullet.damage;
                        bullet.age = 100.;
                        impacts.push((bullet.p, 12., 0));
                        if e.hp <= 0. {
                            dead.push((i, e.p, e.elite));
                        }
                        break;
                    }
                }
                if bullet.age < 90. {
                    for r in &mut self.rocks {
                        if r.hp > 0. && segment_distance(bullet.old, bullet.p, r.p) < r.radius {
                            r.hp -= bullet.damage;
                            bullet.age = 100.;
                            impacts.push((bullet.p, 10., 0));
                            break;
                        }
                    }
                }
                if bullet.age < 90. && self.mode == Mode::Boss {
                    if let Some(b) = &mut self.boss {
                        if segment_distance(bullet.old, bullet.p, b.p) < b.core_radius() {
                            bullet.age = 100.;
                            if b.exposed() {
                                b.hp -= bullet.damage;
                                b.hit = 0.075;
                                impacts.push((bullet.p, 18., 0));
                            } else {
                                impacts.push((bullet.p, 12., 2));
                            }
                        }
                    }
                }
            } else {
                let distance = segment_distance(bullet.old, bullet.p, self.player.p);
                if distance < 8. + bullet.radius {
                    damaged = true;
                    bullet.age = 100.;
                } else if distance < 30. && !bullet.grazed {
                    bullet.grazed = true;
                    near += 1;
                }
            }
        }
        self.grazes += near;
        self.player.energy = (self.player.energy + near as f32 * 2.).min(100.);
        self.score += near as u64 * 10;
        for (_, p, elite) in dead {
            self.kills += 1;
            self.combo = (self.combo + 1).min(50);
            self.combo_time = 4.;
            self.score += 100 * (1 + self.combo / 10) as u64;
            self.player.energy = (self.player.energy + 3.).min(100.);
            self.events.push(SoundEvent::Explosion);
            self.add_fx(p, 50., 1);
            if elite || self.kills.is_multiple_of(4) {
                let phase = self.rng.range(0., 6.);
                self.pickups.push(Pickup {
                    p,
                    age: 0.,
                    kind: if self.kills.is_multiple_of(13) { 1 } else { 0 },
                    phase,
                });
            }
        }
        self.combo_time = (self.combo_time - DT).max(0.);
        if self.combo_time == 0. {
            self.combo = 0;
        }
        for (p, s, k) in impacts {
            self.add_fx(p, s, k);
        }
        for r in &self.rocks {
            if r.hp > 0. && r.p.dist(self.player.p) < r.radius + 9. {
                damaged = true;
            }
        }
        let broken: Vec<_> = self
            .rocks
            .iter()
            .filter(|r| r.hp <= 0.)
            .map(|r| r.p)
            .collect();
        for p in broken {
            self.add_fx(p, 75., 1);
            self.events.push(SoundEvent::Explosion);
            self.score += 150;
        }
        for gate in &self.gates {
            if gate.fired
                && (gate.x - self.player.p.x).abs() < 15.
                && (gate.gap - self.player.p.y).abs() > 90.
            {
                damaged = true;
            }
        }
        for e in &self.enemies {
            if e.hp > 0. && e.p.dist(self.player.p) < e.radius + 7. {
                damaged = true;
            }
        }
        let (top, bottom) = self.walls(self.player.p.x);
        if self.player.p.y < top + 9. || self.player.p.y > bottom - 9. {
            damaged = true;
        }
        if let Some(b) = &self.boss {
            if b.p.dist(self.player.p) < 125. {
                damaged = true;
            }
        }
        for p in &mut self.pickups {
            p.age += DT;
            p.p.x -= DT * 82.;
            p.p.y += ((p.age * 2. + p.phase).sin()) * DT * 8.;
            if p.p.dist(self.player.p) < 160. {
                p.p += (self.player.p - p.p).norm() * DT * 180.;
            }
            if p.p.dist(self.player.p) < 32. {
                if p.kind == 1 {
                    self.player.hull = (self.player.hull + 1).min(3);
                } else {
                    self.meter = self.meter % 6 + 1;
                }
                self.player.energy = (self.player.energy + 7.).min(100.);
                self.events.push(SoundEvent::Pickup);
                p.age = 100.;
            }
        }
        self.enemies
            .retain(|e| e.hp > 0. && e.p.x > -120. && e.age < 28.);
        self.rocks.retain(|r| r.hp > 0. && r.p.x > -100.);
        self.pickups.retain(|p| p.p.x > -50. && p.age < 25.);
        self.bullets.retain(|b| {
            b.age < 9. && b.p.x > -100. && b.p.x < W + 200. && b.p.y > 60. && b.p.y < 840.
        });
        if damaged {
            self.damage();
        }
        if self.mode == Mode::Boss && self.boss.as_ref().is_some_and(|b| b.hp <= 0.) {
            let pos = self.boss.as_ref().unwrap().p;
            self.mode = Mode::Clear;
            self.mode_time = 0.;
            self.score += 10000 + 5000 * self.sector as u64;
            self.record = self.record.max(self.score);
            self.bullets.clear();
            self.enemies.clear();
            self.rocks.clear();
            self.gates.clear();
            self.shake = 20.;
            self.flash = 0.3;
            self.add_fx(pos, 240., 3);
            self.events.push(SoundEvent::Clear);
        }
    }
}
impl V {
    pub fn lerp(self, b: Self, t: f32) -> Self {
        self + (b - self) * t.clamp(0., 1.)
    }
}
pub fn walls(sector: usize, x: f32, time: f32) -> (f32, f32) {
    let world = x + time * 105.;
    let wave = (world * 0.004).sin();
    let wave2 = (world * 0.009 + 1.4).sin();
    let narrow = if sector == 2 {
        (time * 0.13).sin().max(0.) * 50.
    } else {
        0.
    };
    (
        121. + wave * 12. + wave2 * 5. + narrow,
        770. + wave * 11. - wave2 * 6. - narrow,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn swept_collision_catches_fast_projectiles() {
        assert!(segment_distance(V::new(0., 0.), V::new(100., 0.), V::new(50., 2.)) < 3.);
        assert!(segment_distance(V::new(0., 0.), V::new(100., 0.), V::new(50., 20.)) > 10.);
    }
    #[test]
    fn upgrade_caps_and_weapon_switches() {
        let mut g = Game::default();
        g.start(0);
        for _ in 0..3 {
            g.meter = 5;
            g.upgrade();
        }
        assert_eq!(g.player.drones, 2);
        g.meter = 4;
        assert!(g.upgrade());
        assert_eq!(g.player.weapon, Weapon::Beam);
        g.meter = 6;
        g.upgrade();
        assert_eq!(g.player.shield, 4);
        assert_eq!(g.meter, 0);
    }
    #[test]
    fn dash_cost_and_overdrive() {
        let mut g = Game::default();
        g.start(0);
        g.player.energy = 100.;
        g.tick(Input {
            dash: true,
            ..Default::default()
        });
        assert!(g.player.dash > 0.);
        assert!(g.player.energy < 80.);
        g.player.energy = 100.;
        g.tick(Input {
            overdrive: true,
            ..Default::default()
        });
        assert!(g.player.overload > 5.);
        assert!(g.player.energy < 1.);
    }
    #[test]
    fn campaign_reaches_victory() {
        let mut g = Game::default();
        g.start(0);
        g.god = true;
        for (stage, sector) in SECTORS.iter().enumerate() {
            assert_eq!(g.sector, stage);
            g.elapsed = sector.duration;
            g.tick(Input::default());
            assert_eq!(g.mode, Mode::BossIntro);
            for _ in 0..320 {
                g.tick(Input::default());
            }
            assert_eq!(g.mode, Mode::Boss);
            g.boss.as_mut().unwrap().hp = 0.;
            g.tick(Input::default());
            assert_eq!(g.mode, Mode::Clear);
            for _ in 0..510 {
                g.tick(Input::default());
            }
        }
        assert_eq!(g.mode, Mode::Victory);
    }
    #[test]
    fn deterministic_and_bounded_simulation() {
        let mut a = Game::new(45);
        let mut b = Game::new(45);
        a.start(0);
        b.start(0);
        a.god = true;
        b.god = true;
        for t in 0..18000 {
            let input = Input {
                fire: true,
                axis: V::new(0., (t as f32 * 0.007).sin() * 0.7),
                ..Default::default()
            };
            a.tick(input);
            b.tick(input);
            a.events.clear();
            b.events.clear();
            assert!(a.bullets.len() <= 1400);
            assert!(a.enemies.len() <= 60);
        }
        assert_eq!(a.score, b.score);
        assert_eq!(a.kills, b.kills);
        assert_eq!(a.player.p, b.player.p);
        assert!(a.player.p.x.is_finite());
    }
    #[test]
    fn death_respawns_then_gameover() {
        let mut g = Game::default();
        g.start(0);
        g.player.invul = 0.;
        g.player.hull = 1;
        g.damage();
        assert_eq!(g.mode, Mode::Dying);
        for _ in 0..175 {
            g.tick(Input::default());
        }
        assert_eq!(g.mode, Mode::Flight);
        assert_eq!(g.player.hull, 3);
        g.player.lives = 1;
        g.player.hull = 1;
        g.player.invul = 0.;
        g.damage();
        for _ in 0..175 {
            g.tick(Input::default());
        }
        assert_eq!(g.mode, Mode::GameOver);
    }
}
