use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::library::MediaItem;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Next,
    Previous,
    Quit,
    VolumeUp,
    VolumeDown,
    ToggleMute,
    ToggleInfo,
    TogglePause,
}

pub fn action_for_key(key: KeyEvent) -> Option<Action> {
    if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return None;
    }
    match key.code {
        KeyCode::Down | KeyCode::Char('n' | 'N') => Some(Action::Next),
        KeyCode::Up | KeyCode::Char('p' | 'P') => Some(Action::Previous),
        KeyCode::Esc | KeyCode::Char('q' | 'Q') => Some(Action::Quit),
        KeyCode::Char('+' | '=') => Some(Action::VolumeUp),
        KeyCode::Char('-' | '_') => Some(Action::VolumeDown),
        KeyCode::Char('m' | 'M') => Some(Action::ToggleMute),
        KeyCode::Char('v' | 'V') => Some(Action::ToggleInfo),
        KeyCode::Char(' ') => Some(Action::TogglePause),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing,
    Paused,
    Buffering,
}

pub struct App {
    pub tracks: Vec<MediaItem>,
    pub selected: usize,
    pub playback: PlaybackState,
    pub volume: f64,
    pub muted: bool,
    pub position: Duration,
    pub duration: Duration,
    pub buffered_percent: u8,
    pub info_visible: bool,
    pub status: Option<(String, Instant)>,
}

impl App {
    pub fn new(tracks: Vec<MediaItem>) -> Self {
        Self {
            tracks,
            selected: 0,
            playback: PlaybackState::Stopped,
            volume: 1.0,
            muted: false,
            position: Duration::ZERO,
            duration: Duration::ZERO,
            buffered_percent: 0,
            info_visible: false,
            status: None,
        }
    }

    pub fn current_track(&self) -> Option<&MediaItem> {
        self.tracks.get(self.selected)
    }

    pub fn next(&mut self) {
        if !self.tracks.is_empty() {
            self.selected = (self.selected + 1) % self.tracks.len();
            self.reset_progress();
        }
    }

    pub fn previous(&mut self) {
        if !self.tracks.is_empty() {
            self.selected = self
                .selected
                .checked_sub(1)
                .unwrap_or(self.tracks.len() - 1);
            self.reset_progress();
        }
    }

    pub fn change_volume(&mut self, delta: f64) {
        self.volume = (self.volume + delta).clamp(0.0, 1.0);
        self.notify(format!("Volume: {}%", (self.volume * 100.0).round()));
    }

    pub fn notify(&mut self, message: impl Into<String>) {
        self.status = Some((message.into(), Instant::now()));
    }

    pub fn expire_status(&mut self) {
        if self
            .status
            .as_ref()
            .is_some_and(|(_, at)| at.elapsed() >= Duration::from_secs(5))
        {
            self.status = None;
        }
    }

    fn reset_progress(&mut self) {
        self.position = Duration::ZERO;
        self.duration = Duration::ZERO;
        self.buffered_percent = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn preserves_original_key_bindings() {
        assert_eq!(action_for_key(key(KeyCode::Char('N'))), Some(Action::Next));
        assert_eq!(action_for_key(key(KeyCode::Up)), Some(Action::Previous));
        assert_eq!(action_for_key(key(KeyCode::Esc)), Some(Action::Quit));
        assert_eq!(
            action_for_key(key(KeyCode::Char('_'))),
            Some(Action::VolumeDown)
        );
        assert_eq!(
            action_for_key(key(KeyCode::Char(' '))),
            Some(Action::TogglePause)
        );
    }

    #[test]
    fn navigation_wraps() {
        let tracks = vec![
            MediaItem {
                title: "one".into(),
                uri: url::Url::parse("file:///one.mp3").unwrap(),
                kind: crate::library::MediaKind::Local,
            },
            MediaItem {
                title: "two".into(),
                uri: url::Url::parse("https://radio.example/live").unwrap(),
                kind: crate::library::MediaKind::Stream,
            },
        ];
        let mut app = App::new(tracks);
        app.previous();
        assert_eq!(app.selected, 1);
        app.next();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn volume_is_clamped() {
        let mut app = App::new(vec![]);
        app.change_volume(1.0);
        assert_eq!(app.volume, 1.0);
        app.change_volume(-2.0);
        assert_eq!(app.volume, 0.0);
    }
}
