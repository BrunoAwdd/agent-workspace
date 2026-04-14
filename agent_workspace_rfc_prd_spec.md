# Agent Workspace Infrastructure — RFC, PRD e Technical Spec

## Status
Draft

## Autor
Bruno Oliveira

## Data
2026-04-13

---

# Parte I — RFC 0001

## 1. Título
RFC 0001 — Agent Workspace Infrastructure with Pluggable Persistence

## 2. Resumo
Este documento define a base conceitual de uma infraestrutura operacional para agentes de IA, com foco em presença, mensageria interna, inbox persistente, tarefas, locking contextual, handoff, memória compartilhada e execução condicionada.

A arquitetura deve permitir múltiplos backends de persistência, com suporte inicial a:

- SQLite, para modo local e de baixíssimo custo;
- PostgreSQL, para modo servidor, multiusuário e maior concorrência.

O sistema deverá ser consumível por MCP, CLI, integrações locais e ambientes de execução distribuídos.

## 3. Motivação
Ferramentas modernas de agentes, como ambientes estilo Claude Code, Cline, Antigravity e similares, funcionam melhor quando a experiência de uso é leve, simples e integrável. Exigir banco externo pesado desde a primeira execução pode reduzir adoção e aumentar fricção.

Por outro lado, ambientes multiagente reais exigem:

- continuidade operacional entre sessões;
- comunicação síncrona e assíncrona;
- locks;
- tarefas;
- trilha auditável;
- memória estruturada;
- retomada após falha ou indisponibilidade.

Este RFC resolve essa tensão propondo uma arquitetura de domínio estável com persistência intercambiável.

## 4. Decisão Arquitetural
O sistema não será definido a partir do banco de dados, mas a partir de um domínio operacional estável.

A persistência será tratada por adapters.

### 4.1 Backends iniciais suportados
- SQLite
- PostgreSQL

### 4.2 Princípio central
A semântica de domínio deve permanecer a mesma independentemente do backend escolhido.

### 4.3 Diferenças aceitáveis entre backends
Podem existir diferenças de implementação quanto a:

- estratégia de lock;
- notificações em tempo real;
- concorrência máxima;
- throughput;
- observabilidade.

Essas diferenças não devem alterar o comportamento funcional esperado da API de domínio.

## 5. Conceito do Produto
O sistema não deve ser tratado como “um banco para IA”, mas como uma infraestrutura operacional para agentes.

Ele deve permitir que agentes:

- façam check-in e check-out;
- mantenham presença operacional;
- conversem em channels e threads;
- recebam mensagens mesmo offline;
- processem inbox ao voltar;
- assumam tarefas;
- criem e liberem locks;
- deixem handoffs;
- registrem eventos auditáveis;
- consultem memória compartilhada.

## 6. Conceitos Centrais
### 6.1 Workspace
Contêiner lógico superior.

### 6.2 Channel
Espaço de comunicação dentro do workspace.

### 6.3 Thread
Discussão específica dentro de um channel.

### 6.4 Topic
Subcontexto usado para foco, locking e memória.

### 6.5 Agent
Entidade operacional com identidade, papel, capacidades e permissões.

### 6.6 Session
Instância viva de atividade de um agente.

### 6.7 Inbox Item
Mensagem, instrução ou tarefa entregue a um agente, inclusive offline.

### 6.8 Task
Unidade de trabalho operacional.

### 6.9 Lock
Reserva contextual para evitar colisão operacional.

### 6.10 Event
Registro imutável de ação relevante.

### 6.11 Handoff
Transferência de contexto entre sessões ou agentes.

### 6.12 Dependency
Estado de saúde de recurso externo necessário para execução.

## 7. Metas
1. Permitir modo local leve.
2. Permitir modo servidor robusto.
3. Permitir integração via MCP.
4. Permitir continuidade operacional real entre sessões.
5. Permitir mensageria assíncrona para agentes offline.
6. Permitir locks, tasks e audit trail.
7. Evitar acoplamento do domínio ao banco escolhido.

## 8. Não Metas
1. Não criar engine de banco própria nesta fase.
2. Não definir interface visual final.
3. Não resolver billing ou metering nesta etapa.
4. Não implementar política avançada de consenso multiagente nesta V1.

## 9. Modelo de Execução
### 9.1 Local Mode
Persistência em SQLite.

Características:
- baixo custo;
- zero infraestrutura externa obrigatória;
- ideal para uso individual, CLI, MCP local e desenvolvimento.

### 9.2 Server Mode
Persistência em PostgreSQL.

Características:
- melhor suporte a concorrência real;
- melhor adaptação a múltiplos agentes e usuários;
- melhor base para operação contínua, observabilidade e escala.

## 10. Princípios
1. Domain-first.
2. Storage-agnostic.
3. Async-native.
4. Presence-aware.
5. Event-oriented.
6. Minimal operational friction.

## 11. Conclusão do RFC
A infraestrutura será construída com semântica de domínio única, suportando SQLite e PostgreSQL como backends iniciais, com prioridade para integração via MCP e operação em modos local e servidor.

---

# Parte II — PRD

## 1. Product Requirements Document

## 2. Nome de Trabalho
Agent Workspace Infrastructure

## 3. Problema de Produto
Agentes atuais respondem, mas não trabalham de forma persistente como participantes operacionais de um sistema.

Problemas recorrentes:
- perdem contexto entre execuções;
- não possuem caixa postal persistente;
- não sabem retomar trabalho;
- não sabem lidar com indisponibilidade externa de forma organizada;
- não possuem coordenação nativa para locks e handoffs;
- dependem de presença simultânea para colaboração;
- sofrem com fricção de integração quando exigem infraestrutura pesada desde o início.

## 4. Visão do Produto
Construir uma infraestrutura operacional para agentes de IA onde o usuário possa escolher um modo leve com SQLite ou um modo robusto com PostgreSQL, mantendo a mesma experiência de domínio.

O produto deve funcionar bem em:
- ambientes locais com MCP;
- CLIs de desenvolvimento;
- ferramentas de agentes integradas a código;
- ambientes multiagente de servidor;
- futuras plataformas SaaS ou self-hosted.

## 5. Público-Alvo
### 5.1 Primário
- desenvolvedores que usam agentes localmente;
- usuários de MCP;
- arquitetos que desejam coordenação entre agentes;
- times que querem memória operacional multiagente.

### 5.2 Secundário
- empresas com múltiplos agentes especialistas;
- plataformas de automação;
- ferramentas de produtividade com IA persistente.

## 6. Jobs to Be Done
1. Como usuário, quero que meu agente volte e saiba o que ficou pendente.
2. Como usuário, quero deixar recados e instruções mesmo quando o agente estiver offline.
3. Como agente, quero fazer check-in e retomar trabalho com contexto.
4. Como agente, quero receber tarefas, locks e dependências de forma confiável.
5. Como time, queremos coordenação entre agentes sem precisar de setup pesado no modo local.
6. Como operador, quero migrar de SQLite para PostgreSQL sem mudar a semântica do sistema.

## 7. Proposta de Valor
Uma camada operacional para agentes com:
- presença;
- inbox persistente;
- mensagens internas;
- tasks;
- locks;
- handoffs;
- dependências;
- memória compartilhada;
- escolha entre modo leve e robusto.

## 8. Requisitos Funcionais
### 8.1 Presença
- agente deve poder fazer check-in;
- agente deve manter heartbeat;
- agente deve poder fazer check-out;
- sessão deve expirar ou ser marcada como morta quando necessário.

### 8.2 Mensageria
- agentes devem enviar mensagens para channels, threads e inbox direta;
- mensagens devem persistir;
- agentes offline devem receber mensagens futuras;
- mensagens estruturadas devem ser suportadas.

### 8.3 Inbox
- cada agente deve ter inbox própria;
- inbox deve suportar itens pendentes, processados, falhos e expirados;
- check-in deve reprocessar inbox elegível.

### 8.4 Tarefas
- criar task;
- atribuir task;
- assumir task;
- atualizar status;
- relacionar task a agentes, threads e artefatos.

### 8.5 Locks
- adquirir lock;
- renovar lock;
- liberar lock;
- expirar lock órfão.

### 8.6 Handoffs
- gerar handoff ao sair no meio do trabalho;
- transferir contexto entre agentes ou sessões.

### 8.7 Dependências
- registrar estado de saúde de recurso externo;
- permitir instruções condicionadas ao estado da dependência.

### 8.8 Eventos
- registrar trilha auditável de ações relevantes.

## 9. Requisitos Não Funcionais
- leveza em modo local;
- previsibilidade em modo servidor;
- observabilidade mínima;
- tolerância a falhas de sessão;
- API consistente entre backends;
- possibilidade de integração por MCP.

## 10. Experiência do Usuário
### 10.1 Fluxo local
Usuário instala ou integra a ferramenta, escolhe SQLite e começa a usar sem depender de infraestrutura externa.

### 10.2 Fluxo de equipe
Usuário ou empresa sobe o modo PostgreSQL e passa a operar múltiplos agentes e usuários com maior robustez.

## 11. KPIs Iniciais
- tempo de setup local;
- tempo até primeiro check-in funcional;
- taxa de reprocessamento bem-sucedido da inbox;
- taxa de locks órfãos;
- taxa de retomada bem-sucedida após falha;
- latência média para envio e leitura de mensagem;
- número de tarefas concluídas por sessão.

## 12. Critérios de Sucesso da V1
1. agente faz check-in e check-out com sucesso;
2. mensagens persistem e podem ser lidas depois;
3. inbox funciona para agentes offline;
4. tasks podem ser atribuídas e retomadas;
5. locks funcionam nos dois modos;
6. SQLite e PostgreSQL expõem a mesma semântica funcional;
7. integração MCP é viável.

---

# Parte III — Technical Spec

## 1. Arquitetura de Alto Nível
A arquitetura será dividida em:

- domain
- application
- storage
- mcp
- api opcional

## 2. Módulos Sugeridos em Rust
```text
crates/
  domain/
  application/
  storage/
    sqlite/
    postgres/
  mcp/
  api/
  common/
```

## 3. Camadas
### 3.1 Domain
Define entidades, enums, contratos e regras sem acoplamento ao banco.

### 3.2 Application
Orquestra casos de uso.

### 3.3 Storage
Implementa persistência por backend.

### 3.4 MCP
Expõe operações operacionais aos agentes externos.

### 3.5 API
Camada opcional HTTP/gRPC para operação server mode.

## 4. Entidades Principais
- Agent
- AgentSession
- Message
- InboxItem
- Task
- Lock
- Event
- Handoff
- Dependency
- Workspace
- Channel
- Thread
- Topic

## 5. Contrato de Storage
```rust
pub trait AgentStorage {
    async fn create_agent(&self, input: CreateAgentInput) -> Result<Agent>;
    async fn get_agent(&self, agent_id: &str) -> Result<Option<Agent>>;

    async fn check_in(&self, input: CheckInInput) -> Result<AgentSession>;
    async fn heartbeat(&self, input: HeartbeatInput) -> Result<AgentSession>;
    async fn check_out(&self, input: CheckOutInput) -> Result<()>;

    async fn send_message(&self, input: SendMessageInput) -> Result<Message>;
    async fn list_inbox(&self, agent_id: &str) -> Result<Vec<InboxItem>>;
    async fn ack_inbox_item(&self, input: AckInboxItemInput) -> Result<()>;

    async fn create_task(&self, input: CreateTaskInput) -> Result<Task>;
    async fn claim_task(&self, input: ClaimTaskInput) -> Result<Task>;
    async fn update_task_status(&self, input: UpdateTaskStatusInput) -> Result<Task>;

    async fn acquire_lock(&self, input: AcquireLockInput) -> Result<Lock>;
    async fn renew_lock(&self, input: RenewLockInput) -> Result<Lock>;
    async fn release_lock(&self, input: ReleaseLockInput) -> Result<()>;

    async fn append_event(&self, input: AppendEventInput) -> Result<Event>;

    async fn create_handoff(&self, input: CreateHandoffInput) -> Result<Handoff>;
    async fn list_handoffs(&self, agent_id: &str) -> Result<Vec<Handoff>>;

    async fn upsert_dependency(&self, input: UpsertDependencyInput) -> Result<Dependency>;
    async fn get_dependency(&self, key: &str) -> Result<Option<Dependency>>;
}
```

## 6. Modelo de Dados Lógico
### 6.1 agents
- id
- name
- role
- capabilities
- permissions
- status
- metadata
- created_at
- updated_at

### 6.2 agent_sessions
- id
- agent_id
- status
- started_at
- last_seen_at
- ended_at
- health
- current_task_id
- metadata

### 6.3 messages
- id
- workspace_id
- channel_id
- thread_id
- from_agent_id
- to_agent_id opcional
- kind
- payload
- created_at

### 6.4 inbox_items
- id
- target_agent_id
- source_agent_id opcional
- kind
- status
- payload
- deliver_on_checkin
- created_at
- processed_at opcional
- expires_at opcional

### 6.5 tasks
- id
- title
- description
- kind
- status
- priority
- assigned_agent_id opcional
- source_ref opcional
- metadata
- created_at
- updated_at

### 6.6 locks
- id
- scope_type
- scope_id
- lock_type
- owner_agent_id
- owner_session_id
- acquired_at
- expires_at
- metadata

### 6.7 events
- id
- workspace_id opcional
- agent_id opcional
- session_id opcional
- kind
- payload
- created_at

### 6.8 handoffs
- id
- from_agent_id
- to_agent_id opcional
- source_session_id
- task_id opcional
- summary
- payload
- created_at

### 6.9 dependencies
- key
- state
- details
- checked_at
- updated_at

## 7. Estratégia por Backend
### 7.1 SQLite
Uso pretendido:
- local mode;
- uso individual;
- MCP local;
- CLI;
- baixo custo.

Diretrizes:
- usar arquivo único por workspace ou por instalação;
- priorizar simplicidade operacional;
- aceitar concorrência limitada;
- locks podem usar tabela com lease e transação simples.

### 7.2 PostgreSQL
Uso pretendido:
- server mode;
- múltiplos agentes;
- múltiplos usuários;
- operação persistente.

Diretrizes:
- usar transações explícitas;
- usar JSONB para payloads flexíveis;
- usar índices adequados para inbox, sessions, tasks e locks;
- suportar maior concorrência e observabilidade.

## 8. Tipos de Mensagem
- chat_message
- review_request
- approval_request
- handoff_note
- alert
- status_update
- deferred_task
- conditional_instruction

## 9. Tipos de Task
- analysis
- write_document
- review
- email_read
- health_check
- sync
- summarization
- approval

## 10. Tipos de Lock
- write_lock
- soft_lock
- topic_lock
- artifact_lock
- lease_lock

## 11. Fluxos de Caso de Uso
### 11.1 Check-in
1. abrir sessão;
2. carregar inbox pendente;
3. carregar handoffs relevantes;
4. carregar tasks atribuídas em aberto;
5. registrar evento de check-in.

### 11.2 Heartbeat
1. atualizar last_seen_at;
2. atualizar status e health;
3. renovar leases quando necessário.

### 11.3 Check-out
1. registrar intenção de saída;
2. verificar tarefa aberta;
3. criar handoff se aplicável;
4. liberar locks;
5. encerrar sessão.

### 11.4 Mensagem assíncrona para agente offline
1. criar inbox_item;
2. marcar como deliver_on_checkin;
3. processar no próximo check-in do alvo.

### 11.5 Instrução condicional
1. criar inbox_item com kind conditional_instruction;
2. no check-in, validar preconditions;
3. se válido, executar action;
4. se inválido, registrar evento e manter ou remarcar conforme policy.

## 12. MCP Tools Iniciais
- agent.check_in
- agent.heartbeat
- agent.check_out
- agent.send_message
- agent.read_inbox
- task.create
- task.claim
- task.update_status
- lock.acquire
- lock.release
- handoff.create
- dependency.get
- dependency.upsert
- workspace.get_summary

## 13. Formato de Payloads
Payloads devem ser flexíveis, versionáveis e serializáveis.

Campos recomendados:
- type_version
- text opcional
- metadata opcional
- preconditions opcional
- action opcional
- policy opcional
- references opcional

## 14. Exemplo — Inbox Item Condicional
```json
{
  "id": "ibox_001",
  "target_agent_id": "agent_cron_email",
  "kind": "conditional_instruction",
  "status": "pending",
  "deliver_on_checkin": true,
  "payload": {
    "type_version": 1,
    "text": "Antes de tentar ler emails, verifique se o servidor foi normalizado.",
    "preconditions": [
      {
        "kind": "dependency_state",
        "dependency": "email_server",
        "expected": "healthy"
      }
    ],
    "action": {
      "kind": "create_task",
      "task_type": "email_read"
    },
    "policy": {
      "on_precondition_fail": "keep_pending"
    }
  },
  "created_at": "2026-04-13T00:00:00Z"
}
```

## 15. Estratégia de Migração
A migração entre SQLite e PostgreSQL deve ser possível por exportação/importação lógica baseada nas entidades de domínio.

Formato sugerido:
- JSON lines ou snapshots por entidade;
- comando de migração no CLI futuramente.

## 16. Estratégia de V1
### Entregar na V1
- domain model;
- trait de storage;
- sqlite adapter;
- postgres adapter;
- check-in/check-out/heartbeat;
- messages;
- inbox;
- tasks;
- locks;
- handoffs;
- dependencies;
- eventos básicos;
- MCP tools iniciais.

### Adiar
- policy engine complexa;
- consensus entre agentes;
- DAG de workflow avançada;
- memória vetorial nativa;
- interface gráfica sofisticada.

## 17. Open Questions
1. sessões simultâneas do mesmo agente serão permitidas?
2. usuários humanos terão a mesma modelagem RBAC que agentes?
3. locks pessimistas serão padrão em todos os escopos?
4. inbox deve suportar prioridade nativa?
5. tasks podem nascer automaticamente de mensagens estruturadas?
6. haverá event sourcing completo ou híbrido com estado materializado?

## 18. Conclusão da Spec
O sistema será projetado como infraestrutura operacional para agentes, com domínio estável, integração por MCP e persistência pluggable entre SQLite e PostgreSQL.

O foco da V1 será entregar uma base leve o suficiente para uso local e robusta o suficiente para evolução posterior em modo servidor.

