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
        let text = vec![
            Line::from(vec![Span::raw("")]),
            Line::from(vec![Span::styled(
                "Partido:",
                Style::default().bold().fg(Color::Cyan),
            )]),
            Line::from(vec![
                Span::raw(&p.local),
                Span::raw(" vs "),
                Span::raw(&p.visitante),
            ]),
            Line::from(vec![Span::raw("")]),
            Line::from(vec![
                Span::styled("Competición:", Style::default().bold()),
                Span::raw(&p.competicion),
            ]),
            Line::from(vec![
                Span::styled("Fecha:", Style::default().bold()),
                Span::raw(&p.data),
            ]),
            Line::from(vec![
                Span::styled("Hora:", Style::default().bold()),
                Span::raw(&p.hora),
            ]),
            Line::from(vec![
                Span::styled("Resultado:", Style::default().bold()),
                Span::raw(&p.resultado),
            ]),
            Line::from(vec![
                Span::styled("Pista:", Style::default().bold()),
                Span::raw(&p.pista),
            ]),
        ];
        let paragraph = Paragraph::new(text).block(
            Block::bordered()
                .title(" Detalles ")
                .border_style(Style::default().fg(Color::Cyan))
                .borders(Borders::ALL),
        );
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
