import { createServer } from "node:http";
import { readFileSync, existsSync } from "node:fs";
import { join, extname } from "node:path";
import { fileURLToPath } from "node:url";

// ── Paths ─────────────────────────────────────────────────────────────────────

const __dirname = fileURLToPath(new URL(".", import.meta.url));
const CLIENT_DIR = join(__dirname, "dist");
const SSR_MODULE = join(__dirname, "dist/server/entry-server.js");

// ── SSR renderer (loaded lazily so build step can run first) ──────────────────

let ssrRender;
async function getRender() {
  if (!ssrRender) {
    const mod = await import(SSR_MODULE);
    ssrRender = mod.render;
  }
  return ssrRender;
}

// ── Seed data ─────────────────────────────────────────────────────────────────

const SEED_USERS = [
  { id: 1, name: "Alice",   email: "alice@example.test",   role: "admin",  tags: ["rust", "web"], active: true  },
  { id: 2, name: "Bruno",   email: "bruno@example.test",   role: "editor", tags: ["web"],          active: true  },
  { id: 3, name: "Carla",   email: "carla@example.test",   role: "viewer", tags: [],               active: false },
  { id: 4, name: "Dmitri",  email: "dmitri@example.test",  role: "editor", tags: ["rust"],         active: true  },
  { id: 5, name: "Evelyn",  email: "evelyn@example.test",  role: "viewer", tags: ["web", "rust"],  active: true  },
  { id: 6, name: "Fatima",  email: "fatima@example.test",  role: "admin",  tags: [],               active: false },
  { id: 7, name: "Gustavo", email: "gustavo@example.test", role: "viewer", tags: ["rust"],         active: true  },
];

const SEED_ROLES = [
  { id: "admin",  name: "Admin"  },
  { id: "editor", name: "Editor" },
  { id: "viewer", name: "Viewer" },
];

const SEED_TAGS = [
  { id: "rust", name: "Rust" },
  { id: "web",  name: "Web"  },
];

let users = SEED_USERS.map((u) => ({ ...u }));
let roles = SEED_ROLES.map((r) => ({ ...r }));
let tags  = SEED_TAGS.map((t) => ({ ...t }));

function resetData() {
  users = SEED_USERS.map((u) => ({ ...u }));
  roles = SEED_ROLES.map((r) => ({ ...r }));
  tags  = SEED_TAGS.map((t) => ({ ...t }));
}

// ── Lookups helper ────────────────────────────────────────────────────────────

function makeLookups() {
  const roleMap = {};
  for (const r of roles) roleMap[r.id] = r;
  const tagMap = {};
  for (const t of tags) tagMap[t.id] = t;
  return { role: roleMap, tag: tagMap };
}

// ── Filter + sort + paginate helpers ─────────────────────────────────────────

function applyFilters(rows, params) {
  let result = rows;

  if (params.name) {
    const q = params.name.toLowerCase();
    result = result.filter((u) => u.name.toLowerCase().includes(q));
  }
  if (params.email) {
    const q = params.email.toLowerCase();
    result = result.filter((u) => u.email.toLowerCase().includes(q));
  }
  if (params.role) {
    result = result.filter((u) => u.role === params.role);
  }
  if (params.active !== undefined && params.active !== "") {
    const want = params.active === "true" || params.active === true;
    result = result.filter((u) => u.active === want);
  }
  // tags can be repeated: ?tags=rust&tags=web
  const tagFilter = params.tags;
  if (tagFilter && tagFilter.length > 0) {
    const wanted = Array.isArray(tagFilter) ? tagFilter : [tagFilter];
    if (wanted.length > 0) {
      result = result.filter((u) => wanted.every((t) => u.tags.includes(t)));
    }
  }

  return result;
}

function applySort(rows, sortField, ascending) {
  if (!sortField) return [...rows].sort((a, b) => String(a.name).localeCompare(String(b.name)));
  return [...rows].sort((a, b) => {
    const va = a[sortField] ?? "";
    const vb = b[sortField] ?? "";
    const cmp = String(va).localeCompare(String(vb));
    return ascending ? cmp : -cmp;
  });
}

function paginate(rows, page, perPage) {
  const start = (page - 1) * perPage;
  return rows.slice(start, start + perPage);
}

// ── Body parsing ──────────────────────────────────────────────────────────────

function readBody(req) {
  return new Promise((resolve, reject) => {
    let data = "";
    req.on("data", (chunk) => (data += chunk));
    req.on("end", () => resolve(data));
    req.on("error", reject);
  });
}

function parseFormBody(raw) {
  const params = new URLSearchParams(raw);
  const result = {};
  for (const [k, v] of params.entries()) {
    if (k in result) {
      result[k] = Array.isArray(result[k]) ? [...result[k], v] : [result[k], v];
    } else {
      result[k] = v;
    }
  }
  return result;
}

function parseJsonBody(raw) {
  try { return JSON.parse(raw || "{}"); } catch { return {}; }
}

// ── Validation ────────────────────────────────────────────────────────────────

function validateUser(data) {
  const errors = [];
  const name = (data.name ?? "").trim();
  if (!name) errors.push({ field: "name", message: "Name is required" });
  else if (name.length < 2) errors.push({ field: "name", message: "Name must be at least 2 characters" });
  return errors.length > 0 ? errors : null;
}

// ── HTML template ─────────────────────────────────────────────────────────────

function buildHtml(appHtml, ssrData, assetBase) {
  // Read the built index.html and inject the SSR content + data.
  const templatePath = join(CLIENT_DIR, "index.html");
  let template;
  try {
    template = readFileSync(templatePath, "utf-8");
  } catch {
    // Fallback minimal template (for dev / before build)
    template = `<!doctype html><html lang="en"><head><meta charset="UTF-8"/><meta name="viewport" content="width=device-width,initial-scale=1.0"/><title>rentity React example</title></head><body><div id="root"><!--app--></div></body></html>`;
  }

  const dataScript = `<script>window.__SSR_DATA__ = ${JSON.stringify(ssrData)};</script>`;

  return template
    .replace("<!--app-->", appHtml)
    .replace(/<div id="root"><\/div>/, `<div id="root">${appHtml}</div>${dataScript}`);
}

// ── MIME types ────────────────────────────────────────────────────────────────

const MIME = {
  ".html": "text/html",
  ".js":   "application/javascript",
  ".css":  "text/css",
  ".wasm": "application/wasm",
  ".svg":  "image/svg+xml",
  ".png":  "image/png",
  ".ico":  "image/x-icon",
  ".json": "application/json",
  ".map":  "application/json",
};

function serveStatic(res, filePath) {
  if (!existsSync(filePath)) return false;
  const ext = extname(filePath);
  const mime = MIME[ext] ?? "application/octet-stream";
  const content = readFileSync(filePath);
  res.setHeader("Content-Type", mime);
  res.writeHead(200);
  res.end(content);
  return true;
}

// ── SSR page renderer ─────────────────────────────────────────────────────────

async function renderPage(res, url, ssrData) {
  const render = await getRender();
  let appHtml;
  try {
    appHtml = render(url, ssrData);
  } catch (err) {
    console.error("SSR render error:", err);
    appHtml = "<div>Server render failed. Enable JavaScript to continue.</div>";
  }
  const html = buildHtml(appHtml, ssrData, "/");
  res.setHeader("Content-Type", "text/html; charset=utf-8");
  res.writeHead(200);
  res.end(html);
}

// ── Server ────────────────────────────────────────────────────────────────────

const server = createServer(async (req, res) => {
  const url = new URL(req.url, "http://localhost");
  const pathname = url.pathname;

  // ── CORS for API ──────────────────────────────────────────────────────────
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "Content-Type");

  if (req.method === "OPTIONS") {
    res.writeHead(204);
    res.end();
    return;
  }

  // ── Reset ─────────────────────────────────────────────────────────────────
  if (req.method === "GET" && pathname === "/__reset") {
    resetData();
    res.writeHead(200);
    res.end();
    return;
  }

  // ── JSON API ──────────────────────────────────────────────────────────────

  if (req.method === "GET" && pathname === "/api/source") {
    res.setHeader("Content-Type", "application/json");
    res.writeHead(200);
    res.end(JSON.stringify({
      user: { result: users.map((u) => ({ ...u })), total: users.length },
      role: { result: roles.map((r) => ({ ...r })), total: roles.length },
      tag:  { result: tags.map((t) => ({ ...t })),  total: tags.length  },
    }));
    return;
  }

  if (req.method === "POST" && pathname === "/api/user") {
    const raw = await readBody(req);
    const body = parseJsonBody(raw);
    if ("active" in body) body.active = body.active === "true" || body.active === true;
    if ("tags" in body && !Array.isArray(body.tags)) body.tags = body.tags ? [body.tags] : [];
    const id = users.length > 0 ? Math.max(...users.map((u) => u.id)) + 1 : 1;
    const user = { id, name: "", email: "", role: "viewer", tags: [], active: true, ...body };
    users.push(user);
    res.setHeader("Content-Type", "application/json");
    res.writeHead(201);
    res.end(JSON.stringify(user));
    return;
  }

  const putMatch = pathname.match(/^\/api\/user\/(.+)$/);
  if (req.method === "PUT" && putMatch) {
    const id = isNaN(Number(putMatch[1])) ? putMatch[1] : Number(putMatch[1]);
    const idx = users.findIndex((u) => u.id === id);
    if (idx === -1) { res.writeHead(404); res.end(JSON.stringify({ error: "Not found" })); return; }
    const body = parseJsonBody(await readBody(req));
    if ("active" in body) body.active = body.active === "true" || body.active === true;
    if ("tags" in body && !Array.isArray(body.tags)) body.tags = body.tags ? [body.tags] : [];
    users[idx] = { ...users[idx], ...body, id: users[idx].id };
    res.setHeader("Content-Type", "application/json");
    res.writeHead(200);
    res.end(JSON.stringify(users[idx]));
    return;
  }

  const deleteApiMatch = pathname.match(/^\/api\/user\/(.+)$/);
  if (req.method === "DELETE" && deleteApiMatch) {
    const id = isNaN(Number(deleteApiMatch[1])) ? deleteApiMatch[1] : Number(deleteApiMatch[1]);
    const idx = users.findIndex((u) => u.id === id);
    if (idx === -1) { res.writeHead(404); res.end(); return; }
    users.splice(idx, 1);
    res.writeHead(204);
    res.end();
    return;
  }

  // ── Form POST handlers ────────────────────────────────────────────────────

  // POST /users/new — create user
  if (req.method === "POST" && pathname === "/users/new") {
    const raw = await readBody(req);
    const body = parseFormBody(raw);
    if ("active" in body) body.active = body.active === "true" || body.active === true;
    if ("tags" in body) body.tags = Array.isArray(body.tags) ? body.tags : (body.tags ? [body.tags] : []);
    else body.tags = [];

    const errors = validateUser(body);
    if (errors) {
      // Re-render the new user form with errors
      return renderPage(res, "/users/new", {
        form: { values: body, errors },
      });
    }
    const id = users.length > 0 ? Math.max(...users.map((u) => u.id)) + 1 : 1;
    users.push({ id, name: "", email: "", role: "viewer", tags: [], active: true, ...body });
    res.setHeader("Location", "/");
    res.writeHead(303);
    res.end();
    return;
  }

  // POST /users/:id/edit — update user
  const editMatch = pathname.match(/^\/users\/(\d+)\/edit$/);
  if (req.method === "POST" && editMatch) {
    const id = Number(editMatch[1]);
    const idx = users.findIndex((u) => u.id === id);
    if (idx === -1) { res.writeHead(404); res.end("Not found"); return; }

    const raw = await readBody(req);
    const body = parseFormBody(raw);
    if ("active" in body) body.active = body.active === "true" || body.active === true;
    if ("tags" in body) body.tags = Array.isArray(body.tags) ? body.tags : (body.tags ? [body.tags] : []);
    else body.tags = [];

    const errors = validateUser(body);
    if (errors) {
      const user = users[idx];
      return renderPage(res, `/users/${id}/edit`, {
        form: { values: { ...user, ...body }, errors },
      });
    }
    users[idx] = { ...users[idx], ...body, id };
    res.setHeader("Location", "/");
    res.writeHead(303);
    res.end();
    return;
  }

  // POST /users/:id/delete — delete user
  const deleteMatch = pathname.match(/^\/users\/(\d+)\/delete$/);
  if (req.method === "POST" && deleteMatch) {
    const id = Number(deleteMatch[1]);
    const idx = users.findIndex((u) => u.id === id);
    if (idx !== -1) users.splice(idx, 1);
    res.setHeader("Location", "/");
    res.writeHead(303);
    res.end();
    return;
  }

  // ── SSR page routes ───────────────────────────────────────────────────────

  if (req.method === "GET" && pathname === "/") {
    const p = url.searchParams;
    const page    = Math.max(1, Number(p.get("page")  ?? 1));
    const perPage = Number(p.get("perPage") ?? 5);
    const sortField    = p.get("_sort") ?? "name";
    const sortAscending = (p.get("_asc") ?? "1") !== "0";

    const filterParams = {
      name:   p.get("name")   ?? "",
      email:  p.get("email")  ?? "",
      role:   p.get("role")   ?? "",
      active: p.get("active") ?? "",
      tags:   p.getAll("tags"),
    };

    let filtered = applyFilters(users, filterParams);
    const total = filtered.length;
    filtered = applySort(filtered, sortField, sortAscending);
    const rows = paginate(filtered, page, perPage);
    const lookups = makeLookups();

    return renderPage(res, req.url, {
      list: { rows, total, lookups, page, perPage, sortField, sortAscending, filters: filterParams },
    });
  }

  if (req.method === "GET" && pathname === "/users/new") {
    return renderPage(res, "/users/new", { form: { values: {} } });
  }

  const editPageMatch = pathname.match(/^\/users\/(\d+)\/edit$/);
  if (req.method === "GET" && editPageMatch) {
    const id = Number(editPageMatch[1]);
    const user = users.find((u) => u.id === id);
    if (!user) {
      res.setHeader("Location", "/");
      res.writeHead(303);
      res.end();
      return;
    }
    return renderPage(res, pathname, {
      form: { values: { ...user } },
    });
  }

  // ── Static assets ─────────────────────────────────────────────────────────

  if (req.method === "GET") {
    // Try exact path, then with /index.html
    const candidates = [
      join(CLIENT_DIR, pathname),
      join(CLIENT_DIR, pathname, "index.html"),
    ];
    for (const candidate of candidates) {
      if (serveStatic(res, candidate)) return;
    }
  }

  res.writeHead(404);
  res.end("Not found");
});

const PORT = process.env.PORT ?? 4173;
server.listen(PORT, () => {
  console.log(`hyle SSR server running at http://localhost:${PORT}`);
});
