mod audio;
mod controller;
mod core;
mod render;
include!(concat!(env!("OUT_DIR"), "/assets.rs"));
use crate::core::{Game, Input, Mode, Weapon, DT, V};
use macroquad::prelude::*;
use std::path::PathBuf;
fn conf() -> Conf {
    Conf {
        window_title: "ASTRA FRACTURE".into(),
        window_width: 1440,
        window_height: 810,
        high_dpi: true,
        window_resizable: true,
        sample_count: 1,
        platform: miniquad::conf::Platform {
            apple_gfx_api: miniquad::conf::AppleGfxApi::OpenGl,
            ..Default::default()
        },
        ..Default::default()
    }
}
fn record_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(|p| PathBuf::from(p).join("Library/Application Support/Astra Fracture/record.txt"))
}
fn read_record() -> u64 {
    record_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}
fn save_record(score: u64) {
    if let Some(p) = record_path() {
        if let Some(parent) = p.parent() {
            if std::fs::create_dir_all(parent).is_ok() {
                let _ = std::fs::write(p, score.to_string());
            }
        }
    }
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--gamepad-test") {
        controller::probe(args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(3));
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--compose") {
        audio::compose(&PathBuf::from(args.get(i + 1).expect("output directory")));
        return;
    }
    if args.iter().any(|a| a == "--self-test") {
        for stage in 0..3 {
            let mut g = Game::new(0x543212);
            g.start(stage);
            g.god = true;
            g.player.spread = 2;
            g.player.missiles = 2;
            g.player.drones = 2;
            g.player.weapon = Weapon::Spread;
            for frame in 0..60000 {
                let target_y = g
                    .boss
                    .as_ref()
                    .map(|b| b.p.y)
                    .unwrap_or(450. + (frame as f32 * 0.006).sin() * 230.);
                let y = ((target_y - g.player.p.y) / 90.).clamp(-0.9, 0.9);
                g.tick(Input {
                    axis: V::new(0., y),
                    fire: true,
                    ..Default::default()
                });
                g.events.clear();
                if g.mode == Mode::Victory {
                    break;
                }
                assert!(g.player.p.x.is_finite());
                assert!(g.bullets.len() <= 1400);
                assert!(g.enemies.len() <= 60);
            }
            assert_eq!(
                g.mode,
                Mode::Victory,
                "autoplay did not complete the campaign"
            );
            println!(
                "sector {} simulation passed; state {:?}, score {}, kills {}",
                stage + 1,
                g.mode,
                g.score,
                g.kills
            );
        }
        println!("SELF TEST PASS");
        return;
    }
    macroquad::Window::from_config(conf(), run());
}
fn demo(g: &mut Game, scene: usize) {
    let sector = if scene < 3 {
        0
    } else if scene < 5 {
        1
    } else {
        2
    };
    g.start(sector);
    g.god = true;
    g.elapsed = 42.;
    g.time = 17.;
    g.player.invul = 0.;
    g.player.spread = 2;
    g.player.beam = 2;
    g.player.missiles = 2;
    g.player.drones = 2;
    g.player.shield = 3;
    g.player.weapon = Weapon::Beam;
    g.player.energy = 83.;
    g.player.p = V::new(330., 450.);
    g.trail = vec![V::new(210., 480.); 90];
    g.trail[50] = V::new(140., 535.);
    g.score = 18750;
    g.combo = 14;
    g.combo_time = 3.;
    g.meter = 5;
    g.enemies.clear();
    g.bullets.clear();
    if scene == 0 {
        g.mode = Mode::Title;
        return;
    }
    if scene == 7 {
        g.mode = Mode::Victory;
        return;
    }
    if scene == 2 || scene == 4 || scene == 6 {
        let max = [420., 620., 840.][sector];
        g.mode = Mode::Boss;
        g.boss = Some(core::Boss {
            p: V::new(1230., 435.),
            hp: max * 0.58,
            max,
            age: 13.,
            shoot: 1.,
            volley: 10,
            phase: 2,
            hit: 0.,
        });
        for i in 0..18 {
            let a = 2.2 + i as f32 * 0.055;
            let p = V::new(1140., 435.) + V::angle(a) * 340.;
            g.bullets.push(core::Bullet {
                p,
                old: p,
                vel: V::angle(a) * 210.,
                friendly: false,
                kind: sector as u8,
                radius: 6.,
                damage: 1.,
                age: 1.,
                grazed: false,
            });
        }
    } else {
        for (kind, p) in [
            (core::Kind::Scout, V::new(1020., 320.)),
            (core::Kind::Weaver, V::new(1180., 585.)),
            (core::Kind::Carrier, V::new(1360., 420.)),
            (core::Kind::Lancer, V::new(845., 650.)),
        ] {
            g.spawn(kind, p, false);
        }
        g.rocks.push(core::Rock {
            p: V::new(1430., 640.),
            radius: 45.,
            hp: 10.,
            angle: 0.8,
            seed: 123,
        });
    }
    for i in 0..4 {
        let p = V::new(490. + i as f32 * 125., 450.);
        g.bullets.push(core::Bullet {
            p,
            old: p,
            vel: V::new(1750., 0.),
            friendly: true,
            kind: 1,
            radius: 4.,
            damage: 3.,
            age: 0.2,
            grazed: false,
        });
    }
    for i in 0..3 {
        g.pickups.push(core::Pickup {
            p: V::new(645. + i as f32 * 95., 570. - i as f32 * 25.),
            age: 1.,
            kind: 0,
            phase: 0.,
        });
    }
    g.add_fx(V::new(810., 285.), 60., 1);
    if let Some(f) = g.fx.last_mut() {
        f.age = 0.25;
    }
}
async fn run() {
    let args: Vec<_> = std::env::args().collect();
    let smoke = args.iter().any(|a| a == "--smoke-test");
    let mut renderer = render::Renderer::new();
    if args.iter().any(|a| a == "--4k") {
        renderer.cycle_quality();
        renderer.cycle_quality();
    }
    let mut controller =
        controller::Controller::new(!smoke && !args.iter().any(|a| a == "--no-gamepad"));
    let mut game = Game::default();
    if !smoke {
        game.record = read_record();
    }
    renderer.draw(
        &game,
        &render::View {
            paused: false,
            focus: false,
            selected: 0,
            help: false,
            muted: false,
            gamepad: controller.status(),
            gamepad_connected: false,
        },
    );
    next_frame().await;
    let mut audio = audio::Audio::new().await;
    let mut paused = false;
    let mut help = false;
    let mut selected = 0;
    let mut fullscreen = false;
    let mut accumulator = 0f64;
    let mut last = get_time();
    let mut frames = 0u32;
    let mut saved = game.record;
    let mut pending = Input::default();
    loop {
        let now = get_time();
        let dt = (now - last).min(0.12);
        last = now;
        if is_key_pressed(KeyCode::F2) {
            controller.toggle();
        }
        let pad = controller.poll(now);
        let pad_start = pad.start && !help;
        if pad.start && help {
            help = false;
        }
        if pad.disconnected && !matches!(game.mode, Mode::Title | Mode::GameOver | Mode::Victory) {
            paused = true;
        }
        let mut one_shot = Input {
            dash: is_key_pressed(KeyCode::C) || is_mouse_button_pressed(MouseButton::Right),
            overdrive: is_key_pressed(KeyCode::V) || pad.overdrive,
            upgrade: is_key_pressed(KeyCode::X)
                || (is_key_pressed(KeyCode::Enter) && game.mode != Mode::Title),
            weapon: if is_key_pressed(KeyCode::Key1) {
                Some(Weapon::Pulse)
            } else if is_key_pressed(KeyCode::Key2) {
                Some(Weapon::Spread)
            } else if is_key_pressed(KeyCode::Key3) {
                Some(Weapon::Beam)
            } else {
                pad.weapon
            },
            ..Default::default()
        };
        if is_key_pressed(KeyCode::Escape) {
            if help {
                help = false;
            } else if game.mode == Mode::Title {
                break;
            } else {
                game.mode = Mode::Title;
                game.mode_time = 0.;
                paused = false;
            }
        }
        if is_key_pressed(KeyCode::H) {
            help = !help;
        }
        if (is_key_pressed(KeyCode::P) || pad_start)
            && !matches!(game.mode, Mode::Title | Mode::GameOver | Mode::Victory)
        {
            paused = !paused;
        }
        if is_key_pressed(KeyCode::M) {
            audio.enabled = !audio.enabled;
        }
        if is_key_pressed(KeyCode::F) {
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
        }
        if is_key_pressed(KeyCode::F3) || pad.effects {
            renderer.effects = !renderer.effects;
        }
        if is_key_pressed(KeyCode::F4) {
            renderer.cycle_quality();
        }
        if is_key_pressed(KeyCode::F1) {
            game.god = !game.god;
        }
        if game.mode == Mode::Title && !help {
            if is_key_pressed(KeyCode::Right) || pad.menu_x > 0 {
                selected = (selected + 1) % 3;
            }
            if is_key_pressed(KeyCode::Left) || pad.menu_x < 0 {
                selected = (selected + 2) % 3;
            }
            let (mx, my) = mouse_position();
            let fit = (screen_width() / core::W).min(screen_height() / core::H);
            let mouse = vec2(
                (mx - (screen_width() - core::W * fit) / 2.) / fit,
                (my - (screen_height() - core::H * fit) / 2.) / fit,
            );
            let clicked = is_mouse_button_pressed(MouseButton::Left)
                && Rect::new(104., 552., 422., 69.).contains(mouse);
            if is_key_pressed(KeyCode::Enter) || clicked || pad_start {
                game.start(selected);
                paused = false;
                accumulator = 0.;
                one_shot = Input::default();
                pending = Input::default();
            }
        } else if matches!(game.mode, Mode::GameOver | Mode::Victory)
            && (is_key_pressed(KeyCode::Enter) || pad_start)
        {
            game.start(0);
            paused = false;
            pending = Input::default();
            one_shot = Input::default();
        }
        let axis = V::new(
            (is_key_down(KeyCode::Right) || is_key_down(KeyCode::D)) as i32 as f32
                - (is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) || is_key_down(KeyCode::Q))
                    as i32 as f32,
            (is_key_down(KeyCode::Down) || is_key_down(KeyCode::S)) as i32 as f32
                - (is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) || is_key_down(KeyCode::Z))
                    as i32 as f32,
        );
        // Keyboard takes priority while a direction is held; otherwise use the analogue stick.
        let axis = if axis.len() > 0. { axis } else { pad.axis };
        let focus = is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift);
        let fire = is_key_down(KeyCode::Space)
            || is_key_down(KeyCode::J)
            || is_mouse_button_down(MouseButton::Left)
            || pad.fire;
        let mut sounds = vec![];
        if !paused && !help && !smoke {
            pending.dash |= one_shot.dash;
            pending.overdrive |= one_shot.overdrive;
            pending.upgrade |= one_shot.upgrade;
            pending.weapon = one_shot.weapon.or(pending.weapon);
            accumulator += dt;
            while accumulator >= DT as f64 {
                game.tick(Input {
                    axis,
                    fire,
                    focus,
                    ..pending
                });
                sounds.append(&mut game.events);
                pending = Input::default();
                accumulator -= DT as f64;
            }
        } else {
            accumulator = 0.;
            pending = Input::default();
        }
        if !smoke {
            let track = if game.mode == Mode::Title {
                0
            } else if matches!(game.mode, Mode::Boss | Mode::BossIntro) {
                4
            } else {
                1 + game.sector
            };
            audio.update(track, &sounds, dt as f32).await;
            if matches!(game.mode, Mode::GameOver | Mode::Victory)
                && game.record > saved
                && !game.god
            {
                save_record(game.record);
                saved = game.record;
            }
        }
        frames += 1;
        if smoke {
            let scene = match frames {
                0..=8 => 0,
                9..=20 => 1,
                21..=32 => 2,
                33..=44 => 3,
                45..=56 => 4,
                57..=68 => 5,
                69..=80 => 6,
                _ => 7,
            };
            demo(&mut game, scene);
        }
        renderer.draw(
            &game,
            &render::View {
                paused,
                focus,
                selected,
                help,
                muted: !audio.enabled,
                gamepad: controller.status(),
                gamepad_connected: controller.name.is_some(),
            },
        );
        if smoke && [8, 20, 32, 44, 56, 68, 80, 92].contains(&frames) {
            let path = std::env::var("ASTRA_CAPTURE_DIR").unwrap_or_else(|_| ".".into());
            std::fs::create_dir_all(&path).unwrap();
            get_screen_data().export_png(&format!("{path}/astra-{frames:03}.png"));
        }
        if smoke && frames >= 94 {
            break;
        }
        next_frame().await;
    }
    audio.stop();
    if !smoke && game.record > saved && !game.god {
        save_record(game.record);
    }
}
