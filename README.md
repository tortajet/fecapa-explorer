# FECAPA Explorer

Aplicación TUI en Rust para explorar partidos de hockey patines de la FECAPA.

## Requisitos

- Rust (stable)
- Node.js (para el scraping)

## Instalación

```bash
# Instalar dependencias Node.js
npm install

# Compilar aplicación Rust
cargo build --release
```

## Uso

```bash
# Ejecutar aplicación
./target/release/fecapa-explorer
```

### Opciones

```bash
# Ejecutar con modo debug
./target/release/fecapa-explorer --debug
```

## Controles

- **↑/↓** - Navegar por los partidos
- **Enter** - Ver detalles del partido
- **F** - Seleccionar filtro
- **R** - Refrescar (hace scraping de la web)
- **Q** - Salir

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

## Android

El scraping usa Playwright que no está soportado en Android. Para usar en Android:

1. **Opción 1**: Ejecutar el scraping en un PC/servidor y copiar `partidos.json` al móvil
2. **Opción 2**: Usar la aplicación solo para ver datos (sin hacer scraping)

### En Termux:

```bash
# Instalar Rust
pkg install rust

# Instalar Node.js
pkg install nodejs

# Clonar y entrar en el proyecto
git clone https://github.com/tortajet/fecapa-explorer
cd fecapa-explorer

# Instalar dependencias
npm install

# Compilar
cargo build --release

# Ejecutar (solo para ver datos, el scraping no funcionará)
./target/release/fecapa-explorer
```

Para obtener datos actualizados en Android, copia el archivo `partidos.json` desde un PC.

## Estructura

```
fecapa-explorer/
├── src/main.rs        # Aplicación principal en Rust
├── scrape-hockey.js  # Script de scraping en Node.js
├── equipos.json      # Configuración de filtros
├── Cargo.toml        # Dependencias Rust
└── README.md        # Este archivo
```

## Licencia

MIT
