import React, { useEffect, useState } from "react";
import {
  Activity,
  Inbox,
  Lock,
  Boxes,
  Server,
  LayoutDashboard,
  BrainCircuit,
  GitPullRequestDraft,
  Terminal,
  Search,
  Code2,
  X,
  List,
  ActivitySquare,
  MessageSquare,
  Star,
  ThumbsUp,
  ThumbsDown,
  Award,
} from "lucide-react";

interface Agent {
  id: string;
  name: string;
  role: string;
  status: string;
  capabilities: string[];
}

interface Task {
  id: string;
  title: string;
  kind: string;
  status: string;
  priority: string;
  assigned_agent_id: string | null;
  description: string | null;
  metadata: any | null;
  created_at: string;
}

interface EventItem {
  id: string;
  kind: string;
  agent_id: string | null;
  task_id: string | null;
  payload: any;
  created_at: string;
}

interface MessageItem {
  id: string;
  workspace_id: string;
  from_agent_id: string | null;
  to_agent_id: string | null;
  kind: string;
  payload: any;
  created_at: string;
}

interface HumanReview {
  id: string;
  reviewer_id: string;
  task_id: string | null;
  stars: number;
  praise: string | null;
  criticism: string | null;
  domain_context: string | null;
  updated_at: string;
}

interface AgentPeerReview {
  id: string;
  from_agent_id: string;
  task_id: string | null;
  stars: number;
  praise: string | null;
  criticism: string | null;
  domain_context: string | null;
  updated_at: string;
}

interface AgentCapability {
  id: string;
  domain: string;
  level: number; // 0-5
  source: string;
  confidence: number;
}

interface AgentEndorsement {
  id: string;
  from_agent_id: string;
  sentiment: string;
  reason: string | null;
  created_at: string;
}

interface Reputation {
  agent_id: string;
  // Legacy fields (from /reputation endpoint)
  avg_score?: number | null;
  review_count?: number;
  positive_endorsements?: number;
  negative_endorsements?: number;
  reviews?: Array<{
    id: string;
    reviewer_id: string;
    score: number;
    review_text: string | null;
    updated_at: string;
  }>;
  endorsements?: AgentEndorsement[];
  // Phase 1 dual-channel fields (from /full-reputation)
  human_star_avg?: number | null;
  human_review_count?: number;
  recent_human_praise?: string[];
  recent_human_criticism?: string[];
  human_reviews?: HumanReview[];
  agent_star_avg?: number | null;
  agent_review_count?: number;
  recent_agent_praise?: string[];
  recent_agent_criticism?: string[];
  agent_peer_reviews?: AgentPeerReview[];
  capabilities?: AgentCapability[];
}

interface Summary {
  active_agents: Agent[];
  open_tasks: Task[];
  pending_inbox_total: number;
  active_locks_count: number;
}

export default function App() {
  const [summary, setSummary] = useState<Summary | null>(null);
  const [allTasks, setAllTasks] = useState<Task[]>([]);
  const [events, setEvents] = useState<EventItem[]>([]);
  const [messages, setMessages] = useState<MessageItem[]>([]);
  const [reputations, setReputations] = useState<Record<string, Reputation>>(
    {},
  );
  const [error, setError] = useState(false);
  const [activeTab, setActiveTab] = useState<
    "overview" | "tasks" | "messages" | "events" | "reputation"
  >("overview");
  const [selectedTask, setSelectedTask] = useState<Task | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const [sumRes, tasksRes, eventsRes, msgsRes] = await Promise.all([
          fetch("http://localhost:4000/summary"),
          fetch("http://localhost:4000/tasks?limit=50"),
          fetch("http://localhost:4000/events?limit=50"),
          fetch("http://localhost:4000/messages?channel_id=&limit=50"),
        ]);

        if (sumRes.ok && tasksRes.ok && eventsRes.ok && msgsRes.ok) {
          const sumData: Summary = await sumRes.json();
          setSummary(sumData);

          const tData = await tasksRes.json();
          setAllTasks(tData.items || tData);

          const eData = await eventsRes.json();
          setEvents(eData.items || eData);

          const mData = await msgsRes.json();
          setMessages(mData.items || mData);

          // Fetch full dual-channel reputation for each active agent in parallel
          if (sumData.active_agents?.length) {
            const repMap: Record<string, Reputation> = {};
            await Promise.all(
              sumData.active_agents.map(async (agent) => {
                try {
                  const r = await fetch(
                    `http://localhost:4000/agents/${agent.id}/full-reputation`,
                  );
                  if (r.ok) repMap[agent.id] = await r.json();
                } catch {}
              }),
            );
            setReputations(repMap);
          }

          setError(false);
        } else {
          setError(true);
        }
      } catch (err) {
        setError(true);
      }
    };

    fetchData();
    const interval = setInterval(fetchData, 1000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="min-h-screen bg-[#0D1117] text-gray-200 font-sans flex flex-col">
      {/* Header */}
      <header className="flex-none px-8 py-5 border-b border-gray-800 bg-[#0D1117]/80 backdrop-blur sticky top-0 z-10 flex items-center justify-between">
        <div className="flex items-center gap-4">
          <div className="p-2.5 bg-purple-500/10 rounded-xl border border-purple-500/20">
            <Boxes className="w-6 h-6 text-purple-400" />
          </div>
          <div>
            <h1 className="text-xl font-bold text-white tracking-tight leading-tight">
              Agent Workspace{" "}
              <span className="font-light text-gray-500">OS</span>
            </h1>
            <p className="text-xs text-gray-400 font-mono mt-0.5">
              <span className="flex items-center gap-1.5 inline-flex">
                <Server className="w-3 h-3 text-green-400" />
                http://localhost:4000
              </span>
            </p>
          </div>
        </div>

        {/* Tabs */}
        <div className="flex bg-[#161B22] p-1 rounded-xl border border-gray-800">
          <TabButton
            active={activeTab === "overview"}
            onClick={() => setActiveTab("overview")}
            icon={<LayoutDashboard size={14} />}
          >
            Overview
          </TabButton>
          <TabButton
            active={activeTab === "tasks"}
            onClick={() => setActiveTab("tasks")}
            icon={<List size={14} />}
          >
            Local Tasks
          </TabButton>
          <TabButton
            active={activeTab === "messages"}
            onClick={() => setActiveTab("messages")}
            icon={<MessageSquare size={14} />}
          >
            Global Messages
          </TabButton>
          <TabButton
            active={activeTab === "reputation"}
            onClick={() => setActiveTab("reputation")}
            icon={<Award size={14} />}
          >
            Reputation
          </TabButton>
          <TabButton
            active={activeTab === "events"}
            onClick={() => setActiveTab("events")}
            icon={<ActivitySquare size={14} />}
          >
            Audit Log
          </TabButton>
        </div>

        <div className="flex flex-col items-end w-48">
          <div className="flex items-center gap-2 mb-1">
            <div
              className={`w-2 h-2 rounded-full ${!error && summary ? "bg-green-500 animate-pulse shadow-[0_0_8px_rgba(34,197,94,0.6)]" : "bg-red-500"}`}
            />
            <span className="font-mono text-xs tracking-widest text-gray-300 uppercase">
              {!error && summary ? "System Live" : "Offline"}
            </span>
          </div>
          <p className="text-[10px] text-gray-500 uppercase tracking-wider">
            Auto-Sync 1000ms
          </p>
        </div>
      </header>

      {/* Main Content Area */}
      <main className="flex-1 p-8 overflow-y-auto">
        {activeTab === "overview" && (
          <OverviewTab
            summary={summary}
            error={error}
            reputations={reputations}
          />
        )}
        {activeTab === "tasks" && (
          <TasksTab tasks={allTasks} onInspectTask={setSelectedTask} />
        )}
        {activeTab === "messages" && <MessagesTab messages={messages} />}
        {activeTab === "reputation" && (
          <ReputationTab summary={summary} reputations={reputations} />
        )}
        {activeTab === "events" && <EventsTab events={events} />}
      </main>

      {/* Task Modal Details */}
      {selectedTask && (
        <TaskModal
          task={selectedTask}
          summary={summary}
          onClose={() => setSelectedTask(null)}
        />
      )}
    </div>
  );
}

// -------------------------------------------------------------
// COMPONENTS
// -------------------------------------------------------------

function TabButton({ active, onClick, children, icon }: any) {
  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-2 px-4 py-1.5 rounded-lg text-sm font-medium transition-all ${
        active
          ? "bg-purple-500/20 text-purple-300 shadow-sm border border-purple-500/30"
          : "text-gray-400 hover:text-gray-200 hover:bg-gray-800 border border-transparent"
      }`}
    >
      {icon}
      {children}
    </button>
  );
}

function OverviewTab({ summary, error, reputations }: any) {
  return (
    <div className="animate-in fade-in duration-300">
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8">
        <StatCard
          icon={<BrainCircuit />}
          label="Active Sessions"
          value={
            summary?.active_agents.filter((a: any) => a.status === "active")
              .length ?? 0
          }
        />
        <StatCard
          icon={<GitPullRequestDraft />}
          label="Open Tasks"
          value={summary?.open_tasks.length ?? 0}
        />
        <StatCard
          icon={<Inbox />}
          label="Pending Inbox"
          value={summary?.pending_inbox_total ?? 0}
        />
        <StatCard
          icon={<Lock />}
          label="Active Locks"
          value={summary?.active_locks_count ?? 0}
        />
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Agents List */}
        <div className="lg:col-span-1 border border-gray-800 bg-[#161B22] rounded-2xl overflow-hidden shadow-2xl">
          <div className="p-4 border-b border-gray-800 bg-white/[0.02]">
            <h2 className="text-sm font-semibold text-white flex items-center gap-2">
              <Activity className="w-4 h-4 text-purple-400" /> Fleet Overview
            </h2>
          </div>
          <div className="p-3">
            {!summary && !error && (
              <p className="p-3 text-xs text-gray-500 italic">Waiting...</p>
            )}
            {summary?.active_agents.map((agent: any) => (
              <div
                key={agent.id}
                className="p-4 mb-2 rounded-xl border border-gray-800 bg-black/20 hover:bg-black/40 transition-colors"
              >
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-3">
                    <div className="relative">
                      <div className="w-9 h-9 rounded-full bg-gradient-to-tr from-purple-500/20 to-blue-500/20 border border-purple-500/30 flex items-center justify-center">
                        <Terminal className="w-4 h-4 text-purple-300" />
                      </div>
                      <div
                        className={`absolute bottom-0 right-0 w-2.5 h-2.5 rounded-full border-2 border-[#161B22] ${agent.status === "active" ? "bg-green-500" : "bg-yellow-500"}`}
                      />
                    </div>
                    <div>
                      <h3 className="font-medium text-gray-200 text-sm">
                        {agent.name}
                      </h3>
                      <p className="text-[10px] text-gray-500 font-mono tracking-widest">
                        {agent.id}
                      </p>
                    </div>
                  </div>
                </div>
                <div className="flex items-center justify-between mt-3">
                  <div className="flex flex-wrap gap-2">
                    <span className="px-2 py-0.5 rounded-full bg-blue-500/10 text-blue-400 border border-blue-500/20 text-[9px] uppercase font-bold tracking-wider">
                      {agent.role}
                    </span>
                  </div>
                  {reputations && reputations[agent.id] && (
                    <div className="flex items-center gap-2">
                      {reputations[agent.id].avg_score !== null && (
                        <div className="flex items-center gap-1 text-[10px] font-bold text-yellow-500 bg-yellow-500/10 px-1.5 py-0.5 rounded-md border border-yellow-500/20">
                          <Star size={10} fill="currentColor" />
                          {reputations[agent.id].avg_score.toFixed(1)}
                        </div>
                      )}
                      {(reputations[agent.id].positive_endorsements > 0 ||
                        reputations[agent.id].negative_endorsements > 0) && (
                        <div className="flex items-center gap-1.5 text-[10px] font-bold">
                          {reputations[agent.id].positive_endorsements > 0 && (
                            <span className="flex items-center gap-0.5 text-green-400 bg-green-500/10 px-1.5 py-0.5 rounded-md border border-green-500/20">
                              <ThumbsUp size={10} />{" "}
                              {reputations[agent.id].positive_endorsements}
                            </span>
                          )}
                          {reputations[agent.id].negative_endorsements > 0 && (
                            <span className="flex items-center gap-0.5 text-red-400 bg-red-500/10 px-1.5 py-0.5 rounded-md border border-red-500/20">
                              <ThumbsDown size={10} />{" "}
                              {reputations[agent.id].negative_endorsements}
                            </span>
                          )}
                        </div>
                      )}
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Board */}
        <div className="lg:col-span-2 border border-gray-800 bg-[#161B22] rounded-2xl overflow-hidden shadow-2xl">
          <div className="p-4 border-b border-gray-800 bg-white/[0.02]">
            <h2 className="text-sm font-semibold text-white flex items-center gap-2">
              <LayoutDashboard className="w-4 h-4 text-blue-400" /> Main Task
              Board
            </h2>
          </div>
          <div className="p-5 grid gap-3">
            {!summary?.open_tasks.length && (
              <div className="py-12 text-center border border-dashed border-gray-800 rounded-xl bg-black/10">
                <p className="text-gray-500 font-mono text-xs">
                  No active tasks.
                </p>
              </div>
            )}
            {summary?.open_tasks.map((task: any) => (
              <div
                key={task.id}
                className="flex flex-col md:flex-row items-start md:items-center justify-between p-4 rounded-xl border border-gray-800 bg-black/20"
              >
                <div>
                  <h3 className="font-semibold text-gray-200 mb-1 text-sm">
                    {task.title}
                  </h3>
                  <div className="flex items-center gap-2 font-mono text-[10px] text-gray-500">
                    <span>{task.id.slice(0, 8)}</span>
                    <span>•</span>
                    <span className="uppercase text-blue-400">
                      {String(task.kind)}
                    </span>
                  </div>
                </div>
                <div className="flex flex-col items-end mt-3 md:mt-0">
                  <span
                    className={`text-[9px] uppercase font-bold tracking-widest px-2 py-1 rounded-md border
                    ${
                      task.status === "open"
                        ? "bg-gray-800 text-gray-300 border-gray-700"
                        : task.status === "claimed"
                          ? "bg-yellow-500/10 text-yellow-400 border-yellow-500/20"
                          : task.status === "in_progress"
                            ? "bg-blue-500/10 text-blue-400 border-blue-500/20"
                            : "bg-green-500/10 text-green-400 border-green-500/20"
                    }
                  `}
                  >
                    {task.status.replace("_", " ")}
                  </span>
                  {task.assigned_agent_id && (
                    <span className="text-[10px] text-gray-400 font-mono mt-2">
                      → {task.assigned_agent_id}
                    </span>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

function TasksTab({ tasks, onInspectTask }: any) {
  return (
    <div className="animate-in fade-in duration-300 max-w-6xl mx-auto">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-lg font-semibold text-white">
          All Tasks Directory
        </h2>
        <div className="text-xs text-gray-400 font-mono">
          {tasks.length} items
        </div>
      </div>

      <div className="border border-gray-800 bg-[#161B22] rounded-xl overflow-hidden shadow-2xl">
        <table className="w-full text-left text-sm">
          <thead className="bg-[#0D1117] text-gray-400 text-xs font-mono border-b border-gray-800">
            <tr>
              <th className="px-5 py-3 font-medium uppercase tracking-wider">
                ID
              </th>
              <th className="px-5 py-3 font-medium uppercase tracking-wider">
                Title
              </th>
              <th className="px-5 py-3 font-medium uppercase tracking-wider">
                Status
              </th>
              <th className="px-5 py-3 font-medium uppercase tracking-wider">
                Assignee
              </th>
              <th className="px-5 py-3 font-medium uppercase tracking-wider">
                Action
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-800">
            {tasks.map((task: any) => (
              <tr
                key={task.id}
                className="hover:bg-white/[0.02] transition-colors group"
              >
                <td className="px-5 py-4 font-mono text-gray-500 text-xs">
                  {task.id.slice(0, 8)}
                </td>
                <td className="px-5 py-4 font-medium text-gray-200">
                  {task.title}
                  <div className="text-[10px] text-gray-500 font-mono mt-1 w-64 truncate">
                    {task.description || "No description"}
                  </div>
                </td>
                <td className="px-5 py-4">
                  <span
                    className={`text-[10px] uppercase font-bold tracking-widest px-2 py-1 rounded-md border
                    ${
                      task.status === "open"
                        ? "bg-gray-800 text-gray-300 border-gray-700"
                        : task.status === "claimed"
                          ? "bg-yellow-500/10 text-yellow-400 border-yellow-500/20"
                          : task.status === "in_progress"
                            ? "bg-blue-500/10 text-blue-400 border-blue-500/20"
                            : task.status === "done"
                              ? "bg-green-500/10 text-green-400 border-green-500/20"
                              : "bg-red-500/10 text-red-400 border-red-500/20"
                    }
                  `}
                  >
                    {task.status.replace("_", " ")}
                  </span>
                </td>
                <td className="px-5 py-4 font-mono text-xs text-gray-400">
                  {task.assigned_agent_id || "—"}
                </td>
                <td className="px-5 py-4">
                  <button
                    onClick={() => onInspectTask(task)}
                    className="flex items-center gap-1.5 px-3 py-1.5 bg-purple-500/10 hover:bg-purple-500/20 text-purple-400 border border-purple-500/20 rounded-lg text-xs font-medium transition-colors"
                  >
                    <Search size={12} /> Inspect
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function MessagesTab({ messages }: any) {
  return (
    <div className="animate-in fade-in duration-300 max-w-5xl mx-auto">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-lg font-semibold text-white">
          Global Message Stream
        </h2>
        <div className="text-xs text-gray-400 font-mono">
          {messages.length} messages
        </div>
      </div>

      <div className="space-y-4">
        {messages.map((msg: any) => (
          <div
            key={msg.id}
            className="p-5 border border-gray-800 bg-[#161B22] rounded-xl relative overflow-hidden shadow-sm"
          >
            <div className="absolute top-0 left-0 w-1 h-full bg-blue-500/50"></div>
            <div className="flex items-start justify-between mb-4">
              <div className="flex items-center gap-3">
                <div className="p-2 bg-black/40 border border-gray-800 rounded-lg">
                  <MessageSquare size={16} className="text-blue-400" />
                </div>
                <div>
                  <h3 className="text-sm font-bold text-gray-200">
                    {msg.from_agent_id || "System"}{" "}
                    <span className="text-gray-500 font-normal mx-1">➜</span>{" "}
                    {msg.to_agent_id || "Broadcast"}
                  </h3>
                  <p className="text-[10px] text-gray-500 font-mono tracking-wide uppercase mt-0.5">
                    Kind: {String(msg.kind)}
                  </p>
                </div>
              </div>
              <span className="text-[10px] text-gray-500 font-mono">
                {new Date(msg.created_at).toLocaleTimeString()}
              </span>
            </div>

            <div className="bg-black/50 border border-gray-800 rounded-lg p-3 overflow-x-auto">
              <pre className="text-xs font-mono text-gray-300">
                {JSON.stringify(msg.payload, null, 2)}
              </pre>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function EventsTab({ events }: any) {
  return (
    <div className="animate-in fade-in duration-300 max-w-4xl mx-auto">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-lg font-semibold text-white">Audit Log (Events)</h2>
        <div className="text-xs text-gray-400 font-mono">
          Tailing latest {events.length}
        </div>
      </div>
      <div className="space-y-4">
        {events.map((evt: any) => (
          <div
            key={evt.id}
            className="p-4 border border-gray-800 bg-[#161B22] rounded-xl flex items-start gap-4 hover:border-gray-700 transition-colors"
          >
            <div className="mt-1 p-2 bg-gray-800 rounded-lg">
              <ActivitySquare size={16} className="text-gray-400" />
            </div>
            <div className="flex-1">
              <div className="flex justify-between items-start">
                <span className="font-mono text-xs text-blue-400 bg-blue-500/10 px-2 py-0.5 rounded border border-blue-500/20">
                  {evt.kind}
                </span>
                <span className="text-[10px] text-gray-500 font-mono">
                  {new Date(evt.created_at).toLocaleTimeString()}
                </span>
              </div>
              <div className="mt-2 text-sm text-gray-300 font-mono grid grid-cols-2 gap-4">
                {evt.agent_id && (
                  <div>
                    <span className="text-gray-500">Agent:</span> {evt.agent_id}
                  </div>
                )}
                {evt.task_id && (
                  <div>
                    <span className="text-gray-500">Task:</span>{" "}
                    {evt.task_id.slice(0, 8)}
                  </div>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function TaskModal({ task, summary, onClose }: any) {
  const [eligibilityMap, setEligibilityMap] = React.useState<
    Record<string, any>
  >({});

  React.useEffect(() => {
    if (!summary?.active_agents) return;

    // Check claim eligibility for all active agents against this task kind
    summary.active_agents.forEach((agent: any) => {
      fetch(
        `http://localhost:4000/agents/${agent.id}/eligibility?task_kind=${task.kind}&action=claim`,
      )
        .then((res) => res.json())
        .then((data) => {
          setEligibilityMap((prev) => ({ ...prev, [agent.id]: data }));
        })
        .catch((err) => console.error(err));
    });
  }, [task.kind, summary?.active_agents]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-in fade-in duration-200">
      <div className="bg-[#0D1117] border border-gray-700 w-full max-w-3xl rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
        <div className="px-6 py-4 border-b border-gray-800 flex items-center justify-between bg-[#161B22]">
          <h2 className="text-lg font-bold text-gray-100 flex items-center gap-2">
            <Code2 size={18} className="text-purple-400" /> Task Inspector
          </h2>
          <button
            onClick={onClose}
            className="p-1.5 text-gray-400 hover:text-white hover:bg-gray-800 rounded-lg transition-colors"
          >
            <X size={18} />
          </button>
        </div>

        <div className="p-6 overflow-y-auto space-y-6">
          <div>
            <h3 className="text-xl font-bold text-white mb-1">{task.title}</h3>
            <p className="text-sm text-gray-400">
              {task.description || "No description provided."}
            </p>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="p-3 bg-[#161B22] border border-gray-800 rounded-xl">
              <span className="text-xs text-gray-500 font-mono uppercase">
                ID
              </span>
              <p className="text-sm font-mono text-gray-200 mt-1">{task.id}</p>
            </div>
            <div className="p-3 bg-[#161B22] border border-gray-800 rounded-xl">
              <span className="text-xs text-gray-500 font-mono uppercase">
                Assignee
              </span>
              <p className="text-sm font-mono text-gray-200 mt-1">
                {task.assigned_agent_id || "Unassigned"}
              </p>
            </div>
          </div>

          <div>
            <h4 className="text-sm font-semibold text-gray-300 mb-2 uppercase tracking-wider flex items-center gap-2 border-b border-gray-800 pb-2">
              Deep Metadata Engine
            </h4>
            <div className="bg-black/50 border border-gray-800 rounded-xl p-4 overflow-x-auto mb-4">
              {task.metadata && Object.keys(task.metadata).length > 0 ? (
                <pre className="text-xs font-mono text-green-400">
                  {JSON.stringify(task.metadata, null, 2)}
                </pre>
              ) : (
                <p className="text-xs font-mono text-gray-600 italic">
                  No structured metadata attached.
                </p>
              )}
            </div>

            <h4 className="text-sm font-semibold text-gray-300 mb-2 uppercase tracking-wider flex items-center gap-2 border-b border-gray-800 pb-2">
              Claim Eligibility Check
            </h4>
            <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
              {summary?.active_agents?.map((agent: any) => {
                const elig = eligibilityMap[agent.id];
                return (
                  <div
                    key={agent.id}
                    className="p-3 bg-black/40 border border-gray-800 rounded-lg"
                  >
                    <div className="flex justify-between items-center mb-1">
                      <span className="text-xs font-bold text-gray-300 truncate">
                        {agent.name}
                      </span>
                      {elig ? (
                        elig.eligible ? (
                          <span className="text-[10px] text-green-400 font-bold bg-green-500/10 px-1.5 py-0.5 rounded uppercase">
                            Eligible
                          </span>
                        ) : (
                          <span className="text-[10px] text-red-400 font-bold bg-red-500/10 px-1.5 py-0.5 rounded uppercase">
                            Blocked
                          </span>
                        )
                      ) : (
                        <span className="text-[10px] text-gray-500 italic">
                          Checking...
                        </span>
                      )}
                    </div>
                    {elig && !elig.eligible && (
                      <p className="text-[9px] text-red-500 font-mono mt-1">
                        Missing: {elig.missing.join(", ")}
                      </p>
                    )}
                  </div>
                );
              })}
              {(!summary?.active_agents ||
                summary.active_agents.length === 0) && (
                <p className="text-xs text-gray-500 italic">
                  No active agents to check.
                </p>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function StatCard({ icon, label, value }: any) {
  return (
    <div className="p-4 border border-gray-800 bg-[#161B22] rounded-xl flex items-center gap-4 hover:border-gray-700 transition-colors">
      <div className="p-2.5 bg-gray-800/50 rounded-lg text-gray-400 border border-gray-800/80">
        {React.cloneElement(icon, { className: "w-5 h-5" })}
      </div>
      <div>
        <h3 className="text-[11px] font-medium tracking-wide text-gray-400 uppercase">
          {label}
        </h3>
        <p className="text-2xl font-bold text-white font-mono">{value}</p>
      </div>
    </div>
  );
}

function ReputationTab({
  summary,
  reputations,
}: {
  summary: Summary | null;
  reputations: Record<string, Reputation>;
}) {
  if (!summary?.active_agents.length) {
    return (
      <div className="text-gray-500 text-sm font-mono text-center mt-12">
        No agents active to rank.
      </div>
    );
  }

  // Sort agents by avg score primarily, then by endorsements
  const ranked = [...summary.active_agents].sort((a, b) => {
    const ra = reputations[a.id];
    const rb = reputations[b.id];
    const scoreA = ra?.avg_score || 0;
    const scoreB = rb?.avg_score || 0;
    if (scoreA !== scoreB) return scoreB - scoreA;
    const endA =
      (ra?.positive_endorsements || 0) - (ra?.negative_endorsements || 0);
    const endB =
      (rb?.positive_endorsements || 0) - (rb?.negative_endorsements || 0);
    return endB - endA;
  });

  return (
    <div className="animate-in fade-in duration-300 max-w-6xl mx-auto space-y-6">
      <div className="flex items-center justify-between mb-2">
        <div>
          <h2 className="text-xl font-bold text-white flex items-center gap-2">
            <Award className="text-yellow-400" /> Global Agent Reputation
          </h2>
          <p className="text-xs text-gray-500 mt-1">
            Live ranking of fleet agents based on human reviews and peer-to-peer
            endorsements.
          </p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {ranked.map((agent, i) => {
          const rep = reputations[agent.id];
          return (
            <div
              key={agent.id}
              className="relative group p-6 bg-[#161B22] border border-gray-800 rounded-2xl shadow-xl overflow-hidden hover:border-gray-700 transition-all"
            >
              {/* Leaderboard Rank Medal */}
              <div className="absolute top-0 right-0 w-16 h-16 overflow-hidden">
                <div
                  className={`absolute top-4 -right-5 w-24 text-center transform rotate-45 text-[10px] font-bold uppercase tracking-wider shadow-sm 
                  ${
                    i === 0
                      ? "bg-yellow-500/20 text-yellow-500 border border-yellow-500/30"
                      : i === 1
                        ? "bg-gray-400/20 text-gray-300 border border-gray-400/30"
                        : i === 2
                          ? "bg-amber-700/20 text-amber-600 border border-amber-700/30"
                          : "hidden"
                  }`}
                >
                  Rank #{i + 1}
                </div>
              </div>

              <div className="flex items-center gap-4 mb-6">
                <div className="p-3 bg-gradient-to-br from-purple-500/10 to-blue-500/10 border border-purple-500/20 rounded-xl">
                  <Terminal className="text-purple-400 w-6 h-6" />
                </div>
                <div>
                  <h3 className="text-lg font-bold text-gray-200">
                    {agent.name}
                  </h3>
                  <div className="text-[10px] text-gray-500 font-mono mt-0.5">
                    {agent.id}
                  </div>
                </div>
              </div>

              {/* Dual-Channel Score Bar */}
              <div className="grid grid-cols-2 gap-3 mb-5">
                {/* Human Channel */}
                <div className="p-3 bg-black/30 rounded-xl border border-gray-800/60 text-center">
                  <div className="text-[10px] uppercase tracking-widest text-blue-400 font-semibold mb-2">
                    🧑 Human Score
                  </div>
                  <div className="text-2xl font-bold text-yellow-500 mb-0.5">
                    {rep?.human_star_avg != null
                      ? rep.human_star_avg.toFixed(1)
                      : "—"}
                  </div>
                  <div className="text-[9px] text-gray-500">
                    {rep?.human_review_count || 0} reviews
                  </div>
                </div>
                {/* Agent Channel */}
                <div className="p-3 bg-black/30 rounded-xl border border-gray-800/60 text-center">
                  <div className="text-[10px] uppercase tracking-widest text-purple-400 font-semibold mb-2">
                    🤖 Agent Score
                  </div>
                  <div className="text-2xl font-bold text-purple-400 mb-0.5">
                    {rep?.agent_star_avg != null
                      ? rep.agent_star_avg.toFixed(1)
                      : "—"}
                  </div>
                  <div className="text-[9px] text-gray-500">
                    {rep?.agent_review_count || 0} peer reviews
                  </div>
                </div>
              </div>

              {/* Capability Map */}
              {rep?.capabilities && rep.capabilities.length > 0 && (
                <div className="mb-5">
                  <h4 className="text-[11px] uppercase tracking-widest text-gray-500 font-semibold mb-3">
                    Capability Map
                  </h4>
                  <div className="space-y-2">
                    {rep.capabilities.map((cap) => (
                      <div key={cap.id} className="flex items-center gap-3">
                        <span className="text-[10px] font-mono text-gray-400 w-20 text-right shrink-0">
                          {cap.domain}
                        </span>
                        <div className="flex-1 h-1.5 bg-gray-800 rounded-full overflow-hidden">
                          <div
                            className={`h-full rounded-full transition-all ${
                              cap.level >= 5
                                ? "bg-yellow-500"
                                : cap.level >= 4
                                  ? "bg-green-500"
                                  : cap.level >= 3
                                    ? "bg-blue-500"
                                    : cap.level >= 2
                                      ? "bg-gray-400"
                                      : "bg-gray-700"
                            }`}
                            style={{ width: `${(cap.level / 5) * 100}%` }}
                          />
                        </div>
                        <span
                          className={`text-[11px] font-bold w-4 shrink-0 ${
                            cap.level >= 5
                              ? "text-yellow-500"
                              : cap.level >= 4
                                ? "text-green-500"
                                : cap.level >= 3
                                  ? "text-blue-400"
                                  : "text-gray-500"
                          }`}
                        >
                          {cap.level}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Human Reviews + Praise/Criticism */}
              {rep?.human_reviews && rep.human_reviews.length > 0 && (
                <div className="mb-4">
                  <h4 className="text-[11px] uppercase tracking-widest text-gray-500 font-semibold mb-3">
                    Human Reviews
                  </h4>
                  <div className="space-y-2 max-h-32 overflow-y-auto pr-2">
                    {rep.human_reviews.map((rev) => (
                      <div
                        key={rev.id}
                        className="p-3 rounded-lg bg-[#0D1117]/80 border border-gray-800 flex flex-col gap-1.5"
                      >
                        <div className="flex items-center justify-between">
                          <span className="text-[10px] text-blue-400 font-mono px-1.5 py-0.5 bg-blue-500/10 rounded">
                            {rev.reviewer_id}
                          </span>
                          <span className="flex items-center gap-0.5 text-xs font-bold text-yellow-500">
                            {rev.stars} <Star size={10} fill="currentColor" />
                          </span>
                        </div>
                        {rev.praise && (
                          <p className="text-xs text-green-400 italic">
                            👍 "{rev.praise}"
                          </p>
                        )}
                        {rev.criticism && (
                          <p className="text-xs text-red-400 italic">
                            👎 "{rev.criticism}"
                          </p>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Agent Peer Reviews + Gossip */}
              {rep?.agent_peer_reviews && rep.agent_peer_reviews.length > 0 && (
                <div className="mb-4">
                  <h4 className="text-[11px] uppercase tracking-widest text-gray-500 font-semibold mb-3">
                    Peer Opinions
                  </h4>
                  <div className="space-y-2 max-h-32 overflow-y-auto pr-2">
                    {rep.agent_peer_reviews.map((rev) => (
                      <div
                        key={rev.id}
                        className="p-2.5 rounded-lg bg-black/40 border border-gray-800/60 flex flex-col gap-1"
                      >
                        <div className="flex items-center justify-between">
                          <div className="text-[10px] text-gray-400 font-mono">
                            From: {rev.from_agent_id}
                          </div>
                          <span className="flex items-center gap-0.5 text-xs font-bold text-purple-400">
                            {rev.stars} <Star size={9} fill="currentColor" />
                          </span>
                        </div>
                        {rev.praise && (
                          <p className="text-xs text-green-300 italic">
                            👍 "{rev.praise}"
                          </p>
                        )}
                        {rev.criticism && (
                          <p className="text-xs text-red-300 italic">
                            👎 "{rev.criticism}"
                          </p>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Legacy endorsements (positive/negative) */}
              {rep?.endorsements && rep.endorsements.length > 0 && (
                <div className="mb-2">
                  <h4 className="text-[11px] uppercase tracking-widest text-gray-500 font-semibold mb-3">
                    Free Endorsements
                  </h4>
                  <div className="space-y-2 max-h-24 overflow-y-auto pr-2">
                    {rep.endorsements.map((end) => (
                      <div
                        key={end.id}
                        className="p-2.5 rounded-lg bg-black/40 border border-gray-800/60 flex items-start gap-2.5"
                      >
                        <div className="mt-0.5 shrink-0">
                          {end.sentiment === "positive" ? (
                            <ThumbsUp size={12} className="text-green-500" />
                          ) : (
                            <ThumbsDown size={12} className="text-red-500" />
                          )}
                        </div>
                        <div className="min-w-0 flex-1">
                          <div className="text-[10px] text-gray-400 font-mono">
                            From: {end.from_agent_id}
                          </div>
                          <p className="text-xs text-gray-300 break-words pt-1">
                            {end.reason || (
                              <span className="italic text-gray-600">
                                No reason
                              </span>
                            )}
                          </p>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {!rep && (
                <div className="py-8 text-center border border-dashed border-gray-800 rounded-xl bg-black/20 mt-4">
                  <p className="text-xs text-gray-500 font-mono">
                    No reputation data.
                  </p>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
