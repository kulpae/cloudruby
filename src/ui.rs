#![expect(
    clippy::cast_sign_loss,
    clippy::indexing_slicing,
    clippy::string_slice,
    reason = "Rendering math and fixed-width terminal data are bounded by layout invariants."
)]

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, Gauge, List, ListItem, ListState, Padding, Paragraph,
        Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
};
use std::time::Duration;

use crate::{
    app::{App, PlaybackState},
    config::{StyleConfig, StyleSpec, UiConfig},
    library::MediaKind,
};

pub fn playlist_index_at(
    area: Rect,
    x: u16,
    y: u16,
    track_count: usize,
    offset: usize,
) -> Option<usize> {
    let inner = area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });
    if track_count == 0 || x < inner.x || x >= inner.x + inner.width || y < inner.y {
        return None;
    }
    let visible_height = usize::from(inner.height);
    let index = offset + usize::from(y - inner.y);
    (index < track_count && index < offset + visible_height).then_some(index)
}

pub fn playlist_area(area: Rect) -> Option<Rect> {
    let inner = Block::default().borders(Borders::ALL).inner(area);
    if inner.height < 8 || inner.width < 32 {
        return None;
    }
    let rows = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(1),
        Constraint::Length(2),
    ])
    .split(inner);
    if rows[2].width >= 92 {
        Some(
            Layout::horizontal([Constraint::Percentage(64), Constraint::Percentage(36)])
                .spacing(1)
                .split(rows[2])[0],
        )
    } else {
        Some(rows[2])
    }
}

pub fn progress_area(area: Rect) -> Option<Rect> {
    let inner = Block::default().borders(Borders::ALL).inner(area);
    if inner.height < 8 || inner.width < 32 {
        return None;
    }
    Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(1),
        Constraint::Length(2),
    ])
    .split(inner)
    .get(1)
    .copied()
}

pub fn progress_click_position(area: Rect, x: u16, y: u16, duration: Duration) -> Option<Duration> {
    if duration.is_zero() {
        return None;
    }
    let inner = Block::default().borders(Borders::ALL).inner(area);
    if x < inner.x || x >= inner.x + inner.width || y < inner.y || y >= inner.y + inner.height {
        return None;
    }
    let width = f64::from(inner.width.saturating_sub(1).max(1));
    let ratio = f64::from(x - inner.x) / width;
    Some(duration.mul_f64(ratio.clamp(0.0, 1.0)))
}

pub fn render(frame: &mut Frame<'_>, app: &mut App, config: &UiConfig) {
    let area = frame.area();
    let local_count = app
        .tracks
        .iter()
        .filter(|item| item.kind == MediaKind::Local)
        .count();
    let stream_count = app.tracks.len() - local_count;
    let track_stats = format!(
        " all {} · local {local_count} · stream {stream_count} ",
        app.tracks.len()
    );
    frame.render_widget(
        Block::default().style(themed(
            config,
            "app",
            Style::default()
                .fg(Color::Rgb(190, 205, 220))
                .bg(Color::Rgb(7, 17, 33)),
        )),
        area,
    );

    let shell = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◈ ", themed(config, "logo", Style::new().cyan().bold())),
            Span::styled(
                "CloudRuby ",
                themed(config, "header", Style::new().white().bold()),
            ),
        ]))
        .title_bottom(Line::from(track_stats).alignment(Alignment::Right))
        .borders(Borders::ALL)
        .border_type(border_type(config))
        .border_style(themed(config, "border", Style::new().dark_gray()));
    let inner = shell.inner(area);
    frame.render_widget(shell, area);

    if inner.height < 8 || inner.width < 32 {
        render_compact(frame, app, inner, config);
        render_source_input(frame, app, area, config);
        render_search_input(frame, app, area, config);
        render_help(frame, app, area, config);
        return;
    }

    let rows = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(1),
        Constraint::Length(2),
    ])
    .split(inner);
    render_header(frame, app, rows[0], config);
    render_progress(frame, app, rows[1], config);

    if rows[2].width >= 92 {
        let columns = Layout::horizontal([Constraint::Percentage(64), Constraint::Percentage(36)])
            .spacing(1)
            .split(rows[2]);
        render_playlist(frame, app, columns[0], config);
        render_visualizer(frame, app, columns[1], config);
    } else {
        render_playlist(frame, app, rows[2], config);
    }
    render_status(frame, app, rows[3], config);
    render_footer(frame, rows[4], config);

    if app.info_visible {
        let width = if area.width >= 100 { 58 } else { 82 };
        render_details(frame, app, centered_rect(width, 68, area), config, true);
    }
    render_source_input(frame, app, area, config);
    render_search_input(frame, app, area, config);
    render_help(frame, app, area, config);
}

fn render_compact(frame: &mut Frame<'_>, app: &App, area: Rect, config: &UiConfig) {
    let title = app
        .stream_title
        .as_deref()
        .or_else(|| app.playing_track().map(|item| item.title.as_str()))
        .or_else(|| app.current_track().map(|item| item.title.as_str()))
        .unwrap_or("No media");
    let symbol = playback_symbol(app.playback, config.unicode);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                format!("{symbol} {title}"),
                themed(config, "title", Style::new().cyan().bold()),
            )),
            Line::from("n/p navigate · Space pause · q quit"),
        ])
        .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_source_input(frame: &mut Frame<'_>, app: &App, area: Rect, config: &UiConfig) {
    let Some(input) = &app.source_input else {
        return;
    };
    let popup = centered_rect(78, 20, area);
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(format!(" {input}"))
            .block(panel(config, " Add source · Enter add · Esc cancel "))
            .style(themed(config, "input", Style::new().fg(Color::White)))
            .wrap(Wrap { trim: false }),
        popup,
    );
}

pub fn search_dialog_rect(area: Rect, result_count: usize) -> Rect {
    let width = (area.width * 78 / 100).clamp(32, area.width);
    let height = (result_count as u16 + 4)
        .min(area.height.saturating_sub(2))
        .max(4);
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}

pub fn search_results_area(dialog: Rect) -> Rect {
    let inner = dialog.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });
    Rect::new(
        inner.x,
        inner.y + 1,
        inner.width,
        inner.height.saturating_sub(1),
    )
}

pub fn search_index_at(
    area: Rect,
    x: u16,
    y: u16,
    result_count: usize,
    offset: usize,
) -> Option<usize> {
    if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
        return None;
    }
    let index = offset + usize::from(y - area.y);
    (index < result_count).then_some(index)
}

fn render_search_input(frame: &mut Frame<'_>, app: &mut App, area: Rect, config: &UiConfig) {
    let Some(input) = app.search_input.clone() else {
        return;
    };
    let matches = app.search_matches();
    let dialog = search_dialog_rect(area, matches.len());
    let results = search_results_area(dialog);
    app.ensure_search_offset(usize::from(results.height));
    frame.render_widget(Clear, dialog);
    frame.render_widget(
        Block::default()
            .title(format!(
                " Search · {} matches · Enter play · Esc close ",
                matches.len()
            ))
            .borders(Borders::ALL)
            .border_type(border_type(config))
            .border_style(themed(config, "border", Style::new().cyan())),
        dialog,
    );
    frame.render_widget(
        Paragraph::new(format!(" /{input}")).style(themed(
            config,
            "input",
            Style::new().fg(Color::White),
        )),
        Rect::new(results.x, results.y.saturating_sub(1), results.width, 1),
    );
    let items = matches
        .iter()
        .map(|&index| ListItem::new(Line::from(format!(" {}", app.tracks[index].title))));
    let list = List::new(items)
        .style(themed(config, "playlist", Style::new().white()))
        .highlight_symbol(if config.unicode { "▌" } else { ">" })
        .highlight_style(themed(
            config,
            "playlist_active",
            Style::new().black().on_cyan().bold(),
        ));
    let selected = matches
        .first()
        .map(|_| app.search_selected.min(matches.len() - 1));
    let mut state = ListState::default()
        .with_selected(selected)
        .with_offset(app.search_offset);
    frame.render_stateful_widget(list, results, &mut state);
}

fn render_help(frame: &mut Frame<'_>, app: &App, area: Rect, config: &UiConfig) {
    if !app.help_visible {
        return;
    }
    let dialog = centered_rect(78, 76, area);
    let bindings = [
        ("n / N", "play next track"),
        ("p / P", "play previous track"),
        ("Down / wheel down", "select next track"),
        ("Up / wheel up", "select previous track"),
        ("Enter", "play the selected track"),
        ("Click progress bar", "seek to that position"),
        ("Space", "pause or resume playback"),
        ("+ / =", "raise volume"),
        ("- / _", "lower volume"),
        ("m", "toggle mute"),
        ("s", "toggle shuffle"),
        ("a", "add a source"),
        ("v", "toggle track information"),
        ("/", "search track titles"),
        ("h / Esc", "close this help"),
        ("q / Esc", "quit"),
    ];
    let mut lines = bindings
        .iter()
        .map(|(key, action)| {
            Line::from(vec![
                Span::styled(
                    format!(" {key:<30}"),
                    themed(config, "key", Style::new().cyan().bold()),
                ),
                Span::styled(*action, themed(config, "footer", Style::new().white())),
            ])
        })
        .collect::<Vec<_>>();
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Search: type to filter matches; Enter or click a result to play it.",
        themed(config, "footer", Style::new().dark_gray()),
    )));
    frame.render_widget(Clear, dialog);
    frame.render_widget(
        Paragraph::new(lines)
            .block(panel(config, " Keyboard shortcuts "))
            .wrap(Wrap { trim: false }),
        dialog,
    );
}

fn render_header(frame: &mut Frame<'_>, app: &App, area: Rect, config: &UiConfig) {
    let item = app.playing_track().or_else(|| app.current_track());
    let title = app
        .stream_title
        .as_deref()
        .or_else(|| item.map(|item| item.title.as_str()))
        .unwrap_or("Nothing selected");
    let symbol = playback_symbol(app.playback, config.unicode);
    let volume = format!(
        " VOL {:>3}%{} ",
        (app.volume * 100.0).round(),
        if app.muted { " MUTE" } else { "" }
    );
    let header = Layout::horizontal([Constraint::Min(20), Constraint::Length(volume.len() as u16)])
        .split(area);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!(" {symbol} "),
                themed(config, "playback", Style::new().green().bold()),
            ),
            Span::styled(title, themed(config, "title", Style::new().cyan().bold())),
        ])),
        header[0],
    );
    frame.render_widget(
        Paragraph::new(volume)
            .alignment(Alignment::Right)
            .style(themed(config, "volume", Style::new().yellow())),
        header[1],
    );
}

fn render_progress(frame: &mut Frame<'_>, app: &App, area: Rect, config: &UiConfig) {
    let ratio = if app.duration.is_zero() {
        f64::from(app.buffered_percent) / 100.0
    } else {
        app.position.as_secs_f64() / app.duration.as_secs_f64()
    };
    let label = if app.duration.is_zero() {
        format!(" LIVE · buffered {}% ", app.buffered_percent)
    } else {
        format!(
            " {} / {} ",
            format_time(app.position.as_secs()),
            format_time(app.duration.as_secs())
        )
    };
    let block = panel(config, "");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    // Keep the visualizer's canvas in a deep blue-black so low-frequency
    // glyphs never read as a green background.
    frame.render_widget(
        Block::default().style(themed(
            config,
            "visualizer_background",
            Style::default().bg(Color::Rgb(5, 14, 28)),
        )),
        inner,
    );
    frame.render_widget(
        Gauge::default()
            .ratio(ratio.clamp(0.0, 1.0))
            .label(label)
            .use_unicode(config.unicode)
            .style(themed(config, "progress_track", Style::new().dark_gray()))
            .gauge_style(themed(
                config,
                "progress_fill",
                Style::new().cyan().on_black().bold(),
            )),
        inner,
    );
}

fn render_playlist(frame: &mut Frame<'_>, app: &mut App, area: Rect, config: &UiConfig) {
    let position = if app.tracks.is_empty() {
        "0/0".to_owned()
    } else {
        format!("{}/{}", app.selected + 1, app.tracks.len())
    };
    let panel_title = format!(" Queue · {position} ");
    let block = panel(config, &panel_title);
    let inner = block.inner(area);
    app.ensure_playlist_offset(
        Some(app.selected),
        app.tracks.len(),
        usize::from(inner.height),
    );
    let items = app.tracks.iter().enumerate().map(|(index, item)| {
        let marker = match (item.kind, config.unicode) {
            (MediaKind::Local, true) => "♪",
            (MediaKind::Stream, true) => "◉",
            (MediaKind::Local, false) => "L",
            (MediaKind::Stream, false) => "R",
        };
        ListItem::new(Line::from(vec![
            Span::styled(
                format!(" {:>3} ", index + 1),
                themed(config, "playlist_index", Style::new().dark_gray()),
            ),
            Span::styled(
                format!("{marker} "),
                themed(config, "media_icon", Style::new().magenta()),
            ),
            Span::raw(&item.title),
        ]))
    });
    let list = List::new(items)
        .block(block)
        .style(themed(config, "playlist", Style::new().white()))
        .highlight_symbol(if config.unicode { "▌" } else { ">" })
        .highlight_style(themed(
            config,
            "playlist_active",
            Style::new().black().on_cyan().bold(),
        ));
    let mut state = ListState::default()
        .with_selected(Some(app.selected))
        .with_offset(app.playlist_offset);
    frame.render_stateful_widget(list, area, &mut state);

    if app.tracks.len() > usize::from(inner.height) {
        let mut scrollbar_state = ScrollbarState::new(app.tracks.len())
            .position(app.selected)
            .viewport_content_length(usize::from(inner.height));
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(Some(if config.unicode { "│" } else { "|" }))
            .thumb_symbol(if config.unicode { "█" } else { "#" })
            .track_style(themed(config, "scrollbar_track", Style::new().dark_gray()))
            .thumb_style(themed(config, "scrollbar_thumb", Style::new().cyan()));
        frame.render_stateful_widget(
            scrollbar,
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

fn render_visualizer(frame: &mut Frame<'_>, app: &App, area: Rect, config: &UiConfig) {
    let energy = app.spectrum_activity;
    let title = format!(
        " Spectrum · RADIAL · {:>3}🗲 · {:>3.0}° ",
        (energy * 100.0).round(),
        app.visualizer_rotation.to_degrees().rem_euclid(360.0)
    );
    let mut block = panel(config, &title);
    if energy > 0.58 {
        block = block.border_style(themed(
            config,
            "visualizer_hot",
            Style::new().light_magenta().bold(),
        ));
    }
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.is_empty() {
        return;
    }
    if config.unicode {
        render_radial_braille(frame, inner, app, config);
    } else {
        render_radial_ascii(frame, inner, app, config);
    }

    if !app.spectrum_active || app.spectrum.is_empty() {
        render_waiting_indicator(frame, inner, app, config);
    }
}

fn render_waiting_indicator(frame: &mut Frame<'_>, area: Rect, app: &App, config: &UiConfig) {
    let unicode_spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let ascii_spinner = ["|", "/", "-", "\\"];
    let spinner = if config.unicode {
        unicode_spinner[(app.visualizer_frame / 2) as usize % unicode_spinner.len()]
    } else {
        ascii_spinner[(app.visualizer_frame / 3) as usize % ascii_spinner.len()]
    };
    let x = area.x + area.width / 2;
    let y = area.y + area.height / 2;
    frame.buffer_mut().set_string(
        x,
        y,
        spinner,
        themed(config, "visualizer_idle", Style::new().bold()),
    );
}

#[expect(
    dead_code,
    reason = "Retained as an alternate renderer for terminal compatibility."
)]
fn render_braille_spectrum(
    frame: &mut Frame<'_>,
    area: Rect,
    levels: &[f32],
    peaks: &[f32],
    config: &UiConfig,
) {
    let pixel_height = usize::from(area.height) * 4;
    let heights = levels
        .iter()
        .map(|value| (value * pixel_height as f32).round() as usize)
        .collect::<Vec<_>>();
    let peak_heights = peaks
        .iter()
        .map(|value| (value * pixel_height as f32).round() as usize)
        .collect::<Vec<_>>();
    let mut lines = Vec::with_capacity(usize::from(area.height));
    for row in 0..usize::from(area.height) {
        let mut spans = Vec::with_capacity(usize::from(area.width));
        for cell in 0..usize::from(area.width) {
            let left = cell * 2;
            let right = left + 1;
            let (glyph, has_peak) = braille_cell(
                heights.get(left).copied().unwrap_or(0),
                heights.get(right).copied().unwrap_or(0),
                peak_heights.get(left).copied().unwrap_or(0),
                peak_heights.get(right).copied().unwrap_or(0),
                row,
                usize::from(area.height),
            );
            spans.push(Span::styled(
                glyph.to_string(),
                visualizer_style(config, row, usize::from(area.height), has_peak),
            ));
        }
        lines.push(Line::from(spans));
    }
    frame.render_widget(Paragraph::new(lines), area);
}

#[expect(
    dead_code,
    reason = "Retained as an alternate renderer for terminal compatibility."
)]
fn render_ascii_spectrum(
    frame: &mut Frame<'_>,
    area: Rect,
    levels: &[f32],
    peaks: &[f32],
    config: &UiConfig,
) {
    let height = usize::from(area.height);
    let mut lines = Vec::with_capacity(height);
    for row in 0..height {
        let threshold = (height - row) as f32 / height as f32;
        let spans = levels
            .iter()
            .zip(peaks)
            .map(|(level, peak)| {
                let is_peak = *peak >= threshold && *peak < threshold + 1.0 / height as f32;
                Span::styled(
                    if *level >= threshold {
                        "#"
                    } else if is_peak {
                        "-"
                    } else {
                        " "
                    },
                    visualizer_style(config, row, height, is_peak),
                )
            })
            .collect::<Vec<_>>();
        lines.push(Line::from(spans));
    }
    frame.render_widget(Paragraph::new(lines), area);
}

#[expect(
    dead_code,
    reason = "Retained as an alternate renderer for terminal compatibility."
)]
fn spectrum_columns(
    bands: &[f32],
    count: usize,
    sensitivity: f32,
    frame: u64,
    energy: f32,
) -> Vec<f32> {
    if bands.is_empty() || count == 0 {
        return Vec::new();
    }
    (0..count)
        .map(|column| {
            let start_ratio = column as f32 / count as f32;
            let end_ratio = (column + 1) as f32 / count as f32;
            let start = (start_ratio.powf(2.15) * bands.len() as f32).floor() as usize;
            let end = (end_ratio.powf(2.15) * bands.len() as f32).ceil() as usize;
            let level = bands[start.min(bands.len() - 1)..end.clamp(start + 1, bands.len())]
                .iter()
                .copied()
                .fold(0.0_f32, f32::max);
            let phase = frame as f32 * 0.14 + column as f32 * 0.41;
            let shimmer = 1.0 + phase.sin() * energy * 0.09;
            (level * sensitivity.max(0.1) * shimmer).clamp(0.0, 1.0)
        })
        .collect()
}

fn braille_cell(
    left_height: usize,
    right_height: usize,
    left_peak: usize,
    right_peak: usize,
    row: usize,
    rows: usize,
) -> (char, bool) {
    const LEFT: [u32; 4] = [0x01, 0x02, 0x04, 0x40];
    const RIGHT: [u32; 4] = [0x08, 0x10, 0x20, 0x80];
    let base = (rows.saturating_sub(row + 1)) * 4;
    let mut bits = 0;
    let mut has_peak = false;
    for dot_row in 0..4 {
        let level = base + (3 - dot_row);
        if left_height > level {
            bits |= LEFT[dot_row];
        }
        if right_height > level {
            bits |= RIGHT[dot_row];
        }
        if left_peak.checked_sub(1) == Some(level) {
            bits |= LEFT[dot_row];
            has_peak = true;
        }
        if right_peak.checked_sub(1) == Some(level) {
            bits |= RIGHT[dot_row];
            has_peak = true;
        }
    }
    (char::from_u32(0x2800 + bits).unwrap_or(' '), has_peak)
}

fn visualizer_style(config: &UiConfig, row: usize, rows: usize, peak: bool) -> Style {
    if peak {
        return themed(
            config,
            "visualizer_peak",
            Style::new().light_yellow().bold(),
        );
    }
    let height = 1.0 - row as f32 / rows.max(1) as f32;
    if height > 0.72 {
        themed(config, "visualizer_high", Style::new().light_magenta())
    } else if height > 0.38 {
        themed(config, "visualizer_mid", Style::new().light_cyan())
    } else {
        themed(config, "visualizer_low", Style::new().light_green())
    }
}

fn render_radial_braille(frame: &mut Frame<'_>, area: Rect, app: &App, config: &UiConfig) {
    let width = usize::from(area.width) * 2;
    let height = usize::from(area.height) * 4;
    let cells = usize::from(area.width) * usize::from(area.height);
    let mut bits = vec![0_u32; cells];
    let mut peak_cells = vec![false; cells];
    let mut bands = vec![0_usize; cells];
    let mut star_cells = vec![false; cells];
    let mut trail_cells = vec![false; cells];
    let cx = (width as f32 - 1.0) / 2.0;
    let cy = (height as f32 - 1.0) / 2.0;
    let radius = (width.min(height) as f32 * 0.46).max(2.0);
    let inner = radius * 0.27;
    let count = app.spectrum.len().max(24);
    // A deterministic star field sits behind the FFT spokes. Its twinkle and
    // density are driven by the smoothed spectral activity, so silence settles
    // into a quiet blue-black sky.
    let star_count = (cells / 5).clamp(8, 72);
    for star in 0..star_count {
        let seed = star as f32 * 12.9898 + 78.233;
        let radius_ratio = (seed.sin() * 43_758.547).fract().abs();
        let phase = app.visualizer_frame as f32 * (0.035 + radius_ratio * 0.08) + seed;
        let twinkle = (phase.sin() * 0.5 + 0.5) * (0.25 + app.spectrum_activity * 0.9);
        if twinkle < 0.16 {
            continue;
        }
        let angle = seed.cos() * std::f32::consts::PI + phase * 0.02;
        let r = radius * (0.90 + radius_ratio * 0.14);
        for (scale, is_core) in [(0.80, false), (0.90, false), (1.0, true)] {
            let sx = (cx + angle.cos() * r * scale).round() as i32 / 2;
            let sy = (cy + angle.sin() * r * scale).round() as i32 / 4;
            if sx >= 0
                && sy >= 0
                && (sx as usize) < usize::from(area.width)
                && (sy as usize) < usize::from(area.height)
            {
                let index = sy as usize * usize::from(area.width) + sx as usize;
                if is_core {
                    star_cells[index] = true;
                    let growth = (1.0 + twinkle * 2.0) as i32;
                    for dy in -growth..=growth {
                        for dx in -growth..=growth {
                            if dx * dx + dy * dy <= growth * growth {
                                plot_braille(
                                    &mut bits,
                                    &mut peak_cells,
                                    &mut bands,
                                    usize::from(area.width),
                                    sx * 2 + dx,
                                    sy * 4 + dy,
                                    false,
                                    0,
                                );
                            }
                        }
                    }
                } else {
                    trail_cells[index] = true;
                }
            }
        }
        /*
        let sx = (cx + angle.cos() * r).round() as i32 / 2;
        let sy = (cy + angle.sin() * r).round() as i32 / 4;
        if sx >= 0
            && sy >= 0
            && (sx as usize) < usize::from(area.width)
            && (sy as usize) < usize::from(area.height)
        {
            star_cells[sy as usize * usize::from(area.width) + sx as usize] = true;
            let star_style = themed(
                config,
                "visualizer_stars",
                Style::new()
                    .fg(Color::Rgb(75, 145, 205))
                    .add_modifier(Modifier::DIM),
            );
            frame.buffer_mut().set_string(
                area.x + sx as u16,
                area.y + sy as u16,
                if twinkle > 0.72 { "✦" } else { "·" },
                star_style,
            );
        }
        */
        plot_braille(
            &mut bits,
            &mut peak_cells,
            &mut bands,
            usize::from(area.width),
            (cx + angle.cos() * r) as i32,
            (cy + angle.sin() * r) as i32,
            false,
            0,
        );
    }
    for band in 0..count {
        let source = band * app.spectrum.len() / count;
        let level = app.spectrum.get(source).copied().unwrap_or(0.0)
            * config.visualizer_sensitivity.max(0.1);
        let peak = app.spectrum_peaks.get(source).copied().unwrap_or(level);
        let angle = app.visualizer_rotation + std::f32::consts::TAU * band as f32 / count as f32;
        let end = inner + radius * 0.68 * level.clamp(0.0, 1.0);
        let steps = ((end - inner).max(1.0) * 2.0) as usize;
        for step in 0..=steps {
            let r = inner + (end - inner) * step as f32 / steps.max(1) as f32;
            plot_braille(
                &mut bits,
                &mut peak_cells,
                &mut bands,
                usize::from(area.width),
                (cx + angle.cos() * r) as i32,
                (cy + angle.sin() * r) as i32,
                false,
                band,
            );
        }
        let peak_r = inner + radius * 0.68 * peak.clamp(0.0, 1.0);
        plot_braille(
            &mut bits,
            &mut peak_cells,
            &mut bands,
            usize::from(area.width),
            (cx + angle.cos() * peak_r) as i32,
            (cy + angle.sin() * peak_r) as i32,
            true,
            band,
        );
    }
    for ring in [inner, radius * (0.82 + app.spectrum_activity * 0.08)] {
        for step in 0..360 {
            let angle = step as f32 * std::f32::consts::TAU / 360.0 + app.visualizer_rotation * 0.2;
            plot_braille(
                &mut bits,
                &mut peak_cells,
                &mut bands,
                usize::from(area.width),
                (cx + angle.cos() * ring) as i32,
                (cy + angle.sin() * ring) as i32,
                false,
                step % count,
            );
        }
    }
    for y in 0..usize::from(area.height) {
        for x in 0..usize::from(area.width) {
            let index = y * usize::from(area.width) + x;
            let glyph = char::from_u32(0x2800 + bits[index]).unwrap_or(' ');
            if glyph != ' ' {
                let style = gradient_style(config, bands[index], count, peak_cells[index]);
                frame.buffer_mut().set_string(
                    area.x + x as u16,
                    area.y + y as u16,
                    glyph.to_string(),
                    style,
                );
            }
        }
    }
    let star_style = themed(
        config,
        "visualizer_stars",
        Style::new()
            .fg(Color::Rgb(75, 145, 205))
            .add_modifier(Modifier::DIM),
    );
    let trail_style = themed(
        config,
        "visualizer_star_trail",
        Style::new()
            .fg(Color::Rgb(20, 55, 95))
            .add_modifier(Modifier::DIM),
    );
    for y in 0..usize::from(area.height) {
        for x in 0..usize::from(area.width) {
            let index = y * usize::from(area.width) + x;
            if (trail_cells[index] || star_cells[index]) && !peak_cells[index] {
                let style = if star_cells[index] {
                    star_style
                } else {
                    trail_style
                };
                let glyph = char::from_u32(0x2800 + bits[index]).unwrap_or(' ');
                if glyph != ' ' {
                    frame.buffer_mut().set_string(
                        area.x + x as u16,
                        area.y + y as u16,
                        glyph.to_string(),
                        style,
                    );
                }
            }
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "The renderer receives independent buffers for each visual layer."
)]
fn plot_braille(
    bits: &mut [u32],
    peaks: &mut [bool],
    bands: &mut [usize],
    width: usize,
    x: i32,
    y: i32,
    peak: bool,
    band: usize,
) {
    const DOTS: [[u32; 4]; 2] = [[0x01, 0x02, 0x04, 0x40], [0x08, 0x10, 0x20, 0x80]];
    if x < 0 || y < 0 {
        return;
    }
    let (x, y) = (x as usize, y as usize);
    let index = (y / 4) * width + x / 2;
    if index >= bits.len() {
        return;
    }
    bits[index] |= DOTS[x % 2][y % 4];
    peaks[index] |= peak;
    bands[index] = bands[index].max(band);
}

fn gradient_style(config: &UiConfig, band: usize, count: usize, peak: bool) -> Style {
    if peak {
        return themed(
            config,
            "visualizer_peak",
            Style::new().light_yellow().bold(),
        );
    }
    let t = band as f32 / count.max(1) as f32;
    let (r, g, b) = if t < 0.5 {
        let p = t * 2.0;
        (
            (18.0 * (1.0 - p) + 0.0 * p) as u8,
            (70.0 * (1.0 - p) + 210.0 * p) as u8,
            (130.0 * (1.0 - p) + 255.0 * p) as u8,
        )
    } else {
        let p = (t - 0.5) * 2.0;
        (
            (0.0 * (1.0 - p) + 255.0 * p) as u8,
            (210.0 * (1.0 - p) + 80.0 * p) as u8,
            (255.0 * (1.0 - p) + 180.0 * p) as u8,
        )
    };
    let key = if t < 0.38 {
        "visualizer_low"
    } else if t < 0.72 {
        "visualizer_mid"
    } else {
        "visualizer_high"
    };
    themed(config, key, Style::new().fg(Color::Rgb(r, g, b)))
}

fn render_radial_ascii(frame: &mut Frame<'_>, area: Rect, app: &App, config: &UiConfig) {
    let width = usize::from(area.width);
    let height = usize::from(area.height);
    let cx = (width as f32 - 1.0) / 2.0;
    let cy = (height as f32 - 1.0) / 2.0;
    let radius = (width.min(height) as f32 * 0.42).max(1.0);
    let inner = radius * 0.28;
    let count = app.spectrum.len().max(24);
    for band in 0..count {
        let source = band * app.spectrum.len() / count;
        let level = app
            .spectrum
            .get(source)
            .copied()
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let angle = app.visualizer_rotation + std::f32::consts::TAU * band as f32 / count as f32;
        let end = inner + radius * 0.7 * level;
        for step in 0..=(end as usize) {
            let r = step as f32;
            let x = (cx + angle.cos() * r).round() as i32;
            let y = (cy + angle.sin() * r).round() as i32;
            if x >= 0 && y >= 0 && (x as usize) < width && (y as usize) < height {
                let glyph = if step + 1 >= end as usize { "*" } else { "·" };
                frame.buffer_mut().set_string(
                    area.x + x as u16,
                    area.y + y as u16,
                    glyph,
                    gradient_style(config, band, count, false),
                );
            }
        }
    }
}

fn render_details(frame: &mut Frame<'_>, app: &App, area: Rect, config: &UiConfig, overlay: bool) {
    if overlay {
        frame.render_widget(Clear, area);
    }
    let item = app.playing_track().or_else(|| app.current_track());
    let kind = item.map_or("—", |item| match item.kind {
        MediaKind::Local => "Local file",
        MediaKind::Stream => "Network stream",
    });
    let lines = vec![
        detail_line(
            config,
            "TITLE",
            app.stream_title
                .as_deref()
                .or_else(|| item.map(|item| item.title.as_str()))
                .unwrap_or("—"),
        ),
        if item.is_some_and(|item| item.kind == MediaKind::Stream) {
            detail_line(
                config,
                "STATION",
                item.map_or("—", |item| item.title.as_str()),
            )
        } else {
            Line::from("")
        },
        Line::from(""),
        detail_line(config, "TYPE", kind),
        detail_line(
            config,
            "SOURCE",
            &item.map_or_else(|| "—".into(), |item| item.source_label()),
        ),
        Line::from(""),
        detail_line(config, "STATE", playback_name(app.playback)),
        detail_line(
            config,
            "VOLUME",
            &format!(
                "{}%{}",
                (app.volume * 100.0).round(),
                if app.muted { " · muted" } else { "" }
            ),
        ),
        detail_line(config, "BUFFER", &format!("{}%", app.buffered_percent)),
        Line::from(""),
        Line::from(Span::styled(
            "Press v to close",
            themed(config, "hint", Style::new().dark_gray().italic()),
        )),
    ];
    let title = if overlay {
        " Now playing · v close "
    } else {
        " Now playing "
    };
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(panel(config, title))
            .style(themed(config, "details", Style::new().white())),
        area,
    );
}

fn detail_line(config: &UiConfig, label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{label:<8}"),
            themed(config, "detail_label", Style::new().cyan().bold()),
        ),
        Span::styled(
            value.to_owned(),
            themed(config, "detail_value", Style::new().white()),
        ),
    ])
}

fn render_status(frame: &mut Frame<'_>, app: &App, area: Rect, config: &UiConfig) {
    let loading_message;
    let (symbol, message) = if app.loading {
        loading_message = format!("Loading sources… {} tracks queued", app.loaded_count);
        ("…", loading_message.as_str())
    } else if let Some((message, _)) = app.status.as_ref() {
        ("◆", message.as_str())
    } else {
        ("", "")
    };
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                format!(" {symbol} "),
                themed(config, "status_icon", Style::new().green()),
            ),
            Span::styled(message, themed(config, "status", Style::new().gray())),
        ])),
        area,
    );
}

fn render_footer(frame: &mut Frame<'_>, area: Rect, config: &UiConfig) {
    let bindings = [
        ("n/p", "track"),
        ("space", "pause"),
        ("/", "search"),
        ("h", "help"),
        ("q", "quit"),
    ];
    let mut spans = vec![Span::raw(" ")];
    for (index, (key, action)) in bindings.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(
                "  ·  ",
                themed(config, "separator", Style::new().dark_gray()),
            ));
        }
        spans.push(Span::styled(
            format!(" {key} "),
            themed(
                config,
                "key",
                Style::new()
                    .fg(Color::Cyan)
                    .bg(Color::Rgb(20, 35, 55))
                    .bold(),
            ),
        ));
        spans.push(Span::styled(
            format!(" {action}"),
            themed(config, "footer", Style::new().dark_gray()),
        ));
    }
    frame.render_widget(
        Paragraph::new(Line::from(spans))
            .block(Block::default().padding(Padding::left(1)))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn panel<'a>(config: &UiConfig, title: &'a str) -> Block<'a> {
    Block::default()
        .title(Span::styled(
            title,
            themed(config, "panel_title", Style::new().gray().bold()),
        ))
        .borders(Borders::ALL)
        .border_type(border_type(config))
        .border_style(themed(config, "border", Style::new().dark_gray()))
        .padding(Padding::horizontal(1))
}

fn themed(config: &UiConfig, key: &str, fallback: Style) -> Style {
    let Some(spec) = config.colors.get(key) else {
        return fallback;
    };
    match spec {
        StyleConfig::Detailed(spec) => apply_spec(fallback, spec),
        StyleConfig::Legacy(values) => {
            let spec = StyleSpec {
                fg: values.first().cloned(),
                bg: values.get(1).cloned(),
                modifiers: values.iter().skip(2).cloned().collect(),
            };
            apply_spec(fallback, &spec)
        }
    }
}

fn apply_spec(mut style: Style, spec: &StyleSpec) -> Style {
    if let Some(color) = spec.fg.as_deref().and_then(parse_color) {
        style = style.fg(color);
    }
    if let Some(color) = spec.bg.as_deref().and_then(parse_color) {
        style = style.bg(color);
    }
    for modifier in &spec.modifiers {
        if let Some(modifier) = parse_modifier(modifier) {
            style = style.add_modifier(modifier);
        }
    }
    style
}

fn parse_color(value: &str) -> Option<Color> {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "reset" | "default" => Some(Color::Reset),
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" | "grey" | "white" => Some(Color::Gray),
        "dark_gray" | "dark_grey" => Some(Color::DarkGray),
        "light_red" => Some(Color::LightRed),
        "light_green" => Some(Color::LightGreen),
        "light_yellow" => Some(Color::LightYellow),
        "light_blue" => Some(Color::LightBlue),
        "light_magenta" => Some(Color::LightMagenta),
        "light_cyan" => Some(Color::LightCyan),
        "bright_white" => Some(Color::White),
        _ => parse_hex(&normalized)
            .or_else(|| parse_rgb(&normalized))
            .or_else(|| parse_indexed(&normalized)),
    }
}

fn parse_hex(value: &str) -> Option<Color> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    Some(Color::Rgb(
        u8::from_str_radix(&hex[0..2], 16).ok()?,
        u8::from_str_radix(&hex[2..4], 16).ok()?,
        u8::from_str_radix(&hex[4..6], 16).ok()?,
    ))
}

fn parse_rgb(value: &str) -> Option<Color> {
    let values = value.strip_prefix("rgb(")?.strip_suffix(')')?;
    let mut channels = values.split(',').map(str::trim).map(str::parse::<u8>);
    let color = Color::Rgb(
        channels.next()?.ok()?,
        channels.next()?.ok()?,
        channels.next()?.ok()?,
    );
    channels.next().is_none().then_some(color)
}

fn parse_indexed(value: &str) -> Option<Color> {
    value
        .strip_prefix("ansi:")
        .unwrap_or(value)
        .parse::<u8>()
        .ok()
        .map(Color::Indexed)
}

fn parse_modifier(value: &str) -> Option<Modifier> {
    match value.trim().to_ascii_lowercase().as_str() {
        "bold" => Some(Modifier::BOLD),
        "dim" => Some(Modifier::DIM),
        "italic" => Some(Modifier::ITALIC),
        "underlined" | "underline" => Some(Modifier::UNDERLINED),
        "reversed" | "reverse" => Some(Modifier::REVERSED),
        "crossed_out" | "strikethrough" => Some(Modifier::CROSSED_OUT),
        "slow_blink" => Some(Modifier::SLOW_BLINK),
        "rapid_blink" => Some(Modifier::RAPID_BLINK),
        "hidden" => Some(Modifier::HIDDEN),
        _ => None,
    }
}

fn border_type(config: &UiConfig) -> BorderType {
    match config.border_type.to_ascii_lowercase().as_str() {
        "plain" => BorderType::Plain,
        "double" => BorderType::Double,
        "thick" => BorderType::Thick,
        "quadrant_inside" => BorderType::QuadrantInside,
        "quadrant_outside" => BorderType::QuadrantOutside,
        _ => BorderType::Rounded,
    }
}

fn playback_symbol(state: PlaybackState, unicode: bool) -> &'static str {
    match (state, unicode) {
        (PlaybackState::Playing, true) => "▶",
        (PlaybackState::Paused, true) => "Ⅱ",
        (PlaybackState::Buffering, true) => "◌",
        (PlaybackState::Stopped, true) => "■",
        (PlaybackState::Playing, false) => ">",
        (PlaybackState::Paused, false) => "||",
        (PlaybackState::Buffering, false) => "~",
        (PlaybackState::Stopped, false) => "#",
    }
}

fn playback_name(state: PlaybackState) -> &'static str {
    match state {
        PlaybackState::Playing => "Playing",
        PlaybackState::Paused => "Paused",
        PlaybackState::Buffering => "Buffering",
        PlaybackState::Stopped => "Stopped",
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
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::{MediaItem, MediaKind};
    use ratatui::{Terminal, backend::TestBackend};

    fn sample_app() -> App {
        let mut app = App::new(vec![
            MediaItem {
                title: "Clockwork Hearts".into(),
                uri: url::Url::parse("file:///music/clockwork-hearts.flac").unwrap(),
                kind: MediaKind::Local,
            },
            MediaItem {
                title: "Night Radio".into(),
                uri: url::Url::parse("https://radio.example/live").unwrap(),
                kind: MediaKind::Stream,
            },
        ]);
        app.update_spectrum(
            &(0..128)
                .map(|band| ((band as f32 * 0.31).sin().abs() * 0.8) + 0.1)
                .collect::<Vec<_>>(),
        );
        app
    }

    #[test]
    fn renders_rich_player_regions() {
        let backend = TestBackend::new(110, 28);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = sample_app();
        terminal
            .draw(|frame| render(frame, &mut app, &UiConfig::default()))
            .unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(rendered.contains("CloudRuby"));
        assert!(rendered.contains("Clockwork Hearts"));
        assert!(rendered.contains("all 2 · local 1 · stream 1"));
        assert!(rendered.contains("Spectrum"));
        assert!(
            rendered
                .chars()
                .any(|value| ('\u{2801}'..='\u{28ff}').contains(&value))
        );
        assert!(!rendered.contains("SOURCE"));
        assert!(rendered.contains("space"));
    }

    #[test]
    fn track_details_only_appear_when_toggled() {
        let backend = TestBackend::new(110, 28);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = sample_app();
        app.info_visible = true;
        terminal
            .draw(|frame| render(frame, &mut app, &UiConfig::default()))
            .unwrap();
        let rendered: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(rendered.contains("Now playing"));
        assert!(rendered.contains("SOURCE"));
    }

    #[test]
    fn braille_cells_use_all_eight_dots() {
        assert_eq!(braille_cell(4, 4, 0, 0, 0, 1), ('\u{28ff}', false));
        assert_ne!(braille_cell(1, 0, 0, 0, 0, 1).0, '\u{2800}');
    }

    #[test]
    fn supports_named_hex_rgb_and_indexed_colors() {
        assert_eq!(parse_color("light_blue"), Some(Color::LightBlue));
        assert_eq!(parse_color("#12a0ff"), Some(Color::Rgb(0x12, 0xa0, 0xff)));
        assert_eq!(parse_color("rgb(1, 2, 3)"), Some(Color::Rgb(1, 2, 3)));
        assert_eq!(parse_color("ansi:236"), Some(Color::Indexed(236)));
        assert_eq!(parse_color("42"), Some(Color::Indexed(42)));
    }

    #[test]
    fn applies_detailed_theme_modifiers() {
        let config: UiConfig = toml::from_str(
            r##"
            [colors]
            title = { fg = "#ff8800", bg = "ansi:236", modifiers = ["bold", "italic"] }
            "##,
        )
        .unwrap();
        let style = themed(&config, "title", Style::default());
        assert_eq!(style.fg, Some(Color::Rgb(255, 136, 0)));
        assert_eq!(style.bg, Some(Color::Indexed(236)));
        assert!(
            style
                .add_modifier
                .contains(Modifier::BOLD | Modifier::ITALIC)
        );
    }
}
