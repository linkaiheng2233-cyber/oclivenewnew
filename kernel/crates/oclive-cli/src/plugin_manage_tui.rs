//! `oclive plugin manage --tui` — a simple TUI overview of slot-to-plugin mappings.

use crate::init_tui::terminal_supports_tui;
use crate::plugin_manage_cmd::{find_role_dir, load_registry};
use anyhow::{bail, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use oclive_validation::PIPELINE_BLUEPRINT_FILENAME;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use std::io::stdout;
use std::path::Path;

pub fn run_plugin_manage_tui(role: Option<&Path>) -> Result<()> {
    if !terminal_supports_tui() {
        bail!("TUI requires a terminal (unset OCLIVE_NO_TUI to force off)");
    }
    let role_dir = find_role_dir(role)?;
    let reg = load_registry(&role_dir)?;

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let result = run_loop(&role_dir, &reg);
    disable_raw_mode().ok();
    let _ = stdout().execute(LeaveAlternateScreen);
    result
}

fn run_loop(
    role_dir: &Path,
    reg: &std::collections::BTreeMap<String, oclive_validation::SlotRegistryEntry>,
) -> Result<()> {
    let mut terminal = ratatui::init();
    let keys: Vec<String> = reg.keys().cloned().collect();
    let mut list_state =
        ListState::default().with_selected(if keys.is_empty() { None } else { Some(0) });

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(8),
                    Constraint::Length(4),
                ])
                .split(f.area());

            let title = format!(
                " oclive plugin manage — {} ",
                role_dir.file_name().unwrap_or_default().to_string_lossy()
            );
            f.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::raw("Ring: memory → emotion → event → complex_emotion → prompt → llm → agent"),
                ]))
                .block(Block::default().title(title).borders(Borders::ALL)),
                chunks[0],
            );

            let items: Vec<ListItem> = keys
                .iter()
                .enumerate()
                .map(|(i, k)| {
                    let e = &reg[k];
                    let plug = e.plugin.as_deref().unwrap_or("—");
                    let sel = list_state.selected() == Some(i);
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{:<18}", k),
                            if sel {
                                Style::default().add_modifier(Modifier::REVERSED)
                            } else {
                                Style::default()
                            },
                        ),
                        Span::raw(format!(
                            " {} backend={} plugin={}",
                            e.slot_type, e.backend, plug
                        )),
                    ]))
                })
                .collect();
            let list = List::new(items).block(
                Block::default()
                    .title(" slot_registry (↑↓ · q/Esc quit) ")
                    .borders(Borders::ALL),
            );
            f.render_stateful_widget(list, chunks[1], &mut list_state);

            let idx = list_state.selected().unwrap_or(0);
            let detail = if keys.is_empty() {
                format!("No slots in {}\nAdd: oclive plugin manage add-slot <type> <label>", PIPELINE_BLUEPRINT_FILENAME)
            } else {
                let k = &keys[idx];
                let e = &reg[k];
                format!(
                    "Key: {k}\nType: {}\nLabel: {}\nBackend: {}\nPosition: {}\nPlugin: {}\n\nCLI: link {k} <plugin-id> | set-backend {k} <backend>",
                    e.slot_type,
                    e.label,
                    e.backend,
                    e.position,
                    e.plugin.as_deref().unwrap_or("—"),
                )
            };
            f.render_widget(
                Paragraph::new(detail).block(Block::default().title(" detail ").borders(Borders::ALL)),
                chunks[2],
            );
        })?;

        if event::poll(std::time::Duration::from_millis(120))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Up => {
                        let i = list_state.selected().unwrap_or(0);
                        list_state.select(Some(i.saturating_sub(1)));
                    }
                    KeyCode::Down => {
                        let i = list_state.selected().unwrap_or(0);
                        let max = keys.len().saturating_sub(1);
                        list_state.select(Some((i + 1).min(max)));
                    }
                    _ => {}
                }
            }
        }
    }
    ratatui::restore();
    Ok(())
}
