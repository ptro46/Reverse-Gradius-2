use crate::core::{Game, Kind, Mode, Weapon, H, SECTORS, V, W};
use macroquad::prelude::*;
const CYAN: Color = Color::new(0.24, 0.89, 0.94, 1.);
const GOLD: Color = Color::new(1., 0.66, 0.30, 1.);
const INK: Color = Color::new(0.025, 0.045, 0.075, 1.);
const MUTED: Color = Color::new(0.43, 0.56, 0.65, 1.);
pub struct Renderer {
    bg: [Texture2D; 3],
    ship: Texture2D,
    enemies: [Texture2D; 5],
    bosses: [Texture2D; 3],
    glow: Texture2D,
    font: Font,
    mono: Font,
    target: RenderTarget,
    post: Material,
    pub quality: usize,
    pub effects: bool,
}
pub struct View {
    pub paused: bool,
    pub focus: bool,
    pub selected: usize,
    pub help: bool,
    pub muted: bool,
    pub gamepad: &'static str,
    pub gamepad_connected: bool,
}
fn pt(p: V) -> Vec2 {
    vec2(p.x, p.y)
}
fn noise(i: u32) -> f32 {
    let mut x = i.wrapping_mul(0x45d9f3b);
    x ^= x >> 16;
    x = x.wrapping_mul(0x45d9f3b);
    (x ^ (x >> 16)) as f32 / u32::MAX as f32
}
fn tint(c: Color, a: f32) -> Color {
    Color { a, ..c }
}
impl Renderer {
    pub fn new() -> Self {
        let bg = [
            "fractured-moon.png",
            "solar-forge.png",
            "glass-cathedral.png",
        ]
        .map(|name| Texture2D::from_file_with_format(crate::asset(name), Some(ImageFormat::Png)));
        let ship = Texture2D::from_file_with_format(
            crate::asset("interceptor.png"),
            Some(ImageFormat::Png),
        );
        for texture in &bg {
            texture.set_filter(FilterMode::Linear);
        }
        ship.set_filter(FilterMode::Linear);
        let enemies = ["scout", "weaver", "lancer", "carrier", "turret"].map(|name| {
            let t = Texture2D::from_file_with_format(
                crate::asset(&format!("enemy-{name}.png")),
                Some(ImageFormat::Png),
            );
            t.set_filter(FilterMode::Linear);
            t
        });
        let bosses = ["sentinel", "solstice", "choir"].map(|name| {
            let t = Texture2D::from_file_with_format(
                crate::asset(&format!("boss-{name}.png")),
                Some(ImageFormat::Png),
            );
            t.set_filter(FilterMode::Linear);
            t
        });
        let font = load_ttf_font_from_bytes(crate::asset("Rajdhani-SemiBold.ttf")).unwrap();
        let mono = load_ttf_font_from_bytes(crate::asset("ShareTechMono-Regular.ttf")).unwrap();
        let mut image = Image::gen_image_color(96, 96, Color::new(0., 0., 0., 0.));
        for y in 0..96 {
            for x in 0..96 {
                let r = ((x as f32 - 47.5).powi(2) + (y as f32 - 47.5).powi(2)).sqrt() / 48.;
                image.set_pixel(x, y, Color::new(1., 1., 1., (1. - r).max(0.).powf(3.2)));
            }
        }
        let glow = Texture2D::from_image(&image);
        glow.set_filter(FilterMode::Linear);
        let target = render_target(1920, 1080);
        target.texture.set_filter(FilterMode::Linear);
        let post = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX,
                fragment: FRAGMENT,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("Size", UniformType::Float2),
                    UniformDesc::new("Power", UniformType::Float1),
                ],
                ..Default::default()
            },
        )
        .expect("presentation shader");
        Self {
            bg,
            ship,
            enemies,
            bosses,
            glow,
            font,
            mono,
            target,
            post,
            quality: 1,
            effects: true,
        }
    }
    pub fn cycle_quality(&mut self) {
        self.quality = (self.quality + 1) % 4;
        let w = [1600, 1920, 2560, 3840][self.quality];
        self.target = render_target(w, w * 9 / 16);
        self.target.texture.set_filter(FilterMode::Linear);
    }
    fn text(&self, s: &str, x: f32, y: f32, size: f32, color: Color) {
        draw_text_ex(
            s,
            x,
            y,
            TextParams {
                font: Some(&self.font),
                font_size: size as u16,
                color,
                ..Default::default()
            },
        );
    }
    fn mono(&self, s: &str, x: f32, y: f32, size: f32, color: Color) {
        draw_text_ex(
            s,
            x,
            y,
            TextParams {
                font: Some(&self.mono),
                font_size: size as u16,
                color,
                ..Default::default()
            },
        );
    }
    fn centered(&self, s: &str, x: f32, y: f32, size: f32, color: Color) {
        let m = measure_text(s, Some(&self.font), size as u16, 1.);
        self.text(s, x - m.width / 2., y, size, color);
    }
    fn panel(&self, r: Rect, c: Color) {
        let radius = 8.;
        draw_rectangle(r.x + radius, r.y, r.w - 2. * radius, r.h, c);
        draw_rectangle(r.x, r.y + radius, r.w, r.h - 2. * radius, c);
        for (x, y) in [
            (r.x + radius, r.y + radius),
            (r.x + r.w - radius, r.y + radius),
            (r.x + radius, r.y + r.h - radius),
            (r.x + r.w - radius, r.y + r.h - radius),
        ] {
            draw_circle(x, y, radius, c);
        }
    }
    fn light(&self, p: V, r: f32, color: Color) {
        if !self.effects {
            return;
        }
        draw_texture_ex(
            &self.glow,
            p.x - r,
            p.y - r,
            color,
            DrawTextureParams {
                dest_size: Some(vec2(r * 2., r * 2.)),
                ..Default::default()
            },
        );
    }
    fn polygon(&self, p: V, angle: f32, points: &[V], fill: Color, edge: Color) {
        let rot = |v: V| {
            V::new(
                v.x * angle.cos() - v.y * angle.sin(),
                v.x * angle.sin() + v.y * angle.cos(),
            ) + p
        };
        for i in 0..points.len() {
            let a = rot(points[i]);
            let b = rot(points[(i + 1) % points.len()]);
            draw_triangle(pt(p), pt(a), pt(b), fill);
            draw_line(a.x, a.y, b.x, b.y, 1.5, edge);
        }
    }
    fn background(&self, g: &Game) {
        clear_background(Color::from_rgba(2, 6, 13, 255));
        let shift = if g.mode == Mode::Title {
            (g.time * 0.04).sin() * 14.
        } else {
            -(g.elapsed * 0.55).min(45.)
        };
        let col = WHITE;
        draw_texture_ex(
            &self.bg[g.sector],
            shift - 32.,
            -20.,
            col,
            DrawTextureParams {
                dest_size: Some(vec2(W + 64., H + 40.)),
                ..Default::default()
            },
        );
        draw_rectangle(0., 0., W, H, Color::new(0., 0.012, 0.025, 0.18));
        for layer in 0..3 {
            for i in 0..65 {
                let seed = i * 7 + layer * 331;
                let speed = 8. + layer as f32 * 17.;
                let x = (noise(seed) * W - g.time * speed).rem_euclid(W);
                let y = 100. + noise(seed + 18) * 670.;
                let a = 0.22 + layer as f32 * 0.15;
                let radius = 0.6 + layer as f32 * 0.55;
                draw_circle(x, y, radius, Color::new(0.64, 0.83, 0.9, a));
                if layer == 2 && i % 9 == 0 {
                    self.light(V::new(x, y), 8., Color::new(0.25, 0.75, 1., 0.22));
                }
            }
        }
        if g.sector == 1 {
            let p = V::new(1370., 470.);
            for i in 0..6 {
                draw_circle_lines(
                    p.x,
                    p.y,
                    160. + i as f32 * 23.,
                    2.,
                    Color::new(1., 0.4, 0.12, 0.08),
                );
            }
            self.light(p, 390., Color::new(1., 0.35, 0.05, 0.3));
        }
        if g.sector == 2 {
            for i in 0..9 {
                let x = (i as f32 * 240. - g.time * 36.).rem_euclid(W + 240.) - 120.;
                draw_line(
                    x,
                    140.,
                    x - 210.,
                    760.,
                    1.,
                    Color::new(0.58, 0.45, 1., 0.12),
                );
                draw_line(
                    x + 10.,
                    140.,
                    x - 200.,
                    760.,
                    5.,
                    Color::new(0.25, 0.25, 0.4, 0.07),
                );
            }
        }
    }
    fn walls(&self, g: &Game) {
        let edge = match g.sector {
            0 => Color::new(0.17, 0.42, 0.46, 1.),
            1 => Color::new(0.56, 0.29, 0.16, 1.),
            _ => Color::new(0.39, 0.26, 0.57, 1.),
        };
        for i in 0..41 {
            let x = i as f32 * 40.;
            let (a, b) = g.walls(x);
            let (c, d) = g.walls(x + 40.);
            draw_triangle(vec2(x, 0.), vec2(x + 40., 0.), vec2(x, a), INK);
            draw_triangle(vec2(x, a), vec2(x + 40., 0.), vec2(x + 40., c), INK);
            draw_triangle(vec2(x, b), vec2(x + 40., d), vec2(x, H), INK);
            draw_triangle(vec2(x, H), vec2(x + 40., d), vec2(x + 40., H), INK);
            draw_line(x, a, x + 40., c, 1.5, edge);
            draw_line(x, b, x + 40., d, 1.5, edge);
            if i % 4 == 0 {
                draw_line(x + 5., a - 5., x + 27., a - 5., 2., tint(edge, 0.5));
                draw_line(x + 5., b + 5., x + 27., b + 5., 2., tint(edge, 0.5));
            }
        }
    }
    fn player(&self, g: &Game) {
        let p = &g.player;
        if g.mode == Mode::Dying {
            return;
        }
        for i in (0..p.drones as usize).rev() {
            let d = g.drone(i);
            self.light(d, 30., tint(CYAN, 0.5));
            draw_poly(
                d.x,
                d.y,
                6,
                10.,
                g.time * 40.,
                Color::new(0.18, 0.38, 0.45, 1.),
            );
            draw_poly_lines(d.x, d.y, 6, 12., g.time * 40., 1.5, CYAN);
            draw_circle(d.x, d.y, 4., WHITE);
        }
        if p.dash > 0. {
            for i in 1..6 {
                let old = p.p - p.vel * (i as f32 * 0.008);
                draw_texture_ex(
                    &self.ship,
                    old.x - 59.,
                    old.y - 26.,
                    Color::new(0.2, 0.8, 1., 0.12 * (1. - i as f32 / 6.)),
                    DrawTextureParams {
                        dest_size: Some(vec2(118., 52.)),
                        rotation: p.roll,
                        ..Default::default()
                    },
                );
            }
        }
        let plume = 24. + (g.time * 38.).sin() * 5. + if p.dash > 0. { 60. } else { 0. };
        let engine = p.p + V::new(-47., 0.);
        self.light(engine, 70., Color::new(0.05, 0.65, 1., 0.6));
        for dy in [-9., 9.] {
            let at = engine + V::new(0., dy);
            draw_triangle(
                pt(at + V::new(0., -4.)),
                pt(at - V::new(plume, 0.)),
                pt(at + V::new(0., 4.)),
                Color::new(0.12, 0.7, 1., 0.65),
            );
            draw_triangle(
                pt(at + V::new(0., -1.8)),
                pt(at - V::new(plume * 0.6, 0.)),
                pt(at + V::new(0., 1.8)),
                Color::new(0.8, 0.98, 1., 0.95),
            );
        }
        draw_texture_ex(
            &self.ship,
            p.p.x - 59.,
            p.p.y - 26.,
            if p.invul > 0. {
                Color::new(0.65, 0.93, 1., 1.)
            } else {
                WHITE
            },
            DrawTextureParams {
                dest_size: Some(vec2(118., 52.)),
                rotation: p.roll,
                ..Default::default()
            },
        );
        if p.shield > 0 {
            draw_circle_lines(p.p.x, p.p.y, 61., 1.5, tint(CYAN, 0.3));
            self.light(p.p, 70., tint(CYAN, 0.08));
            for i in 0..p.shield {
                let a = -1.3 + i as f32 * 0.36;
                let d = p.p + V::angle(a) * 63.;
                draw_circle(d.x, d.y, 2., CYAN);
            }
        }
        if p.overload > 0. {
            self.light(p.p, 130., Color::new(0.3, 0.55, 1., 0.3));
        }
    }
    fn enemy(&self, e: &crate::core::Enemy, g: &Game) {
        let index = match e.kind {
            Kind::Scout => 0,
            Kind::Weaver => 1,
            Kind::Lancer => 2,
            Kind::Carrier => 3,
            Kind::Turret => 4,
        };
        let texture = &self.enemies[index];
        let width = e.radius * 3.4;
        let height = width * texture.height() / texture.width();
        let rotation = match e.kind {
            Kind::Scout => (e.age * 2.1).cos() * 0.10,
            Kind::Weaver => e.age * 0.35,
            Kind::Turret => {
                let d = g.player.p - e.p;
                d.y.atan2(d.x) - std::f32::consts::PI
            }
            _ => 0.,
        };
        self.light(e.p, e.radius * 2.0, Color::new(1., 0.35, 0.1, 0.23));
        draw_texture_ex(
            texture,
            e.p.x - width / 2.,
            e.p.y - height / 2.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(width, height)),
                rotation,
                ..Default::default()
            },
        );
        if e.elite {
            draw_circle(e.p.x + e.radius * 0.6, e.p.y - e.radius, 3.5, GOLD);
        }
        if e.kind == Kind::Lancer && e.fire < 0.3 {
            let aim = (g.player.p - e.p).norm();
            let a = e.p + aim * 40.;
            let b = e.p + aim * 140.;
            draw_line(a.x, a.y, b.x, b.y, 1., tint(GOLD, 0.35));
        }
    }
    fn boss(&self, b: &crate::core::Boss, g: &Game) {
        let p = b.p;
        let texture = &self.bosses[g.sector];
        let width = [540., 445., 480.][g.sector];
        let height = width * texture.height() / texture.width();
        // Align the illustrated reactor with the simulated weak point.
        let anchor = [(0.556, 0.48), (0.503, 0.49), (0.5, 0.57)][g.sector];
        let hot = match g.sector {
            0 => GOLD,
            1 => Color::new(1., 0.28, 0.10, 1.),
            _ => Color::new(0.75, 0.4, 1., 1.),
        };
        let angle = match g.sector {
            1 => b.age * 0.04,
            2 => (b.age * 0.2).sin() * 0.05,
            _ => (b.age * 0.3).sin() * 0.012,
        };
        self.light(p, width * 0.6, tint(hot, 0.3));
        draw_texture_ex(
            texture,
            p.x - width * anchor.0,
            p.y - height * anchor.1,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(width, height)),
                rotation: angle,
                pivot: Some(vec2(p.x, p.y)),
                ..Default::default()
            },
        );
        for i in 0..b.phase {
            let a = b.age * 0.3 + i as f32 * std::f32::consts::TAU / b.phase as f32;
            let at = p + V::angle(a) * 64.;
            self.light(at, 24., tint(hot, 0.55));
            draw_circle(at.x, at.y, 3.5, hot);
        }
        if b.exposed() {
            self.light(p, 110., tint(hot, 0.65));
            draw_circle_lines(p.x, p.y, 40., 1.5, tint(hot, 0.8));
            if b.hit > 0. {
                self.light(p, 100., tint(WHITE, 0.8));
            }
        } else {
            draw_circle(p.x, p.y, 43., Color::new(0.05, 0.16, 0.19, 0.94));
            draw_circle_lines(p.x, p.y, 47., 2., CYAN);
            for i in 0..5 {
                draw_line(
                    p.x - 34.,
                    p.y - 24. + i as f32 * 12.,
                    p.x + 34.,
                    p.y - 24. + i as f32 * 12.,
                    2.5,
                    CYAN,
                );
            }
        }
    }
    fn objects(&self, g: &Game) {
        for r in &g.rocks {
            let points: Vec<_> = (0..11)
                .map(|i| {
                    let a = i as f32 * std::f32::consts::TAU / 11.;
                    V::angle(a) * r.radius * (0.75 + noise(r.seed.wrapping_add(i)) * 0.25)
                })
                .collect();
            self.polygon(
                r.p,
                r.angle,
                &points,
                Color::from_rgba(44, 48, 53, 255),
                Color::from_rgba(100, 104, 107, 255),
            );
            for i in 0..4 {
                let a = i as f32 * 1.6 + r.angle;
                let p = r.p + V::angle(a) * r.radius * 0.7;
                draw_line(
                    r.p.x,
                    r.p.y,
                    p.x,
                    p.y,
                    1.3,
                    Color::from_rgba(65, 73, 81, 255),
                );
            }
        }
        for gate in &g.gates {
            let hot = if gate.fired {
                Color::new(1., 0.3, 0.12, 1.)
            } else {
                Color::new(1., 0.65, 0.2, 0.3)
            };
            for (a, b) in [(145., gate.gap - 90.), (gate.gap + 90., 760.)] {
                draw_line(
                    gate.x,
                    a,
                    gate.x,
                    b,
                    if gate.fired { 12. } else { 1.5 },
                    hot,
                );
                if gate.fired {
                    draw_line(gate.x, a, gate.x, b, 3., WHITE);
                }
            }
            for y in [gate.gap - 100., gate.gap + 100.] {
                draw_poly(gate.x, y, 6, 17., 30., Color::from_rgba(51, 58, 66, 255));
                draw_circle(gate.x, y, 6., hot);
                self.light(V::new(gate.x, y), 44., tint(hot, 0.5));
            }
        }
        for p in &g.pickups {
            let c = if p.kind == 1 {
                Color::new(0.4, 1., 0.6, 1.)
            } else {
                CYAN
            };
            self.light(p.p, 40., tint(c, 0.45));
            draw_poly(p.p.x, p.p.y, 4, 13., p.age * 50., tint(c, 0.2));
            draw_poly_lines(p.p.x, p.p.y, 4, 14., p.age * 50., 1.5, c);
            if p.kind == 0 {
                draw_circle(p.p.x, p.p.y, 4., WHITE);
            } else {
                draw_line(p.p.x - 5., p.p.y, p.p.x + 5., p.p.y, 2., WHITE);
                draw_line(p.p.x, p.p.y - 5., p.p.x, p.p.y + 5., 2., WHITE);
            }
        }
    }
    fn projectiles(&self, g: &Game) {
        for b in &g.bullets {
            if b.friendly {
                let c = if b.kind == 2 { GOLD } else { CYAN };
                let dir = b.vel.norm();
                let length = if b.kind == 1 {
                    82.
                } else if b.kind == 2 {
                    18.
                } else {
                    22.
                };
                let tail = b.p - dir * length;
                self.light(b.p, if b.kind == 1 { 28. } else { 14. }, tint(c, 0.5));
                draw_line(
                    tail.x,
                    tail.y,
                    b.p.x,
                    b.p.y,
                    if b.kind == 1 { 7. } else { 3. },
                    c,
                );
                draw_line(tail.x, tail.y, b.p.x, b.p.y, 1.5, WHITE);
            } else {
                let c = match b.kind {
                    0 => GOLD,
                    1 => Color::new(1., 0.3, 0.55, 1.),
                    _ => Color::new(0.83, 0.4, 1., 1.),
                };
                self.light(b.p, 23., tint(c, 0.5));
                draw_circle(b.p.x, b.p.y, b.radius, c);
                draw_circle(b.p.x - 1., b.p.y - 1., 2.5, Color::new(1., 0.98, 0.85, 1.));
            }
        }
    }
    fn explosions(&self, g: &Game) {
        for f in &g.fx {
            let life = (1. - f.age / 1.25).max(0.);
            let c = if f.kind == 2 { CYAN } else { GOLD };
            let size = f.size * (0.3 + f.age * 1.5);
            self.light(f.p, size * 1.6, tint(c, life * 0.8));
            if f.kind == 0 {
                draw_circle_lines(f.p.x, f.p.y, size * 0.25, 1., tint(c, life));
                continue;
            }
            draw_circle_lines(f.p.x, f.p.y, size, 1.5, tint(c, life * 0.65));
            let n = if self.effects { 36 } else { 12 };
            for i in 0..n {
                let a = noise(f.seed.wrapping_add(i)) * std::f32::consts::TAU;
                let speed = 0.3 + noise(f.seed.wrapping_add(i + 300)) * 0.9;
                let p = f.p + V::angle(a) * (f.age * f.size * speed * 2.);
                let back = p - V::angle(a) * 8. * life;
                draw_line(p.x, p.y, back.x, back.y, 1.3 + 2. * noise(i), tint(c, life));
            }
        }
    }
    fn ui(&self, g: &Game, view: &View) {
        draw_rectangle(0., 0., W, 92., Color::new(0.015, 0.027, 0.045, 0.95));
        draw_line(32., 91., W - 32., 91., 1., Color::new(0.14, 0.34, 0.4, 0.5));
        self.mono("ASTRA / FRACTURE", 40., 40., 20., CYAN);
        self.text(SECTORS[g.sector].name, 40., 71., 24., WHITE);
        self.mono("SCORE", 440., 30., 14., MUTED);
        self.text(&format!("{:08}", g.score), 440., 68., 37., WHITE);
        self.mono(
            &format!("X{:02}", 1 + g.combo / 10),
            625.,
            63.,
            20.,
            if g.combo > 0 { GOLD } else { MUTED },
        );
        self.mono("COQUE", 765., 30., 14., MUTED);
        for i in 0..3 {
            self.panel(
                Rect::new(765. + i as f32 * 28., 44., 21., 12.),
                if i < g.player.hull {
                    CYAN
                } else {
                    Color::new(0.12, 0.2, 0.23, 1.)
                },
            );
        }
        self.mono(
            &format!("VIES      {:02}", g.player.lives.max(0)),
            765.,
            77.,
            14.,
            MUTED,
        );
        self.mono(
            &format!("BOUCLIER  {:02}", g.player.shield),
            930.,
            56.,
            17.,
            MUTED,
        );
        self.mono(
            "SURCHARGE [V]",
            1190.,
            30.,
            14.,
            if g.player.energy >= 99. { CYAN } else { MUTED },
        );
        draw_rectangle(1190., 43., 225., 5., Color::new(0.08, 0.17, 0.21, 1.));
        draw_rectangle(
            1190.,
            43.,
            225. * g.player.energy / 100.,
            5.,
            if g.player.overload > 0. { GOLD } else { CYAN },
        );
        self.mono(
            &format!("{:03}%", g.player.energy as i32),
            1437.,
            49.,
            19.,
            WHITE,
        );
        self.mono(
            if view.muted {
                "M : SON OFF"
            } else {
                "M : AUDIO"
            },
            1450.,
            78.,
            12.,
            MUTED,
        );
        draw_rectangle(0., 798., W, 102., Color::new(0.015, 0.027, 0.045, 0.97));
        self.mono(
            "CHOISIR EN COLLECTANT  /  ACTIVER [X]",
            40.,
            824.,
            13.,
            MUTED,
        );
        let names = [
            "PROPULSION",
            "EVENTAIL",
            "MISSILES",
            "LANCE PHASE",
            "DRONE",
            "BOUCLIER",
        ];
        let p = g.player;
        for (i, name) in names.iter().enumerate() {
            let r = Rect::new(40. + i as f32 * 186., 837., 174., 47.);
            let active = g.meter == i + 1;
            let owned = match i {
                0 => p.speed > 0,
                1 => p.spread > 0,
                2 => p.missiles > 0,
                3 => p.beam > 0,
                4 => p.drones > 0,
                _ => p.shield > 0,
            };
            self.panel(
                r,
                if active {
                    Color::from_rgba(55, 61, 45, 255)
                } else if owned {
                    Color::from_rgba(15, 42, 52, 255)
                } else {
                    Color::from_rgba(15, 25, 38, 255)
                },
            );
            if active {
                draw_rectangle(r.x, r.y + 44., r.w, 2., GOLD);
            }
            self.mono(
                &format!("0{}", i + 1),
                r.x + 11.,
                r.y + 18.,
                11.,
                if active { GOLD } else { MUTED },
            );
            self.text(
                name,
                r.x + 11.,
                r.y + 38.,
                19.,
                if active {
                    GOLD
                } else if owned {
                    CYAN
                } else {
                    MUTED
                },
            );
        }
        self.mono("ARME [1 / 2 / 3]", 1225., 828., 13., MUTED);
        self.text(
            match p.weapon {
                Weapon::Pulse => "IMPULSION",
                Weapon::Spread => "EVENTAIL",
                Weapon::Beam => "LANCE PHASE",
            },
            1225.,
            860.,
            29.,
            WHITE,
        );
        self.mono("C : DASH   /   MAJ : PRECISION", 1225., 885., 12., MUTED);
        if g.mode == Mode::Flight {
            let progress = g.elapsed / SECTORS[g.sector].duration;
            draw_rectangle(0., 795., W * progress, 2., tint(CYAN, 0.6));
        }
        if let Some(b) = &g.boss {
            if matches!(g.mode, Mode::Boss | Mode::BossIntro) {
                let width = 600.;
                self.mono(SECTORS[g.sector].boss, 500., 116., 13., GOLD);
                draw_rectangle(500., 124., width, 5., Color::new(0.2, 0.13, 0.11, 1.));
                draw_rectangle(500., 124., width * (b.hp / b.max).clamp(0., 1.), 5., GOLD);
                self.mono(&format!("PHASE 0{}", b.phase), 1020., 116., 13., MUTED);
            }
        }
        if g.mode == Mode::Flight && g.elapsed < 5. {
            let a = (g.elapsed.min(1.) * (5. - g.elapsed).min(1.)).max(0.);
            self.centered(
                &format!("0{}   /   {}", g.sector + 1, SECTORS[g.sector].name),
                W / 2.,
                315.,
                44.,
                tint(WHITE, a),
            );
            self.centered(
                SECTORS[g.sector].subtitle,
                W / 2.,
                348.,
                21.,
                tint(MUTED, a),
            );
        }
        if g.mode == Mode::BossIntro {
            draw_rectangle(0., 350., W, 150., Color::new(0.08, 0.015, 0.01, 0.73));
            self.centered("SIGNATURE MASSIVE DETECTEE", W / 2., 395., 27., GOLD);
            self.centered(SECTORS[g.sector].boss, W / 2., 455., 61., WHITE);
        }
        if g.mode == Mode::Clear {
            self.overlay(
                "SECTEUR SECURISE",
                &format!("Prime de combat  +{}", 10000 + g.sector * 5000),
                "Signal stabilise. Saut vers le prochain secteur...",
            );
        }
        if g.mode == Mode::GameOver {
            self.overlay(
                "SIGNAL PERDU",
                &format!("Score final  {:08}", g.score),
                "ENTREE / B10 pour relancer   /   ECHAP pour le menu",
            );
        }
        if g.mode == Mode::Victory {
            self.overlay(
                "LA FRACTURE EST REFERMEE",
                &format!(
                    "Score {:08}   /   {} cibles   /   {} esquives proches",
                    g.score, g.kills, g.grazes
                ),
                "ENTREE / B10 : nouvelle expedition   /   ECHAP : menu",
            );
        }
        if view.paused {
            self.overlay(
                "PAUSE",
                "Le vide peut attendre.",
                "P / B10 pour reprendre   /   H : commandes   /   ECHAP : menu",
            );
        }
        if view.focus {
            draw_circle_lines(p.p.x, p.p.y, 8., 1., WHITE);
            draw_circle(p.p.x, p.p.y, 2., CYAN);
        }
        if g.god {
            self.mono("ENTRAINEMENT", W - 220., 770., 16., GOLD);
        }
    }
    fn overlay(&self, title: &str, line: &str, hint: &str) {
        draw_rectangle(0., 92., W, 706., Color::new(0.005, 0.015, 0.025, 0.73));
        draw_line(470., 340., 1130., 340., 1., tint(CYAN, 0.5));
        self.centered(title, W / 2., 410., 59., WHITE);
        self.centered(line, W / 2., 458., 27., CYAN);
        self.centered(hint, W / 2., 522., 21., MUTED);
    }
    fn title(&self, g: &Game, view: &View) {
        let hero = V::new(1190., 455. + (g.time * 0.8).sin() * 15.);
        self.light(
            hero + V::new(-190., 0.),
            180.,
            Color::new(0.12, 0.65, 1., 0.4),
        );
        draw_texture_ex(
            &self.ship,
            hero.x - 245.,
            hero.y - 107.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(490., 214.)),
                rotation: -0.035,
                ..Default::default()
            },
        );
        draw_rectangle(0., 0., W, H, Color::new(0.005, 0.01, 0.02, 0.12));
        self.mono("EXPEDITION  /  01.07.2249", 104., 116., 17., CYAN);
        draw_line(104., 143., 285., 143., 2., CYAN);
        self.text("ASTRA", 96., 300., 163., WHITE);
        self.text("FRACTURE", 96., 440., 163., WHITE);
        self.text(
            "Traversez le silence. Refermez la fracture.",
            104.,
            490.,
            26.,
            MUTED,
        );
        self.panel(
            Rect::new(104., 552., 422., 69.),
            Color::new(0.18, 0.83, 0.87, 0.96),
        );
        self.text("ENGAGER L'EXPEDITION", 128., 596., 29., INK);
        self.mono(
            if view.gamepad_connected {
                "B10"
            } else {
                "ENTREE"
            },
            437.,
            592.,
            17.,
            INK,
        );
        self.text(
            &format!("0{} / {}", view.selected + 1, SECTORS[view.selected].name),
            104.,
            675.,
            28.,
            WHITE,
        );
        self.mono(
            "FLECHES / STICK : SECTEUR   /   H : COMMANDES",
            104.,
            706.,
            14.,
            MUTED,
        );
        self.mono(view.gamepad, 104., 739., 13., CYAN);
        self.mono(
            &format!("RECORD PERSONNEL   {:08}", g.record),
            104.,
            790.,
            16.,
            CYAN,
        );
        self.mono(
            "UN SHOOT'EM UP ORIGINAL  /  MUSIQUES & VISUELS INEDITS",
            104.,
            844.,
            12.,
            MUTED,
        );
        self.mono(
            &format!(
                "{}   /   F4 : QUALITE   /   F3 : EFFETS",
                ["STANDARD", "HAUTE", "ULTRA", "4K"][self.quality]
            ),
            1000.,
            842.,
            12.,
            MUTED,
        );
        self.mono(
            if view.muted {
                "M : SON OFF"
            } else {
                "M : AUDIO ON"
            },
            1380.,
            116.,
            14.,
            MUTED,
        );
    }
    fn help(&self, view: &View) {
        draw_rectangle(0., 0., W, H, Color::new(0.005, 0.01, 0.02, 0.92));
        self.text("PILOTAGE", 104., 140., 64., WHITE);
        self.mono("CLAVIER / SOURIS", 104., 190., 19., CYAN);
        self.mono("LOGITECH DUAL ACTION", 1010., 190., 19., CYAN);
        let lines = [
            ("FLECHES / ZQSD / WASD", "Deplacement"),
            ("ESPACE / CLIC GAUCHE", "Tir maintenu"),
            ("MAJ", "Vol de precision : hitbox visible"),
            ("C", "Dash invulnerable — coute 22 energie"),
            ("V", "Surcharge — armes amplifiees pendant 6 s"),
            ("X / ENTREE EN VOL", "Activer l'amelioration selectionnee"),
            ("1 / 2 / 3", "Changer d'arme debloquee"),
            ("P / M", "Pause / couper le son"),
            ("F / F3 / F4", "Plein ecran / effets / qualite"),
            ("F1", "Mode entrainement invulnerable"),
        ];
        for (i, (key, desc)) in lines.iter().enumerate() {
            let y = 245. + i as f32 * 43.;
            self.mono(key, 104., y, 16., CYAN);
            self.text(desc, 380., y, 23., WHITE);
        }
        let pad_lines = [
            ("STICK G.", "Deplacement analogique"),
            ("B8", "Tir maintenu"),
            ("B6", "Surcharge / 100 energie"),
            ("B1", "Impulsion"),
            ("B4", "Eventail si debloque"),
            ("B3", "Lance phase si debloquee"),
            ("B2", "Halos et particules"),
            ("B10", "Start / pause / reprendre"),
            ("F2", "Activer / desactiver"),
        ];
        for (i, (key, desc)) in pad_lines.iter().enumerate() {
            let y = 245. + i as f32 * 43.;
            self.mono(key, 1010., y, 16., CYAN);
            self.text(desc, 1145., y, 23., WHITE);
        }
        self.mono(view.gamepad, 1010., 680., 14., GOLD);
        self.text(
            "B11 est le clic du stick : inutile de le maintenir.",
            1010.,
            710.,
            19.,
            MUTED,
        );
        self.text("Les cristaux font avancer le choix de bonus. Frolez les tirs pour gagner de l'energie.",260.,755.,23.,MUTED);
        self.mono("H POUR FERMER", 260., 803., 17., GOLD);
    }
    pub fn draw(&self, g: &Game, view: &View) {
        let shake = if self.effects { g.shake } else { 0. };
        let offset = vec2(
            (g.time * 73.).sin() * shake,
            (g.time * 91.).cos() * shake * 0.5,
        );
        set_camera(&Camera2D {
            render_target: Some(self.target.clone()),
            target: vec2(W / 2., H / 2.) + offset,
            ..Camera2D::from_display_rect(Rect::new(0., 0., W, H))
        });
        self.background(g);
        if g.mode != Mode::Title {
            self.walls(g);
            self.objects(g);
            for e in &g.enemies {
                self.enemy(e, g);
            }
            if let Some(b) = &g.boss {
                if matches!(g.mode, Mode::BossIntro | Mode::Boss) {
                    self.boss(b, g);
                }
            }
            self.projectiles(g);
            self.player(g);
            self.explosions(g);
            if g.flash > 0. && self.effects {
                draw_rectangle(0., 92., W, 706., Color::new(0.5, 0.8, 1., g.flash * 0.3));
            }
        }
        // World post-processing; draw the interface afterward so text stays crisp.
        set_default_camera();
        let fit = (screen_width() / W).min(screen_height() / H);
        let viewport = Rect::new(
            (screen_width() - W * fit) / 2.,
            (screen_height() - H * fit) / 2.,
            W * fit,
            H * fit,
        );
        clear_background(Color::from_rgba(2, 5, 10, 255));
        self.post.set_uniform(
            "Size",
            vec2(self.target.texture.width(), self.target.texture.height()),
        );
        self.post
            .set_uniform("Power", if self.effects { 1f32 } else { 0f32 });
        gl_use_material(&self.post);
        draw_texture_ex(
            &self.target.texture,
            viewport.x,
            viewport.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(viewport.w, viewport.h)),
                flip_y: true,
                ..Default::default()
            },
        );
        gl_use_default_material();
        set_camera(&Camera2D {
            zoom: vec2(2. / W, 2. / H),
            viewport: Some((
                viewport.x as i32,
                viewport.y as i32,
                viewport.w as i32,
                viewport.h as i32,
            )),
            ..Camera2D::from_display_rect(Rect::new(0., 0., W, H))
        });
        if g.mode == Mode::Title {
            self.title(g, view);
        } else {
            self.ui(g, view);
        }
        if view.help {
            self.help(view);
        }
        set_default_camera();
    }
}
const VERTEX: &str = r#"#version 100
attribute vec3 position;attribute vec2 texcoord;varying highp vec2 uv;uniform mat4 Model;uniform mat4 Projection;
void main(){gl_Position=Projection*Model*vec4(position,1.0);uv=texcoord;}
"#;
const FRAGMENT: &str = r#"#version 100
precision highp float;varying highp vec2 uv;uniform sampler2D Texture;uniform vec2 Size;uniform float Power;
vec3 bloom(vec2 p){return max(texture2D(Texture,p).rgb-vec3(0.50),vec3(0.0));}
void main(){vec3 c=texture2D(Texture,uv).rgb;vec2 d=vec2(3.8)/Size;
vec3 glow=bloom(uv+vec2(d.x,0.))+bloom(uv-vec2(d.x,0.))+bloom(uv+vec2(0.,d.y))+bloom(uv-vec2(0.,d.y));
glow+=(bloom(uv+d*2.)+bloom(uv-d*2.)+bloom(uv+vec2(d.x,-d.y)*2.)+bloom(uv+vec2(-d.x,d.y)*2.))*0.5;
c+=glow*0.17*Power;vec2 edge=(uv-.5)*2.;c*=1.-dot(edge,edge)*0.1;
gl_FragColor=vec4(clamp(c,0.,1.),1.);}
"#;
