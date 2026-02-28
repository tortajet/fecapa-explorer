const { chromium } = require('playwright');
const fs = require('fs');

async function scrapeMatches() {
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
  
  fs.writeFileSync('partidos.json', JSON.stringify(matches, null, 2));
  console.log(`Guardados ${matches.length} partidos en partidos.json`);
  
  await browser.close();
  return matches;
}

scrapeMatches().catch(console.error);
