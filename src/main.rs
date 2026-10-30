mod audio;
mod core;
mod graphics;
include!(concat!(env!("OUT_DIR"), "/assets.rs"));
use core::{Data, Game, Input, Mode};
use macroquad::prelude::*;
use std::{collections::HashMap, path::PathBuf};
fn conf() -> Conf {
    Conf {
        window_title: "Gradius 2 · Native Rust".into(),
        window_width: 1024,
        window_height: 896,
        window_resizable: true,
        high_dpi: true,
        platform: miniquad::conf::Platform {
            apple_gfx_api: miniquad::conf::AppleGfxApi::OpenGl,
            ..Default::default()
        },
        ..Default::default()
    }
}
struct Art {
    textures: HashMap<String, Texture2D>,
}
impl Art {
    fn new() -> Self {
        let mut a = Self {
            textures: HashMap::new(),
        };
        for i in 0..12 {
            for prefix in ["map", "enemies", "sprites"] {
                a.add(format!("{prefix}-{i:02}.png"));
            }
        }
        for i in 1..=5 {
            a.add(format!("boss-{i}.png"));
        }
        for n in ["title.png", "ship.png", "font.png"] {
            a.add(n.into());
        }
        a
    }
    fn add(&mut self, name: String) {
        let t = Texture2D::from_file_with_format(asset(&name), Some(ImageFormat::Png));
        t.set_filter(FilterMode::Nearest);
        self.textures.insert(name, t);
    }
    fn tex(&self, name: &str) -> &Texture2D {
        &self.textures[name]
    }
    fn text(&self, s: &str, x: f32, y: f32, c: Color) {
        for (i, ch) in s.to_ascii_uppercase().chars().enumerate() {
            let xx = x + i as f32 * 6.;
            if ch == ':' {
                draw_rectangle(xx + 2., y + 2., 1., 1., c);
                draw_rectangle(xx + 2., y + 5., 1., 1., c);
                continue;
            }
            if ch == '/' {
                draw_line(xx + 1., y + 6., xx + 4., y + 1., 1., c);
                continue;
            }
            let n = ch as usize;
            if (48..96).contains(&n) {
                let n = n - 48;
                draw_texture_ex(
                    self.tex("font.png"),
                    x + i as f32 * 6.,
                    y,
                    c,
                    DrawTextureParams {
                        source: Some(Rect::new((n % 16 * 8) as f32, (n / 16 * 8) as f32, 8., 8.)),
                        dest_size: Some(vec2(6., 8.)),
                        ..Default::default()
                    },
                );
            }
        }
    }
    fn enemy(&self, stage: usize, t: usize, x: f32, y: f32) {
        draw_texture_ex(
            self.tex(&format!("enemies-{stage:02}.png")),
            x - 4.,
            y + 20.,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    (t % 16 * 24) as f32,
                    (t / 16 * 24) as f32,
                    24.,
                    24.,
                )),
                ..Default::default()
            },
        );
    }
    fn sprite(&self, stage: usize, n: usize, x: f32, y: f32, color: Color) {
        let n = n / 4;
        draw_texture_ex(
            self.tex(&format!("sprites-{stage:02}.png")),
            x,
            y + 24.,
            color,
            DrawTextureParams {
                source: Some(Rect::new(
                    (n % 16 * 16) as f32,
                    (n / 16 * 16) as f32,
                    16.,
                    16.,
                )),
                ..Default::default()
            },
        );
    }
}
fn draw_game(g: &Game, a: &Art, menu: bool, selected: usize, paused: bool) {
    clear_background(BLACK);
    if g.mode == Mode::Title {
        draw_texture(a.tex("title.png"), 0., 0., WHITE);
        draw_rectangle(0., 104., 256., 120., BLACK);
        a.text("METALION", 104., 128., SKYBLUE);
        a.text("ESPACE : TIR  /  SHIFT : BONUS", 44., 148., GRAY);
        a.text("ENTER  START", 91., 169., WHITE);
        a.text("FLECHES : CHOISIR SECTEUR", 45., 190., GRAY);
        a.text(&format!("SECTEUR {:02}", selected), 99., 202., YELLOW);
        return;
    }
    let st = &g.data.stages[g.stage];
    let map = a.tex(&format!("map-{:02}.png", g.stage));
    draw_texture_ex(
        map,
        0.,
        24.,
        WHITE,
        DrawTextureParams {
            source: Some(Rect::new(g.scroll, 0., 256., 176.)),
            ..Default::default()
        },
    );
    a.text(&format!("1P {:07}", g.score), 4., 2., WHITE);
    a.text(&format!("HI {:07}", g.high), 141., 2., WHITE);
    a.text(&format!("LIVES {}", g.lives.max(0)), 4., 13., SKYBLUE);
    a.text(&format!("STAGE {:02}", g.stage), 105., 13., SKYBLUE);
    a.text(
        if g.campaign >= 7 {
            "RETURN"
        } else {
            "METALION"
        },
        201.,
        13.,
        GRAY,
    );
    for e in &g.enemies {
        a.enemy(g.stage, e.kind, e.p.x, e.p.y);
    }
    for p in &g.pickups {
        a.enemy(g.stage, 0x29 + p.kind as usize, p.p.x, p.p.y);
    }
    if let Some(b) = &g.boss {
        if b.hp > 0 || b.defeated % 8 < 4 {
            if b.kind > 0 {
                draw_texture(
                    a.tex(&format!("boss-{}.png", b.kind)),
                    b.p.x,
                    b.p.y + 24.,
                    WHITE,
                );
            } else {
                draw_rectangle(b.p.x, b.p.y + 24., 56., 80., DARKBLUE);
                for i in 0..5 {
                    draw_rectangle_lines(
                        b.p.x + 4. + i as f32 * 3.,
                        b.p.y + 30. + i as f32 * 5.,
                        40. - i as f32 * 4.,
                        66. - i as f32 * 8.,
                        1.,
                        SKYBLUE,
                    );
                }
                draw_circle(
                    b.p.x + 20.,
                    b.p.y + 64.,
                    7.,
                    if g.frame % 10 < 5 { RED } else { ORANGE },
                );
            }
        }
        if b.hp > 0 {
            draw_rectangle(40., 26., 176., 2., DARKGRAY);
            draw_rectangle(40., 26., 176. * b.hp as f32 / b.max_hp as f32, 2., RED);
        } else {
            a.text("ENTER THE CORE", 78., 44., YELLOW);
        }
    }
    for s in &g.shots {
        if s.friendly {
            draw_rectangle(
                s.p.x,
                s.p.y + 23.,
                if s.kind == 1 { 24. } else { 6. },
                if s.kind == 2 { 4. } else { 2. },
                if s.kind == 1 { SKYBLUE } else { WHITE },
            );
        } else {
            draw_circle(s.p.x, s.p.y + 24., 2., ORANGE);
            draw_circle(s.p.x, s.p.y + 24., 1., WHITE);
        }
    }
    if !matches!(g.mode, Mode::Dead | Mode::GameOver) {
        for i in 0..g.options {
            let p = g.trail[(i + 1) * 16];
            a.sprite(g.stage, 0x1c, p.x, p.y, ORANGE);
        }
        {
            // Keep the ship visible throughout the invulnerability period.
            let frame = if menu {
                0
            } else if is_key_down(KeyCode::Up) {
                1
            } else if is_key_down(KeyCode::Down) {
                2
            } else {
                0
            };
            draw_texture_ex(
                a.tex("ship.png"),
                g.player.x - 4.,
                g.player.y + 20.,
                if g.invul > 0 {
                    Color::new(0.65, 0.85, 1., 1.)
                } else {
                    WHITE
                },
                DrawTextureParams {
                    source: Some(Rect::new(frame as f32 * 24., 0., 24., 24.)),
                    ..Default::default()
                },
            );
            if g.shield > 0 {
                a.sprite(g.stage, 0x2c, g.player.x + 12., g.player.y, SKYBLUE);
            }
        }
    }
    for f in &g.fx {
        let size = 3. + f.age as f32 * 0.6;
        draw_circle_lines(
            f.p.x + 8.,
            f.p.y + 32.,
            size,
            2.,
            if f.age < 12 { YELLOW } else { RED },
        );
    }
    draw_rectangle(0., 200., 256., 24., BLACK);
    let names = ["SPEED", "MISSIL", "DOUBLE", "LASER", "OPTION", "SHIELD"];
    for (i, n) in names.iter().enumerate() {
        let x = 2. + i as f32 * 42.;
        let active = g.meter == i + 1;
        let owned = match i {
            0 => g.speed > 0,
            1 => g.missile,
            2 => g.double,
            3 => g.laser,
            4 => g.options > 0,
            _ => g.shield > 0,
        };
        draw_rectangle_lines(
            x,
            204.,
            40.,
            15.,
            1.,
            if active { ORANGE } else { DARKBLUE },
        );
        a.text(
            n,
            x + 2.,
            208.,
            if active {
                YELLOW
            } else if owned {
                SKYBLUE
            } else {
                GRAY
            },
        );
    }
    let progress = (g.scroll / (st.end as f32 * 8.)).clamp(0., 1.);
    draw_rectangle(0., 222., 256. * progress, 2., DARKBLUE);
    if paused || matches!(g.mode, Mode::GameOver | Mode::Clear | Mode::Victory) {
        draw_rectangle(16., 88., 224., 55., Color::new(0., 0., 0., 0.88));
        let title = if paused {
            "PAUSE"
        } else {
            match g.mode {
                Mode::GameOver => "GAME OVER",
                Mode::Victory => "MISSION COMPLETE",
                _ => {
                    if g.stage == 8 {
                        "YOU CAPTURED"
                    } else {
                        "SECTOR CLEAR"
                    }
                }
            }
        };
        a.text(title, 128. - title.len() as f32 * 3., 99., YELLOW);
        if paused {
            a.text("P : REPRENDRE", 91., 118., WHITE);
        } else if g.mode != Mode::Clear {
            a.text("ENTER : MENU", 94., 118., WHITE);
        }
    }
    if g.god {
        a.text("TRAINING", 201., 188., YELLOW);
    }
}
fn load_game() -> Game {
    let data: Data = serde_json::from_slice(asset("game.json")).expect("game data");
    let masks = (0..12)
        .map(|i| asset(&format!("collision-{i:02}.bin")).to_vec())
        .collect();
    Game::new(data, masks)
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--export-audio") {
        let dir = PathBuf::from(args.get(i + 1).expect("output directory"));
        std::fs::create_dir_all(&dir).unwrap();
        for id in (1..=0x42).chain(std::iter::once(0x5b)) {
            let r = audio::render(asset(&format!("audio/{id:02x}.bin")));
            std::fs::write(dir.join(format!("{id:02x}.wav")), audio::wav(&r.pcm)).unwrap();
        }
        println!("67 WAV exported: {}", dir.display());
        return;
    }
    if args.iter().any(|a| a == "--self-test") {
        let mut g = load_game();
        g.start();
        g.god = true;
        for stage in 0..12 {
            g.load(stage, 0.);
            for t in 0..1800 {
                g.tick(Input {
                    x: if t % 240 < 120 { 0.2 } else { -0.2 },
                    y: if t % 120 < 60 { 0.5 } else { -0.5 },
                    fire: true,
                    power: t % 600 == 0,
                });
            }
            assert!(g.player.x.is_finite());
            assert!(g.scroll > 0.);
            println!(
                "stage {stage}: 1800 native ticks OK, scroll {}, {} enemies",
                g.scroll,
                g.enemies.len()
            );
        }
        println!("SELF TEST PASS");
        return;
    }
    macroquad::Window::from_config(conf(), run());
}
async fn run() {
    let args: Vec<_> = std::env::args().collect();
    let smoke = args.iter().any(|a| a == "--smoke-test");
    let sprite_check = args.iter().any(|a| a == "--sprite-check");
    let visual_check = args.iter().any(|a| a == "--visual-check");
    let mut modern = !args.iter().any(|a| a == "--retro") && !sprite_check;
    let mut effects = !args.iter().any(|a| a == "--no-effects");
    let presentation = graphics::Modern::new();
    let a = Art::new();
    let mut g = load_game();
    let mut sound = audio::Audio::new();
    let mut selected = 1;
    let mut paused = false;
    let mut fullscreen = false;
    let mut accumulator = 0.;
    let mut last = get_time();
    let mut smoke_frame = 0;
    let rt = render_target(256, 224);
    rt.texture.set_filter(FilterMode::Nearest);
    let camera = Camera2D {
        render_target: Some(rt.clone()),
        ..Camera2D::from_display_rect(Rect::new(0., 0., 256., 224.))
    };
    loop {
        if is_key_pressed(KeyCode::Escape) {
            if g.mode == Mode::Title {
                break;
            }
            g.mode = Mode::Title;
            sound.stop();
            paused = false;
        }
        if is_key_pressed(KeyCode::M) {
            sound.enabled = !sound.enabled;
        }
        if is_key_pressed(KeyCode::F) {
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
        }
        if is_key_pressed(KeyCode::P) && g.mode == Mode::Playing {
            paused = !paused;
            if paused {
                sound.stop();
            }
        }
        if is_key_pressed(KeyCode::F2) {
            modern = !modern;
        }
        if is_key_pressed(KeyCode::F3) {
            effects = !effects;
        }
        if is_key_pressed(KeyCode::F1) {
            g.god = !g.god;
        }
        if g.mode == Mode::Title {
            if is_key_pressed(KeyCode::Right) {
                selected = (selected + 1) % 12;
            }
            if is_key_pressed(KeyCode::Left) {
                selected = (selected + 11) % 12;
            }
            if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                g.start();
                if selected != 1 {
                    g.load(selected, 0.);
                    g.practice = true;
                }
                accumulator = 0.;
                last = get_time();
            }
        } else if matches!(g.mode, Mode::GameOver | Mode::Victory) && is_key_pressed(KeyCode::Enter)
        {
            g.mode = Mode::Title;
        }
        let now = get_time();
        accumulator += (now - last).min(0.1);
        last = now;
        let mut sounds = vec![];
        let mut power = is_key_pressed(KeyCode::LeftShift)
            || is_key_pressed(KeyCode::RightShift)
            || is_key_pressed(KeyCode::X);
        while accumulator >= 1. / 60. {
            if !paused {
                let input = Input {
                    x: (is_key_down(KeyCode::Right) as i32 - is_key_down(KeyCode::Left) as i32)
                        as f32,
                    y: (is_key_down(KeyCode::Down) as i32 - is_key_down(KeyCode::Up) as i32) as f32,
                    fire: is_key_down(KeyCode::Space) || is_key_down(KeyCode::Z),
                    power,
                };
                g.tick(input);
                sounds.extend_from_slice(&g.sounds);
                power = false;
            }
            accumulator -= 1. / 60.;
        }
        if !paused && !sprite_check && !visual_check {
            sound.update(g.music, &sounds, now).await;
        }
        if smoke {
            smoke_frame += 1;
            if smoke_frame == 30 {
                g.start();
                g.god = true;
                g.scroll = 1280.;
                g.meter = 5;
                g.power();
            }
            if smoke_frame == 90 {
                g.load(3, 1800.);
            }
            if smoke_frame == 150 {
                g.load(1, g.data.stages[1].end as f32 * 8.);
            }
            g.invul = 0;
        }
        if sprite_check {
            smoke_frame += 1;
            g.start();
            g.scroll = 1280.;
            g.invul = 120;
            g.frame = smoke_frame;
        }
        if visual_check {
            smoke_frame += 1;
            g.frame = 120;
            if smoke_frame <= 3 {
                g.mode = Mode::Title;
            } else {
                g.start();
                g.frame = 120;
                g.scroll = 1280.;
                g.invul = 0;
                g.options = 2;
                g.trail[16] = core::P::new(44., 86.);
                g.trail[32] = core::P::new(25., 96.);
                g.enemies.push(core::Enemy {
                    p: core::P::new(159., 66.),
                    origin: core::P::new(159., 66.),
                    kind: 11,
                    age: 0,
                    hp: 1,
                    capsule: true,
                    turret: false,
                });
                g.enemies.push(core::Enemy {
                    p: core::P::new(205., 115.),
                    origin: core::P::new(205., 115.),
                    kind: 4,
                    age: 0,
                    hp: 1,
                    capsule: false,
                    turret: false,
                });
                g.pickups.push(core::Pickup {
                    p: core::P::new(128., 105.),
                    kind: 0,
                });
                for (x, y, friendly) in [(100., 88., true), (119., 88., true), (165., 110., false)]
                {
                    g.shots.push(core::Shot {
                        p: core::P::new(x, y),
                        v: core::P::default(),
                        friendly,
                        kind: 0,
                        age: 0,
                    });
                }
                if smoke_frame > 6 {
                    g.scroll = g.data.stages[1].end as f32 * 8.;
                    g.boss = Some(core::Boss {
                        kind: 1,
                        p: core::P::new(166., 40.),
                        hp: 18,
                        max_hp: 18,
                        age: 0,
                        defeated: 0,
                    });
                }
            }
        }
        set_camera(&camera);
        draw_game(&g, &a, false, selected, paused);
        if sprite_check {
            for y in 0..8 {
                for x in 0..32 {
                    draw_rectangle(
                        x as f32 * 8.,
                        132. + y as f32 * 8.,
                        8.,
                        8.,
                        if (x + y) % 2 == 0 {
                            Color::from_rgba(32, 64, 112, 255)
                        } else {
                            Color::from_rgba(72, 112, 160, 255)
                        },
                    );
                }
            }
            for (i, t) in [1, 11, 41, 42, 43, 44, 45].iter().enumerate() {
                a.enemy(1, *t, 16. + i as f32 * 32., 136.);
            }
        }
        set_default_camera();
        if modern {
            presentation.upscale(&rt.texture);
        }
        clear_background(Color::from_rgba(6, 8, 15, 255));
        let fit = (screen_width() / 256.).min(screen_height() / 224.);
        let scale = if modern {
            fit.max(0.25)
        } else {
            fit.floor().max(1.)
        };
        let viewport = Rect::new(
            (screen_width() - 256. * scale) / 2.,
            (screen_height() - 224. * scale) / 2.,
            256. * scale,
            224. * scale,
        );
        if modern {
            presentation.draw(&g, viewport, effects, selected);
        } else {
            draw_texture_ex(
                &rt.texture,
                viewport.x,
                viewport.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(viewport.w, viewport.h)),
                    flip_y: true,
                    ..Default::default()
                },
            );
            if g.mode == Mode::Title && !sprite_check {
                draw_text(
                    "RETRO / F2 : MODERNE",
                    viewport.x + 10.,
                    viewport.y + viewport.h - 10.,
                    16.,
                    SKYBLUE,
                );
            }
        }
        if visual_check && [3, 6, 9].contains(&smoke_frame) {
            let path =
                PathBuf::from(std::env::var("GRADIUS_SMOKE_DIR").unwrap_or_else(|_| ".".into()));
            std::fs::create_dir_all(&path).unwrap();
            let scene = if smoke_frame == 3 {
                "title"
            } else if smoke_frame == 6 {
                "stage"
            } else {
                "boss"
            };
            let style = if modern { "modern" } else { "retro" };
            get_screen_data()
                .export_png(path.join(format!("{style}-{scene}.png")).to_str().unwrap());
            if smoke_frame == 6 && modern {
                rt.texture
                    .get_texture_data()
                    .export_png(path.join("base.png").to_str().unwrap());
                presentation.export_scale(&path.join("scale4x.png"));
            }
        }
        if sprite_check || (smoke && [20, 80, 140, 200].contains(&smoke_frame)) {
            let path = std::env::var("GRADIUS_SMOKE_DIR").unwrap_or_else(|_| ".".into());
            std::fs::create_dir_all(&path).unwrap();
            get_screen_data().export_png(&format!(
                "{path}/{}-{smoke_frame}.png",
                if sprite_check { "sprites" } else { "smoke" }
            ));
        }
        if (visual_check && smoke_frame >= 9)
            || (sprite_check && smoke_frame >= 6)
            || (smoke && smoke_frame >= 220)
        {
            break;
        }
        next_frame().await;
    }
    sound.stop();
}
