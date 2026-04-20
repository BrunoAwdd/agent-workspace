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
}

interface Summary {
  active_agents: Agent[];
  open_tasks: Task[];
  pending_inbox_total: number;
  active_locks_count: number;
}

export default function App() {
  const [summary, setSummary] = useState<Summary | null>(null);
  const [error, setError] = useState(false);

  useEffect(() => {
    const fetchSummary = async () => {
      try {
        const res = await fetch("http://localhost:4000/summary");
        if (res.ok) {
          const data = await res.json();
          setSummary(data);
          setError(false);
        } else {
          setError(true);
        }
      } catch (err) {
        setError(true);
      }
    };

    fetchSummary();
    const interval = setInterval(fetchSummary, 1000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="min-h-screen bg-[#0D1117] text-gray-200 font-sans p-6 md:p-12">
      {/* Header */}
      <header className="flex items-center justify-between mb-10 pb-6 border-b border-gray-800">
        <div className="flex items-center gap-4">
          <div className="p-3 bg-purple-500/10 rounded-xl border border-purple-500/20">
            <Boxes className="w-8 h-8 text-purple-400" />
          </div>
          <div>
            <h1 className="text-2xl font-bold text-white tracking-tight">
              Agent Workspace
            </h1>
            <p className="text-sm text-gray-400 font-mono mt-1">
              <span className="flex items-center gap-1.5 inline-flex">
                <Server className="w-3.5 h-3.5 text-green-400" />
                http://localhost:4000
              </span>
            </p>
          </div>
        </div>
        <div className="text-right flex flex-col items-end">
          <div className="flex items-center gap-2 mb-1">
            <div
              className={`w-2.5 h-2.5 rounded-full ${!error && summary ? "bg-green-500 animate-pulse" : "bg-red-500"}`}
            />
            <span className="font-mono text-sm tracking-widest text-gray-400 uppercase">
              {!error && summary ? "LIVE" : "DISCONNECTED"}
            </span>
          </div>
          <p className="text-xs text-gray-500">Polling /summary (1s)</p>
        </div>
      </header>

      {/* Stats Grid */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-10">
        <StatCard
          icon={<BrainCircuit />}
          label="Active Agents"
          value={
            summary?.active_agents.filter((a) => a.status === "active")
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
          <div className="p-5 border-b border-gray-800 bg-white/[0.02]">
            <h2 className="text-md font-semibold text-white flex items-center gap-2">
              <Activity className="w-5 h-5 text-purple-400" /> Fleet Overview
            </h2>
          </div>
          <div className="p-3">
            {!summary && !error && (
              <p className="p-4 pl-2 text-gray-500 italic">
                Waiting for agents...
              </p>
            )}
            {error && (
              <p className="p-4 pl-2 text-red-400 italic">API unreachable.</p>
            )}
            {summary?.active_agents.map((agent) => (
              <div
                key={agent.id}
                className="p-4 mb-2 rounded-xl border border-gray-800 bg-black/20 hover:bg-black/40 transition-colors group"
              >
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-3">
                    <div className="relative">
                      <div className="w-10 h-10 rounded-full bg-gradient-to-tr from-purple-500/20 to-blue-500/20 border border-purple-500/30 flex items-center justify-center">
                        <Terminal className="w-4 h-4 text-purple-300" />
                      </div>
                      <div
                        className={`absolute bottom-0 right-0 w-3 h-3 rounded-full border-2 border-[#161B22] ${agent.status === "active" ? "bg-green-500" : "bg-yellow-500"}`}
                      />
                    </div>
                    <div>
                      <h3 className="font-medium text-gray-100">
                        {agent.name}
                      </h3>
                      <p className="text-xs text-gray-500 font-mono">
                        {agent.id}
                      </p>
                    </div>
                  </div>
                </div>
                <div className="flex flex-wrap gap-2 mt-3">
                  <span className="px-2 py-0.5 rounded-full bg-blue-500/10 text-blue-400 border border-blue-500/20 text-[10px] uppercase font-bold tracking-wider">
                    {agent.role}
                  </span>
                  {agent.capabilities.map((cap) => (
                    <span
                      key={cap}
                      className="px-2 py-0.5 rounded-full bg-gray-800 text-gray-400 border border-gray-700 text-[10px] uppercase font-bold tracking-wider"
                    >
                      {cap}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Tasks List */}
        <div className="lg:col-span-2 border border-gray-800 bg-[#161B22] rounded-2xl overflow-hidden shadow-2xl">
          <div className="p-5 border-b border-gray-800 bg-white/[0.02]">
            <h2 className="text-md font-semibold text-white flex items-center gap-2">
              <LayoutDashboard className="w-5 h-5 text-blue-400" /> Task Board
            </h2>
          </div>
          <div className="p-5">
            {!summary?.open_tasks.length && (
              <div className="py-12 text-center border border-dashed border-gray-800 rounded-xl bg-black/10">
                <p className="text-gray-500 font-mono text-sm">
                  No active tasks in the workspace.
                </p>
              </div>
            )}

            <div className="grid gap-3">
              {summary?.open_tasks.map((task) => (
                <div
                  key={task.id}
                  className="flex flex-col md:flex-row items-start md:items-center justify-between p-4 rounded-xl border border-gray-800 bg-black/20 hover:border-gray-700 transition-colors"
                >
                  <div className="mb-3 md:mb-0">
                    <h3 className="font-semibold text-gray-200 mb-1">
                      {task.title}
                    </h3>
                    <div className="flex items-center gap-3 font-mono text-xs text-gray-500">
                      <span>{task.id.slice(0, 8)}</span>
                      <span>•</span>
                      <span className="uppercase text-blue-400">
                        {task.kind}
                      </span>
                    </div>
                  </div>

                  <div className="flex items-center gap-4">
                    <div className="flex flex-col items-end">
                      <span
                        className={`text-[10px] uppercase font-bold tracking-widest px-2.5 py-1 rounded-md border
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
                        <span className="text-xs text-gray-400 font-mono mt-2 flex items-center gap-1">
                          → {task.assigned_agent_id}
                        </span>
                      )}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function StatCard({
  icon,
  label,
  value,
}: {
  icon: React.ReactNode;
  label: string;
  value: string | number;
}) {
  return (
    <div className="p-5 border border-gray-800 bg-[#161B22]/50 rounded-2xl flex items-center gap-4 hover:bg-[#161B22] transition-colors relative overflow-hidden group">
      <div className="absolute top-0 right-0 p-8 opacity-5 transform translate-x-4 -translate-y-4 group-hover:scale-110 transition-transform">
        {icon}
      </div>
      <div className="p-3 bg-gray-800/50 rounded-xl text-gray-400 border border-gray-800/80">
        {React.cloneElement(icon as React.ReactElement<any>, {
          className: "w-6 h-6",
        })}
      </div>
      <div>
        <h3 className="text-sm font-medium tracking-wide text-gray-400 mb-0.5">
          {label}
        </h3>
        <p className="text-3xl font-bold text-white font-mono">{value}</p>
      </div>
    </div>
  );
}
