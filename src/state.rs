use crate::commands::{cargar_filtros, cargar_partidos};
use crate::models::{Filtro, Partido, Vista};

pub struct App {
    pub partidos: Vec<Partido>,
    pub todos_partidos: Vec<Partido>,
    pub filtros: Vec<Filtro>,
    pub filtro_seleccionado: usize,
    pub partido_seleccionado: usize,
    pub vista_actual: Vista,
    pub mensaje: String,
    pub scraping: bool,
    pub buscar_texto: String,
}

impl App {
    pub fn new() -> Self {
        let mut filtros = cargar_filtros();

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

    pub fn aplicar_filtro(&mut self) {
        let filtro = &self.filtros[self.filtro_seleccionado];

        if filtro.buscar.is_empty() {
            self.partidos = self.todos_partidos.clone();
            self.mensaje = format!("Mostrando todos los partidos: {}", self.partidos.len());
            return;
        }

        let buscar = filtro.buscar.to_uppercase();
        let categoria = filtro.categoria.to_uppercase();

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

    pub fn recargar_datos(&mut self) {
        let partidos = cargar_partidos();
        self.todos_partidos = partidos.clone();
        self.partidos = partidos;
        if !self.filtros.is_empty() {
            self.aplicar_filtro();
        }
    }

    pub fn aplicar_busqueda(&mut self) {
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

    pub fn reset_busqueda(&mut self) {
        self.buscar_texto.clear();
        self.partidos = self.todos_partidos.clone();
        self.mensaje = format!("{} partidos", self.partidos.len());
    }
}
