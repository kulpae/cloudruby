use std::{sync::Arc, time::Duration};

use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub enum PlayerEvent {
    EndOfStream,
    Error(String),
    Buffering(u8),
    State(PlayerState),
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
            let bus = playbin
                .bus()
                .ok_or_else(|| anyhow::anyhow!("GStreamer playbin has no bus"))?;
            let watched = playbin.clone();
            let running = Arc::new(AtomicBool::new(true));
            let watcher_running = Arc::clone(&running);
            thread::Builder::new()
                .name("cloudruby-gstreamer".into())
                .spawn(move || {
                    while watcher_running.load(Ordering::Relaxed) {
                        let Some(message) = bus.timed_pop(gst::ClockTime::from_mseconds(100))
                        else {
                            continue;
                        };
                        use gst::MessageView;
                        match message.view() {
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
