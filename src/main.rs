#![allow(non_ascii_idents)]

mod commands;
mod models;
mod state;
mod ui;

use commands::{download_partidos, find_file_path, is_android, scrape_partidos};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use models::Vista;
use ratatui::{backend::CrosstermBackend, layout::Layout, Terminal};
use state::App;
use std::{fs, io, thread, time::Duration};

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
                .constraints([
                    ratatui::layout::Constraint::Length(3),
                    ratatui::layout::Constraint::Min(0),
                    ratatui::layout::Constraint::Length(3),
                ])
                .split(f.area());

            ui::render_title(f, chunks[0]);

            match app.vista_actual {
                Vista::Partidos => {
                    ui::render_partidos_table(f, chunks[1], &app);
                }
                Vista::Filtros => {
                    ui::render_filtros_list(f, chunks[1], &app);
                }
                Vista::Detalles => {
                    ui::render_detalles(f, chunks[1], &app);
                }
                Vista::Buscar => {
                    ui::render_buscar(f, chunks[1], &app);
                }
                Vista::Confirm => {
                    ui::render_confirm(f, chunks[1], &app);
                }
                Vista::Help => {
                    ui::render_help(f, chunks[1], &app);
                }
            }

            ui::render_status(f, chunks[2], &app);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.vista_actual {
                        Vista::Partidos => match key.code {
                            crossterm::event::KeyCode::Char('q') => break,
                            crossterm::event::KeyCode::Char('?') => {
                                app.vista_actual = Vista::Help;
                            }
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
                            crossterm::event::KeyCode::PageDown => {
                                let block_size = 20;
                                let max = app.partidos.len().saturating_sub(1);
                                app.partido_seleccionado =
                                    (app.partido_seleccionado + block_size).min(max);
                            }
                            crossterm::event::KeyCode::PageUp => {
                                let block_size = 20;
                                app.partido_seleccionado =
                                    app.partido_seleccionado.saturating_sub(block_size);
                            }
                            _ => {}
                        },
                        Vista::Filtros => match key.code {
                            crossterm::event::KeyCode::Esc => {
                                app.vista_actual = Vista::Partidos;
                            }
                            crossterm::event::KeyCode::Char('?') => {
                                app.vista_actual = Vista::Help;
                            }
                            crossterm::event::KeyCode::Enter => {
                                app.aplicar_filtro();
                                app.vista_actual = Vista::Partidos;
                            }
                            crossterm::event::KeyCode::Char('d')
                            | crossterm::event::KeyCode::Char('D') => {
                                if app.filtro_seleccionado > 0 {
                                    app.confirm_type = Some(models::ConfirmType::DeleteFilter);
                                    app.confirm_seleccion = 1;
                                    app.vista_actual = Vista::Confirm;
                                }
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
                                app.detalle_seleccion = 0;
                            }
                            crossterm::event::KeyCode::Char('?') => {
                                app.vista_actual = Vista::Help;
                            }
                            crossterm::event::KeyCode::Up => {
                                if app.detalle_seleccion > 0 {
                                    app.detalle_seleccion -= 1;
                                }
                            }
                            crossterm::event::KeyCode::Down => {
                                if app.detalle_seleccion < 5 {
                                    app.detalle_seleccion += 1;
                                }
                            }
                            crossterm::event::KeyCode::Char('a')
                            | crossterm::event::KeyCode::Char('A') => {
                                let (nombre, mensaje) = if let Some(p) =
                                    app.partidos.get(app.partido_seleccionado)
                                {
                                    (p.local.clone(), format!("✅ Filtro '{}' añadido", p.local))
                                } else {
                                    (String::new(), String::new())
                                };
                                if !nombre.is_empty() {
                                    app.agregar_filtro(nombre.clone(), nombre, String::new());
                                    app.mensaje = mensaje;
                                    app.vista_actual = Vista::Partidos;
                                }
                            }
                            crossterm::event::KeyCode::Char('c')
                            | crossterm::event::KeyCode::Char('C') => {
                                let (nombre, mensaje) =
                                    if let Some(p) = app.partidos.get(app.partido_seleccionado) {
                                        (
                                            p.competicion.clone(),
                                            format!("✅ Filtro '{}' añadido", p.competicion),
                                        )
                                    } else {
                                        (String::new(), String::new())
                                    };
                                if !nombre.is_empty() {
                                    app.agregar_filtro(nombre.clone(), String::new(), nombre);
                                    app.mensaje = mensaje;
                                    app.vista_actual = Vista::Partidos;
                                }
                            }
                            _ => {}
                        },
                        Vista::Buscar => match key.code {
                            crossterm::event::KeyCode::Esc => {
                                app.reset_busqueda();
                                app.vista_actual = Vista::Partidos;
                            }
                            crossterm::event::KeyCode::Char('?') => {
                                app.vista_actual = Vista::Help;
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
                        Vista::Confirm => match key.code {
                            crossterm::event::KeyCode::Esc => {
                                app.confirm_type = None;
                                app.vista_actual = Vista::Filtros;
                            }
                            crossterm::event::KeyCode::Char('?') => {
                                app.vista_actual = Vista::Help;
                            }
                            crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Down => {
                                if app.confirm_seleccion == 0 {
                                    app.confirm_seleccion = 1;
                                } else {
                                    app.confirm_seleccion = 0;
                                }
                            }
                            crossterm::event::KeyCode::Enter => {
                                if app.confirm_seleccion == 0 {
                                    match app.confirm_type {
                                        Some(models::ConfirmType::DeleteFilter) => {
                                            app.eliminar_filtro();
                                        }
                                        Some(models::ConfirmType::AddFilter) => {
                                            if let Some(p) =
                                                app.partidos.get(app.partido_seleccionado)
                                            {
                                                let nombre = p.local.clone();
                                                let buscar = p.local.clone();
                                                let categoria = p.competicion.clone();
                                                app.agregar_filtro(nombre, buscar, categoria);
                                                app.mensaje = "✅ Filtro añadido".to_string();
                                            }
                                        }
                                        None => {}
                                    }
                                }
                                app.confirm_type = None;
                                app.vista_actual = Vista::Partidos;
                            }
                            _ => {}
                        },
                        Vista::Help => match key.code {
                            crossterm::event::KeyCode::Esc => {
                                app.vista_actual = Vista::Partidos;
                            }
                            _ => {}
                        },
                    }
                }
            }

            if app.scraping {
                let mut waited = 0;
                let max_wait = 30000;
                let mut found = false;

                while waited < max_wait && !found {
                    thread::sleep(Duration::from_millis(500));
                    waited += 500;

                    let part_path = find_file_path("partidos.json");

                    if let Ok(data) = fs::read_to_string(&part_path) {
                        if let Ok(partidos) = serde_json::from_str::<Vec<models::Partido>>(&data) {
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
