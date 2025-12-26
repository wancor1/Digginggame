use anyhow::{Context, Result};
use ratatui::crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::{fs, io, path::Path, time::Duration};
use tui_textarea::TextArea;

use digginggame::managers::persistence::SaveData;

const SAVE_DIR: &str = "saves";

enum AppMode {
    FileList,
    Editing,
    Error(String),
    Success(String),
}

struct App<'a> {
    mode: AppMode,
    files: Vec<String>,
    list_state: ListState,
    textarea: TextArea<'a>,
    current_filename: Option<String>,
}

impl App<'_> {
    fn new() -> Self {
        Self {
            mode: AppMode::FileList,
            files: Vec::new(),
            list_state: ListState::default(),
            textarea: TextArea::default(),
            current_filename: None,
        }
    }

    fn refresh_file_list(&mut self) {
        self.files.clear();
        if let Ok(entries) = fs::read_dir(SAVE_DIR) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = entry
                    .file_name()
                    .to_str()
                    .filter(|_| path.extension().is_some_and(|ext| ext == "dat"))
                {
                    self.files.push(name.to_string());
                }
            }
        }
        if self.list_state.selected().is_none() && !self.files.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    fn load_selected_file(&mut self) {
        if let Some(idx) = self.list_state.selected().filter(|&i| i < self.files.len()) {
            let filename = &self.files[idx];
            self.current_filename = Some(filename.clone());

            let path = Path::new(SAVE_DIR).join(filename);
            match Self::read_and_decode(&path) {
                Ok(json_content) => {
                    self.textarea = TextArea::from(json_content.lines());
                    self.textarea
                        .set_block(Block::default().borders(Borders::ALL).title(format!(
                            " Editing: {filename} (Ctrl+S to Save, Esc to Cancel) "
                        )));
                    self.textarea
                        .set_line_number_style(Style::default().fg(Color::DarkGray));
                    self.mode = AppMode::Editing;
                }
                Err(e) => {
                    self.mode = AppMode::Error(format!("Failed to load: {e}"));
                }
            }
        }
    }

    fn read_and_decode(path: &Path) -> Result<String> {
        let compressed = fs::read(path).context("Failed to read file")?;
        let decompressed = zstd::decode_all(&compressed[..]).context("Failed to decompress")?;
        // First deserialize to SaveData to validate structure (optional, but good practice)
        // Or directly to Value to allow flexible editing.
        // Let's decode to Value to preserve formatting as much as possible,
        // but decoding to SaveData ensures we are editing what we think we are.
        // However, user might want to see pretty JSON.
        let data: serde_json::Value =
            serde_json::from_slice(&decompressed).context("Invalid JSON")?;
        let pretty_json = serde_json::to_string_pretty(&data).context("Failed to format JSON")?;
        Ok(pretty_json)
    }

    fn save_current_file(&mut self) {
        if let Some(filename) = &self.current_filename {
            let content = self.textarea.lines().join("\n");
            let path = Path::new(SAVE_DIR).join(filename);

            // Validate JSON
            match serde_json::from_str::<SaveData>(&content) {
                Ok(data) => {
                    // Re-serialize to minified JSON for storage (or standard format)
                    match serde_json::to_vec(&data) {
                        Ok(json_bytes) => {
                            // Compress
                            match zstd::encode_all(&json_bytes[..], 0) {
                                Ok(compressed) => {
                                    // Write
                                    if let Err(e) = fs::write(&path, compressed) {
                                        self.mode =
                                            AppMode::Error(format!("Failed to write file: {e}"));
                                    } else {
                                        self.mode = AppMode::Success(
                                            "Saved successfully! Press Enter.".to_string(),
                                        );
                                    }
                                }
                                Err(e) => {
                                    self.mode = AppMode::Error(format!("Compression failed: {e}"));
                                }
                            }
                        }
                        Err(e) => {
                            self.mode = AppMode::Error(format!("Serialization failed: {e}"));
                        }
                    }
                }
                Err(e) => {
                    self.mode = AppMode::Error(format!("Invalid Save Data Structure: {e}"));
                }
            }
        }
    }

    fn next_file(&mut self) {
        if self.files.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.files.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous_file(&mut self) {
        if self.files.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.files.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    app.refresh_file_list();

    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            match app.mode {
                AppMode::FileList => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => app.next_file(),
                    KeyCode::Up => app.previous_file(),
                    KeyCode::Enter => app.load_selected_file(),
                    KeyCode::Char('r') => app.refresh_file_list(),
                    _ => {} // Ignore other keys
                },
                AppMode::Editing => {
                    // Check for Ctrl+S
                    if key.code == KeyCode::Char('s')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        app.save_current_file();
                    } else if key.code == KeyCode::Esc {
                        app.mode = AppMode::FileList;
                        app.current_filename = None;
                    } else {
                        // Forward to textarea
                        // Convert crossterm key to tui-textarea key
                        app.textarea.input(key);
                    }
                }
                AppMode::Error(_) | AppMode::Success(_) => {
                    if key.code == KeyCode::Enter || key.code == KeyCode::Esc {
                        app.mode = AppMode::Editing; // Return to editing
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(0)].as_ref())
        .split(f.area());

    match &app.mode {
        AppMode::FileList => {
            let items: Vec<ListItem> = app
                .files
                .iter()
                .map(|i| ListItem::new(i.as_str()))
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Select Save File (Enter to Edit, q to Quit) "),
                )
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Yellow),
                )
                .highlight_symbol(">>");

            f.render_stateful_widget(list, chunks[0], &mut app.list_state);
        }
        AppMode::Editing => {
            f.render_widget(&app.textarea, chunks[0]);
        }
        AppMode::Error(msg) => {
            let p = Paragraph::new(format!("Error: {msg}\n\nPress Enter to continue.")).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Error ")
                    .style(Style::default().fg(Color::Red)),
            );
            f.render_widget(p, chunks[0]);
        }
        AppMode::Success(msg) => {
            let p = Paragraph::new(format!("{msg}\n\nPress Enter to continue editing.")).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Success ")
                    .style(Style::default().fg(Color::Green)),
            );
            f.render_widget(p, chunks[0]);
        }
    }
}
