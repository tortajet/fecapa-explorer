const blessed = require('blessed');
const fs = require('fs');
const { chromium } = require('playwright');

const EQUIPOS_FILE = 'equipos.json';
const PARTIDOS_FILE = 'partidos.json';

let partidos = [];
let filtros = [];
let filtroActual = null;
let screen, table, statusBox, filterList;

async function cargarPartidos() {
  if (fs.existsSync(PARTIDOS_FILE)) {
    const data = fs.readFileSync(PARTIDOS_FILE, 'utf-8');
    return JSON.parse(data);
  }
  return [];
}

async function cargarFiltros() {
  if (fs.existsSync(EQUIPOS_FILE)) {
    const data = fs.readFileSync(EQUIPOS_FILE, 'utf-8');
    return JSON.parse(data).filtros;
  }
  return [];
}

async function scrapePartidos() {
  statusBox.setContent('⏳ Extrayendo partidos de la web...');
  screen.render();

  const browser = await chromium.launch();
  const page = await browser.newPage();
  
  await page.goto('https://www.hoqueipatins.fecapa.cat/ag/');
  await page.waitForTimeout(3000);
  
  const matches = await page.evaluate(() => {
    const data = [];
    const rows = document.querySelectorAll('table tr');
    
    rows.forEach((row) => {
      const cells = row.querySelectorAll('td');
      if (cells.length >= 7) {
        const competicio = cells[0]?.textContent?.trim() || '';
        const dataPartit = cells[1]?.textContent?.trim() || '';
        const hora = cells[2]?.textContent?.trim() || '';
        const local = cells[4]?.textContent?.trim() || '';
        const visitant = cells[6]?.textContent?.trim() || '';
        const resultat = cells[7]?.textContent?.trim() || '';
        const pista = cells[8]?.textContent?.trim() || '';
        
        if (competicio || local || visitant) {
          data.push({
            competicio,
            data: dataPartit,
            hora,
            local,
            visitant,
            resultat,
            pista
          });
        }
      }
    });
    
    return data;
  });
  
  await browser.close();
  
  fs.writeFileSync(PARTIDOS_FILE, JSON.stringify(matches, null, 2));
  statusBox.setContent(`✅ ${matches.length} partidos guardados en ${PARTIDOS_FILE}`);
  screen.render();
  
  return matches;
}

function aplicarFiltro(filtro) {
  filtroActual = filtro;
  
  let partidosFiltrados = partidos;
  
  if (filtro && filtro.buscar) {
    partidosFiltrados = partidos.filter(p => {
      const texto = `${p.competicio} ${p.local} ${p.visitant} ${p.pista}`.toUpperCase();
      const cumpleBusqueda = texto.includes(filtro.buscar.toUpperCase());
      const cumpleCategoria = !filtro.categoria || p.competicio.toUpperCase().includes(filtro.categoria.toUpperCase());
      return cumpleBusqueda && cumpleCategoria;
    });
  }
  
  actualizarTabla(partidosFiltrados);
  
  if (filterList) {
    filterList.hide();
    filterList = null;
  }
  screen.focusPop();
}

function formatearTabla(partidosMostrar) {
  const cols = ['Competició', 'Data', 'Hora', 'Local', 'Visitante', 'Resultat', 'Pista'];
  const colWidths = [30, 10, 6, 25, 25, 8, 35];
  
  let header = '';
  let separator = '';
  let pos = 0;
  
  for (let i = 0; i < cols.length; i++) {
    header += cols[i].padEnd(colWidths[i]).substring(0, colWidths[i]);
    separator += '-'.repeat(colWidths[i]);
    if (i < cols.length - 1) {
      header += ' │ ';
      separator += '─┼─';
    }
  }
  
  const rows = partidosMostrar.slice(0, 100).map(p => {
    let row = '';
    const values = [
      p.competicio.substring(0, 30),
      p.data,
      p.hora,
      p.local.substring(0, 25),
      p.visitant.substring(0, 25),
      p.resultat || '-',
      p.pista.substring(0, 35)
    ];
    
    for (let i = 0; i < values.length; i++) {
      row += values[i].padEnd(colWidths[i]);
      if (i < values.length - 1) {
        row += ' │ ';
      }
    }
    return row;
  });
  
  return { header, separator, rows };
}

function actualizarTabla(partidosMostrar) {
  const { header, separator, rows } = formatearTabla(partidosMostrar);
  
  const contenido = [
    header,
    separator,
    ...rows
  ].join('\n');
  
  table.setContent(contenido);
  
  const filtroNombre = filtroActual ? filtroActual.nombre : 'Todos los partidos';
  statusBox.setContent(`Mostrando ${partidosMostrar.length} partidos | Filtro: ${filtroNombre} | (R)efrescar | (F)iltros | (Q)uit`);
  screen.render();
}

function mostrarFiltros() {
  if (filterList) {
    filterList.hide();
    filterList = null;
  }
  
  const opciones = filtros.map(f => f.nombre);
  
  filterList = blessed.list({
    parent: screen,
    top: 'center',
    left: 'center',
    width: '60%',
    height: Math.min(filtros.length + 4, 20),
    border: 'line',
    label: ' Seleccionar Filtro ',
    style: {
      selected: {
        bg: 'blue',
        fg: 'white'
      }
    },
    keys: true,
    vi: true,
    items: opciones
  });
  
  filterList.on('select', function(node, index) {
    aplicarFiltro(filtros[index]);
  });
  
  filterList.on('cancel', function() {
    filterList.hide();
    filterList = null;
    screen.render();
  });
  
  screen.append(filterList);
  filterList.focus();
  screen.render();
}

async function init() {
  screen = blessed.screen({
    smartCSR: true,
    title: 'Hockey Patins - Partidos'
  });
  
  const titleBox = blessed.box({
    top: 0,
    left: 0,
    width: '100%',
    height: 3,
    content: '{center}{bold}🏒 HOQUEI PATINS - COMPETICIÓN{/bold}{/center}',
    tags: true,
    style: {
      bold: true,
      fg: 'green'
    }
  });
  
  statusBox = blessed.box({
    bottom: 0,
    left: 0,
    width: '100%',
    height: 3,
    content: 'Cargando...',
    tags: true,
    style: {
      fg: 'white',
      bg: 'blue'
    }
  });
  
  table = blessed.box({
    top: 3,
    left: 0,
    width: '100%',
    bottom: 3,
    content: 'Cargando datos...',
    tags: true,
    scrollable: true,
    alwaysScroll: true,
    style: {
      fg: 'white'
    }
  });
  
  screen.append(titleBox);
  screen.append(table);
  screen.append(statusBox);
  
  screen.key(['escape', 'q', 'Q', 'C-c'], () => {
    return process.exit(0);
  });
  
  screen.key(['r', 'R'], async () => {
    try {
      statusBox.setContent('⏳ Refrescando partidos...');
      screen.render();
      partidos = await scrapePartidos();
      aplicarFiltro(filtroActual);
    } catch (err) {
      statusBox.setContent(`❌ Error: ${err.message}`);
      screen.render();
    }
  });
  
  screen.key(['f', 'F'], () => {
    mostrarFiltros();
  });
  
  screen.key(['pageup'], () => {
    table.scroll(-10);
    screen.render();
  });
  
  screen.key(['pagedown'], () => {
    table.scroll(10);
    screen.render();
  });
  
  screen.key(['up'], () => {
    table.scroll(-1);
    screen.render();
  });
  
  screen.key(['down'], () => {
    table.scroll(1);
    screen.render();
  });
  
  screen.key(['h', 'H', '?'], () => {
    blessed.message({
      parent: screen,
      top: 'center',
      left: 'center',
      width: '50%',
      height: 12,
      border: 'line',
      label: ' AYUDA ',
      tags: true
    }).show('Controles:\n\n↑/↓ - Navegar\nPageUp/PageDown - Página\nF - Filtros\nR - Refrescar\nQ - Salir\nH - Esta ayuda');
  });
  
  try {
    filtros = await cargarFiltros();
    partidos = await cargarPartidos();
    
    if (partidos.length === 0) {
      statusBox.setContent('No hay datos. Presiona R para descargar.');
    } else {
      statusBox.setContent(`Cargados ${partidos.length} partidos. Filtro: Todos`);
      aplicarFiltro(null);
    }
    
    screen.render();
  } catch (err) {
    statusBox.setContent(`Error: ${err.message}`);
    screen.render();
  }
}

init();
