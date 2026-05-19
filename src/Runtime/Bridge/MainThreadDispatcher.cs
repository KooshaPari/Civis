#nullable enable
using System;
using System.Collections.Concurrent;
using System.Threading;
using System.Threading.Tasks;
using UnityEngine;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Dispatches work from background threads onto the Unity main thread.
    ///
    /// IMPORTANT: MonoBehaviour.Update() NEVER fires in DINO (custom PlayerLoop).
    /// The queue is pumped by <see cref="DrainQueue"/> which is called from
    /// <see cref="KeyInputSystem.OnUpdate"/> (ECS SystemBase — survives scene transitions).
    /// The MonoBehaviour.Update() method is kept as a fallback but is not relied upon.
    /// </summary>
    public class MainThreadDispatcher : MonoBehaviour
    {
        private static readonly ConcurrentQueue<Action> _queue = new ConcurrentQueue<Action>();

        /// <summary>
        /// Indicates whether the main-thread pump (DrainQueue) is being ticked.
        /// Set true by <see cref="MarkPumpAlive"/> from KeyInputSystem.OnUpdate; cleared
        /// by <see cref="MarkPumpDead"/> from KeyInputSystem.OnDestroy / RuntimeDriver tear-down.
        ///
        /// Background threads (bridge handlers, etc.) MUST check <see cref="IsPumpAlive"/>
        /// before issuing a blocking <see cref="Task.Wait(int)"/> on a queued work item:
        /// if the pump is dead, the work item will never run and the wait will burn its
        /// full timeout. <see cref="RunOnMainThread{T}(Func{T})"/> short-circuits with a
        /// faulted task in that case so callers fail fast instead of wedging the process.
        ///
        /// Marked volatile so the read/write pair is visible across threads without a lock.
        /// </summary>
        private static volatile bool _pumpIsAlive;

        /// <summary>Background-thread-safe view of <see cref="_pumpIsAlive"/>.</summary>
        public static bool IsPumpAlive => _pumpIsAlive;

        /// <summary>
        /// Called by the main-thread pump owner (KeyInputSystem.OnUpdate) on every tick
        /// to declare the pump is healthy. Cheap volatile write.
        /// </summary>
        public static void MarkPumpAlive() => _pumpIsAlive = true;

        /// <summary>
        /// Called when the pump owner is being torn down (KeyInputSystem.OnDestroy /
        /// RuntimeDriver destroy / scene transition mid-flight) so subsequent
        /// <see cref="RunOnMainThread{T}(Func{T})"/> calls can short-circuit instead of
        /// queuing work that will never be drained.
        /// </summary>
        public static void MarkPumpDead() => _pumpIsAlive = false;

        /// <summary>
        /// Drains the pending action queue. Called from ECS SystemBase.OnUpdate()
        /// (KeyInputSystem) which fires reliably on the main thread.
        /// Also called from MonoBehaviour.Update() as a fallback (rarely fires in DINO).
        /// </summary>
        public static void DrainQueue()
        {
            // Drains imply we're being ticked on the main thread — re-arm the alive flag.
            _pumpIsAlive = true;

            int processed = 0;
            while (_queue.TryDequeue(out Action? action))
            {
                try
                {
                    action.Invoke();
                }
                catch (Exception ex)
                {
                    Debug.LogError($"[MainThreadDispatcher] Exception in queued action: {ex}");
                }

                processed++;
                if (processed > 100)
                    break;
            }
        }

        /// <summary>
        /// MonoBehaviour.Update() fallback — rarely fires in DINO but kept for safety.
        /// </summary>
        private void Update()
        {
            DrainQueue();
        }

        /// <summary>
        /// Enqueue an action to run on the Unity main thread. The result is delivered
        /// through the provided <see cref="TaskCompletionSource{T}"/>.
        /// </summary>
        /// <typeparam name="T">The return type of the work.</typeparam>
        /// <param name="work">The function to execute on the main thread.</param>
        /// <param name="tcs">The TaskCompletionSource to signal when work completes.</param>
        public static void Enqueue<T>(Func<T> work, TaskCompletionSource<T> tcs)
        {
            _queue.Enqueue(() =>
            {
                try
                {
                    T result = work();
                    tcs.TrySetResult(result);
                }
                catch (Exception ex)
                {
                    tcs.TrySetException(ex);
                }
            });
        }

        /// <summary>
        /// Schedule a function to run on the Unity main thread and return a Task
        /// that completes with the result. Safe to call from any thread.
        /// </summary>
        /// <typeparam name="T">The return type.</typeparam>
        /// <param name="work">The function to execute on the main thread.</param>
        /// <returns>A task that completes when the work finishes on the main thread.</returns>
        public static Task<T> RunOnMainThread<T>(Func<T> work)
        {
            // tcs-sync-continuation-ok: Intentional sync continuations on main thread for thread-safety.
            TaskCompletionSource<T> tcs = new TaskCompletionSource<T>();

            // Fast-fail when the main-thread pump is known dead (e.g. KeyInputSystem
            // destroyed during scene transition). Without this, callers that block on
            // .Wait(timeout) would burn their full timeout queueing work that can never
            // run, wedging the bridge thread and ultimately the game process.
            if (!_pumpIsAlive)
            {
                tcs.TrySetException(new InvalidOperationException(
                    "MainThreadDispatcher pump is not running (KeyInputSystem.OnUpdate has not ticked recently). " +
                    "Work item rejected to avoid indefinite wait."));
                return tcs.Task;
            }

            Enqueue(work, tcs);
            return tcs.Task;
        }

        /// <summary>
        /// Schedule an action (no return value) to run on the Unity main thread.
        /// </summary>
        /// <param name="action">The action to execute on the main thread.</param>
        /// <returns>A task that completes when the action finishes.</returns>
        public static Task RunOnMainThread(Action action)
        {
            // tcs-sync-continuation-ok: Intentional sync continuations on main thread for thread-safety.
            TaskCompletionSource<bool> tcs = new TaskCompletionSource<bool>();

            // Fast-fail when the main-thread pump is known dead — see RunOnMainThread<T>.
            if (!_pumpIsAlive)
            {
                tcs.TrySetException(new InvalidOperationException(
                    "MainThreadDispatcher pump is not running (KeyInputSystem.OnUpdate has not ticked recently). " +
                    "Work item rejected to avoid indefinite wait."));
                return tcs.Task;
            }

            _queue.Enqueue(() =>
            {
                try
                {
                    action();
                    tcs.TrySetResult(true);
                }
                catch (Exception ex)
                {
                    tcs.TrySetException(ex);
                }
            });
            return tcs.Task;
        }
    }
}
