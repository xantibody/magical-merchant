// UI 検証用ワンショット: dist を配信し、Tauri IPC をモックして実描画を撮影する
import { chromium } from "playwright";
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { extname, join } from "node:path";

const MIME = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".css": "text/css",
  ".svg": "image/svg+xml",
  ".woff2": "font/woff2",
};

const server = createServer(async (req, res) => {
  const path = req.url === "/" ? "/index.html" : req.url.split("?")[0];
  try {
    const data = await readFile(join("dist", path));
    res.writeHead(200, { "content-type": MIME[extname(path)] ?? "application/octet-stream" });
    res.end(data);
  } catch {
    const data = await readFile(join("dist", "index.html"));
    res.writeHead(200, { "content-type": "text/html" });
    res.end(data);
  }
});
await new Promise((r) => server.listen(4173, r));

const NOTES = [
  {
    path: "/n/1.md",
    filename: "1.md",
    time: "2026-06-12T10:30:00+09:00",
    tags: ["design"],
    preview: "# 設計メモ\nエディタの行幅を70chに揃える",
    title: "設計メモ",
  },
  {
    path: "/n/2.md",
    filename: "2.md",
    time: "2026-06-11T18:00:00+09:00",
    tags: ["recipe", "dinner"],
    preview: "# バジルパスタ\n[[設計メモ]]ではない普通のメモ",
    title: "バジルパスタ",
  },
  {
    path: "/n/3.md",
    filename: "3.md",
    time: "2026-06-10T08:12:00+09:00",
    tags: [],
    preview: "# 旅行計画\n京都へ行く。",
    title: "旅行計画",
  },
];

const MOCK = `
window.__TAURI_INTERNALS__ = {
  metadata: {},
  plugins: {},
  transformCallback: (cb) => cb,
  invoke: (cmd, args) => {
    const notes = ${JSON.stringify(NOTES)};
    switch (cmd) {
      case "list_notes": return Promise.resolve(notes);
      case "read_note": return Promise.resolve("---\\ntime: 2026-06-12T10:30:00+09:00\\ntags: [design]\\n---\\n# 設計メモ\\n\\nエディタの行幅は **70ch** に揃える。詳細は [[バジルパスタ]] と [[存在しないノート]] を参照。\\n\\n> シンプルさは捨てないこと\\n\\n\`\`\`rust\\nfn main() { println!(\\"hello\\"); }\\n\`\`\`");
      case "list_backlinks": return Promise.resolve([notes[1]]);
      case "list_mentions": return Promise.resolve([notes[2]]);
      case "search_notes": return Promise.resolve([{ filename: "3.md", title: "旅行計画", snippet: "…京都へ行く。…" }]);
      case "search_timeline": return Promise.resolve([{ date: "2026-06-11", time: "09:15:00", snippet: "京都の打ち合わせメモ" }]);
      case "resolve_wikilink": return Promise.resolve(args && args.title === "バジルパスタ" ? "2.md" : null);
      case "read_timeline": return Promise.resolve(["- [09:15:00] 京都の打ち合わせ、[[設計メモ]]に追記した {\\"battery\\":82,\\"is_charging\\":false,\\"network_type\\":\\"WiFi\\",\\"os\\":\\"macos\\",\\"arch\\":\\"arm64\\"}", "- [12:40:00] 昼休みに散歩"]);
      case "list_timeline_dates": return Promise.resolve(["2026-06-12", "2026-06-11"]);
      case "read_timeline_by_date": return Promise.resolve(["- [09:15:00] 京都の打ち合わせ、[[設計メモ]]に追記した"]);
      case "auth_status": return Promise.resolve(false);
      case "sync_status": return Promise.resolve({ state: "idle" });
      case "get_sync_config": return Promise.resolve({ workers_url: "" });
      case "is_sync_config_editable": return Promise.resolve(true);
      case "list_projects": return Promise.resolve([{ slug: "app", name: "App", description: "" }]);
      case "list_active_tasks": return Promise.resolve([{ filename: "t1.md", title: "リリース準備", created: "2026-06-10T00:00:00+09:00", tags: [], body: "" }]);
      case "list_done_tasks": return Promise.resolve([]);
      default: return Promise.resolve(null);
    }
  },
};
`;

const browser = await chromium.launch();
const shots = [];
for (const theme of ["light", "dark"]) {
  const ctx = await browser.newContext({
    viewport: { width: 900, height: 640 },
    colorScheme: theme,
  });
  await ctx.addInitScript(MOCK);
  await ctx.addInitScript(`localStorage.setItem("theme", "${theme}");`);
  const page = await ctx.newPage();

  // Timeline (home)
  await page.goto("http://localhost:4173/");
  await page.waitForTimeout(900);
  await page.screenshot({ path: `shot-${theme}-timeline.png` });

  // Notes list
  await page.goto("http://localhost:4173/notes");
  await page.waitForTimeout(700);
  const listBtn = page.locator('button[aria-label="ノート一覧を開く"]');
  await listBtn.focus();
  await listBtn.click();
  await page.waitForTimeout(500);
  await page.screenshot({ path: `shot-${theme}-notes-list.png` });

  // Note preview (with backlinks/mentions)
  await page.locator(".note-list-item").first().click();
  await page.waitForTimeout(700);
  await page.screenshot({ path: `shot-${theme}-note-preview.png` });

  // Command palette
  await page.keyboard.press("ControlOrMeta+k");
  await page.waitForTimeout(400);
  await page.keyboard.type("京都");
  await page.waitForTimeout(600);
  await page.screenshot({ path: `shot-${theme}-palette.png` });

  // Menu open
  await page.keyboard.press("Escape");
  await page.locator('button[aria-label="Toggle menu"]').click();
  await page.waitForTimeout(300);
  await page.screenshot({ path: `shot-${theme}-menu.png` });

  shots.push(theme);
  await ctx.close();
}
await browser.close();
server.close();
console.log("done:", shots.join(", "));
