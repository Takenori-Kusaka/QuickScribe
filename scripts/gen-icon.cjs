// CI/ローカルでアイコン原画(1024x1024 PNG)を生成する。
// バイナリ画像をリポジトリにコミットせず、決定論的に生成して `tauri icon` に渡す。
const fs = require("fs");
const path = require("path");
const { PNG } = require("pngjs");

const size = 1024;
const png = new PNG({ width: size, height: size });

// ブランドカラー(#4F46E5)の単色。後でデザインアセットに差し替える。
const [r, g, b] = [0x4f, 0x46, 0xe5];
for (let y = 0; y < size; y++) {
  for (let x = 0; x < size; x++) {
    const idx = (size * y + x) << 2;
    png.data[idx] = r;
    png.data[idx + 1] = g;
    png.data[idx + 2] = b;
    png.data[idx + 3] = 255;
  }
}

const out = path.join(__dirname, "..", "src-tauri", "icon-src.png");
fs.writeFileSync(out, PNG.sync.write(png));
console.log("wrote", out);
