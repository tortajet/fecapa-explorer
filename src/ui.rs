use crate::state::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Stylize,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table},
    Frame,
};

pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        let chars: String = s.chars().take(max_len - 3).collect();
        format!("{}...", chars)
    } else {
        s.to_string()
    }
}

pub fn render_title(f: &mut Frame, area: Rect) {
    let title = Paragraph::new("🏒 HOQUEI PATINS - COMPETICIÓN")
        .style(Style::default().fg(Color::Green).bold())
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(title, area);
}

pub fn render_partidos_table(f: &mut Frame, area: Rect, app: &App) {
    let table_height = (area.height as usize).saturating_sub(2);
    let offset = app.partido_seleccionado.saturating_sub(table_height / 2);
    let width = area.width as usize;

    let (_, constraints, header): (Vec<&str>, Vec<Constraint>, Vec<&str>) = if width < 90 {
        (
            vec!["COMP", "FECHA", "HORA", "LOCAL", "VIS", "RES"],
            vec![
                Constraint::Min(15),
                Constraint::Length(10),
                Constraint::Length(6),
                Constraint::Min(12),
                Constraint::Min(12),
                Constraint::Length(8),
            ],
            vec!["COMP", "FECHA", "HORA", "LOCAL", "VIS", "RES"],
        )
    } else if width < 120 {
        (
            vec!["COMPETICIÓN", "FECHA", "HORA", "LOCAL", "VISITANTE", "RES"],
            vec![
                Constraint::Min(20),
                Constraint::Length(10),
                Constraint::Length(6),
                Constraint::Min(15),
                Constraint::Min(15),
                Constraint::Length(8),
            ],
            vec!["COMPETICIÓN", "FECHA", "HORA", "LOCAL", "VISITANTE", "RES"],
        )
    } else {
        (
            vec![
                "COMPETICIÓN",
                "FECHA",
                "HORA",
                "LOCAL",
                "VISITANTE",
                "RES",
                "PISTA",
            ],
            vec![
                Constraint::Min(25),
                Constraint::Length(10),
                Constraint::Length(6),
                Constraint::Min(20),
                Constraint::Min(20),
                Constraint::Length(8),
                Constraint::Min(25),
            ],
            vec![
                "COMPETICIÓN",
                "FECHA",
                "HORA",
                "LOCAL",
                "VISITANTE",
                "RES",
                "PISTA",
            ],
        )
    };

    let max_lens = if width < 90 {
        (15, 10, 5, 10, 10)
    } else if width < 120 {
        (20, 10, 5, 13, 13)
    } else {
        (25, 10, 6, 20, 20)
    };

    let rows: Vec<Row> = app
        .partidos
        .iter()
        .skip(offset)
        .take(table_height)
        .enumerate()
        .map(|(i, p)| {
            let real_index = offset + i;
            let style = if real_index == app.partido_seleccionado {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else if p.resultado.is_empty() {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };

            let cells = if width < 90 {
                vec![
                    Cell::from(truncate(&p.competicion, max_lens.0)),
                    Cell::from(truncate(&p.data, max_lens.1)),
                    Cell::from(truncate(&p.hora, max_lens.2)),
                    Cell::from(truncate(&p.local, max_lens.3)),
                    Cell::from(truncate(&p.visitante, max_lens.4)),
                    Cell::from(if p.resultado.is_empty() {
                        "-".to_string()
                    } else {
                        p.resultado.clone()
                    }),
                ]
            } else if width < 120 {
                vec![
                    Cell::from(truncate(&p.competicion, max_lens.0)),
                    Cell::from(truncate(&p.data, max_lens.1)),
                    Cell::from(truncate(&p.hora, max_lens.2)),
                    Cell::from(truncate(&p.local, max_lens.3)),
                    Cell::from(truncate(&p.visitante, max_lens.4)),
                    Cell::from(if p.resultado.is_empty() {
                        "-".to_string()
                    } else {
                        p.resultado.clone()
                    }),
                ]
            } else {
                vec![
                    Cell::from(truncate(&p.competicion, max_lens.0)),
                    Cell::from(truncate(&p.data, max_lens.1)),
                    Cell::from(truncate(&p.hora, max_lens.2)),
                    Cell::from(truncate(&p.local, max_lens.3)),
                    Cell::from(truncate(&p.visitante, max_lens.4)),
                    Cell::from(if p.resultado.is_empty() {
                        "-".to_string()
                    } else {
                        p.resultado.clone()
                    }),
                    Cell::from(truncate(&p.pista, 25)),
                ]
            };

            Row::new(cells).style(style)
        })
        .collect();

    let table = Table::new(rows, constraints)
        .header(Row::new(header).style(Style::default().fg(Color::Yellow).bold()))
        .block(
            Block::bordered()
                .title(" Partidos ")
                .border_style(Style::default().fg(Color::Cyan))
                .borders(Borders::ALL),
        )
        .highlight_symbol(">> ");

    f.render_widget(table, area);
}

pub fn render_filtros_list(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .filtros
        .iter()
        .enumerate()
        .map(|(i, filter)| {
            let style = if i == app.filtro_seleccionado {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(filter.nombre.clone()).style(style)
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

    let render_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Min(10),
            Constraint::Percentage(30),
        ])
        .split(area)[1];

    f.render_widget(list, render_area);
}

pub fn render_detalles(f: &mut Frame, area: Rect, app: &App) {
    if let Some(p) = app.partidos.get(app.partido_seleccionado) {
        let fields = vec![
            ("Partido", format!("{} vs {}", p.local, p.visitante)),
            ("Competición", p.competicion.clone()),
            ("Fecha", p.data.clone()),
            ("Hora", p.hora.clone()),
            (
                "Resultado",
                if p.resultado.is_empty() {
                    "-".to_string()
                } else {
                    p.resultado.clone()
                },
            ),
            ("Pista", p.pista.clone()),
        ];

        let text: Vec<Line> = fields
            .iter()
            .enumerate()
            .map(|(i, (label, value))| {
                let is_selected = i == app.detalle_seleccion;
                let style = if is_selected {
                    Style::default()
                        .bold()
                        .fg(Color::Yellow)
                        .bg(Color::DarkGray)
                } else {
                    Style::default().bold()
                };
                let value_style = if is_selected {
                    Style::default().fg(Color::White).bg(Color::DarkGray)
                } else {
                    Style::default()
                };
                Line::from(vec![
                    Span::styled(format!("{}: ", label), style),
                    Span::styled(value, value_style),
                ])
            })
            .collect();

        let paragraph = Paragraph::new(text)
            .block(
                Block::bordered()
                    .title(" Detalles - ↑↓ Navegar | A Añadir filtro | D Añadir competicion | Esc Volver ")
                    .border_style(Style::default().fg(Color::Cyan))
                    .borders(Borders::ALL),
            )
            .wrap(ratatui::widgets::Wrap { trim: false });
        f.render_widget(paragraph, area);
    }
}

pub fn render_buscar(f: &mut Frame, area: Rect, app: &App) {
    let search_prompt = Paragraph::new(format!("/{}", app.buscar_texto))
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::bordered()
                .title(" BUSCAR (contiene) ")
                .border_style(Style::default().fg(Color::Green))
                .borders(Borders::ALL),
        );
    f.render_widget(search_prompt, area);
}

pub fn render_confirm(f: &mut Frame, area: Rect, app: &App) {
    let pregunta = match app.confirm_type {
        Some(crate::models::ConfirmType::DeleteFilter) => "Eliminar este filtro?",
        Some(crate::models::ConfirmType::AddFilter) => "Añadir este filtro?",
        None => "",
    };

    let opciones = vec!["Sí", "No"];
    let items: Vec<ListItem> = opciones
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let style = if i == app.confirm_seleccion {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };
            ListItem::new(*opt).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(pregunta)
                .border_style(Style::default().fg(Color::Yellow))
                .borders(Borders::ALL),
        )
        .highlight_symbol(">> ");

    let centered_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(5),
            Constraint::Percentage(40),
        ])
        .split(area)[1];

    let horizontal_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(20),
            Constraint::Percentage(30),
        ])
        .split(centered_area)[1];

    f.render_widget(list, horizontal_area);
}

pub fn render_status(f: &mut Frame, area: Rect, app: &App) {
    use crate::commands::is_android;

    let status_text = if app.scraping {
        if is_android() {
            "⏳ Descargando partidos de GitHub...".to_string()
        } else {
            "⏳ Extrayendo partidos de la web...".to_string()
        }
    } else {
        format!(
            "{} | Filtro: {} | ↑↓/Av/Re Pag Navegar | Enter Ver | F Filtros | / Buscar | R Refrescar | Q Salir",
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
    f.render_widget(status, area);
}
