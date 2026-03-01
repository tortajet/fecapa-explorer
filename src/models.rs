use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filtro {
    pub nombre: String,
    pub buscar: String,
    pub categoria: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipoConfig {
    pub filtros: Vec<Filtro>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partido {
    #[serde(rename = "competicio")]
    pub competicion: String,
    pub data: String,
    pub hora: String,
    pub local: String,
    #[serde(rename = "visitant")]
    pub visitante: String,
    #[serde(rename = "resultat")]
    pub resultado: String,
    pub pista: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Vista {
    Partidos,
    Filtros,
    Detalles,
    Buscar,
    Confirm,
    Help,
}

impl Default for Vista {
    fn default() -> Self {
        Vista::Partidos
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmType {
    DeleteFilter,
}
