const BASE_URL = "/api";
let sessionId = null;
async function ensureSession() {
  if (sessionId) return sessionId;
  const resp = await fetch(`${BASE_URL}/sessions`, { method: "POST" });
  if (!resp.ok) throw new Error("Failed to create session");
  const data = await resp.json();
  sessionId = data.id;
  return sessionId;
}
function sessionUrl(path) {
  return `${BASE_URL}/sessions/${sessionId}${path}`;
}
async function apiPut(path, body) {
  await ensureSession();
  const resp = await fetch(sessionUrl(path), {
    method: "PUT",
    headers: body !== void 0 ? { "Content-Type": "application/json" } : {},
    body: body !== void 0 ? JSON.stringify(body) : void 0
  });
  if (!resp.ok) {
    const err = await resp.json().catch(() => ({ error: resp.statusText }));
    throw new Error(err.error || resp.statusText);
  }
  const text = await resp.text();
  return text ? JSON.parse(text) : void 0;
}
async function updateProject(project) {
  return apiPut("/project", project);
}
async function updateCabinet(index, entry) {
  return apiPut(`/cabinets/${index}`, entry);
}
export {
  updateProject as a,
  updateCabinet as u
};
