# Pattern #226 Event Field Exemptions

## Rationale
Events in C# have built-in encapsulation via the event keyword. Callers can only use += and -= operators; direct assignment is compile-time prevented. Wrapping public event fields in properties provides no additional protection and is non-idiomatic C#.

## Exemptions
- SDK/HotReload/PackFileWatcher.cs:34 OnPackContentChanged (inline: public-field-ok marker)
- SDK/HotReload/PackFileWatcher.cs:37 OnPackReloaded (inline: public-field-ok marker)
- SDK/HotReload/PackFileWatcher.cs:40 OnPackReloadFailed (inline: public-field-ok marker)

