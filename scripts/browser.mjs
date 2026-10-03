import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { readFile } from "node:fs/promises";
import { createServer } from "node:net";
import { resolve } from "node:path";
import { chromium } from "playwright";

const root = resolve(import.meta.dirname, "..");
const portPicker = createServer();
await new Promise((resolve, reject) => {
  portPicker.once("error", reject);
  portPicker.listen(0, "127.0.0.1", resolve);
});
const port = portPicker.address().port;
await new Promise(resolve => portPicker.close(resolve));
const server = spawn(process.env.FUSOR_BIN ?? "fusor", [
  "preview", "examples/book/dist", "--port", String(port),
], { cwd: root });
const ready = new Promise((resolve, reject) => {
  const timeout = setTimeout(() => reject(new Error("preview did not start")), 10_000);
  server.once("error", error => { clearTimeout(timeout); reject(error); });
  server.once("exit", code => {
    clearTimeout(timeout);
    reject(new Error(`preview exited: ${code}`));
  });
  server.stderr.on("data", bytes => {
    if (bytes.toString().includes("Ready.")) { clearTimeout(timeout); resolve(); }
  });
  server.stderr.pipe(process.stderr);
});

let browser;
try {
  await ready;
  browser = await chromium.launch();
  const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
  page.setDefaultTimeout(15_000);
  const errors = [];
  page.on("pageerror", error => errors.push(error.message));
  page.on("console", message => {
    if (message.type() === "error") errors.push(message.text());
  });
  const origin = `http://127.0.0.1:${port}/manual/`;
  await checkPages(page, origin);
  await checkNavigation(page, origin);
  await checkMobile(page, origin);
  assert.deepEqual(errors, []);
  console.log("PASS: Markdown, source assets, links, nested navigation, search, theme and mobile layout");
} finally {
  await browser?.close();
  server.kill();
  if (server.exitCode === null) await new Promise(resolve => server.once("exit", resolve));
}

async function checkPages(page, origin) {
  for (const [slug, title] of [
    ["", "A shared documentation shell"], ["topics", "Topics"], ["topics/authoring", "Authoring"],
  ]) {
    await page.goto(origin + slug);
    await page.getByRole("heading", { name: title, exact: true }).waitFor();
    assert.equal(await page.title(), `${title} · Example book`);
    const source = `content/${slug || "index"}.md`;
    assert.equal(await page.getByRole("link", { name: "View Markdown source" })
      .getAttribute("href"), `/manual/${source}`);
    const response = await page.request.get(origin + source);
    assert.equal(response.status(), 200);
    assert.equal(await response.text(),
      await readFile(resolve(root, "examples/book/docs", `${slug || "index"}.md`), "utf8"));
  }
  assert.equal(await page.locator("pre code").textContent(), "let count = signal(0);\ncount.set(1);\n");
  assert.deepEqual(await page.locator("table th").allTextContents(), ["File", "Purpose"]);
  await page.goto(origin);
  const authoring = page.locator(".lead, .sections").getByRole("link", { name: "authoring" });
  assert.equal(await authoring.getAttribute("href"), "/manual/topics/authoring#code");
  assert.equal(await page.getByRole("link", { name: "build script" }).getAttribute("href"),
    "https://github.com/fusor-rs/docs-base/blob/main/examples/book/build.rs");
  await authoring.click();
  await page.getByRole("heading", { name: "Authoring", exact: true }).waitFor();
  assert.equal(page.url(), origin + "topics/authoring#code");
}

async function checkNavigation(page, origin) {
  const sidebar = page.locator(".sidebar");
  const branch = sidebar.getByRole("button", { name: "Toggle Topics subpages" });
  assert.equal(await branch.getAttribute("aria-expanded"), "true");
  assert.equal(await sidebar.getByRole("link", { name: "Authoring", exact: true })
    .getAttribute("aria-current"), "page");
  await page.getByRole("navigation", { name: "Breadcrumb" }).getByRole("link", { name: "Topics" }).click();
  await page.getByRole("heading", { name: "Topics", exact: true }).waitFor();
  await branch.click();
  await sidebar.getByRole("link", { name: "Authoring", exact: true }).waitFor({ state: "hidden" });
  const search = page.getByRole("searchbox");
  await search.fill("signal");
  await sidebar.getByRole("link", { name: "Authoring", exact: true }).waitFor();
  assert.equal(await branch.getAttribute("aria-expanded"), "true");
  await search.fill("no-matching-topic");
  await page.locator(".search-empty").waitFor();
  await search.press("Escape");
  assert.equal(await search.inputValue(), "");
  await page.getByRole("button", { name: "Toggle color theme" }).click();
  await page.reload();
  await page.locator(".site.dark").waitFor();
  await page.goto(origin + "missing");
  await page.getByRole("heading", { name: "Page not found" }).waitFor();
}

async function checkMobile(page, origin) {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(origin + "topics/authoring");
  await page.getByRole("heading", { name: "Authoring", exact: true }).waitFor();
  assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  await page.locator(".mobile-toc summary").click();
  await page.locator(".mobile-toc").getByRole("link", { name: "Code" }).click();
  assert(page.url().endsWith("#code"));
  await page.getByRole("button", { name: "Toggle navigation" }).click();
  await page.locator(".sidebar").getByRole("link", { name: "Introduction" }).click();
  await page.getByRole("heading", { name: "A shared documentation shell" }).waitFor();
  await page.locator(".sidebar").waitFor({ state: "hidden" });
}
