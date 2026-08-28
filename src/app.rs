use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use rand::seq::SliceRandom;

use crate::{library::MediaItem, player::SpectrumFrame};

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
    ToggleShuffle,
    AddSource,
}

pub fn action_for_key(key: KeyEvent) -> Option<Action> {
    if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
        return None;
    }
    match key.code {
        KeyCode::Down | KeyCode::Char('n' | 'N') => Some(Action::Next),
        KeyCode::Up | KeyCode::Char('p' | 'P') => Some(Action::Previous),
        KeyCode::Esc | KeyCode::Char('q' | 'Q') => Some(Action::Quit),
        KeyCode::Char('+' | '=' | '*') => Some(Action::VolumeUp),
        KeyCode::Char('-' | '_') => Some(Action::VolumeDown),
        KeyCode::Char('m' | 'M') => Some(Action::ToggleMute),
        KeyCode::Char('v' | 'V') => Some(Action::ToggleInfo),
        KeyCode::Char(' ') => Some(Action::TogglePause),
        KeyCode::Char('s' | 'S') => Some(Action::ToggleShuffle),
        KeyCode::Char('a' | 'A') => Some(Action::AddSource),
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
    pub spectrum: Vec<f32>,
    pub spectrum_peaks: Vec<f32>,
    pub spectrum_active: bool,
    pub spectrum_activity: f32,
    pub visualizer_rotation: f32,
    pub visualizer_frame: u64,
    spectrum_pending: VecDeque<SpectrumFrame>,
    spectrum_generation: u64,
    pub info_visible: bool,
    pub stream_title: Option<String>,
    pub status: Option<(String, Instant)>,
    pub loading: bool,
    pub loaded_count: usize,
    pub shuffle_enabled: bool,
    pub source_input: Option<String>,
    pending_loads: usize,
}

impl App {
    pub fn new(tracks: Vec<MediaItem>) -> Self {
        let loaded_count = tracks.len();
        Self {
            tracks,
            selected: 0,
            playback: PlaybackState::Stopped,
            volume: 1.0,
            muted: false,
            position: Duration::ZERO,
            duration: Duration::ZERO,
            buffered_percent: 0,
            spectrum: Vec::new(),
            spectrum_peaks: Vec::new(),
            spectrum_active: false,
            spectrum_activity: 0.0,
            visualizer_rotation: 0.0,
            visualizer_frame: 0,
            spectrum_pending: VecDeque::new(),
            spectrum_generation: 0,
            info_visible: false,
            stream_title: None,
            status: None,
            loading: false,
            loaded_count,
            shuffle_enabled: false,
            source_input: None,
            pending_loads: 0,
        }
    }

    pub fn start_loading(&mut self) {
        if !self.loading {
            self.loaded_count = self.tracks.len();
        }
        self.loading = true;
        self.pending_loads += 1;
        self.notify("Loading sources…");
    }

    pub fn add_track(&mut self, track: MediaItem) -> bool {
        if self.tracks.iter().any(|item| item.uri == track.uri) {
            return false;
        }
        self.tracks.push(track);
        self.loaded_count = self.tracks.len();
        true
    }

    pub fn finish_loading(&mut self, shuffle: bool) -> bool {
        self.pending_loads = self.pending_loads.saturating_sub(1);
        if self.pending_loads != 0 {
            return false;
        }
        self.loading = false;
        self.shuffle_enabled = shuffle;
        if shuffle {
            self.shuffle_remaining();
            self.notify(format!("Loaded {} tracks · shuffle on", self.tracks.len()));
        } else {
            self.notify(format!("Loaded {} tracks", self.tracks.len()));
        }
        true
    }

    pub fn begin_add_source(&mut self) {
        self.source_input = Some(String::new());
    }

    pub fn cancel_add_source(&mut self) {
        self.source_input = None;
    }

    pub fn append_source_character(&mut self, character: char) {
        if let Some(input) = &mut self.source_input {
            input.push(character);
        }
    }

    pub fn remove_source_character(&mut self) {
        if let Some(input) = &mut self.source_input {
            input.pop();
        }
    }

    pub fn submit_source(&mut self) -> Option<String> {
        self.source_input
            .take()
            .filter(|source| !source.trim().is_empty())
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle_enabled = !self.shuffle_enabled;
        if self.shuffle_enabled {
            self.shuffle_remaining();
        }
        self.notify(if self.shuffle_enabled {
            "Shuffle on"
        } else {
            "Shuffle off"
        });
    }

    fn shuffle_remaining(&mut self) {
        if self.tracks.len() > self.selected + 1 {
            self.tracks[(self.selected + 1)..].shuffle(&mut rand::rng());
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

    pub fn update_spectrum(&mut self, bands: &[f32]) {
        self.apply_spectrum(bands);
    }

    pub fn queue_spectrum(&mut self, frame: SpectrumFrame) {
        if frame.generation == self.spectrum_generation {
            self.spectrum_pending.push_back(frame);
        }
    }

    pub fn start_spectrum_stream(&mut self, generation: u64) {
        self.spectrum_generation = generation;
        self.spectrum_pending.clear();
        self.spectrum.fill(0.0);
        self.spectrum_peaks.fill(0.0);
        self.spectrum_active = false;
        self.spectrum_activity = 0.0;
        self.stream_title = None;
    }

    pub fn update_stream_title(&mut self, generation: u64, title: String) {
        if generation == self.spectrum_generation {
            self.stream_title = Some(title);
        }
    }

    pub fn present_spectrum(&mut self, position: Duration) {
        while self
            .spectrum_pending
            .front()
            .is_some_and(|frame| frame.running_time.saturating_sub(frame.duration / 2) <= position)
        {
            if let Some(frame) = self.spectrum_pending.pop_front() {
                self.apply_spectrum(&frame.bands);
            }
        }
    }

    fn apply_spectrum(&mut self, bands: &[f32]) {
        if bands.is_empty() {
            return;
        }
        if self.spectrum.len() != bands.len() {
            self.spectrum.resize(bands.len(), 0.0);
            self.spectrum_peaks.resize(bands.len(), 0.0);
        }
        let mut activity = 0.0;
        for ((level, peak), target) in self
            .spectrum
            .iter_mut()
            .zip(&mut self.spectrum_peaks)
            .zip(bands.iter().copied())
        {
            let target = target.clamp(0.0, 1.0);
            activity += target;
            let response = if target > *level { 0.38 } else { 0.30 };
            *level += (target - *level) * response;
            *peak = peak.max(*level);
        }
        activity /= bands.len() as f32;
        self.spectrum_activity += (activity - self.spectrum_activity) * 0.28;
        self.spectrum_active = true;
    }

    pub fn animate_spectrum(&mut self) {
        self.visualizer_frame = self.visualizer_frame.wrapping_add(1);
        let speed = if self.playback == PlaybackState::Playing {
            0.012
        } else {
            0.003
        };
        // Keep this unwrapped: reducing at TAU can cause a visible boundary
        // snap on terminals that redraw between the two frames.
        self.visualizer_rotation += speed + self.spectrum_activity * 0.14;
        for (level, peak) in self.spectrum.iter_mut().zip(&mut self.spectrum_peaks) {
            *level *= if self.playback == PlaybackState::Playing {
                0.975
            } else {
                0.88
            };
            *peak = (*peak - 0.018).max(*level);
        }
    }

    fn reset_progress(&mut self) {
        self.position = Duration::ZERO;
        self.duration = Duration::ZERO;
        self.buffered_percent = 0;
        self.spectrum.fill(0.0);
        self.spectrum_peaks.fill(0.0);
        self.spectrum_active = false;
        self.spectrum_pending.clear();
        self.spectrum_activity = 0.0;
        self.visualizer_rotation = 0.0;
        self.stream_title = None;
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
        assert_eq!(
            action_for_key(key(KeyCode::Char('s'))),
            Some(Action::ToggleShuffle)
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

    #[test]
    fn spectrum_uses_fast_attack_and_peak_decay() {
        let mut app = App::new(vec![]);
        app.playback = PlaybackState::Playing;
        for _ in 0..16 {
            app.update_spectrum(&[1.0, 0.5]);
        }
        assert!(app.spectrum[0] > 0.3);
        assert_eq!(app.spectrum_peaks[0], app.spectrum[0]);
        for _ in 0..17 {
            app.update_spectrum(&[0.0, 0.0]);
        }
        app.animate_spectrum();
        assert!(app.spectrum[0] < 0.8);
        assert!(app.spectrum_active);
    }
}
