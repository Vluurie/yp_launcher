use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, TrySendError};
use std::sync::Arc;
use std::time::{Duration, Instant};

use gilrs::{Axis, Button, EventType, Gamepad, Gilrs};
use yp_core::{CancelToken, SettingKey};

pub const CONTROLLER_NAVIGATION: SettingKey<bool> = SettingKey::new("controller_navigation", || true);
pub const POLL_INTERVAL: Duration = Duration::from_millis(16);
const REPEAT_DELAY: Duration = Duration::from_millis(350);
const REPEAT_INTERVAL: Duration = Duration::from_millis(100);
const STICK_THRESHOLD: f32 = 0.55;
const STICK_RELEASE: f32 = 0.35;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerAction {
    Move(Direction),
    Confirm,
    Back,
    PreviousTab,
    NextTab,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug, Clone, Copy, Default)]
struct Snapshot {
    dpad: Option<Direction>,
    stick_x: f32,
    stick_y: f32,
    scroll_y: f32,
    confirm: bool,
    back: bool,
    previous_tab: bool,
    next_tab: bool,
}

impl Snapshot {
    fn from_gamepad(gamepad: Gamepad<'_>) -> Self {
        let dpad = [
            (Button::DPadUp, Direction::Up),
            (Button::DPadDown, Direction::Down),
            (Button::DPadLeft, Direction::Left),
            (Button::DPadRight, Direction::Right),
        ]
        .into_iter()
        .find_map(|(button, direction)| gamepad.is_pressed(button).then_some(direction));
        Self {
            dpad,
            stick_x: gamepad.value(Axis::LeftStickX),
            stick_y: gamepad.value(Axis::LeftStickY),
            scroll_y: gamepad.value(Axis::RightStickY),
            confirm: gamepad.is_pressed(Button::South),
            back: gamepad.is_pressed(Button::East),
            previous_tab: gamepad.is_pressed(Button::LeftTrigger),
            next_tab: gamepad.is_pressed(Button::RightTrigger),
        }
    }

    fn direction(self, previous: Option<Direction>) -> Option<Direction> {
        if let Some(dpad) = self.dpad {
            return Some(dpad);
        }
        let (direction, value) = if self.stick_x.abs() > self.stick_y.abs() {
            (
                if self.stick_x < 0.0 {
                    Direction::Left
                } else {
                    Direction::Right
                },
                self.stick_x.abs(),
            )
        } else {
            (
                if self.stick_y < 0.0 {
                    Direction::Down
                } else {
                    Direction::Up
                },
                self.stick_y.abs(),
            )
        };
        let threshold = if previous == Some(direction) {
            STICK_RELEASE
        } else {
            STICK_THRESHOLD
        };
        (value >= threshold).then_some(direction)
    }
}

#[derive(Default)]
struct Repeater {
    held: Option<ControllerAction>,
    next_repeat: Option<Instant>,
}

impl Repeater {
    fn update(&mut self, action: Option<ControllerAction>, now: Instant) -> Option<ControllerAction> {
        if self.held != action {
            self.held = action;
            self.next_repeat = action.map(|_| now + REPEAT_DELAY);
            return action;
        }
        if self.next_repeat.is_some_and(|deadline| now >= deadline) {
            self.next_repeat = Some(now + REPEAT_INTERVAL);
            return action;
        }
        None
    }
}

#[derive(Default)]
struct NavigationInput {
    previous: Snapshot,
    movement: Repeater,
    scrolling: Repeater,
}

impl NavigationInput {
    fn update(&mut self, snapshot: Snapshot, now: Instant, enabled: bool) -> Vec<ControllerAction> {
        let mut actions = Vec::new();
        let previous_direction = match self.movement.held {
            Some(ControllerAction::Move(direction)) => Some(direction),
            _ => None,
        };
        let direction = snapshot.direction(previous_direction).map(ControllerAction::Move);
        let scroll = if snapshot.scroll_y >= STICK_THRESHOLD {
            Some(ControllerAction::ScrollUp)
        } else if snapshot.scroll_y <= -STICK_THRESHOLD {
            Some(ControllerAction::ScrollDown)
        } else {
            None
        };
        if enabled {
            if let Some(action) = self.movement.update(direction, now) {
                actions.push(action);
            }
            if let Some(action) = self.scrolling.update(scroll, now) {
                actions.push(action);
            }
            for (pressed, previous, action) in [
                (snapshot.confirm, self.previous.confirm, ControllerAction::Confirm),
                (snapshot.back, self.previous.back, ControllerAction::Back),
                (
                    snapshot.previous_tab,
                    self.previous.previous_tab,
                    ControllerAction::PreviousTab,
                ),
                (snapshot.next_tab, self.previous.next_tab, ControllerAction::NextTab),
            ] {
                if pressed && !previous {
                    actions.push(action);
                }
            }
        } else {
            self.movement.held = direction;
            self.movement.next_repeat = direction.map(|_| now + REPEAT_DELAY);
            self.scrolling.held = scroll;
            self.scrolling.next_repeat = scroll.map(|_| now + REPEAT_DELAY);
        }
        self.previous = snapshot;
        actions
    }
}

pub struct ControllerInput {
    receiver: Receiver<ControllerAction>,
    enabled: Arc<AtomicBool>,
    cancel: CancelToken,
}

impl ControllerInput {
    pub fn start() -> std::io::Result<Self> {
        let (sender, receiver) = sync_channel(32);
        let enabled = Arc::new(AtomicBool::new(false));
        let worker_enabled = enabled.clone();
        let cancel = CancelToken::new();
        let worker_cancel = cancel.clone();
        std::thread::Builder::new()
            .name("yp-controller".to_owned())
            .spawn(move || {
                let mut gilrs = match Gilrs::new() {
                    Ok(gilrs) => gilrs,
                    Err(error) => {
                        log::warn!("controller input is unavailable: {error}");
                        return;
                    }
                };
                let mut active = None;
                let mut navigation = NavigationInput::default();
                while !worker_cancel.is_cancelled() {
                    while let Some(event) = gilrs.next_event() {
                        match event.event {
                            EventType::Disconnected if active == Some(event.id) => {
                                active = None;
                                navigation = NavigationInput::default();
                            }
                            EventType::ButtonPressed(_, _) => active = Some(event.id),
                            EventType::AxisChanged(_, value, _) if value.abs() >= STICK_THRESHOLD => {
                                active = Some(event.id)
                            }
                            _ => {}
                        }
                    }
                    let snapshot = active
                        .and_then(|id| gilrs.connected_gamepad(id))
                        .map(Snapshot::from_gamepad)
                        .unwrap_or_default();
                    for action in navigation.update(snapshot, Instant::now(), worker_enabled.load(Ordering::Relaxed)) {
                        match sender.try_send(action) {
                            Ok(()) | Err(TrySendError::Full(_)) => {}
                            Err(TrySendError::Disconnected(_)) => return,
                        }
                    }
                    std::thread::sleep(POLL_INTERVAL);
                }
            })?;
        Ok(Self {
            receiver,
            enabled,
            cancel,
        })
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn drain(&self) -> impl Iterator<Item = ControllerAction> + '_ {
        self.receiver.try_iter()
    }
}

impl Drop for ControllerInput {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stick_drift_is_ignored_and_direction_has_hysteresis() {
        let mut input = NavigationInput::default();
        let now = Instant::now();
        let mut snapshot = Snapshot {
            stick_y: 0.2,
            ..Snapshot::default()
        };
        assert!(input.update(snapshot, now, true).is_empty());
        snapshot.stick_y = 0.8;
        assert_eq!(
            input.update(snapshot, now, true),
            [ControllerAction::Move(Direction::Up)]
        );
        snapshot.stick_y = 0.4;
        assert!(input.update(snapshot, now, true).is_empty());
        snapshot.stick_y = 0.2;
        assert!(input.update(snapshot, now, true).is_empty());
        snapshot.stick_y = -0.8;
        assert_eq!(
            input.update(snapshot, now, true),
            [ControllerAction::Move(Direction::Down)]
        );
    }

    #[test]
    fn dpad_takes_precedence_over_the_stick_and_repeats_at_a_controlled_rate() {
        let mut input = NavigationInput::default();
        let now = Instant::now();
        let snapshot = Snapshot {
            dpad: Some(Direction::Left),
            stick_y: 1.0,
            ..Snapshot::default()
        };
        let expected = [ControllerAction::Move(Direction::Left)];
        assert_eq!(input.update(snapshot, now, true), expected);
        assert!(input
            .update(snapshot, now + REPEAT_DELAY - Duration::from_millis(1), true)
            .is_empty());
        assert_eq!(input.update(snapshot, now + REPEAT_DELAY, true), expected);
        assert!(input
            .update(snapshot, now + REPEAT_DELAY + Duration::from_millis(1), true)
            .is_empty());
        assert_eq!(
            input.update(snapshot, now + REPEAT_DELAY + REPEAT_INTERVAL, true),
            expected
        );
    }

    #[test]
    fn confirm_back_and_shoulders_fire_once_per_press() {
        let mut input = NavigationInput::default();
        let now = Instant::now();
        let snapshot = Snapshot {
            confirm: true,
            back: true,
            previous_tab: true,
            next_tab: true,
            ..Snapshot::default()
        };
        let expected = [
            ControllerAction::Confirm,
            ControllerAction::Back,
            ControllerAction::PreviousTab,
            ControllerAction::NextTab,
        ];
        assert_eq!(input.update(snapshot, now, true), expected);
        assert!(input.update(snapshot, now + Duration::from_secs(1), true).is_empty());
        input.update(Snapshot::default(), now, true);
        assert_eq!(input.update(snapshot, now, true), expected);
    }

    #[test]
    fn background_confirm_is_not_replayed_when_the_launcher_regains_focus() {
        let mut input = NavigationInput::default();
        let now = Instant::now();
        let pressed = Snapshot {
            confirm: true,
            ..Snapshot::default()
        };
        assert!(input.update(pressed, now, false).is_empty());
        assert!(input.update(pressed, now, true).is_empty());
        input.update(Snapshot::default(), now, true);
        assert_eq!(input.update(pressed, now, true), [ControllerAction::Confirm]);
    }

    #[test]
    fn releasing_a_direction_stops_repeat_and_right_stick_scrolls() {
        let mut input = NavigationInput::default();
        let now = Instant::now();
        input.update(
            Snapshot {
                dpad: Some(Direction::Down),
                ..Snapshot::default()
            },
            now,
            true,
        );
        assert!(input.update(Snapshot::default(), now + REPEAT_DELAY, true).is_empty());
        assert!(input
            .update(Snapshot::default(), now + Duration::from_secs(1), true)
            .is_empty());
        let snapshot = Snapshot {
            scroll_y: -1.0,
            ..Snapshot::default()
        };
        assert_eq!(input.update(snapshot, now, true), [ControllerAction::ScrollDown]);
    }
}
