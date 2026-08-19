// Generates src-tauri/icons/ from design/app-icon*.svg.
//
// App icon (Dock / exe / dmg / ico / icns): app-icon-tile.svg — rounded green
// tile + white eye (the two-pupil mark is too dense at small sizes).
// Tray icon (menu bar / taskbar): app-icon-16.svg — single-pupil variant sized
// for small rendering (the two-pupil mark blurs below ~32px).
// Requires: sharp (svg rasterize), png-to-ico. icns is hand-assembled
// (PNG-in-icns blocks).

import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import sharp from "sharp";
import pngToIco from "png-to-ico";

const TILE = "design/app-icon-tile.svg";
const TRAY = "design/app-icon-16.svg";
const OUT = "src-tauri/icons";
mkdirSync(OUT, { recursive: true });

async function png(svg, size, file) {
  await sharp(svg, { density: 384 })
    .resize(size, size)
    .withMetadata({ density: 72 })
    .png()
    .toFile(`${OUT}/${file}`);
  return `${OUT}/${file}`;
}

// icns: 'icns' magic + typed blocks containing raw PNG data (accepted since 10.7)
function buildIcns(entries) {
  // entries: [{type, file}] — type must match the PNG's pixel size (ic07=128,
  // ic08=256, ic09=512, ic10=1024).
  const blocks = entries.map(({ type, file }) => {
    const data = readFileSync(file);
    const block = Buffer.alloc(8 + data.length);
    block.write(type, 0, "ascii");
    block.writeUInt32BE(8 + data.length, 4);
    data.copy(block, 8);
    return block;
  });
  const total = blocks.reduce((s, b) => s + b.length, 0);
  const head = Buffer.alloc(8);
  head.write("icns", 0, "ascii");
  head.writeUInt32BE(8 + total, 4);
  return Buffer.concat([head, ...blocks]);
}

// App icon — rounded green tile.
await png(TILE, 16, "icon-16.png");
await png(TILE, 32, "32x32.png");
await png(TILE, 128, "128x128.png");
await png(TILE, 256, "128x128@2x.png");
const p512 = await png(TILE, 512, "icon-512.png");
const p1024 = await png(TILE, 1024, "icon-1024.png");

// Tray icon — single-pupil variant.
await png(TRAY, 32, "tray-icon.png");
await png(TRAY, 16, "tray-icon-16.png");

const ico = await pngToIco([`${OUT}/icon-16.png`, `${OUT}/32x32.png`, `${OUT}/128x128.png`, p512]);
writeFileSync(`${OUT}/icon.ico`, ico);

writeFileSync(
  `${OUT}/icon.icns`,
  buildIcns([
    { type: "ic07", file: `${OUT}/128x128.png` },
    { type: "ic08", file: `${OUT}/128x128@2x.png` },
    { type: "ic09", file: p512 },
    { type: "ic10", file: p1024 },
  ])
);

console.log("icons written to", OUT);
