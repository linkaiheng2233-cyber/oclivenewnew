//! `oclive market` — 浏览 / 搜索 / 安装插件与模板。

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use std::io::stdout;
use std::path::PathBuf;
use std::process::Command;

use crate::market_index::{fetch_market_index, find_item, search_items, MarketKind, MarketItem};
use crate::plugin_ext::PluginInstallArgs;
use crate::publish_cmd;

#[derive(Parser, Debug)]
pub struct MarketCli {
    #[command(subcommand)]
    pub command: MarketCommands,
}

#[derive(Subcommand, Debug)]
pub enum MarketCommands {
    /// 搜索插件、模板与角色包
    Search(MarketSearchArgs),
    /// TUI 分类浏览（Enter 安装，Esc 退出）
    Browse(MarketBrowseArgs),
    /// 安装插件 / 模板 / 角色包
    Install(MarketInstallArgs),
    /// 查看条目详情
    Info(MarketInfoArgs),
}

#[derive(Parser, Debug)]
pub struct MarketSearchArgs {
    pub keyword: String,
    #[arg(long)]
    pub json: bool,
}

#[derive(Parser, Debug)]
pub struct MarketBrowseArgs {
    #[arg(short = 'o', long, default_value = "./plugins")]
    pub plugins_dir: PathBuf,
    #[arg(long, default_value = ".")]
    pub template_output: PathBuf,
}

#[derive(Parser, Debug)]
pub struct MarketInstallArgs {
    pub id: String,
    #[arg(short = 'o', long, default_value = "./plugins")]
    pub plugins_dir: PathBuf,
    #[arg(long, default_value = ".")]
    pub template_output: PathBuf,
}

#[derive(Parser, Debug)]
pub struct MarketInfoArgs {
    pub id: String,
    #[arg(long)]
    pub json: bool,
}

pub fn run(cli: MarketCli) -> Result<()> {
    match cli.command {
        MarketCommands::Search(a) => run_search(a),
        MarketCommands::Browse(a) => run_browse(a),
        MarketCommands::Install(a) => run_install(a),
        MarketCommands::Info(a) => run_info(a),
    }
}

fn run_search(args: MarketSearchArgs) -> Result<()> {
    let index = fetch_market_index()?;
    let hits = search_items(&index, &args.keyword);
    if args.json {
        println!("{}", serde_json::to_string_pretty(&hits)?);
        return Ok(());
    }
    if hits.is_empty() {
        println!("（无匹配）");
        return Ok(());
    }
    for p in hits {
        print_item_line(&p);
    }
    Ok(())
}

fn run_info(args: MarketInfoArgs) -> Result<()> {
    let index = fetch_market_index()?;
    let item = find_item(&index, &args.id)
        .ok_or_else(|| anyhow::anyhow!("市场索引中无: {}", args.id))?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&item)?);
        return Ok(());
    }
    print_item_detail(&item);
    Ok(())
}

fn run_install(args: MarketInstallArgs) -> Result<()> {
    let index = fetch_market_index()?;
    let item = find_item(&index, &args.id)
        .ok_or_else(|| anyhow::anyhow!("市场索引中无: {}", args.id))?;
    install_item(&item, &args.plugins_dir, &args.template_output)?;
    Ok(())
}

pub fn install_item(item: &MarketItem, plugins_dir: &PathBuf, template_out: &PathBuf) -> Result<()> {
    let kind: MarketKind = item.kind.into();
    match kind {
        MarketKind::Plugin => install_plugin_item(item, plugins_dir),
        MarketKind::Template => install_template_item(item, template_out),
        MarketKind::RolePack => install_role_pack_item(item, template_out),
    }
}

fn install_plugin_item(item: &MarketItem, plugins_dir: &PathBuf) -> Result<()> {
    if let Some(git) = item.git.as_deref().filter(|s| !s.is_empty()) {
        let dst = plugins_dir.join(&item.id);
        std::fs::create_dir_all(plugins_dir)?;
        if dst.exists() {
            std::fs::remove_dir_all(&dst).ok();
        }
        let st = Command::new("git")
            .args(["clone", "--depth", "1", git, dst.to_string_lossy().as_ref()])
            .status()
            .context("git clone")?;
        if !st.success() {
            bail!("git clone 失败: {git}");
        }
        println!("✓ 已从 Git 安装插件 {} → {}", item.id, dst.display());
        return Ok(());
    }
    if let Some(url) = item.download_url.as_deref().filter(|s| !s.is_empty()) {
        bail!(
            "插件 {} 需提供 git 或本地路径；download_url 解压安装尚未实现: {url}",
            item.id
        );
    }
    crate::plugin_ext::run_install(PluginInstallArgs {
        id: item.id.clone(),
        plugins_dir: plugins_dir.clone(),
        source: None,
    })
}

fn install_template_item(item: &MarketItem, out: &PathBuf) -> Result<()> {
    if let Some(url) = item.download_url.as_deref().filter(|s| !s.is_empty()) {
        if out.exists() {
            bail!("输出目录已存在: {}", out.display());
        }
        publish_cmd::init_from_template_url(url, out)?;
        println!("✓ 已从 URL 安装模板 → {}", out.display());
        return Ok(());
    }
    let tid = item
        .template_id
        .as_deref()
        .or_else(|| item.id.strip_prefix("template:"))
        .unwrap_or(item.id.as_str());
    if out.exists() {
        bail!("输出目录已存在: {}", out.display());
    }
    let exe = std::env::current_exe().context("current_exe")?;
    let st = Command::new(exe)
        .args([
            "init",
            "--non-interactive",
            "--quiet",
            "--template",
            tid,
            "-o",
            &out.to_string_lossy(),
            "--project-name",
            tid.replace('-', "_").as_str(),
        ])
        .status()?;
    if !st.success() {
        bail!("模板 init 失败");
    }
    println!("✓ 已用内置模板 `{tid}` 生成工程 → {}", out.display());
    Ok(())
}

fn install_role_pack_item(item: &MarketItem, out: &PathBuf) -> Result<()> {
    let url = item
        .download_url
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("角色包 {} 缺少 download_url", item.id))?;
    let roles = out.join("roles").join(&item.id);
    if roles.exists() {
        bail!("已存在: {}", roles.display());
    }
    std::fs::create_dir_all(roles.parent().unwrap_or(out))?;
    if url.ends_with(".tar.gz") || url.ends_with(".tgz") {
        let tmp = tempfile::tempdir()?;
        let archive = tmp.path().join("pack.tar.gz");
        let resp = ureq::get(url).call().context("download role pack")?;
        let mut reader = resp.into_reader();
        let mut file = std::fs::File::create(&archive)?;
        std::io::copy(&mut reader, &mut file)?;
        publish_cmd::extract_tar_gz(&archive, roles.parent().unwrap_or(out))?;
    } else {
        bail!("角色包仅支持 .tar.gz download_url");
    }
    println!("✓ 角色包 {} → {}", item.id, roles.display());
    Ok(())
}

fn run_browse(args: MarketBrowseArgs) -> Result<()> {
    if !crate::init_tui::terminal_supports_tui() {
        bail!("需要交互式终端；可设置 OCLIVE_NO_TUI=0 或改用 `oclive market search`");
    }
    let index = fetch_market_index()?;
    enable_raw_mode().context("raw")?;
    stdout().execute(EnterAlternateScreen).context("alt")?;
    let r = browse_loop(&index, &args);
    disable_raw_mode().ok();
    let _ = stdout().execute(LeaveAlternateScreen);
    r
}

fn browse_loop(index: &crate::market_index::MarketIndexFile, args: &MarketBrowseArgs) -> Result<()> {
    let categories = [
        MarketKind::Plugin,
        MarketKind::Template,
        MarketKind::RolePack,
    ];
    let mut cat_state = ListState::default().with_selected(Some(0));
    let mut item_state = ListState::default().with_selected(Some(0));
    let mut items: Vec<MarketItem> = index.items_for_kind(categories[0]);
    let mut message = String::new();
    let mut terminal = ratatui::init();

    loop {
        let cat_i = cat_state.selected().unwrap_or(0);
        let item_i = item_state.selected().unwrap_or(0);
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(28), Constraint::Percentage(72)])
                .split(f.area());
            let cat_items: Vec<ListItem> = categories
                .iter()
                .enumerate()
                .map(|(i, k)| {
                    let n = index.items_for_kind(*k).len();
                    let sel = cat_i == i;
                    ListItem::new(Line::from(format!(
                        "{} ({n})",
                        k.label()
                    )))
                    .style(if sel {
                        Style::default().add_modifier(Modifier::REVERSED)
                    } else {
                        Style::default()
                    })
                })
                .collect();
            f.render_stateful_widget(
                List::new(cat_items).block(
                    Block::default()
                        .title(" 分类 ")
                        .borders(Borders::ALL),
                ),
                chunks[0],
                &mut cat_state,
            );

            let right = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(chunks[1]);
            let list_lines: Vec<ListItem> = items
                .iter()
                .enumerate()
                .map(|(i, it)| {
                    ListItem::new(Line::from(format!("{} v{}", it.name, it.version))).style(
                        if i == item_i {
                            Style::default().add_modifier(Modifier::REVERSED)
                        } else {
                            Style::default()
                        },
                    )
                })
                .collect();
            f.render_stateful_widget(
                List::new(list_lines).block(
                    Block::default()
                        .title(" 条目 (↑↓ Enter 安装 Esc 退出) ")
                        .borders(Borders::ALL),
                ),
                right[0],
                &mut item_state,
            );
            let detail = if let Some(it) = items.get(item_i) {
                item_detail_text(it)
            } else {
                "（无条目）".into()
            };
            let foot = if message.is_empty() {
                String::new()
            } else {
                format!("\n\n{message}")
            };
            f.render_widget(
                Paragraph::new(format!("{detail}{foot}"))
                    .wrap(Wrap { trim: true })
                    .block(Block::default().title(" 详情 ").borders(Borders::ALL)),
                right[1],
            );
        })?;

        if event::poll(std::time::Duration::from_millis(120))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Up => {
                        let i = item_state.selected().unwrap_or(0);
                        item_state.select(Some(i.saturating_sub(1)));
                    }
                    KeyCode::Down => {
                        let i = item_state.selected().unwrap_or(0);
                        let max = items.len().saturating_sub(1);
                        item_state.select(Some((i + 1).min(max)));
                    }
                    KeyCode::Left => {
                        let i = cat_state.selected().unwrap_or(0);
                        cat_state.select(Some(i.saturating_sub(1)));
                        let k = categories[cat_state.selected().unwrap_or(0)];
                        items = index.items_for_kind(k);
                        item_state.select(Some(0));
                    }
                    KeyCode::Right => {
                        let i = cat_state.selected().unwrap_or(0);
                        let max = categories.len().saturating_sub(1);
                        cat_state.select(Some((i + 1).min(max)));
                        let k = categories[cat_state.selected().unwrap_or(0)];
                        items = index.items_for_kind(k);
                        item_state.select(Some(0));
                    }
                    KeyCode::Enter => {
                        if let Some(it) = items.get(item_state.selected().unwrap_or(0)) {
                            match install_item(it, &args.plugins_dir, &args.template_output) {
                                Ok(()) => message = format!("✓ 已安装 {}", it.id),
                                Err(e) => message = format!("❌ {e}"),
                            }
                        }
                    }
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }
    ratatui::restore();
    Ok(())
}

fn print_item_line(p: &MarketItem) {
    let kind = MarketKind::from(p.kind);
    println!(
        "[{}] {} v{} — {} — {}",
        kind.label(),
        p.id,
        p.version,
        p.author,
        p.description
    );
}

fn print_item_detail(p: &MarketItem) {
    let kind = MarketKind::from(p.kind);
    println!("类型: {}", kind.label());
    println!("ID: {}", p.id);
    println!("名称: {}", p.name);
    println!("版本: {}", p.version);
    println!("作者: {}", p.author);
    println!("描述: {}", p.description);
    if !p.tags.is_empty() {
        println!("标签: {}", p.tags.join(", "));
    }
    println!("安装量: {}", p.install_count);
    if let Some(u) = &p.download_url {
        println!("下载: {u}");
    }
    if let Some(g) = &p.git {
        println!("Git: {g}");
    }
    if let Some(t) = &p.template_id {
        println!("模板 ID: {t}");
    }
}

fn item_detail_text(p: &MarketItem) -> String {
    let kind = MarketKind::from(p.kind);
    format!(
        "类型: {}\nID: {}\n名称: {}\n版本: {}\n作者: {}\n\n{}\n\n标签: {}\n安装量: {}\n{}\n{}",
        kind.label(),
        p.id,
        p.name,
        p.version,
        p.author,
        p.description,
        if p.tags.is_empty() {
            "—".into()
        } else {
            p.tags.join(", ")
        },
        p.install_count,
        p.download_url
            .as_ref()
            .map(|u| format!("下载: {u}"))
            .unwrap_or_default(),
        p.git.as_ref().map(|g| format!("Git: {g}")).unwrap_or_default(),
    )
}
