// Generates src-tauri/icons/ from design/app-icon.svg.
// Outputs: 32x32.png, 128x128.png, 128x128@2x.png, tray-icon.png, icon.ico, icon.icns
// Requires: sharp (svg rasterize), png-to-ico. icns is hand-assembled (PNG-in-icns blocks).

import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import sharp from "sharp";
import pngToIco from "png-to-ico";

const SRC = "design/app-icon.svg";
const OUT = "src-tauri/icons";
mkdirSync(OUT, { recursive: true });

const svg = readFileSync(SRC);

async function png(size, file) {
  await sharp(svg, { density: 384 }).resize(size, size).png().toFile(`${OUT}/${file}`);
  return `${OUT}/${file}`;
}

// icns: 'icns' magic + typed blocks containing raw PNG data (accepted since 10.7)
function buildIcns(entries) {
  // entries: [{type, file}]
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

await png(32, "32x32.png");
await png(128, "128x128.png");
await png(256, "128x128@2x.png");
await png(32, "tray-icon.png");
await png(16, "tray-icon-16.png");
const p512 = await png(512, "icon-512.png");
const p1024 = await png(1024, "icon-1024.png");

const ico = await pngToIco([`${OUT}/tray-icon-16.png`, `${OUT}/32x32.png`, `${OUT}/128x128.png`, p512]);
writeFileSync(`${OUT}/icon.ico`, ico);

writeFileSync(
  `${OUT}/icon.icns`,
  buildIcns([
    { type: "ic08", file: `${OUT}/128x128.png` }, // 256
    { type: "ic09", file: p512 }, // 512
    { type: "ic10", file: p1024 }, // 1024
  ])
);

console.log("icons written to", OUT);
