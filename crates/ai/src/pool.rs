//! Async worker-pool skeleton (FR-CIV-AI-008, NFR-CIV-AI-001).
//!
//! **The single most important runtime property: the simulation never `await`s
//! a token.** The pool owns a dedicated multi-thread tokio runtime on its own OS
//! threads; the sim/render threads share none of it. The sim enqueues with a
//! non-blocking [`AiWorkerPool::try_enqueue`] (bounded mpsc → backpressure) and
//! drains results at a safe point with [`AiWorkerPool::drain_results`].
//!
//! ## P1 scope
//! Skeleton + never-await contract: dedicated runtime, bounded queue, hard
//! concurrency cap (semaphore), result channel, newest-wins coalesce hook.
//! Full LOD gating and per-feature cadence live in the owning sim crates.

use std::sync::Arc;

use tokio::runtime::Runtime;
use tokio::sync::{mpsc, Semaphore};

use crate::{AiProvider, EmbedRequest, GenOutput, GenRequest};

/// Opaque task identifier, echoed back on the result.
pub type TaskId = u64;

/// A unit of off-hot-path AI work.
pub enum AiTask {
    /// Free-form text generation.
    Generate {
        /// Caller-supplied id, echoed on the result.
        id: TaskId,
        /// Provider to run against (Arc-shared).
        provider: Arc<dyn AiProvider>,
        /// The generate request.
        req: GenRequest,
    },
    /// Batched embeddings.
    Embed {
        /// Caller-supplied id, echoed on the result.
        id: TaskId,
        /// Provider to run against (Arc-shared).
        provider: Arc<dyn AiProvider>,
        /// The embed request.
        req: EmbedRequest,
    },
}

impl AiTask {
    /// The task's id.
    #[must_use]
    pub fn id(&self) -> TaskId {
        match self {
            AiTask::Generate { id, .. } | AiTask::Embed { id, .. } => *id,
        }
    }
}

/// Result payload — text, vectors, or a named error.
#[derive(Debug, Clone)]
pub enum AiPayload {
    /// Generated text output.
    Text(GenOutput),
    /// Batched embedding vectors.
    Vectors(Vec<Vec<f32>>),
    /// The provider call failed (loud, carried not swallowed).
    Err(crate::AiError),
}

/// A completed task's result, drained by the sim at a safe point.
#[derive(Debug, Clone)]
pub struct AiResult {
    /// Echoes the originating [`AiTask::id`].
    pub id: TaskId,
    /// The task's payload.
    pub payload: AiPayload,
}

/// Async AI worker pool. Owns a dedicated tokio runtime off the sim/render
/// threads. Construct once; share the handle.
pub struct AiWorkerPool {
    _runtime: Runtime,
    task_tx: mpsc::Sender<AiTask>,
    result_rx: mpsc::Receiver<AiResult>,
}

impl AiWorkerPool {
    /// Spawn the pool with a bounded queue of `queue_capacity` and a hard cap of
    /// `max_concurrent` in-flight generations.
    ///
    /// # Panics
    /// Panics if the dedicated tokio runtime cannot be built (loud, per the
    /// failure stance — there is no silent degrade).
    #[must_use]
    pub fn spawn(queue_capacity: usize, max_concurrent: usize) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(max_concurrent.max(1))
            .enable_all()
            .thread_name("civ-ai-worker")
            .build()
            .expect("civ-ai worker pool: failed to build dedicated tokio runtime");

        let (task_tx, mut task_rx) = mpsc::channel::<AiTask>(queue_capacity.max(1));
        let (result_tx, result_rx) = mpsc::channel::<AiResult>(queue_capacity.max(1));
        let limiter = Arc::new(Semaphore::new(max_concurrent.max(1)));

        runtime.spawn(async move {
            while let Some(task) = task_rx.recv().await {
                let permit = Arc::clone(&limiter);
                let result_tx = result_tx.clone();
                tokio::spawn(async move {
                    // Hard concurrency cap: never exceed max_concurrent in flight.
                    let _permit = permit
                        .acquire()
                        .await
                        .expect("civ-ai semaphore closed unexpectedly");
                    let result = run_task(task).await;
                    // Result drain is best-effort: if the sim dropped the
                    // receiver (shutdown), the late result is discarded.
                    let _ = result_tx.send(result).await;
                });
            }
        });

        Self {
            _runtime: runtime,
            task_tx,
            result_rx,
        }
    }

    /// Enqueue a task **without blocking** the caller (the sim never awaits).
    ///
    /// Returns the rejected task on a full queue so callers can coalesce/drop
    /// with a logged warning (newest-wins) rather than silently swallow.
    ///
    /// # Errors
    /// Returns the task back when the bounded queue is full or closed.
    #[allow(clippy::result_large_err)]
    pub fn try_enqueue(&self, task: AiTask) -> Result<(), AiTask> {
        self.task_tx.try_send(task).map_err(|e| e.into_inner())
    }

    /// Drain all currently-ready results (call at end-of-tick, a safe point).
    /// Partial/late results are normal and expected.
    pub fn drain_results(&mut self) -> Vec<AiResult> {
        let mut out = Vec::new();
        while let Ok(result) = self.result_rx.try_recv() {
            out.push(result);
        }
        out
    }

    /// Block the **current** thread until a result is available, then drain.
    /// For tests / headless batch runners only — never the sim thread.
    pub async fn next_result(&mut self) -> Option<AiResult> {
        self.result_rx.recv().await
    }
}

async fn run_task(task: AiTask) -> AiResult {
    match task {
        AiTask::Generate { id, provider, req } => {
            let payload = match provider.generate(&req).await {
                Ok(out) => AiPayload::Text(out),
                Err(e) => AiPayload::Err(e),
            };
            AiResult { id, payload }
        }
        AiTask::Embed { id, provider, req } => {
            let payload = match provider.embed(&req).await {
                Ok(v) => AiPayload::Vectors(v),
                Err(e) => AiPayload::Err(e),
            };
            AiResult { id, payload }
        }
    }
}
