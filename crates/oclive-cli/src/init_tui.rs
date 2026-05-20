//! `oclive init --tui` 模板可视化选择器（ratatui）。

use crate::init::InitTemplateArg;
use crate::template_catalog::{project_config_from_template, CATALOG, template_from_id};
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
    stdout().execute(EnterAlternateScreen).context("alt screen")?;
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
                    .title(" 内核工厂模板 (↑↓ Enter Esc) ")
                    .borders(Borders::ALL),
            );
            f.render_stateful_widget(list, chunks[0], &mut list_state);

            let idx = list_state.selected().unwrap_or(0);
            let preview = preview_text(project_name, idx);
            let para = Paragraph::new(preview).block(
                Block::default()
                    .title(" 预览 ")
                    .borders(Borders::ALL),
            );
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
        println!("已取消 TUI，回退到常规交互。");
        return Ok(None);
    }
    Ok(chosen)
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
        "模板: {}\n场景: {}\n描述: {}\n\npreset: {}\nMonolith: {}\nproject-type: {:?}\n角色包: {}\n\nEnter 确认 · Esc 手动配置",
        entry.id,
        entry.scene,
        entry.description,
        entry.preset,
        entry.monolith,
        cfg.project_type,
        crate::template_catalog::role_pack_label(cfg.role_pack_kind),
    )
}
