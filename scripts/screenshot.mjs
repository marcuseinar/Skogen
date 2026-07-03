import { chromium } from 'playwright';

const URL  = process.argv[2] || 'http://localhost:3000';
const OUT  = process.argv[3] || 'screenshot.png';
const WAIT = parseInt(process.argv[4] || '4000', 10);

const browser = await chromium.launch({
  executablePath: process.env.PLAYWRIGHT_BROWSERS_PATH
    ? `${process.env.PLAYWRIGHT_BROWSERS_PATH}/chromium`
    : undefined,
  args: ['--no-sandbox', '--disable-setuid-sandbox', '--use-gl=angle', '--use-angle=swiftshader'],
});

const ctx  = await browser.newContext({ viewport: { width: 1280, height: 720 } });
const page = await ctx.newPage();

page.on('console', m => process.stderr.write(`[browser] ${m.text()}\n`));
page.on('pageerror', e => process.stderr.write(`[error] ${e.message}\n`));

await page.goto(URL, { waitUntil: 'networkidle' });

// Wait for game to signal it's ready, or fall back to timeout
await Promise.race([
  page.waitForFunction('window.gameReady === true', { timeout: 10000 }),
  new Promise(r => setTimeout(r, WAIT)),
]);

// Extra frame time for the render loop to fire
await page.waitForTimeout(1000);

await page.screenshot({ path: OUT, fullPage: false });
await browser.close();

console.log(`Saved: ${OUT}`);
