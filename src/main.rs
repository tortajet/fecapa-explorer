#![allow(non_ascii_idents)]

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    prelude::Stylize,
    style::{Color, Style},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table},
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};
use std::{fs, io, thread, time::Duration};

fn is_android() -> bool {
    std::env::consts::OS == "android"
        || std::env::var("ANDROID_ROOT").is_ok()
        || std::env::var("TERMUX_VERSION").is_ok()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Filtro {
    nombre: String,
    buscar: String,
    categoria: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EquipoConfig {
    filtros: Vec<Filtro>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Partido {
    #[serde(rename = "competicio")]
    competicion: String,
    data: String,
    hora: String,
    local: String,
    #[serde(rename = "visitant")]
    visitante: String,
    #[serde(rename = "resultat")]
    resultado: String,
    pista: String,
}

#[derive(Debug, Clone, PartialEq)]
enum Vista {
    Partidos,
    Filtros,
    Detalles,
    Buscar,
}

struct App {
    partidos: Vec<Partido>,
    todos_partidos: Vec<Partido>, // Keep original list
    filtros: Vec<Filtro>,
    filtro_seleccionado: usize,
    partido_seleccionado: usize,
    vista_actual: Vista,
    mensaje: String,
    scraping: bool,
    buscar_texto: String,
}

impl App {
    fn new() -> Self {
        let mut filtros = cargar_filtros();

        // Always have "Todos" as first filter if list is empty or doesn't have it
        if filtros.is_empty() || !filtros.iter().any(|f| f.nombre == "Todos") {
            filtros.insert(
                0,
                Filtro {
                    nombre: "Todos".to_string(),
                    buscar: "".to_string(),
                    categoria: "".to_string(),
                },
            );
        }

        let partidos = cargar_partidos();
        let num_partidos = partidos.len();

        Self {
            partidos: partidos.clone(),
            todos_partidos: partidos,
            filtros,
            filtro_seleccionado: 0,
            partido_seleccionado: 0,
            vista_actual: Vista::Partidos,
            mensaje: format!("{} partidos cargados", num_partidos),
            scraping: false,
            buscar_texto: String::new(),
        }
    }

    fn aplicar_filtro(&mut self) {
        let filtro = &self.filtros[self.filtro_seleccionado];

        if filtro.buscar.is_empty() {
            self.partidos = self.todos_partidos.clone();
            self.mensaje = format!("Mostrando todos los partidos: {}", self.partidos.len());
            return;
        }

        let buscar = filtro.buscar.to_uppercase();
        let categoria = filtro.categoria.to_uppercase();

        // Always filter from the complete list, not from filtered results
        self.partidos = self
            .todos_partidos
            .iter()
            .filter(|p| {
                let texto = format!("{} {} {} {}", p.competicion, p.local, p.visitante, p.pista)
                    .to_uppercase();
                let cumple_buscar = texto.contains(&buscar);
                let cumple_cat =
                    categoria.is_empty() || p.competicion.to_uppercase().contains(&categoria);
                cumple_buscar && cumple_cat
            })
            .cloned()
            .collect();

        self.mensaje = format!(
            "Filtro: {} - {} partidos",
            filtro.nombre,
            self.partidos.len()
        );
        self.partido_seleccionado = 0;
    }

    fn recargar_datos(&mut self) {
        let partidos = cargar_partidos();
        self.todos_partidos = partidos.clone();
        self.partidos = partidos;
        if !self.filtros.is_empty() {
            self.aplicar_filtro();
        }
    }

    fn aplicar_busqueda(&mut self) {
        if self.buscar_texto.is_empty() {
            self.partidos = self.todos_partidos.clone();
            self.mensaje = format!("Mostrando todos los partidos: {}", self.partidos.len());
            return;
        }

        let buscar = self.buscar_texto.to_uppercase();

        self.partidos = self
            .todos_partidos
            .iter()
            .filter(|p| {
                let texto = format!(
                    "{} {} {} {} {}",
                    p.competicion, p.local, p.visitante, p.pista, p.resultado
                )
                .to_uppercase();
                texto.contains(&buscar)
            })
            .cloned()
            .collect();

        self.mensaje = format!(
            "Buscando: \"{}\" - {} partidos",
            self.buscar_texto,
            self.partidos.len()
        );
        self.partido_seleccionado = 0;
    }
}

fn get_data_dir() -> std::path::PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();

    // Check both exe directory and current directory
    let current_dir = std::env::current_dir().unwrap_or_default();

    // Prefer exe directory if it has the files
    if exe_dir.join("partidos.json").exists() && exe_dir.join("equipos.json").exists() {
        return exe_dir;
    }

    // Otherwise use current directory
    current_dir
}

fn cargar_filtros() -> Vec<Filtro> {
    let data_dir = get_data_dir();
    let eq_path = data_dir.join("equipos.json");

    // Also try current directory
    let current_path = std::path::PathBuf::from("equipos.json");
    let final_path = if eq_path.exists() {
        eq_path.clone()
    } else if current_path.exists() {
        current_path
    } else {
        eq_path
    };

    if let Ok(data) = fs::read_to_string(&final_path) {
        if let Ok(config) = serde_json::from_str::<EquipoConfig>(&data) {
            return config.filtros;
        }
    }

    // Fallback default
    vec![Filtro {
        nombre: "Todos".to_string(),
        buscar: "".to_string(),
        categoria: "".to_string(),
    }]
}

fn cargar_partidos() -> Vec<Partido> {
    let data_dir = get_data_dir();
    let part_path = data_dir.join("partidos.json");

    // Also try current directory
    let current_path = std::path::PathBuf::from("partidos.json");
    let final_path = if part_path.exists() {
        part_path.clone()
    } else if current_path.exists() {
        current_path
    } else {
        part_path
    };

    if let Ok(data) = fs::read_to_string(&final_path) {
        if let Ok(partidos) = serde_json::from_str::<Vec<Partido>>(&data) {
            return partidos;
        }
    }
    vec![]
}

fn scrape_partidos() -> Result<Vec<Partido>, String> {
    // Get the directory where the binary is located
    let data_dir = get_data_dir();
    let scrape_script = data_dir.join("scrape-hockey.js");

    // Also try current directory
    let current_script = std::path::PathBuf::from("scrape-hockey.js");
    let script_path = if scrape_script.exists() {
        scrape_script
    } else if current_script.exists() {
        current_script
    } else {
        return Err("No se encontró scrape-hockey.js".to_string());
    };

    // Execute the Node.js script
    let output = std::process::Command::new("node")
        .arg(script_path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Error ejecutando node: {}", stderr));
    }

    // Now read the generated partidos.json
    let part_path = if data_dir.join("partidos.json").exists() {
        data_dir.join("partidos.json")
    } else {
        std::path::PathBuf::from("partidos.json")
    };

    let data =
        fs::read_to_string(&part_path).map_err(|e| format!("Error leyendo archivo: {}", e))?;

    let partidos: Vec<Partido> =
        serde_json::from_str(&data).map_err(|e| format!("Error parseando JSON: {}", e))?;

    Ok(partidos)
}

fn download_partidos() -> Result<Vec<Partido>, String> {
    let url = "https://raw.githubusercontent.com/tortajet/fecapa-explorer/main/partidos.json";

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(url).send().map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let data = response.text().map_err(|e| e.to_string())?;
    let partidos: Vec<Partido> = serde_json::from_str(&data).map_err(|e| e.to_string())?;

    Ok(partidos)
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    app.recargar_datos();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(f.area());

            let title = Paragraph::new("🏒 HOQUEI PATINS - COMPETICIÓN")
                .style(Style::default().fg(Color::Green).bold())
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(title, chunks[0]);

            match app.vista_actual {
                Vista::Partidos => {
                    let rows: Vec<Row> = app
                        .partidos
                        .iter()
                        .enumerate()
                        .map(|(i, p)| {
                            let style = if i == app.partido_seleccionado {
                                Style::default().bg(Color::Blue).fg(Color::White)
                            } else if p.resultado.is_empty() {
                                Style::default().fg(Color::Green)
                            } else {
                                Style::default()
                            };

                            Row::new(vec![
                                Cell::from(truncate(&p.competicion, 25)),
                                Cell::from(truncate(&p.data, 10)),
                                Cell::from(truncate(&p.hora, 6)),
                                Cell::from(truncate(&p.local, 20)),
                                Cell::from(truncate(&p.visitante, 20)),
                                Cell::from(if p.resultado.is_empty() {
                                    "-".to_string()
                                } else {
                                    p.resultado.clone()
                                }),
                                Cell::from(truncate(&p.pista, 25)),
                            ])
                            .style(style)
                        })
                        .collect();

                    let table = Table::new(
                        rows,
                        [
                            Constraint::Min(25),
                            Constraint::Length(10),
                            Constraint::Length(6),
                            Constraint::Min(20),
                            Constraint::Min(20),
                            Constraint::Length(8),
                            Constraint::Min(25),
                        ],
                    )
                    .header(
                        Row::new(vec![
                            "COMPETICIÓN",
                            "FECHA",
                            "HORA",
                            "LOCAL",
                            "VISITANTE",
                            "RES",
                            "PISTA",
                        ])
                        .style(Style::default().fg(Color::Yellow).bold()),
                    )
                    .block(
                        Block::bordered()
                            .title(" Partidos ")
                            .border_style(Style::default().fg(Color::Cyan))
                            .borders(Borders::ALL),
                    )
                    .highlight_symbol(">> ");

                    f.render_widget(table, chunks[1]);
                }
                Vista::Filtros => {
                    let items: Vec<ListItem> = app
                        .filtros
                        .iter()
                        .enumerate()
                        .map(|(i, f)| {
                            let style = if i == app.filtro_seleccionado {
                                Style::default().bg(Color::Blue).fg(Color::White)
                            } else {
                                Style::default()
                            };
                            ListItem::new(f.nombre.clone()).style(style)
                        })
                        .collect();

                    let list = List::new(items)
                        .block(
                            Block::bordered()
                                .title(" Filtros ")
                                .border_style(Style::default().fg(Color::Cyan))
                                .borders(Borders::ALL),
                        )
                        .highlight_symbol(">> ");

                    let area = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage(30),
                            Constraint::Min(10),
                            Constraint::Percentage(30),
                        ])
                        .split(chunks[1])[1];
                    f.render_widget(list, area);
                }
                Vista::Detalles => {
                    if let Some(p) = app.partidos.get(app.partido_seleccionado) {
                        let text = vec![
                            ratatui::text::Line::from(vec![ratatui::text::Span::raw("")]),
                            ratatui::text::Line::from(vec![ratatui::text::Span::styled(
                                "Partido:",
                                Style::default().bold().fg(Color::Cyan),
                            )]),
                            ratatui::text::Line::from(vec![
                                ratatui::text::Span::raw(&p.local),
                                ratatui::text::Span::raw(" vs "),
                                ratatui::text::Span::raw(&p.visitante),
                            ]),
                            ratatui::text::Line::from(vec![ratatui::text::Span::raw("")]),
                            ratatui::text::Line::from(vec![
                                ratatui::text::Span::styled(
                                    "Competición:",
                                    Style::default().bold(),
                                ),
                                ratatui::text::Span::raw(&p.competicion),
                            ]),
                            ratatui::text::Line::from(vec![
                                ratatui::text::Span::styled("Fecha:", Style::default().bold()),
                                ratatui::text::Span::raw(&p.data),
                            ]),
                            ratatui::text::Line::from(vec![
                                ratatui::text::Span::styled("Hora:", Style::default().bold()),
                                ratatui::text::Span::raw(&p.hora),
                            ]),
                            ratatui::text::Line::from(vec![
                                ratatui::text::Span::styled("Resultado:", Style::default().bold()),
                                ratatui::text::Span::raw(&p.resultado),
                            ]),
                            ratatui::text::Line::from(vec![
                                ratatui::text::Span::styled("Pista:", Style::default().bold()),
                                ratatui::text::Span::raw(&p.pista),
                            ]),
                        ];
                        let paragraph = Paragraph::new(text).block(
                            Block::bordered()
                                .title(" Detalles ")
                                .border_style(Style::default().fg(Color::Cyan))
                                .borders(Borders::ALL),
                        );
                        f.render_widget(paragraph, chunks[1]);
                    }
                }
                Vista::Buscar => {
                    let search_prompt = Paragraph::new(format!("/{}", app.buscar_texto))
                        .style(Style::default().fg(Color::Yellow))
                        .block(
                            Block::bordered()
                                .title(" BUSCAR (contiene) ")
                                .border_style(Style::default().fg(Color::Green))
                                .borders(Borders::ALL),
                        );
                    f.render_widget(search_prompt, chunks[1]);
                }
            }

            let status_text = if app.scraping {
                if is_android() {
                    "⏳ Descargando partidos de GitHub...".to_string()
                } else {
                    "⏳ Extrayendo partidos de la web...".to_string()
                }
            } else {
                format!(
                    "{} | Filtro: {} | ↑↓ Navegar | Enter Ver | F Filtros | / Buscar | R Refrescar | Q Salir",
                    app.mensaje,
                    app.filtros
                        .get(app.filtro_seleccionado)
                        .map(|f| f.nombre.as_str())
                        .unwrap_or("Todos")
                )
            };
            let status = Paragraph::new(status_text)
                .style(Style::default().fg(Color::White).bg(Color::Blue))
                .alignment(ratatui::layout::Alignment::Left);
            f.render_widget(status, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.vista_actual {
                        Vista::Partidos => match key.code {
                            crossterm::event::KeyCode::Char('q') => break,
                            crossterm::event::KeyCode::Char('r')
                            | crossterm::event::KeyCode::Char('R') => {
                                if !app.scraping {
                                    app.scraping = true;
                                    let exe_dir = std::env::current_exe()
                                        .ok()
                                        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                                        .unwrap_or_default();

                                    let save_path = if exe_dir.join("partidos.json").exists() {
                                        exe_dir.join("partidos.json")
                                    } else {
                                        std::path::PathBuf::from("partidos.json")
                                    };

                                    if is_android() {
                                        thread::spawn(move || {
                                            if let Ok(partidos) = download_partidos() {
                                                let _ = fs::write(
                                                    &save_path,
                                                    serde_json::to_string_pretty(&partidos)
                                                        .unwrap_or_default(),
                                                );
                                            }
                                        });
                                    } else {
                                        thread::spawn(move || {
                                            if let Ok(partidos) = scrape_partidos() {
                                                let _ = fs::write(
                                                    &save_path,
                                                    serde_json::to_string_pretty(&partidos)
                                                        .unwrap_or_default(),
                                                );
                                            }
                                        });
                                    }
                                }
                            }
                            crossterm::event::KeyCode::Char('f')
                            | crossterm::event::KeyCode::Char('F') => {
                                app.vista_actual = Vista::Filtros;
                            }
                            crossterm::event::KeyCode::Char('/') => {
                                app.buscar_texto.clear();
                                app.vista_actual = Vista::Buscar;
                            }
                            crossterm::event::KeyCode::Char('e')
                            | crossterm::event::KeyCode::Enter => {
                                app.vista_actual = Vista::Detalles;
                            }
                            crossterm::event::KeyCode::Up => {
                                if app.partido_seleccionado > 0 {
                                    app.partido_seleccionado -= 1;
                                }
                            }
                            crossterm::event::KeyCode::Down => {
                                if app.partido_seleccionado < app.partidos.len().saturating_sub(1) {
                                    app.partido_seleccionado += 1;
                                }
                            }
                            _ => {}
                        },
                        Vista::Filtros => match key.code {
                            crossterm::event::KeyCode::Esc => {
                                app.vista_actual = Vista::Partidos;
                            }
                            crossterm::event::KeyCode::Enter => {
                                app.aplicar_filtro();
                                app.vista_actual = Vista::Partidos;
                            }
                            crossterm::event::KeyCode::Up => {
                                if app.filtro_seleccionado > 0 {
                                    app.filtro_seleccionado -= 1;
                                }
                            }
                            crossterm::event::KeyCode::Down => {
                                if app.filtro_seleccionado < app.filtros.len().saturating_sub(1) {
                                    app.filtro_seleccionado += 1;
                                }
                            }
                            _ => {}
                        },
                        Vista::Detalles => match key.code {
                            crossterm::event::KeyCode::Esc => {
                                app.vista_actual = Vista::Partidos;
                            }
                            _ => {}
                        },
                        Vista::Buscar => match key.code {
                            crossterm::event::KeyCode::Esc => {
                                app.buscar_texto.clear();
                                app.partidos = app.todos_partidos.clone();
                                app.mensaje = format!("{} partidos", app.partidos.len());
                                app.vista_actual = Vista::Partidos;
                            }
                            crossterm::event::KeyCode::Enter => {
                                app.vista_actual = Vista::Partidos;
                            }
                            crossterm::event::KeyCode::Backspace => {
                                app.buscar_texto.pop();
                                app.aplicar_busqueda();
                            }
                            crossterm::event::KeyCode::Char(c) => {
                                app.buscar_texto.push(c);
                                app.aplicar_busqueda();
                            }
                            _ => {}
                        },
                    }
                }
            }

            if app.scraping {
                // Wait longer for scraping to complete - poll every 500ms for up to 30 seconds
                let mut waited = 0;
                let max_wait = 30000;
                let mut found = false;

                while waited < max_wait && !found {
                    thread::sleep(Duration::from_millis(500));
                    waited += 500;

                    let data_dir = get_data_dir();
                    let current_path = std::path::PathBuf::from("partidos.json");
                    let part_path = if current_path.exists() {
                        current_path.clone()
                    } else {
                        data_dir.join("partidos.json")
                    };

                    if let Ok(data) = fs::read_to_string(&part_path) {
                        if let Ok(partidos) = serde_json::from_str::<Vec<Partido>>(&data) {
                            if !partidos.is_empty() {
                                app.todos_partidos = partidos.clone();
                                app.partidos = partidos;
                                app.aplicar_filtro();
                                app.scraping = false;
                                app.mensaje =
                                    format!("✅ {} partidos guardados", app.todos_partidos.len());
                                found = true;
                            }
                        }
                    }
                }

                if !found && waited >= max_wait {
                    app.scraping = false;
                    app.mensaje = "❌ Timeout esperando datos".to_string();
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        // Take characters, not bytes, to avoid invalid UTF-8 boundaries
        let chars: String = s.chars().take(max_len - 3).collect();
        format!("{}...", chars)
    } else {
        s.to_string()
    }
}
