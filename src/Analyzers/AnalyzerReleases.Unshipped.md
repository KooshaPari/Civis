## Unshipped Analyzer Rules

### Tier 1 Rules (Syntactic)

Rule ID | Category | Severity | Notes
--------|----------|----------|-------
DF0094  | Design   | Warning  | Unbounded version constraint - detects version constraints without upper bounds (Pattern #94)
DF0096  | Logging  | Warning  | LogError discards exception stack trace - detects ex.Message member access, $"{ex.Message}" interpolation, and "msg" + ex.Message concatenation patterns in LogError/LogCritical/LogWarning/LogException calls. Suppress with `// pattern-96-ok: <reason>`. Mirrors detect_logerror_no_stack.py (Pattern #96). Formalized as Tier 1 in iter-144 with 19 firing tests.
DF0097  | Concurrency | Warning | TaskCompletionSource missing RunContinuationsAsynchronously - detects sync-continuation deadlock risks
DF0098  | Async    | Info     | await missing ConfigureAwait(false) in library code - detects library-scope awaits that should suppress context capture (Pattern #98)
DF0099  | Performance | Warning | Dictionary<string, T> without explicit StringComparer - detects unprotected string keys requiring Ordinal comparison
DF0102  | Resource Management | Warning | Process.Start without using or assignment - detects handle leaks on discarded Process.Start calls
DF0103  | Performance | Info    | DateTime.Now used in logging context - detects local-time-dependent timestamps (Pattern #103)
DF0105  | Resource Management | Warning | Event subscription without matching unsubscribe - detects += without matching -= in cleanup methods (Pattern #105)
DF0106  | Reliability | Warning  | File.ReadAllText/WriteAllText without explicit Encoding - detects implicit system-default encoding that causes silent data loss on non-UTF-8 systems (Pattern #106)
DF0108  | Reliability | Warning | Sleep-based test sync in test method - detects fragile fixed-duration delays (Pattern #108)
DF0111  | Observability | Warning | Empty catch block silently swallows exceptions - detects bare catch {} blocks without logging
DF0114  | Async    | Warning  | CancellationToken not threaded to inner async call - detects await calls that don't pass CT parameter
DF0116  | Reliability | Warning | Sync-over-async blocking (.Result/.Wait()) - detects blocking on tasks that risk deadlock without captured context
DF0117  | Performance | Info    | StringBuilder created without capacity hint - detects pre-sizing opportunities
DF0120  | Serialization | Warning | JsonSerializer.Deserialize without explicit options - detects calls missing canonical JsonSerializerOptions
DF0123  | NuGetAPI | Warning  | Public class exposes mutable collection property - detects public List<T>/IList<T>/Collection<T>/ICollection<T> properties that break encapsulation

### Tier 2 Rules (Semantic)

Rule ID | Category | Severity | Notes
--------|----------|----------|-------
DF1001  | Concurrency | Warning | Static mutable collection field modified without lock - detects `static List<T>`/`Dictionary<K,V>`/`HashSet<T>` fields prone to race conditions (Tier 2 prototype)
DF1002  | Resource Management | Info | Static event subscription without weak reference - detects subscriptions to long-lived events (SceneManager.sceneLoaded, AppDomain.UnhandledException, etc.) from instance objects (Tier 2 prototype)
DF1003  | Concurrency | Warning | `await` inside `lock` block - detects Monitor-based locks with await continuations that risk IllegalMonitorStateException (Tier 2 prototype)
DF1004  | Performance | Info | Task.WhenAll over potentially unbounded enumeration - detects `Task.WhenAll(items.Select(...))` patterns that allocate unbounded tasks; recommends `Parallel.ForEachAsync` for >10 items (Tier 2 prototype)
DF1005  | Reliability | Warning | `async void` method outside event-handler context - detects `async void` methods that are not legitimate event handlers; recommends `async Task` instead (Tier 2 prototype)
DF1006  | Reliability | Warning | Disposable field in class not implementing IDisposable - detects `private`/`static` fields of disposable types (Stream, Reader, Client, Pipe, etc.) in classes without IDisposable implementation; recommends implementing IDisposable or using field initializer with `using` (Tier 2 prototype)
DF1007  | Reliability | Warning | Float ==/!= comparison without tolerance - detects direct `==`/`!=` comparisons of float/double/decimal types without Math.Abs tolerance; suggests tolerance-based equality for precision-loss prevention in game balance/damage/range checks (Tier 2 prototype)
DF1008  | Reliability | Info    | Dictionary[key] without TryGetValue/ContainsKey guard - detects `dict[key]` indexing on untrusted/user-sourced keys without safety check; recommends `TryGetValue` + explicit missing-key handling to avoid KeyNotFoundException leakage (Tier 2 prototype)
DF1009  | Reliability | Warning | Enum.Parse without TryParse fallback - detects `Enum.Parse<TEnum>(string)` calls on user-sourced data (YAML enums, JSON discriminators, pack content) without fallback; recommends `Enum.TryParse(...)` + explicit error handling (Tier 2 prototype)
DF1010  | Reliability | Warning | Async lambda assigned to Action / fire-and-forget - detects `Action async () => { await Foo(); }` patterns that discard Task and make exceptions unobservable; recommends `Func<Task>` or `Task.Run(async () => { ... })` with error handling (Tier 2 prototype)
