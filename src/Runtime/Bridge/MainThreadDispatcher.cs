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
        /// Drains the pending action queue. Called from ECS SystemBase.OnUpdate()
        /// (KeyInputSystem) which fires reliably on the main thread.
        /// Also called from MonoBehaviour.Update() as a fallback (rarely fires in DINO).
        /// </summary>
        public static void DrainQueue()
        {
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
            TaskCompletionSource<T> tcs = new TaskCompletionSource<T>();
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
            TaskCompletionSource<bool> tcs = new TaskCompletionSource<bool>();
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
