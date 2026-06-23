//! `oclive init --tui` visual template selector (ratatui).

use crate::init::InitTemplateArg;
use crate::template_catalog::{project_config_from_template, template_from_id, CATALOG};
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use std::io::stdout;

pub fn terminal_supports_tui() -> bool {
    if std::env::var("OCLIVE_NO_TUI").is_ok() {
        return false;
    }
    std::io::IsTerminal::is_terminal(&std::io::stdout())
}

pub fn pick_template_tui(project_name: &str) -> Result<Option<InitTemplateArg>> {
    enable_raw_mode().context("enable_raw_mode")?;
    stdout()
        .execute(EnterAlternateScreen)
        .context("alt screen")?;
    let result = run_loop(project_name);
    disable_raw_mode().ok();
    let _ = stdout().execute(LeaveAlternateScreen);
    result
}

fn run_loop(project_name: &str) -> Result<Option<InitTemplateArg>> {
    let mut terminal = ratatui::init();
    let mut list_state = ListState::default().with_selected(Some(0));
    let mut cancelled = false;
    let mut chosen: Option<InitTemplateArg> = None;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
                .split(f.area());
            let items: Vec<ListItem> = CATALOG
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    let sel = list_state.selected() == Some(i);
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{:<16}", e.id),
                            if sel {
                                Style::default().add_modifier(Modifier::REVERSED)
                            } else {
                                Style::default()
                            },
                        ),
                        Span::raw(format!(" {}", e.scene)),
                    ]))
                })
                .collect();
            let list = List::new(items).block(
                Block::default()
                    .title(" kernel factory templates (↑↓ Enter Esc) ")
                    .borders(Borders::ALL),
            );
            f.render_stateful_widget(list, chunks[0], &mut list_state);

            let idx = list_state.selected().unwrap_or(0);
            let preview = preview_text(project_name, idx);
            let para = Paragraph::new(preview)
                .block(Block::default().title(" preview ").borders(Borders::ALL));
            f.render_widget(para, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Up => {
                        let i = list_state.selected().unwrap_or(0);
                        list_state.select(Some(i.saturating_sub(1)));
                    }
                    KeyCode::Down => {
                        let i = list_state.selected().unwrap_or(0);
                        let max = CATALOG.len().saturating_sub(1);
                        list_state.select(Some((i + 1).min(max)));
                    }
                    KeyCode::Enter => {
                        if let Some(e) = CATALOG.get(list_state.selected().unwrap_or(0)) {
                            chosen = template_from_id(e.id);
                        }
                        break;
                    }
                    KeyCode::Esc => {
                        cancelled = true;
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    ratatui::restore();
    if cancelled {
        println!("TUI cancelled; falling back to standard prompts.");
        return Ok(None);
    }
    Ok(chosen)
}

/// Monolith custom welding: Space toggles a slot, Enter confirms, Esc skips (falls back to the preset default welding).
pub fn pick_weld_modules_tui() -> Result<Option<Vec<String>>> {
    use crate::monolith_config::SLOT_IDS;
    enable_raw_mode().context("enable_raw_mode")?;
    stdout()
        .execute(EnterAlternateScreen)
        .context("alt screen")?;
    let result = run_weld_loop(&SLOT_IDS);
    disable_raw_mode().ok();
    let _ = stdout().execute(LeaveAlternateScreen);
    result
}

fn run_weld_loop(slots: &[&str; 7]) -> Result<Option<Vec<String>>> {
    let mut terminal = ratatui::init();
    let mut selected = [true, true, true, false, true, false, false];
    let mut cursor = 0usize;
    let mut skip = false;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(f.area());
            let lines: Vec<Line> = slots
                .iter()
                .enumerate()
                .map(|(i, id)| {
                    let mark = if selected[i] { "[x]" } else { "[ ]" };
                    let cur = if i == cursor { ">" } else { " " };
                    let style = if i == cursor {
                        Style::default().add_modifier(Modifier::REVERSED)
                    } else {
                        Style::default()
                    };
                    Line::styled(format!("{cur} {mark} {id}"), style)
                })
                .collect();
            let list = Paragraph::new(lines).block(
                Block::default()
                    .title(" custom weld (↑↓ · space · Enter · Esc skip) ")
                    .borders(Borders::ALL),
            );
            f.render_widget(list, chunks[0]);
            let n = selected.iter().filter(|&&b| b).count();
            let est_mib = (7 - n) as f64 * 0.35;
            let est_latency = -(n as f64 * 2.5);
            let preview = format!(
                "Welded slots: {n} / 7\nEst. binary reduction: ~{est_mib:.1} MiB (heuristic)\nEst. latency delta: ~{est_latency:.0}ms (heuristic)\n\nEnter → write monolith.toml weld_modules",
            );
            f.render_widget(
                Paragraph::new(preview).block(Block::default().title(" preview ").borders(Borders::ALL)),
                chunks[1],
            );
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Up => cursor = cursor.saturating_sub(1),
                    KeyCode::Down => cursor = (cursor + 1).min(slots.len() - 1),
                    KeyCode::Char(' ') => selected[cursor] = !selected[cursor],
                    KeyCode::Enter => break,
                    KeyCode::Esc => {
                        skip = true;
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    ratatui::restore();
    if skip {
        return Ok(None);
    }
    let out: Vec<String> = slots
        .iter()
        .enumerate()
        .filter(|&(i, _id)| selected[i])
        .map(|(_i, id)| (*id).to_string())
        .collect();
    if out.is_empty() {
        println!("No slots selected; skipping custom weld.");
        return Ok(None);
    }
    Ok(Some(out))
}

fn preview_text(project_name: &str, idx: usize) -> String {
    let Some(entry) = CATALOG.get(idx) else {
        return String::new();
    };
    let Some(t) = template_from_id(entry.id) else {
        return entry.description.to_string();
    };
    let cfg = project_config_from_template(project_name, t);
    format!(
        "Template: {}\nScene: {}\nDescription: {}\n\npreset: {}\nMonolith: {}\nproject-type: {:?}\nRole pack: {}\n\nEnter confirm · Esc manual setup",
        entry.id,
        entry.scene,
        entry.description,
        entry.preset,
        entry.monolith,
        cfg.project_type,
        crate::template_catalog::role_pack_label(cfg.role_pack_kind),
    )
}
