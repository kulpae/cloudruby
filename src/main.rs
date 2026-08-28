use std::{path::PathBuf, sync::Arc, time::Duration};

use anyhow::Context;
use clap::Parser;
use cloudruby::{
    app::{Action, App, PlaybackState, action_for_key},
    config::{Config, ConfigSource},
    library::load_sources,
    player::{AudioPlayer, DefaultPlayer, PlayerEvent, PlayerState},
    ui,
};
use crossterm::event::{Event, EventStream};
use futures_util::StreamExt;
use rand::seq::SliceRandom;
use tokio::sync::mpsc;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Directories, audio files, M3U/M3U8 playlists, or HTTP(S) stream URLs
    sources: Vec<String>,
    #[arg(long)]
    config: Option<PathBuf>,
    /// Preserve source and playlist order
    #[arg(long, alias = "no_shuffle")]
    no_shuffle: bool,
    /// Write the effective configuration to the XDG path and exit
    #[arg(long)]
    write_config: bool,
    /// Ignore all configuration files
    #[arg(long, alias = "noconfig")]
    no_config: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();
    let cli = Cli::parse();
    let (mut config, _) = if cli.no_config {
        (Config::default(), ConfigSource::Defaults)
    } else {
        Config::load(cli.config.as_deref())?
    };
    if !cli.sources.is_empty() {
        config.sources = cli.sources;
    }
    if cli.no_shuffle {
        config.no_shuffle = true;
    }
    if cli.write_config {
        let path = Config::path()?;
        config.save(&path)?;
        println!("Wrote {}", path.display());
        return Ok(());
    }
    if config.sources.is_empty() {
        anyhow::bail!(
            "provide a music directory, audio file, M3U playlist, or stream URL (see --help)"
        );
    }

    let mut tracks = load_sources(&config.sources)?;
    if tracks.is_empty() {
        anyhow::bail!("no supported audio entries found");
    }
    if !config.no_shuffle {
        tracks.shuffle(&mut rand::rng());
    }

    let (player_tx, player_rx) = mpsc::unbounded_channel();
    let player: Arc<dyn AudioPlayer> = DefaultPlayer::new(player_tx)?;
    let mut app = App::new(tracks);
    play_selected(&player, &mut app)?;
    run_tui(player, player_rx, app, config).await
}

fn play_selected(player: &Arc<dyn AudioPlayer>, app: &mut App) -> anyhow::Result<()> {
    let item = app.current_track().context("playlist is empty")?;
    player.play_uri(item.uri.as_str())?;
    app.playback = PlaybackState::Playing;
    app.position = Duration::ZERO;
    app.duration = Duration::ZERO;
    Ok(())
}

async fn run_tui(
    player: Arc<dyn AudioPlayer>,
    mut player_events: mpsc::UnboundedReceiver<PlayerEvent>,
    mut app: App,
    config: Config,
) -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    let result = async {
        let mut input = EventStream::new();
        let mut tick = tokio::time::interval(Duration::from_millis(33));
        loop {
            terminal.draw(|frame| ui::render(frame, &app, &config.ui))?;
            tokio::select! {
                _ = tick.tick() => {
                    let snapshot = player.snapshot();
                    app.position = snapshot.position;
                    if !snapshot.duration.is_zero() { app.duration = snapshot.duration; }
                    app.present_spectrum(snapshot.position);
                    app.animate_spectrum();
                    app.expire_status();
                }
                event = input.next() => {
                    match event {
                        Some(Ok(Event::Key(key))) => {
                            if let Some(action) = action_for_key(key)
                                && handle_action(action, &player, &mut app)?
                            {
                                break;
                            }
                        }
                        Some(Ok(Event::Resize(_, _))) => {}
                        Some(Err(error)) => return Err(error.into()),
                        None => break,
                        _ => {}
                    }
                }
                event = player_events.recv() => match event {
                    Some(PlayerEvent::EndOfStream) => {
                        app.next();
                        play_selected(&player, &mut app)?;
                    }
                    Some(PlayerEvent::Error(message)) => app.notify(format!("Playback error: {message}")),
                    Some(PlayerEvent::Buffering(value)) => {
                        app.buffered_percent = value;
                        app.playback = if value < 100 { PlaybackState::Buffering } else { PlaybackState::Playing };
                    }
                    Some(PlayerEvent::State(state)) => app.playback = match state {
                        PlayerState::Playing => PlaybackState::Playing,
                        PlayerState::Paused => PlaybackState::Paused,
                        PlayerState::Stopped => PlaybackState::Stopped,
                    },
                    Some(PlayerEvent::Spectrum(frame)) => app.queue_spectrum(frame),
                    None => {}
                },
                _ = tokio::signal::ctrl_c() => break,
            }
        }
        Ok::<_, anyhow::Error>(())
    }
    .await;
    let _ = player.stop();
    ratatui::restore();
    result
}

fn handle_action(
    action: Action,
    player: &Arc<dyn AudioPlayer>,
    app: &mut App,
) -> anyhow::Result<bool> {
    match action {
        Action::Quit => return Ok(true),
        Action::Next => {
            app.next();
            play_selected(player, app)?;
        }
        Action::Previous => {
            app.previous();
            play_selected(player, app)?;
        }
        Action::VolumeUp => {
            app.change_volume(0.05);
            player.set_volume(app.volume)?;
        }
        Action::VolumeDown => {
            app.change_volume(-0.05);
            player.set_volume(app.volume)?;
        }
        Action::ToggleMute => {
            app.muted = !app.muted;
            player.set_muted(app.muted)?;
            app.notify(if app.muted { "Muted" } else { "Unmuted" });
        }
        Action::ToggleInfo => app.info_visible = !app.info_visible,
        Action::TogglePause => {
            if app.playback == PlaybackState::Paused {
                player.resume()?;
                app.playback = PlaybackState::Playing;
            } else {
                player.pause()?;
                app.playback = PlaybackState::Paused;
            }
        }
    }
    Ok(false)
}
