//! Optional Logitech Dual Action input. No controller is required to play.
//! Physical HID usages are used on macOS, independent of SDL's axis ordering.
use crate::core::{Weapon, V};
use gilrs::ev::{AxisOrBtn, Code};
use gilrs::{Axis, Button, EventType, Gamepad, GamepadId, Gilrs, GilrsBuilder};

const fn button(number: u32) -> u16 {
    1 << (number - 1)
}

#[derive(Clone, Copy, Default, Debug)]
struct Snapshot {
    x: f32,
    y: f32, // Game coordinates: positive down.
    held: u16,
    pressed: u16,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct PadInput {
    pub axis: V,
    pub fire: bool,
    pub overdrive: bool,
    pub weapon: Option<Weapon>,
    pub effects: bool,
    pub start: bool,
    pub menu_x: i32,
    pub disconnected: bool,
}

#[derive(Default)]
struct Mapper {
    previous: u16,
    menu_direction: i32,
    next_repeat: f64,
}

fn stick(x: f32, y: f32) -> V {
    if !x.is_finite() || !y.is_finite() {
        return V::default();
    }
    let v = V::new(x.clamp(-1., 1.), y.clamp(-1., 1.));
    let length = v.len();
    // Radial deadzone avoids drift without losing diagonal/analogue movement.
    if length <= 0.18 {
        V::default()
    } else {
        v * (((length.min(1.) - 0.18) / 0.82) / length)
    }
}

impl Mapper {
    fn update(&mut self, raw: Snapshot, now: f64, attached: bool) -> PadInput {
        // Connecting or enabling a pad with Start/effects held must not trigger it.
        let edges = if attached {
            0
        } else {
            raw.pressed | (raw.held & !self.previous)
        };
        self.previous = raw.held;
        let axis = stick(raw.x, raw.y);
        let direction = if axis.x > 0.55 {
            1
        } else if axis.x < -0.55 {
            -1
        } else {
            0
        };
        let mut menu_x = 0;
        if attached {
            self.menu_direction = direction;
            self.next_repeat = now + 0.4;
        } else if direction != self.menu_direction {
            self.menu_direction = direction;
            self.next_repeat = now + 0.4;
            menu_x = direction;
        } else if direction != 0 && now >= self.next_repeat {
            menu_x = direction;
            self.next_repeat = now + 0.18;
        }
        PadInput {
            axis,
            fire: (raw.held | edges) & button(8) != 0,
            overdrive: edges & button(6) != 0,
            weapon: if edges & button(1) != 0 {
                Some(Weapon::Pulse)
            } else if edges & button(4) != 0 {
                Some(Weapon::Spread)
            } else if edges & button(3) != 0 {
                Some(Weapon::Beam)
            } else {
                None
            },
            effects: edges & button(2) != 0,
            start: edges & button(10) != 0,
            menu_x,
            disconnected: false,
        }
    }
}

// On macOS gilrs preserves the HID usage page and usage in Code.
// Undo gilrs' semantic Y inversion to recover the device's physical X/Y;
// this remains correct if SDL's axis ordering maps X/Y to the right stick.
#[cfg(target_os = "macos")]
fn physical_axis(value: f32, mapped: Option<AxisOrBtn>) -> f32 {
    match mapped {
        Some(AxisOrBtn::Axis(Axis::LeftStickY | Axis::RightStickY | Axis::DPadY)) => -value,
        _ => value,
    }
}

#[cfg(any(target_os = "macos", test))]
fn hid_button(native: u32) -> u16 {
    if native >> 16 == 9 && (1..=12).contains(&(native & 0xffff)) {
        button(native & 0xffff)
    } else {
        0
    }
}

fn physical_button(code: Code, mapped: Button) -> u16 {
    #[cfg(target_os = "macos")]
    {
        // Synthetic hat/axis events are not numbered Dual Action buttons.
        let _ = mapped;
        hid_button(code.into_u32())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = code;
        semantic_button(mapped)
    }
}

#[cfg(any(not(target_os = "macos"), test))]
fn semantic_button(mapped: Button) -> u16 {
    match mapped {
        Button::West => button(1),
        Button::South => button(2),
        Button::East => button(3),
        Button::North => button(4),
        Button::RightTrigger => button(6),
        Button::LeftTrigger2 => button(7),
        Button::RightTrigger2 => button(8),
        Button::Start => button(10),
        _ => 0,
    }
}

fn snapshot(pad: &Gamepad<'_>) -> Snapshot {
    let mut raw = Snapshot::default();
    for (code, data) in pad.state().buttons() {
        if data.is_pressed() {
            let mapped = match pad.axis_or_btn_name(code) {
                Some(AxisOrBtn::Btn(b)) => b,
                _ => Button::Unknown,
            };
            raw.held |= physical_button(code, mapped);
        }
    }
    #[cfg(target_os = "macos")]
    for (code, data) in pad.state().axes() {
        let value = physical_axis(data.value(), pad.axis_or_btn_name(code));
        match code.into_u32() {
            0x0001_0030 => raw.x = value,
            0x0001_0031 => raw.y = value,
            _ => {}
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        raw.x = pad.value(Axis::LeftStickX);
        raw.y = -pad.value(Axis::LeftStickY);
    }
    raw
}

pub struct Controller {
    gilrs: Option<Gilrs>,
    active: Option<GamepadId>,
    mapper: Mapper,
    enabled: bool,
    error: Option<String>,
    pub name: Option<String>,
}
impl Controller {
    pub fn new(enabled: bool) -> Self {
        Self {
            gilrs: None,
            active: None,
            mapper: Mapper::default(),
            enabled,
            error: None,
            name: None,
        }
    }
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
        self.active = None;
        self.mapper = Mapper::default();
        self.name = None;
        self.error = None; // Allow a fresh attempt after a temporary backend failure.
    }
    pub fn status(&self) -> &'static str {
        if !self.enabled {
            "MANETTE OFF / F2"
        } else if self.error.is_some() {
            "MANETTE INDISPONIBLE / F2"
        } else if self.active.is_some() {
            "DUAL ACTION / F2 : OFF"
        } else {
            "MANETTE AUTO / F2 : OFF"
        }
    }
    pub fn poll(&mut self, now: f64) -> PadInput {
        if !self.enabled || self.error.is_some() {
            return PadInput::default();
        }
        if self.gilrs.is_none() {
            match GilrsBuilder::new()
                .with_default_filters(false)
                .with_force_feedback(false)
                .build()
            {
                Ok(gilrs) => self.gilrs = Some(gilrs),
                Err(e) => {
                    eprintln!("Optional gamepad unavailable: {e}");
                    self.error = Some(e.to_string());
                    return PadInput::default();
                }
            }
        }
        let gilrs = self.gilrs.as_mut().unwrap();
        let mut presses = vec![];
        let mut disconnected = false;
        while let Some(event) = gilrs.next_event() {
            match event.event {
                EventType::ButtonPressed(mapped, code) => {
                    presses.push((event.id, physical_button(code, mapped)))
                }
                EventType::Disconnected if self.active == Some(event.id) => disconnected = true,
                _ => {}
            }
        }
        gilrs.inc();
        let selected = gilrs
            .gamepads()
            .find(|(_, pad)| pad.vendor_id() == Some(0x046d) && pad.product_id() == Some(0xc216))
            .map(|(id, _)| id);
        let changed = selected != self.active;
        disconnected |= self.active.is_some() && changed;
        if changed || disconnected {
            self.mapper = Mapper::default();
            self.active = selected;
            self.name = selected.map(|id| gilrs.gamepad(id).name().to_owned());
            eprintln!(
                "Gamepad: {}",
                self.name.as_deref().unwrap_or("disconnected")
            );
        }
        let Some(id) = selected else {
            return PadInput {
                disconnected,
                ..Default::default()
            };
        };
        let mut raw = snapshot(&gilrs.gamepad(id));
        for (candidate, mask) in presses {
            if candidate == id {
                raw.pressed |= mask;
            }
        }
        let mut input = self.mapper.update(raw, now, changed || disconnected);
        input.disconnected = disconnected;
        input
    }
}

pub fn probe(seconds: u64) {
    let mut controller = Controller::new(true);
    let start = std::time::Instant::now();
    let mut previous = String::new();
    while start.elapsed().as_secs() < seconds.clamp(1, 60) {
        let input = controller.poll(start.elapsed().as_secs_f64());
        let report = format!(
            "{} | {:?} | {:?}",
            controller.status(),
            controller.name,
            input
        );
        if report != previous {
            println!("{report}");
            previous = report;
        }
        std::thread::sleep(std::time::Duration::from_millis(8));
    }
    println!("GAMEPAD PROBE COMPLETE: {}", controller.status());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn physical_numbering_and_action_edges() {
        for (number, expected) in [(1, Weapon::Pulse), (4, Weapon::Spread), (3, Weapon::Beam)] {
            let mut m = Mapper::default();
            let raw = Snapshot {
                held: button(number),
                ..Default::default()
            };
            assert_eq!(m.update(raw, 0., false).weapon, Some(expected));
            assert!(m.update(raw, 0.1, false).weapon.is_none());
        }
        // The old B7 binding is unused; B8 fires and must never trigger overload.
        let mut isolated = Mapper::default();
        let old = isolated.update(
            Snapshot {
                held: button(7),
                ..Default::default()
            },
            0.,
            false,
        );
        assert!(!old.fire && !old.overdrive);
        let fire = isolated.update(
            Snapshot {
                held: button(8),
                ..Default::default()
            },
            0.1,
            false,
        );
        assert!(fire.fire && !fire.overdrive);
        let overload = isolated.update(
            Snapshot {
                held: button(6),
                ..Default::default()
            },
            0.2,
            false,
        );
        assert!(overload.overdrive && !overload.fire);
        let mut m = Mapper::default();
        let raw = Snapshot {
            held: button(2) | button(6) | button(8) | button(10),
            ..Default::default()
        };
        let first = m.update(raw, 0., false);
        assert!(first.fire && first.overdrive && first.effects && first.start);
        let held = m.update(raw, 0.1, false);
        assert!(held.fire && !held.overdrive && !held.effects && !held.start);
        let released = m.update(Snapshot::default(), 0.2, false);
        assert!(!released.fire && !released.overdrive && !released.effects && !released.start);
        assert!(m.update(raw, 0.3, false).start);
    }
    #[test]
    fn short_taps_connect_and_unused_click() {
        let mut m = Mapper::default();
        let tap = m.update(
            Snapshot {
                pressed: button(6),
                ..Default::default()
            },
            0.,
            false,
        );
        assert!(tap.overdrive);
        assert!(!m.update(Snapshot::default(), 0.1, false).overdrive);
        let held = Snapshot {
            held: button(2) | button(6) | button(10) | button(11),
            ..Default::default()
        };
        let connect = m.update(held, 0.2, true);
        assert!(!connect.start && !connect.effects && !connect.overdrive);
        let click = Mapper::default().update(
            Snapshot {
                held: button(11),
                ..Default::default()
            },
            0.,
            false,
        );
        assert_eq!(click.axis.len(), 0.);
        assert!(!click.start && !click.fire && click.weapon.is_none());
    }
    #[test]
    fn analogue_deadzone_orientation_diagonal_and_menu_repeat() {
        assert_eq!(stick(0.1, -0.08).len(), 0.);
        assert_eq!(stick(f32::NAN, 1.).len(), 0.);
        assert!(stick(0., -1.).y < -0.99);
        assert!((stick(1., 1.).len() - 1.).abs() < 0.0001);
        let mut m = Mapper::default();
        let right = Snapshot {
            x: 1.,
            ..Default::default()
        };
        assert_eq!(m.update(right, 0., false).menu_x, 1);
        assert_eq!(m.update(right, 0.1, false).menu_x, 0);
        assert_eq!(m.update(right, 0.41, false).menu_x, 1);
        m.update(Snapshot::default(), 0.5, false);
        assert_eq!(m.update(right, 0.51, false).menu_x, 1);
    }
    #[test]
    fn disabled_pad_does_not_initialize_or_keep_actions() {
        let mut controller = Controller::new(false);
        let input = controller.poll(0.);
        assert_eq!(input.axis.len(), 0.);
        assert!(!input.fire && !input.start);
        assert!(controller.gilrs.is_none());
        controller.toggle();
        assert!(controller.enabled && controller.active.is_none());
        controller.toggle();
        assert!(!controller.enabled && controller.active.is_none());
    }
    #[test]
    fn physical_hid_usages_and_gameplay_locks() {
        use crate::core::{Game, Input};
        for n in 1..=12 {
            assert_eq!(hid_button(0x0009_0000 | n), button(n));
        }
        assert_eq!(hid_button(0x0001_0030), 0);
        assert_eq!(hid_button(0x0009_0000), 0);
        let mut g = Game::default();
        g.start(0);
        let mut m = Mapper::default();
        let mut press = |number| {
            m.update(
                Snapshot {
                    pressed: button(number),
                    ..Default::default()
                },
                0.,
                false,
            )
        };
        let spread = press(4);
        g.tick(Input {
            weapon: spread.weapon,
            ..Default::default()
        });
        assert_eq!(g.player.weapon, Weapon::Pulse);
        g.player.spread = 1;
        g.tick(Input {
            weapon: spread.weapon,
            ..Default::default()
        });
        assert_eq!(g.player.weapon, Weapon::Spread);
        let beam = press(3);
        g.tick(Input {
            weapon: beam.weapon,
            ..Default::default()
        });
        assert_eq!(g.player.weapon, Weapon::Spread);
        g.player.beam = 1;
        g.tick(Input {
            weapon: beam.weapon,
            ..Default::default()
        });
        assert_eq!(g.player.weapon, Weapon::Beam);
        g.player.energy = 99.;
        g.tick(Input {
            overdrive: press(6).overdrive,
            ..Default::default()
        });
        assert_eq!(g.player.overload, 0.);
        g.player.energy = 100.;
        g.tick(Input {
            overdrive: press(6).overdrive,
            ..Default::default()
        });
        assert_eq!(g.player.overload, 6.);
        assert_eq!(g.player.energy, 0.);
    }
    #[test]
    fn portable_semantics_keep_logitech_numbers() {
        for (b, n) in [
            (Button::West, 1),
            (Button::South, 2),
            (Button::East, 3),
            (Button::North, 4),
            (Button::RightTrigger, 6),
            (Button::LeftTrigger2, 7),
            (Button::RightTrigger2, 8),
            (Button::Start, 10),
        ] {
            assert_eq!(semantic_button(b), button(n));
        }
        #[cfg(target_os = "macos")]
        {
            assert_eq!(
                physical_axis(0.8, Some(AxisOrBtn::Axis(Axis::RightStickY))),
                -0.8
            );
            assert_eq!(
                physical_axis(-0.8, Some(AxisOrBtn::Axis(Axis::RightStickX))),
                -0.8
            );
        }
    }
}
