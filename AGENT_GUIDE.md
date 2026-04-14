# Agent Workspace — Guia para Agentes IA

Este documento descreve como um agente IA deve usar o **agent-workspace** como camada de coordenação entre agentes.

---

## O que é

O agent-workspace é um servidor de mensageria e coordenação entre agentes. Ele oferece:

- **Sessões** — ciclo de vida check-in / heartbeat / check-out
- **Mensagens + Inbox** — comunicação assíncrona entre agentes
- **Tasks** — trabalho distribuível e claimable
- **Locks** — exclusão mútua sobre recursos compartilhados
- **Handoffs** — passagem de contexto entre sessões
- **Dependencies** — status de saúde de recursos externos

Dois modos de acesso:
| Modo | Quando usar |
|------|-------------|
| **MCP (stdio)** | Agentes IA com suporte a MCP (Claude, etc.) |
| **HTTP REST** | Scripts, agentes sem MCP, testes diretos |

---

## Setup

### 1. Build

```bash
cd /home/bruno/projects/agent-workspace
cargo build -p aw-mcp -p aw-api
```

### 2. Configuração (`.env`)

```env
STORAGE_BACKEND=sqlite        # ou "postgres"
SQLITE_URL=sqlite://agent-workspace.db
# POSTGRES_URL=postgres://user:pass@localhost:5432/agent_workspace
PORT=4000                     # porta da API HTTP
```

### 3. Registrar o agente (pré-requisito obrigatório)

O agente precisa existir no banco **antes** de fazer check-in.
Faça isso uma vez, via API HTTP ou direto no banco:

```bash
# Subir a API
cargo run -p aw-api

# Registrar o agente (substitua os valores)
curl -s -X POST http://localhost:4000/agents \
  -H 'Content-Type: application/json' \
  -d '{
    "id": "meu-agente-1",
    "name": "Meu Agente",
    "role": "worker",
    "capabilities": ["analysis", "write_document"],
    "permissions": ["read", "write"]
  }'
```

O `id` é livre — pode ser qualquer string única (ex: `"claude-researcher"`, `"gpt-writer"`).
Esta etapa só precisa acontecer uma vez por agente.

### 4. Configurar MCP

Crie `.mcp.json` na raiz do projeto do agente (ou em `~/.claude/settings.json`):

```json
{
  "mcpServers": {
    "agent-workspace": {
      "command": "/home/bruno/projects/agent-workspace/target/debug/aw-mcp",
      "env": {
        "SQLITE_URL": "sqlite:///home/bruno/projects/agent-workspace/agent-workspace.db"
      }
    }
  }
}
```

> **Para produção**: use `target/release/aw-mcp` (compile com `cargo build --release -p aw-mcp`).

---

## Ciclo de vida de uma sessão

```
Startup
  └─► workspace.check_in(agent_id)
        │   retorna: session_id
        │
        ├─► workspace.read_inbox(agent_id)       ← ler mensagens pendentes
        ├─► workspace.list_handoffs(agent_id)    ← ler contexto de sessões anteriores
        │
        │   [loop de trabalho]
        ├─► workspace.heartbeat(session_id)      ← a cada 30-60s
        ├─► workspace.claim_task(...)            ← pegar trabalho
        ├─► workspace.update_task_status(...)    ← reportar progresso
        ├─► workspace.send_message(...)          ← comunicar com outros agentes
        ├─► workspace.acquire_lock(...)          ← proteger recurso compartilhado
        ├─► workspace.release_lock(...)
        │
Shutdown
  └─► workspace.check_out(session_id, create_handoff=true)
```

---

## Referência das Tools

### `workspace.check_in`

Registra uma sessão ativa. Chame uma vez no início.

```json
{
  "agent_id": "meu-agente-1",
  "metadata": { "version": "1.0", "environment": "prod" }
}
```

**Retorna:** objeto `AgentSession` com `id` (o `session_id` para usar nas calls seguintes).

---

### `workspace.heartbeat`

Renova o keepalive da sessão. Chame a cada **30–60 segundos** enquanto estiver ativo.

```json
{
  "session_id": "uuid-da-sessao",
  "health": "healthy",
  "current_task_id": "uuid-da-task-atual"
}
```

Valores de `health`: `"healthy"` | `"degraded"` | `"unknown"`

---

### `workspace.check_out`

Encerra a sessão. Passe `create_handoff: true` para deixar contexto para o próximo agente.

```json
{
  "session_id": "uuid-da-sessao",
  "create_handoff": true,
  "handoff_summary": "Analisei os dados de janeiro. Falta processar fevereiro.",
  "handoff_payload": { "last_processed_month": "2024-01", "next_step": "february" }
}
```

---

### `workspace.send_message`

Envia uma mensagem para outro agente (ou para um canal).

```json
{
  "workspace_id": "main",
  "from_agent_id": "meu-agente-1",
  "to_agent_id": "agente-revisor",
  "kind": "review_request",
  "payload": { "document_id": "doc-42", "urgency": "high" },
  "deliver_to_inbox": true
}
```

Valores de `kind`:
- `"chat_message"` — mensagem livre
- `"review_request"` — solicita revisão
- `"approval_request"` — solicita aprovação
- `"handoff_note"` — nota de passagem
- `"alert"` — alerta urgente
- `"status_update"` — atualização de progresso
- `"deferred_task"` — tarefa para executar depois
- `"conditional_instruction"` — instrução condicional

---

### `workspace.read_inbox`

Lê todas as mensagens pendentes endereçadas a este agente.

```json
{ "agent_id": "meu-agente-1" }
```

**Retorna:** `{ items: [...], count: N }`

Após processar cada item, chame `workspace.ack_inbox`.

---

### `workspace.ack_inbox`

Marca uma mensagem do inbox como processada.

```json
{
  "item_id": "uuid-do-item",
  "agent_id": "meu-agente-1",
  "status": "done"
}
```

Valores de `status`: `"done"` | `"failed"` | `"processing"`

---

### `workspace.create_task`

Cria uma nova task disponível para qualquer agente reclamar.

```json
{
  "title": "Analisar relatório Q1",
  "description": "Extrair métricas principais do relatório de Q1 2024",
  "kind": "analysis",
  "priority": "high",
  "assigned_agent_id": null,
  "metadata": { "report_url": "s3://bucket/q1-report.pdf" }
}
```

Valores de `kind`: `"analysis"` | `"write_document"` | `"review"` | `"email_read"` | `"health_check"` | `"sync"` | `"summarization"` | `"approval"` | `"custom:<nome>"`

Valores de `priority`: `"low"` | `"normal"` | `"high"` | `"critical"`

---

### `workspace.claim_task`

Reclama uma task aberta para trabalhar nela.

```json
{
  "task_id": "uuid-da-task",
  "agent_id": "meu-agente-1",
  "session_id": "uuid-da-sessao"
}
```

Falha com erro de concorrência se outro agente já reclamou a task.

---

### `workspace.update_task_status`

Atualiza o status de progresso de uma task.

```json
{
  "task_id": "uuid-da-task",
  "status": "in_progress",
  "metadata": { "progress": "50%", "note": "Processando página 3 de 6" }
}
```

Valores de `status`: `"open"` | `"claimed"` | `"in_progress"` | `"done"` | `"failed"` | `"cancelled"`

---

### `workspace.acquire_lock`

Adquire exclusividade sobre um recurso. Retorna erro `CONFLICT` se já estiver travado.

```json
{
  "scope_type": "document",
  "scope_id": "doc-42",
  "lock_type": "write_lock",
  "owner_agent_id": "meu-agente-1",
  "owner_session_id": "uuid-da-sessao",
  "ttl_secs": 120
}
```

Valores de `lock_type`: `"write_lock"` | `"soft_lock"` | `"topic_lock"` | `"artifact_lock"` | `"lease_lock"`

> **Importante:** sempre chame `workspace.release_lock` ao terminar. Locks expiram automaticamente após `ttl_secs`, mas liberar explicitamente é a prática correta.

---

### `workspace.release_lock`

Libera um lock adquirido.

```json
{
  "lock_id": "uuid-do-lock",
  "owner_session_id": "uuid-da-sessao"
}
```

---

### `workspace.create_handoff`

Cria um registro de handoff para o próximo agente que assumir este trabalho.

```json
{
  "from_agent_id": "meu-agente-1",
  "to_agent_id": null,
  "source_session_id": "uuid-da-sessao",
  "task_id": "uuid-da-task",
  "summary": "Completei a análise de janeiro. Os dados de fevereiro estão em s3://bucket/feb.",
  "payload": {
    "completed_months": ["2024-01"],
    "pending_months": ["2024-02", "2024-03"],
    "anomalies_found": 3
  }
}
```

---

### `workspace.list_handoffs`

Lista handoffs endereçados a este agente. Leia no início da sessão.

```json
{ "agent_id": "meu-agente-1" }
```

---

### `workspace.get_dependency`

Consulta o status de saúde de uma dependência externa.

```json
{ "key": "db:main" }
```

**Retorna:** `{ key, state, details, checked_at, updated_at }`

Valores de `state`: `"healthy"` | `"degraded"` | `"unhealthy"` | `"unknown"`

---

### `workspace.upsert_dependency`

Reporta o status de uma dependência que este agente monitora.

```json
{
  "key": "api:billing",
  "state": "degraded",
  "details": "Latência acima de 2s nas últimas 5 chamadas"
}
```

---

## Erros comuns

| Erro | Causa | Solução |
|------|-------|---------|
| `storage error: FOREIGN KEY constraint failed` | `check_in` com `agent_id` não registrado | Registrar o agente via `POST /agents` primeiro |
| `CONFLICT` em `acquire_lock` | Outro agente tem o lock | Aguardar e tentar de novo, ou usar TTL menor |
| `CONFLICT` em `claim_task` | Task já foi reclamada | Buscar outra task |
| `not found: session ...` | `heartbeat` com session expirada ou inválida | Fazer novo `check_in` |

---

## Padrão recomendado de startup

```
1. check_in(agent_id)                    → guarda session_id
2. list_handoffs(agent_id)               → lê contexto anterior
3. read_inbox(agent_id)                  → lê mensagens pendentes
4. ack_inbox cada item lido              → limpa inbox
5. [iniciar loop de heartbeat em background a cada 45s]
6. [iniciar trabalho principal]
```

## Padrão recomendado de shutdown

```
1. update_task_status(task_id, "done")   → se estava em uma task
2. release_lock(lock_id)                 → se tinha locks ativos
3. create_handoff(...)                   → se há trabalho a continuar
4. check_out(session_id, create_handoff=false)
```

---

## Padrão de coordenação emergente

Qualquer agente pode assumir o papel de coordenador sem configuração prévia.
O workspace não impõe quem coordena — quem lê o estado e age, coordena.

### Fluxo de coordenação

```
1. GET /summary
   → ver quantos agentes ativos, tasks abertas, inbox pendente

2. GET /tasks?unassigned=true
   → ver o que precisa ser feito

3. GET /agents
   → ver quem está disponível

4. POST /tasks/:id/assign  { "assigned_by": "coord-id", "assigned_to": "worker-id" }
   → delegar trabalho

5. POST /messages  { ..., "deliver_to_inbox": true }
   → notificar o agente assignado
```

Via MCP, use as tools `GetSummaryTool`, `ListTasksTool`, `AssignTaskTool`.

### Listar tasks com filtros (MCP / HTTP)

```json
GET /tasks?unassigned=true
GET /tasks?status=open,claimed
GET /tasks?assigned_to=worker-1
GET /tasks?limit=20
```

Via MCP, `ListTasksTool` aceita os mesmos filtros no payload.

### Assign task (coordenador)

```json
POST /tasks/:id/assign
{
  "assigned_by": "coordinator-1",
  "assigned_to": "worker-1"
}
```

Para desassignar: `"assigned_to": null`.

---

## Observabilidade

### Resumo do workspace

```
GET /summary
```

Snapshot em tempo real: agentes ativos, tasks abertas, inbox pendente, locks ativos.

### Eventos (audit trail)

```
GET /events                          → todos os eventos
GET /events?agent_id=meu-agente-1    → eventos de um agente específico
GET /events?agent_id=meu-agente-1&limit=50
```

Eventos são emitidos automaticamente pelo workspace:

- `session.checked_in` — ao fazer check-in
- `session.checked_out` — ao fazer check-out
- `message.sent` — ao enviar mensagem
- `task.claimed` — ao reclamar task
- `task.assigned` — ao assignar task via coordenador

### Sessões ativas

```
GET /sessions/active
```

Lista todas as sessões com status `active` no momento.

---

## Manutenção automática

O servidor roda um loop de manutenção a cada 60 segundos:

- Sessões sem heartbeat por mais de **5 minutos** são marcadas como `dead` e seus locks são liberados
- Locks além do TTL são expirados

Agentes devem enviar heartbeat ao menos a cada 5 minutos para manter a sessão ativa.

---

## Rodando os testes

```bash
# Testes da camada de storage (SQLite, banco em memória)
cargo test -p aw-storage-sqlite

# Testes da API HTTP (endpoints, banco em memória)
cargo test -p aw-api

# Todos de uma vez
cargo test -p aw-storage-sqlite -p aw-api
```

Os testes de integração usam banco SQLite em memória — sem setup, sem limpeza manual.
