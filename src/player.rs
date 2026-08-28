use std::{sync::Arc, time::Duration};

use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub enum PlayerEvent {
    EndOfStream,
    Error(String),
    Buffering(u8),
    State(PlayerState),
    StreamStarted(u64),
    StreamTitle(u64, String),
    Spectrum(SpectrumFrame),
}

#[derive(Clone, Debug)]
pub struct SpectrumFrame {
    pub bands: Vec<f32>,
    pub running_time: Duration,
    pub duration: Duration,
    pub generation: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlayerState {
    #[default]
    Stopped,
    Playing,
    Paused,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PlayerSnapshot {
    pub position: Duration,
    pub duration: Duration,
}

pub trait AudioPlayer: Send + Sync {
    fn play_uri(&self, uri: &str) -> anyhow::Result<()>;
    fn pause(&self) -> anyhow::Result<()>;
    fn resume(&self) -> anyhow::Result<()>;
    fn stop(&self) -> anyhow::Result<()>;
    fn set_volume(&self, volume: f64) -> anyhow::Result<()>;
    fn set_muted(&self, muted: bool) -> anyhow::Result<()>;
    fn snapshot(&self) -> PlayerSnapshot;
}

#[cfg(feature = "gstreamer-backend")]
mod gst_backend {
    use std::{
        sync::atomic::{AtomicBool, Ordering},
        thread,
    };

    use gst::prelude::*;
    use gstreamer as gst;

    use super::*;

    pub struct GstreamerPlayer {
        playbin: gst::Element,
        running: Arc<AtomicBool>,
    }

    impl GstreamerPlayer {
        pub fn new(events: mpsc::UnboundedSender<PlayerEvent>) -> anyhow::Result<Arc<Self>> {
            gst::init()?;
            let playbin = gst::ElementFactory::make("playbin").build()?;
            if let Ok(spectrum) = gst::ElementFactory::make("spectrum")
                .property("bands", 128_u32)
                .property("threshold", -80_i32)
                .property("interval", 25_000_000_u64)
                .property("post-messages", true)
                .property("message-magnitude", true)
                .property("message-phase", false)
                .build()
            {
                playbin.set_property("audio-filter", &spectrum);
            }
            let bus = playbin
                .bus()
                .ok_or_else(|| anyhow::anyhow!("GStreamer playbin has no bus"))?;
            let watched = playbin.clone();
            let running = Arc::new(AtomicBool::new(true));
            let watcher_running = Arc::clone(&running);
            thread::Builder::new()
                .name("cloudruby-gstreamer".into())
                .spawn(move || {
                    let mut spectrum_generation = 0_u64;
                    while watcher_running.load(Ordering::Relaxed) {
                        let Some(message) = bus.timed_pop(gst::ClockTime::from_mseconds(100))
                        else {
                            continue;
                        };
                        use gst::MessageView;
                        match message.view() {
                            MessageView::StreamStart(..) => {
                                spectrum_generation = spectrum_generation.wrapping_add(1);
                                let _ =
                                    events.send(PlayerEvent::StreamStarted(spectrum_generation));
                            }
                            MessageView::Tag(tag) => {
                                if let Some(title) = tag.tags().get::<gst::tags::Title>() {
                                    let title = title.get().trim();
                                    if !title.is_empty() {
                                        let _ = events.send(PlayerEvent::StreamTitle(
                                            spectrum_generation,
                                            title.to_owned(),
                                        ));
                                    }
                                }
                            }
                            MessageView::Eos(..) => {
                                let _ = events.send(PlayerEvent::EndOfStream);
                            }
                            MessageView::Error(error) => {
                                let _ = events.send(PlayerEvent::Error(error.error().to_string()));
                            }
                            MessageView::Buffering(buffering) => {
                                let _ = events.send(PlayerEvent::Buffering(
                                    buffering.percent().clamp(0, 100) as u8,
                                ));
                            }
                            MessageView::StateChanged(state)
                                if message.src().as_ref().is_some_and(|source| {
                                    *source == watched.upcast_ref::<gst::Object>()
                                }) =>
                            {
                                let state = match state.current() {
                                    gst::State::Playing => PlayerState::Playing,
                                    gst::State::Paused => PlayerState::Paused,
                                    _ => PlayerState::Stopped,
                                };
                                let _ = events.send(PlayerEvent::State(state));
                            }
                            MessageView::Element(element) => {
                                let Some(structure) = element.structure() else {
                                    continue;
                                };
                                if structure.name() != "spectrum" {
                                    continue;
                                }
                                if let Ok(magnitudes) = structure.get::<gst::List>("magnitude") {
                                    let bands = magnitudes
                                        .iter()
                                        .filter_map(|value| value.get::<f32>().ok())
                                        .map(normalize_magnitude)
                                        .collect::<Vec<_>>();
                                    if !bands.is_empty() {
                                        let running_time = structure
                                            .get::<gst::ClockTime>("running-time")
                                            .ok()
                                            .map_or(Duration::ZERO, |value| {
                                                Duration::from_nanos(value.nseconds())
                                            });
                                        let duration = structure
                                            .get::<gst::ClockTime>("duration")
                                            .ok()
                                            .map_or(Duration::from_millis(25), |value| {
                                                Duration::from_nanos(value.nseconds())
                                            });
                                        let _ = events.send(PlayerEvent::Spectrum(SpectrumFrame {
                                            bands,
                                            running_time,
                                            duration,
                                            generation: spectrum_generation,
                                        }));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                })?;
            Ok(Arc::new(Self { playbin, running }))
        }
    }

    impl AudioPlayer for GstreamerPlayer {
        fn play_uri(&self, uri: &str) -> anyhow::Result<()> {
            self.playbin.set_state(gst::State::Null)?;
            self.playbin.set_property("uri", uri);
            self.playbin.set_state(gst::State::Playing)?;
            Ok(())
        }

        fn pause(&self) -> anyhow::Result<()> {
            self.playbin.set_state(gst::State::Paused)?;
            Ok(())
        }

        fn resume(&self) -> anyhow::Result<()> {
            self.playbin.set_state(gst::State::Playing)?;
            Ok(())
        }

        fn stop(&self) -> anyhow::Result<()> {
            self.playbin.set_state(gst::State::Null)?;
            Ok(())
        }

        fn set_volume(&self, volume: f64) -> anyhow::Result<()> {
            self.playbin.set_property("volume", volume);
            Ok(())
        }

        fn set_muted(&self, muted: bool) -> anyhow::Result<()> {
            self.playbin.set_property("mute", muted);
            Ok(())
        }

        fn snapshot(&self) -> PlayerSnapshot {
            PlayerSnapshot {
                position: self
                    .playbin
                    .query_position::<gst::ClockTime>()
                    .map_or(Duration::ZERO, |value| {
                        Duration::from_nanos(value.nseconds())
                    }),
                duration: self
                    .playbin
                    .query_duration::<gst::ClockTime>()
                    .map_or(Duration::ZERO, |value| {
                        Duration::from_nanos(value.nseconds())
                    }),
            }
        }
    }

    impl Drop for GstreamerPlayer {
        fn drop(&mut self) {
            self.running.store(false, Ordering::Relaxed);
            let _ = self.playbin.set_state(gst::State::Null);
        }
    }

    pub use GstreamerPlayer as DefaultPlayer;

    fn normalize_magnitude(db: f32) -> f32 {
        ((db + 80.0) / 80.0).clamp(0.0, 1.0).powf(0.55)
    }

    #[cfg(test)]
    mod tests {
        use super::normalize_magnitude;

        #[test]
        fn normalizes_spectrum_db_with_reactive_curve() {
            assert_eq!(normalize_magnitude(-80.0), 0.0);
            assert_eq!(normalize_magnitude(0.0), 1.0);
            assert!(normalize_magnitude(-40.0) > 0.5);
        }
    }
}

#[cfg(feature = "gstreamer-backend")]
pub use gst_backend::DefaultPlayer;

#[cfg(not(feature = "gstreamer-backend"))]
pub struct DefaultPlayer;

#[cfg(not(feature = "gstreamer-backend"))]
impl DefaultPlayer {
    pub fn new(_: mpsc::UnboundedSender<PlayerEvent>) -> anyhow::Result<Arc<Self>> {
        anyhow::bail!("cloudruby was built without the gstreamer-backend feature")
    }
}

#[cfg(not(feature = "gstreamer-backend"))]
impl AudioPlayer for DefaultPlayer {
    fn play_uri(&self, _: &str) -> anyhow::Result<()> {
        anyhow::bail!("cloudruby was built without the gstreamer-backend feature")
    }

    fn pause(&self) -> anyhow::Result<()> {
        anyhow::bail!("cloudruby was built without the gstreamer-backend feature")
    }

    fn resume(&self) -> anyhow::Result<()> {
        anyhow::bail!("cloudruby was built without the gstreamer-backend feature")
    }

    fn stop(&self) -> anyhow::Result<()> {
        anyhow::bail!("cloudruby was built without the gstreamer-backend feature")
    }

    fn set_volume(&self, _: f64) -> anyhow::Result<()> {
        anyhow::bail!("cloudruby was built without the gstreamer-backend feature")
    }

    fn set_muted(&self, _: bool) -> anyhow::Result<()> {
        anyhow::bail!("cloudruby was built without the gstreamer-backend feature")
    }

    fn snapshot(&self) -> PlayerSnapshot {
        PlayerSnapshot::default()
    }
}
