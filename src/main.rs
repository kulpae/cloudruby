use std::{
    io::{self, IsTerminal, Read},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use clap::Parser;
use cloudruby::{
    app::{Action, App, PlaybackState, action_for_key},
    config::{Config, ConfigSource},
    library::load_sources_incremental,
    player::{AudioPlayer, DefaultPlayer, PlayerEvent, PlayerState},
    ui,
};
use crossterm::event::{Event, EventStream};
use futures_util::StreamExt;
use tokio::sync::mpsc;

enum LoadEvent {
    Track(cloudruby::library::MediaItem),
    Finished(Result<(), String>),
}

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Directories, audio files, M3U/M3U8 playlists, or HTTP(S) stream URLs
    sources: Vec<String>,
    #[arg(long)]
    config: Option<PathBuf>,
    /// Preserve source and playlist order
    #[arg(
        long,
        alias = "no_shuffle",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "true",
        default_value = "false",
        value_parser = clap::value_parser!(bool),
    )]
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
    let mut sources = cli.sources;
    sources.extend(read_stdin_sources()?);
    if !sources.is_empty() {
        config.sources = sources;
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

    let (player_tx, player_rx) = mpsc::unbounded_channel();
    let player: Arc<dyn AudioPlayer> = DefaultPlayer::new(player_tx)?;
    let (load_tx, load_rx) = mpsc::unbounded_channel();
    let sources = config.sources.clone();
    spawn_loader(sources, load_tx.clone());

    let mut app = App::new(Vec::new());
    app.start_loading();
    run_tui(player, player_rx, load_rx, load_tx, app, config).await
}

fn spawn_loader(sources: Vec<String>, load_tx: mpsc::UnboundedSender<LoadEvent>) {
    tokio::task::spawn_blocking(move || {
        let result = load_sources_incremental(&sources, |track| {
            let _ = load_tx.send(LoadEvent::Track(track));
        })
        .map_err(|error| error.to_string());
        let _ = load_tx.send(LoadEvent::Finished(result));
    });
}

fn read_stdin_sources() -> anyhow::Result<Vec<String>> {
    if io::stdin().is_terminal() {
        return Ok(Vec::new());
    }

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(parse_stdin_sources(&input))
}

fn parse_stdin_sources(input: &str) -> Vec<String> {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
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
    mut load_events: mpsc::UnboundedReceiver<LoadEvent>,
    load_tx: mpsc::UnboundedSender<LoadEvent>,
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
                            if app.source_input.is_some() {
                                match key.code {
                                    crossterm::event::KeyCode::Enter => {
                                        if let Some(source) = app.submit_source() {
                                            app.start_loading();
                                            spawn_loader(vec![source], load_tx.clone());
                                        } else {
                                            app.cancel_add_source();
                                        }
                                    }
                                    crossterm::event::KeyCode::Esc => app.cancel_add_source(),
                                    crossterm::event::KeyCode::Backspace => {
                                        app.remove_source_character()
                                    }
                                    crossterm::event::KeyCode::Char(character) => {
                                        app.append_source_character(character)
                                    }
                                    _ => {}
                                }
                                continue;
                            }
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
                        if !app.tracks.is_empty() {
                            app.next();
                            play_selected(&player, &mut app)?;
                        }
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
                    Some(PlayerEvent::StreamStarted(generation)) => {
                        app.start_spectrum_stream(generation);
                    }
                    Some(PlayerEvent::Spectrum(frame)) => app.queue_spectrum(frame),
                    None => {}
                },
                event = load_events.recv() => match event {
                    Some(LoadEvent::Track(track)) => {
                        let should_start = app.tracks.is_empty();
                        if app.add_track(track) && should_start {
                            play_selected(&player, &mut app)?;
                        }
                    }
                    Some(LoadEvent::Finished(result)) => {
                        if let Err(message) = result {
                            if app.finish_loading(false) {
                                app.notify(format!("Loading failed: {message}"));
                            }
                        } else if app.finish_loading(!config.no_shuffle)
                            && app.tracks.is_empty()
                        {
                            app.notify("No supported audio entries found");
                        }
                    }
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
        Action::ToggleShuffle => {
            if app.loading {
                app.notify("Shuffle is available after loading finishes");
            } else {
                app.toggle_shuffle();
            }
        }
        Action::AddSource => app.begin_add_source(),
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    #[test]
    fn stdin_sources_are_trimmed_and_blank_lines_are_ignored() {
        let sources = "  first.mp3\n\nhttps://radio.example/live.ogg  \n";
        let actual = super::parse_stdin_sources(sources);
        assert_eq!(actual, ["first.mp3", "https://radio.example/live.ogg"]);
    }
}
