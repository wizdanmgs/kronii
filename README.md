# Kronii

A distributed job scheduler built with Rust.

It supports:

- Worker pool (bounded concurrency)
- PostgreSQL persistence
- PostgreSQL advisory locks (distributed locking)
- Cron-based scheduling
- YAML job definitions
- Metrics endpoint (Prometheus-compatible)
- Graceful shutdown

---

## Architecture Overview

This scheduler is designed to run **multiple instances safely**.

Each instance:

1. Loads jobs from `jobs.yaml`
2. Persists jobs into PostgreSQL
3. Uses PostgreSQL advisory locks to ensure only ONE node executes a job
4. Executes jobs using a worker pool
5. Exposes metrics over HTTP

---

### Metrics Endpoint

Prometheus-style metrics available at:

```
GET /metrics
```

Example:

```
scheduler_jobs_total 10
scheduler_jobs_running 2
scheduler_jobs_failed 1
scheduler_jobs_success 7
```

---

## Database Setup

Create PostgreSQL database:

```bash
createdb scheduler
```

Run schema:

```sql
CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    cron TEXT NOT NULL,
    next_run TIMESTAMP NOT NULL,
    last_run TIMESTAMP,
    status TEXT NOT NULL
);
```

---

## Configuration

Environment variables:

| Variable     | Description                  | Default  |
| ------------ | ---------------------------- | -------- |
| DATABASE_URL | PostgreSQL connection string | required |
| WORKER_COUNT | Max concurrent jobs          | 5        |
| METRICS_PORT | Metrics server port          | 3000     |

Example:

```bash
export DATABASE_URL=postgres://user:password@localhost/scheduler
export WORKER_COUNT=5
export METRICS_PORT=3000
```

---

## jobs.yaml Example

```yaml
jobs:
  - name: cleanup_temp
    cron: "0/30 * * * * *"
  - name: sync_data
    cron: "0 */1 * * * *"
  - name: email_digest
    cron: "0 0 */1 * * *"
  - name: analytics_rollup
    cron: "0 15 * * * *"
  - name: billing_cycle
    cron: "0 0 0 * * *"
  - name: cache_warmup
    cron: "0/45 * * * * *"
  - name: log_rotation
    cron: "0 0 1 * * *"
  - name: health_check
    cron: "0/20 * * * * *"
  - name: data_backup
    cron: "0 0 3 * * *"
  - name: notification_retry
    cron: "0/10 * * * * *"
```

---

## Run the Scheduler

```bash
cargo run
```

---

## Run Multiple Instances (Distributed Mode)

Simply start multiple processes:

```bash
cargo run
cargo run
cargo run
```

Advisory locks ensure only one executes a given job.

---

## Scaling Strategy

Horizontal scaling:

- Add more scheduler instances
- Increase WORKER_COUNT
- PostgreSQL handles coordination

---

## Failure Handling

If a node crashes:

- Advisory lock is released automatically
- Another node picks up execution
- next_run ensures no job is lost
