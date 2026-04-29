import { createServer } from "node:http";

// ── Seed data ─────────────────────────────────────────────────────────────────

const SEED_USERS = [
  { id: 1, name: "Alice",   email: "alice@example.test",   role: "admin",  active: true  },
  { id: 2, name: "Bruno",   email: "bruno@example.test",   role: "editor", active: true  },
  { id: 3, name: "Carla",   email: "carla@example.test",   role: "viewer", active: false },
  { id: 4, name: "Dmitri",  email: "dmitri@example.test",  role: "editor", active: true  },
  { id: 5, name: "Evelyn",  email: "evelyn@example.test",  role: "viewer", active: true  },
  { id: 6, name: "Fatima",  email: "fatima@example.test",  role: "admin",  active: false },
  { id: 7, name: "Gustavo", email: "gustavo@example.test", role: "viewer", active: true  },
];

const SEED_ROLES = [
  { id: "admin",  name: "Admin"  },
  { id: "editor", name: "Editor" },
  { id: "viewer", name: "Viewer" },
];

// ── In-memory state ───────────────────────────────────────────────────────────

const users = SEED_USERS.map((u) => ({ ...u }));
const roles = SEED_ROLES.map((r) => ({ ...r }));

// ── Helpers ───────────────────────────────────────────────────────────────────

function cors(res) {
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "Content-Type");
}

function json(res, status, body) {
  cors(res);
  res.setHeader("Content-Type", "application/json");
  res.writeHead(status);
  res.end(JSON.stringify(body));
}

function readBody(req) {
  return new Promise((resolve, reject) => {
    let data = "";
    req.on("data", (chunk) => (data += chunk));
    req.on("end", () => {
      try { resolve(JSON.parse(data || "{}")); }
      catch (e) { reject(e); }
    });
    req.on("error", reject);
  });
}

// ── Source shape ──────────────────────────────────────────────────────────────

function makeSource() {
  return {
    user: { result: users.map((u) => ({ ...u })), total: users.length },
    role: { result: roles.map((r) => ({ ...r })), total: roles.length },
  };
}

// ── Server ────────────────────────────────────────────────────────────────────

const server = createServer(async (req, res) => {
  const url = new URL(req.url, "http://localhost");

  // CORS preflight
  if (req.method === "OPTIONS") {
    cors(res);
    res.writeHead(204);
    res.end();
    return;
  }

  // GET /__reset
  if (req.method === "GET" && url.pathname === "/__reset") {
    users.splice(0, users.length, ...SEED_USERS.map((u) => ({ ...u })));
    roles.splice(0, roles.length, ...SEED_ROLES.map((r) => ({ ...r })));
    cors(res);
    res.writeHead(200);
    res.end();
    return;
  }

  // GET /api/source
  if (req.method === "GET" && url.pathname === "/api/source") {
    return json(res, 200, makeSource());
  }

  // POST /api/user
  if (req.method === "POST" && url.pathname === "/api/user") {
    const body = await readBody(req);
    if ("active" in body) body.active = body.active === "true" || body.active === true;
    const id = users.length > 0 ? Math.max(...users.map((u) => u.id)) + 1 : 1;
    const user = { id, name: "", email: "", role: "viewer", active: true, ...body };
    users.push(user);
    return json(res, 201, user);
  }

  // PUT /api/user/:id
  const putUser = url.pathname.match(/^\/api\/user\/(.+)$/);
  if (req.method === "PUT" && putUser) {
    const id = isNaN(Number(putUser[1])) ? putUser[1] : Number(putUser[1]);
    const idx = users.findIndex((u) => u.id === id);
    if (idx === -1) return json(res, 404, { error: "User not found" });

    const body = await readBody(req);
    if ("active" in body) body.active = body.active === "true" || body.active === true;

    users[idx] = { ...users[idx], ...body, id: users[idx].id };
    return json(res, 200, users[idx]);
  }

  // DELETE /api/user/:id
  const deleteUser = url.pathname.match(/^\/api\/user\/(.+)$/);
  if (req.method === "DELETE" && deleteUser) {
    const id = isNaN(Number(deleteUser[1])) ? deleteUser[1] : Number(deleteUser[1]);
    const idx = users.findIndex((u) => u.id === id);
    if (idx === -1) return json(res, 404, { error: "User not found" });
    users.splice(idx, 1);
    cors(res);
    res.writeHead(204);
    res.end();
    return;
  }

  json(res, 404, { error: "Not found" });
});

const PORT = 3001;
server.listen(PORT, () => {
  console.log(`hyle demo server running at http://localhost:${PORT}`);
});
