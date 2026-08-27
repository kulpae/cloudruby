use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{
    app::{App, PlaybackState},
    config::UiConfig,
};

pub fn render(frame: &mut Frame<'_>, app: &App, config: &UiConfig) {
    let area = frame.area();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    let ratio = if app.duration.is_zero() {
        0.0
    } else {
        app.position.as_secs_f64() / app.duration.as_secs_f64()
    };
    let label = if app.duration.is_zero() {
        format!(" LIVE · {}% buffered ", app.buffered_percent)
    } else {
        format!(
            " {} / {} · {}% buffered ",
            format_time(app.position.as_secs()),
            format_time(app.duration.as_secs()),
            app.buffered_percent
        )
    };
    frame.render_widget(
        Gauge::default()
            .ratio(ratio.clamp(0.0, 1.0))
            .label(label)
            .style(style(config, "progress_bar", Color::Blue, Color::White)),
        rows[0],
    );

    let symbol = match app.playback {
        PlaybackState::Playing => "▶",
        PlaybackState::Paused => "Ⅱ",
        PlaybackState::Buffering => "…",
        PlaybackState::Stopped => "■",
    };
    let title = app
        .current_track()
        .map_or("None", |track| track.title.as_str());
    let source = app
        .current_track()
        .map_or_else(String::new, |track| track.source_label());
    frame.render_widget(
        Paragraph::new(format!("{symbol} {title}")).style(style(
            config,
            "title",
            Color::Cyan,
            Color::Reset,
        )),
        rows[1],
    );
    frame.render_widget(
        Paragraph::new(format!("  {source}")).style(style(
            config,
            "source",
            Color::Magenta,
            Color::Reset,
        )),
        rows[2],
    );
    let status = app
        .status
        .as_ref()
        .map_or("", |(message, _)| message.as_str());
    frame.render_widget(
        Paragraph::new(status).style(style(config, "status", Color::Red, Color::Reset)),
        rows[3],
    );

    let items = app
        .tracks
        .iter()
        .map(|track| ListItem::new(format!("{} — {}", track.title, track.source_label())));
    let list = List::new(items)
        .style(style(config, "playlist", Color::Green, Color::Reset))
        .highlight_symbol("> ")
        .highlight_style(
            style(config, "playlist_active", Color::Red, Color::Reset).add_modifier(Modifier::BOLD),
        );
    let mut state = ListState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(list, rows[4], &mut state);

    let footer = "n/p next/previous · Space pause · +/- volume · m mute · v info · q quit";
    frame.render_widget(
        Paragraph::new(footer).style(style(config, "default", Color::White, Color::Reset)),
        rows[5],
    );

    if app.info_visible {
        render_info(frame, app, centered_rect(70, 60, area), config);
    }
}

fn render_info(frame: &mut Frame<'_>, app: &App, area: Rect, config: &UiConfig) {
    frame.render_widget(Clear, area);
    let track = app.current_track();
    let lines = vec![
        Line::from(vec![
            Span::styled("Track: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(track.map_or("", |track| track.title.as_str())),
        ]),
        Line::from(format!(
            "Type: {}",
            track.map_or("", |track| match track.kind {
                crate::library::MediaKind::Local => "local file",
                crate::library::MediaKind::Stream => "network stream",
            })
        )),
        Line::from(format!(
            "Source: {}",
            track.map_or_else(String::new, |track| track.source_label())
        )),
        Line::from(format!(
            "URI: {}",
            track.map_or("", |track| track.uri.as_str())
        )),
        Line::from(format!(
            "Volume: {}%{}",
            (app.volume * 100.0).round(),
            if app.muted { " (muted)" } else { "" }
        )),
        Line::from(format!("Buffered: {}%", app.buffered_percent)),
        Line::from(""),
        Line::from("n/p next/previous · Space pause · m mute · v close"),
    ];
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .title(" CloudRuby info ")
                    .borders(Borders::ALL),
            )
            .style(style(config, "playlist", Color::Green, Color::Black)),
        area,
    );
}

fn style(config: &UiConfig, key: &str, default_fg: Color, default_bg: Color) -> Style {
    let values = config.colors.get(key);
    let foreground = values
        .and_then(|values| values.first())
        .and_then(|value| parse_color(value))
        .unwrap_or(default_fg);
    let background = values
        .and_then(|values| values.get(1))
        .and_then(|value| parse_color(value))
        .unwrap_or(default_bg);
    Style::default().fg(foreground).bg(background)
}

fn parse_color(value: &str) -> Option<Color> {
    match value.to_ascii_lowercase().as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" | "grey" | "white" => Some(Color::White),
        "reset" | "default" => Some(Color::Reset),
        _ => None,
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);
    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}

fn format_time(seconds: u64) -> String {
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::{MediaItem, MediaKind};
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn renders_original_player_regions() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = App::new(vec![MediaItem {
            title: "Clockwork Hearts".into(),
            uri: url::Url::parse("file:///music/clockwork-hearts.flac").unwrap(),
            kind: MediaKind::Local,
        }]);
        terminal
            .draw(|frame| render(frame, &app, &UiConfig::default()))
            .unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(rendered.contains("Clockwork Hearts"));
        assert!(rendered.contains("clockwork-hearts.flac"));
        assert!(rendered.contains("q quit"));
    }
}
