# FECAPA Explorer

Aplicación TUI en Rust para explorar partidos de hockey patines de la FECAPA.

## Requisitos

- Rust (stable)
- Node.js (para el scraping)

## Instalación

```bash
# Compilar
cargo build --release

# O simplemente ejecuta
./target/release/fecapa-explorer
```

## Uso

Controles:
- **↑/↓** - Navegar por los partidos
- **Enter** - Ver detalles del partido
- **F** - Seleccionar filtro
- **R** - Refrescar (hace scraping de la web)
- **Q** - Salir

### Opciones

```bash
# Ejecutar con modo debug
./target/release/fecapa-explorer --debug
```

## Configuración

Los filtros se configuran en `equipos.json`:

```json
{
  "filtros": [
    {
      "nombre": "Nombre del filtro",
      "buscar": "texto a buscar",
      "categoria": "texto en la competición"
    }
  ]
}
```

## Estructura

```
fecapa-explorer/
├── src/main.rs        # Aplicación principal en Rust
├── scrape-hockey.js   # Script de scraping en Node.js
├── equipos.json       # Configuración de filtros
├── Cargo.toml        # Dependencias Rust
└── README.md        # Este archivo
```

## Licencia

MIT
