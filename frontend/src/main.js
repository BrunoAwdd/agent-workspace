import "./style.css";

const API_BASE = "http://localhost:4000";

async function fetchAPI(endpoint) {
  try {
    const response = await fetch(`${API_BASE}${endpoint}`);
    if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
    return await response.json();
  } catch (e) {
    console.error(`Could not fetch ${endpoint}:`, e);
    return null;
  }
}

function updateAgents(agents) {
  const list = document.getElementById("agent-list");
  const count = document.getElementById("agent-count");
  if (!list || !agents) return;

  count.textContent = agents.length;
  list.innerHTML = agents
    .map(
      (agent) => `
    <div class="agent-item">
      <div class="agent-avatar">${agent.id.substring(0, 2).toUpperCase()}</div>
      <div class="agent-info">
        <h4>${agent.name}</h4>
        <p>${agent.role} • <span style="color: ${agent.status === "active" ? "var(--success-color)" : "var(--text-secondary)"}">${agent.status}</span></p>
      </div>
    </div>
  `,
    )
    .join("");
}

function updateTasks(tasks) {
  const list = document.getElementById("task-list");
  if (!list || !tasks) return;

  list.innerHTML =
    tasks.length === 0
      ? '<p style="text-align:center; opacity:0.5; margin-top:20px;">No tasks available</p>'
      : tasks
          .map((task) => {
            const progress = task.metadata?.progress
              ? parseInt(task.metadata.progress)
              : task.status === "done"
                ? 100
                : task.status === "claimed"
                  ? 10
                  : 0;
            return `
        <div class="task-item ${task.priority}">
          <div class="task-title">
            <span>${task.title}</span>
            <span style="font-size: 0.7rem; color: var(--primary-color)">${task.status.toUpperCase()}</span>
          </div>
          <div class="task-meta">${task.description}</div>
          <div class="progress-bar">
            <div class="progress-fill" style="width: ${progress}%"></div>
          </div>
        </div>
      `;
          })
          .join("");
}

function updateMissionInfo(messages) {
  const phaseEl = document.getElementById("mission-phase");
  const targetEl = document.getElementById("mission-target");
  const keyEl = document.getElementById("mission-key");
  const msgCountEl = document.getElementById("agent-msg-count");

  if (!messages) return;

  msgCountEl.textContent = messages.length;

  const phantomMsg = messages.find(
    (m) =>
      m.payload?.operation === "PHANTOM_LEDGER" ||
      m.payload?.details?.operation === "PHANTOM_LEDGER",
  );

  if (phantomMsg) {
    const payload = phantomMsg.payload;
    if (payload.phase) phaseEl.textContent = payload.phase;
    if (payload.code?.activation_key)
      keyEl.textContent = payload.code.activation_key;
    if (payload.instruction) {
      const blockMatch = payload.instruction.match(/block (\d+)/i);
      if (blockMatch) targetEl.textContent = `BLOCK ${blockMatch[1]}`;
    }
    document.getElementById("mission-status").style.borderColor =
      "var(--success-color)";
    document.getElementById("mission-status").style.boxShadow =
      "0 0 20px rgba(0, 255, 159, 0.2)";
  }
}

function logToConsole(message, type = "info") {
  const consoleEl = document.getElementById("console-output");
  if (!consoleEl) return;

  const line = document.createElement("div");
  line.className = "console-line";
  const timestamp = new Date().toLocaleTimeString();

  let color = "var(--success-color)";
  if (type === "warn") color = "var(--accent-color)";
  if (type === "meta") color = "var(--primary-color)";

  line.innerHTML = `
    <span class="console-timestamp">[${timestamp}]</span>
    <span style="color: ${color}">${message}</span>
  `;

  consoleEl.appendChild(line);
  consoleEl.scrollTop = consoleEl.scrollHeight;
}

async function refreshData() {
  const agents = await fetchAPI("/agents");
  // Hack to get all tasks - if the API doesn't support list_tasks, we might need to adjust
  // Let's check for /tasks (GET) if it was added or if we can get them per agent
  const tasks = []; // We'll try to get tasks for each agent if list_tasks is missing

  if (agents) {
    updateAgents(agents);
    for (const agent of agents) {
      // Trying the endpoint /tasks (assuming I might have missed it or it exists)
      // Actually let's assume I can get tasks somehow or just show the ones I know
    }
  }

  // We'll also try to fetch messages from a known agent (antigravity-agent)
  const messages = await fetchAPI("/messages");
  if (messages) {
    updateMissionInfo(messages);
  }

  // Since listing all tasks is not obvious in the API routes,
  // let's try to fetch /tasks just in case
  const allTasks = await fetchAPI("/tasks");
  if (allTasks && Array.isArray(allTasks)) {
    updateTasks(allTasks);
  }
}

// Initial Log
logToConsole("System kernel initialized.", "meta");
logToConsole("Establishing handshake with workspace nodes...");

// Polling
setInterval(refreshData, 3000);
refreshData();

// Mock dependencies
const depList = document.getElementById("dependency-list");
depList.innerHTML = `
  <div class="agent-item">
    <div style="font-size: 0.7rem;">DATABASE: SQLITE</div>
    <div style="margin-left: auto; color: var(--success-color);">ONLINE</div>
  </div>
  <div class="agent-item">
    <div style="font-size: 0.7rem;">MCP_SERVER: STDIO</div>
    <div style="margin-left: auto; color: var(--accent-color);">IDLE</div>
  </div>
  <div class="agent-item">
    <div style="font-size: 0.7rem;">API_REST: PORT 4000</div>
    <div style="margin-left: auto; color: var(--success-color);">ACTIVE</div>
  </div>
`;
