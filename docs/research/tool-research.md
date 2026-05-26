# Tool Research: Relevant Technologies for DINOForge

> Research compiled from web search to identify tools, libraries, and technologies relevant to this Unity modding project.

---

## 1. Modding Frameworks

| Tool | Description | Status |
|------|-------------|--------|
| **BepInEx** | Unity/XNA plugin framework (core dependency) | Already using |
| **Harmony** | Runtime patching library | Already using |
| **MonoMod** | IL2CPP support | Available |
| **Lib.Harmony** | Legacy patching | Available |

**Sources:** BepInEx GitHub, BepInEx Documentation

---

## 2. Game Testing/Automation

| Tool | Description | Relevance |
|------|-------------|-----------|
| **AltTester** | Unity-specific UI automation, element inspection | HIGH |
| **Unity Test Framework** | Built-in Unity testing | HIGH |
| **GameDriver** | Record/playback game automation | HIGH |
| **Unium** | HTTP/JSON automation for Unity games | HIGH |
| **T-Plan** | Visual-based game testing | MEDIUM |
| **Playwright** | Browser automation (reference) | MEDIUM |
| **Ranorex** | General UI automation | LOW |

**Sources:** AltTester Official, Unity Test Framework Docs, GameDriver

---

## 3. JSON/YAML Serialization

| Tool | Description | Status |
|------|-------------|--------|
| **Newtonsoft.Json** | JSON serialization | Already using |
| **YamlDotNet** | YAML serialization | Already using |
| **NJsonSchema** | JSON Schema validation | Already using |
| **System.Text.Json** | Built-in .NET JSON | Available |

**Sources:** NuGet Gallery, Unity Asset Store

---

## 4. Asset Management

| Tool | Description | Status |
|------|-------------|--------|
| **Unity Addressables** | Runtime asset loading | Already using |
| **Asset Bundles** | Legacy asset packaging | Reference |
| **AssimpNet** | 3D model import (GLB/FBX) | Already using |

**Sources:** Unity Addressables Documentation

---

## 5. Interprocess Communication

| Tool | Description | Status |
|------|-------------|--------|
| **Named Pipes** | JSON-RPC transport | Already using |
| **gRPC** | RPC framework | Available |
| **ZeroMQ** | Message queue | Available |
| **SignalR** | Real-time communication | Available |

**Sources:** Microsoft Docs, Stack Overflow

---

## 6. CLI & Build Tools

| Tool | Description | Status |
|------|-------------|--------|
| **dotnet CLI** | .NET SDK | Already using |
| **System.CommandLine** | CLI framework | Already using |
| **Spectre.Console** | Terminal UI | Already using |
| **thunderstore-cli** | Mod package publishing | Available |
| **r2mod_cli** | Mod management CLI | Available |

**Sources:** Thunderstore GitHub

---

## 7. Schema Validation

| Tool | Description | Status |
|------|-------------|--------|
| **NJsonSchema** | JSON Schema validation | Already using |
| **JsonSchema.Net** | .NET JSON Schema | Available |
| **FluentValidation** | Object validation | Available |
| **SchematicNet** | YAML schema validation | Available |

**Sources:** NuGet Gallery

---

## 8. Testing Frameworks

| Tool | Description | Status |
|------|-------------|--------|
| **xUnit** | Test framework | Already using |
| **FluentAssertions** | Assertion library | Already using |
| **Moq** | Mocking library | Available |
| **NSubstitute** | Mocking alternative | Available |

**Sources:** xUnit Official

---

## 9. Unity ECS Tools

| Tool | Description | Source |
|------|-------------|--------|
| **Unity.Entities** | ECS framework | Unity Docs |
| **Entity Debugger** | In-editor ECS debugging | Unity Docs |
| **Burst Compiler** | Performance optimization | Unity Docs |
| **Jobs System** | Multi-threading | Unity Docs |

**Sources:** Unity ECS Documentation

---

## 15. MCP Servers for Unity (HIGHLY RELEVANT)

| Server | Description | Relevance |
|-------|-------------|-----------|
| **IvanMurzak/Unity-MCP** | Full-featured Unity MCP server (1.3k stars, 2.6k commits) - Tools, Resources, Prompts for Unity Editor | **DIRECTLY RELEVANT** |
| **CoderGamester/mcp-unity** | MCP plugin for Unity Editor integration | **DIRECTLY RELEVANT** |
| **CoplayDev/unity-mcp** | Unity MCP bridge for AI assistants | **DIRECTLY RELEVANT** |
| **azreal42-unity-game-engine** | TCP-based Unity MCP with scene manipulation | **DIRECTLY RELEVANT** |
| **coolmew/Unity-MCP** | Script inspection, asset management | **DIRECTLY RELEVANT** |
| **mitchchristow/unity-mcp** | Unity 6+ MCP server | **DIRECTLY RELEVANT** |
| **arodoid-unity** | Real-time Unity Editor interaction | **DIRECTLY RELEVANT** |

### Unity-MCP Key Features (from IvanMurzak/Unity-MCP):

**MCP Tools (Actions):**
- `get_scene_hierarchy` - Get scene hierarchy
- `get_game_objects` - Get GameObjects
- `get_components` - Get components
- `execute_method` - Execute methods
- `create_game_object` - Create GameObjects
- `add_component` - Add components
- `set_property` - Set properties
- `build_project` - Build project
- `open_scene` - Open scenes
- `get_project_settings` - Project settings

**MCP Resources (Read-only):**
- Scene hierarchy
- GameObject properties
- Component details
- Asset information

**MCP Prompts:**
- Coding conventions
- Project structure context
- Workflow instructions

**Sources:** IvanMurzak/Unity-MCP GitHub (1.3k stars)

---

## 16. Godot MCP Servers

---

## 25. MCP Frameworks (for building MCP servers)

| Framework | Language | Description |
|-----------|----------|-------------|
| **FastMCP 3.0** | Python | **RECOMMENDED** - Fast, Pythonic MCP server framework |
| **python-sdk** | Python | Official MCP Python SDK |
| **modelcontextprotocol** | TypeScript | Official MCP TypeScript SDK |
| **mcp-typescript** | TypeScript | TypeScript MCP types |

### FastMCP 3.0 (HIGHLY RECOMMENDED)

**Why FastMCP:**
- Native OpenTelemetry tracing
- Response size limiting
- Background tasks via Docker
- Security fixes
- Fast, Pythonic API

**Installation:**
```bash
pip install fastmcp
```

**Example Structure:**
```python
from fastmcp import FastMCP

mcp = FastMCP("DINOForge")

@mcp.tool()
async def get_game_status():
    """Get current game status"""
    return {"status": "running", "entities": 123}

@mcp.resource("game://scene")
async def get_scene():
    """Get current scene info"""
    return {"name": "main", "objects": [...]}

@mcp.prompt()
def game_debug_prompt():
    return "You are debugging a Unity game. Current issue: {issue}"
```

**Sources:** 
- https://github.com/PrefectHQ/fastmcp
- https://gofastmcp.com/
- https://www.jlowin.dev/blog/fastmcp-3-launch

---

## 26. Architecture Comparison: Editor vs Game MCP

| Aspect | IvanMurzak/Unity-MCP | DINOForge Bridge |
|--------|---------------------|------------------|
| **Runs in** | Unity Editor (plugin) | Game process (BepInEx) |
| **Language** | C# + TypeScript CLI | C# (bridge) + CLI |
| **MCP Framework** | Custom | FastMCP (if added) |
| **Access** | Scene hierarchy, GameObjects | ECS entities, game state |
| **Tools** | create_game_object, execute_method | query, override, ui-* |
| **Resources** | Scene, Assets, Components | Entities, Resources |

### Key Insight

Both follow the **MCP pattern** but at different injection points:
- **Editor MCP** = AI talks to Unity Editor
- **Game MCP** = AI talks to running game

If adding MCP back to DINOForge:
```
FastMCP (Python) → GameClient (C#) → Named Pipe → GameBridgeServer (C#)
```

This gives you the best of both worlds:
- FastMCP 3.0 for modern MCP infrastructure
- Existing C# game bridge logic preserved

---

## 27. Full MCP Server Architecture for DINOForge

### Overview Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Claude Code / AI Agent                      │
└──────────────────────────────┬──────────────────────────────────┘
                               │ stdio / SSE
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    FastMCP Server (Python)                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   Tools     │  │  Resources  │  │   Prompts   │  │
│  │             │  │             │  │             │  │
│  │ game_status │  │ game://state│  │debug_convention│ │
│  │ spawn_unit  │  │ game://ecs  │  │              │  │
│  │ query       │  │ game://ui   │  │              │  │
│  │ ui_automation│ │ game://packs│  │              │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────────┘  │
│         │                │                │                   │
│         └────────────────┼────────────────┘                   │
│                          │                                    │
│                   ┌──────▼──────┐                          │
│                   │ GameClient  │  ← C# Bridge Client   │
│                   │  (Python)  │                      │
│                   └──────┬──────┘                          │
└──────────────────────────┼──────────────────────────────────┘
                           │ JSON-RPC / Named Pipe
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                 GameBridgeServer (C# - BepInEx)                  │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │  Handle methods:                                        │    │
│  │  - get_game_status    → Unity game state               │    │
│  │  - query_entities     → ECS entity queries            │    │
│  │  - get_ui_tree        → UGUI hierarchy                 │    │
│  │  - ui-query/click   → UI automation               │    │
│  │  - apply_override    → Stat modifications            │    │
│  │  - reload_packs      → Hot reload                   │    │
│  └──────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

### Option 1: FastMCP → GameClient (Mixed Stack)

**FastMCP Server (Python):**
```python
# dinoforge_mcp/server.py
from fastmcp import FastMCP
from dinoforge_bridge import GameClient
import asyncio

mcp = FastMCP("DINOForge")

# Initialize game client connection
_game_client = None

def get_client() -> GameClient:
    global _game_client
    if _game_client is None:
        _game_client = GameClient("dinoforge_game")
    return _game_client

# === TOOLS ===

@mcp.tool()
async def game_status() -> dict:
    """Check if game is running and get status"""
    client = get_client()
    return await client.get_status_async()

@mcp.tool()
async def query_entities(component_type: str) -> dict:
    """Query ECS entities by component type"""
    client = get_client()
    return await client.query_entities_async(component_type)

@mcp.tool()
async def spawn_unit(unit_id: str, x: float = 0, z: float = 0) -> dict:
    """Spawn a unit at position"""
    client = get_client()
    return await client.spawn_unit_async(unit_id, x, z)

@mcp.tool()
async def ui_tree(selector: str = None) -> dict:
    """Capture UI hierarchy snapshot"""
    client = get_client()
    return await client.get_ui_tree_async(selector)

@mcp.tool()
async def ui_query(selector: str) -> dict:
    """Query UI elements by selector"""
    client = get_client()
    return await client.query_ui_async(selector)

@mcp.tool()
async def ui_click(selector: str) -> dict:
    """Click UI element"""
    client = get_client()
    return await client.click_ui_async(selector)

@mcp.tool()
async def ui_wait(selector: str, state: str = "visible", timeout: int = 5000) -> dict:
    """Wait for UI state"""
    client = get_client()
    return await client.wait_for_ui_async(selector, state, timeout)

@mcp.tool()
async def ui_expect(selector: str, condition: str) -> dict:
    """Assert UI condition"""
    client = get_client()
    return await client.expect_ui_async(selector, condition)

@mcp.tool()
async def apply_override(path: str, value: float) -> dict:
    """Apply stat override"""
    client = get_client()
    return await client.apply_override_async(path, value)

@mcp.tool()
async def reload_packs() -> dict:
    """Reload all mod packs"""
    client = get_client()
    return await client.reload_packs_async()

@mcp.tool()
async def screenshot() -> dict:
    """Capture game screenshot"""
    client = get_client()
    return await client.screenshot_async()

# === RESOURCES ===

@mcp.resource("game://status")
async def game_status_resource() -> dict:
    """Current game status resource"""
    client = get_client()
    return await client.get_status_async()

@mcp.resource("game://entities/{component_type}")
async def entities_resource(component_type: str) -> dict:
    """ECS entities by type"""
    client = get_client()
    return await client.query_entities_async(component_type)

@mcp.resource("game://ui-tree")
async def ui_tree_resource() -> dict:
    """Full UI hierarchy"""
    client = get_client()
    return await client.get_ui_tree_async()

@mcp.resource("game://packs")
async def packs_resource() -> dict:
    """Loaded mod packs"""
    client = get_client()
    return await client.get_packs_async()

@mcp.resource("game://resources")
async def resources_resource() -> dict:
    """Game resources"""
    client = get_client()
    return await client.get_resources_async()

# === PROMPTS ===

@mcp.prompt()
def debug_convention() -> str:
    return """You are debugging DINOForge mods.

When analyzing issues:
1. Check game status with game_status
2. Query relevant ECS entities
3. Use ui_tree to inspect UI state
4. Apply test overrides to reproduce

Always include entity counts and ECS query results."""

@mcp.prompt()
def testing_convention() -> str:
    return """You are testing DINOForge UI automation.

Test patterns:
1. ui_tree → get baseline
2. ui_wait for panels to appear
3. ui_click to interact
4. ui_expect to verify state
5. screenshot for visual confirmation"""

if __name__ == "__main__":
    mcp.run()
```

**Python GameClient Wrapper:**
```python
# dinoforge_bridge/__init__.py
import asyncio
import json
import os

class GameClient:
    def __init__(self, pipe_name: str = "dinoforge_game"):
        self.pipe_name = pipe_name
    
    async def _call(self, method: str, **params):
        # Use Windows named pipe or TCP
        # Simplified: spawn CLI process
        import subprocess
        result = subprocess.run([
            "dotnet", "run", "--project", "src/Tools/Cli",
            "--", method, *self._args(params)
        ], capture_output=True, text=True)
        return json.loads(result.stdout)
    
    async def get_status_async(self):
        return await self._call("status")
    
    async def query_entities_async(self, component_type: str):
        return await self._call("query", type=component_type)
    
    # ... etc for all methods
```

### Option 2: Pure C# MCP (If no Python wanted)

Using C# MCP SDK directly:
```
C# MCP Server → GameBridgeServer (same process)
```

### Running the MCP Server

```bash
# Option 1: Standalone
python -m dinoforge_mcp.server

# Option 2: Via Claude Code config
# .claude/mcp-servers.json:
{
  "mcpServers": {
    "dinoforge": {
      "command": "python",
      "args": ["-m", "dinoforge_mcp.server"],
      "env": {
        "DINOFORGE_PIPE": "dinoforge_game"
      }
    }
  }
}
```

### Tool Definitions for Claude

```json
{
  "tools": [
    {
      "name": "game_status",
      "description": "Check if game is running",
      "inputSchema": {
        "type": "object",
        "properties": {}
      }
    },
    {
      "name": "query_entities",
      "description": "Query ECS entities by component type",
      "inputSchema": {
        "type": "object",
        "properties": {
          "component_type": {"type": "string"}
        }
      }
    },
    {
      "name": "ui_tree",
      "description": "Capture UI hierarchy snapshot",
      "inputSchema": {
        "type": "object",
        "properties": {
          "selector": {"type": "string"}
        }
      }
    },
    {
      "name": "ui_click",
      "description": "Click UI element by selector",
      "inputSchema": {
        "type": "object",
        "properties": {
          "selector": {"type": "string"}
        },
        "required": ["selector"]
      }
    }
  ]
}
```

| Server | Description | Tools |
|--------|-------------|-------|
| **tugcantopaloglu/godot-mcp** | Full Godot 4.x control | 149 tools |
| **ee0pdt/Godot-MCP** | Scene/resource/script creation | Various |
| **GDAI MCP** | All-in-one Godot AI integration | Comprehensive |
| **Coding-Solo/godot-mcp** | Editor launch, project run, debug | Basic |
| **bradypp/godot-mcp** | Godot engine interaction | Various |

---

## 17. Other Game Engine MCPs

| Engine | Server | Description |
|--------|--------|-------------|
| **Unreal Engine** | Various community projects | In development |
| **Defold** | defold-mcp | Defold game engine |
| **GameMaker** | gmaker-mcp | GameMaker Studio |

---

## 18. Game Development Workflow Tools

| Category | Tools |
|----------|-------|
| **Asset Pipelines** | TexturePacker, DragonBones, Spine |
| **Level Editors** | Tiled, LDtk, Ogmo |
| **Spritesheet Tools** | Aseprite, Piskel, Leshy |
| **Audio** | FMOD, Wwise, Audacity |
| **Version Control** | Git LFS, GitHub Game CI |

---

## 19. CI/CD for Game Development

| Tool | Description |
|------|-------------|
| **GitHub Actions** | Build automation for games |
| **TeamCity** | Game CI/CD by JetBrains |
| **Jenkins** | Open-source CI/CD |
| **GitLab CI** | Git-based automation |
| **Game CI** | Specialized game build automation |

---

## 20. IDEs & Editors for Unity

| Tool | Description | Relevance |
|------|-------------|-----------|
| **JetBrains Rider** | Full Unity debugging, testing, profiling | HIGH |
| **VS Code** | Lightweight with extensions | MEDIUM |
| **Visual Studio** | Standard Unity development | MEDIUM |

**Rider Features:**
- Unity-specific debugging
- Unit test running
- Code coverage
- dotTrace profiling
- In-editor Unity console
- Shader debugging

---

## 21. Code Quality Tools

| Tool | Description |
|------|-------------|
| **SonarQube** | Static analysis |
| **ReSharper** | Code quality |
| **Roslynator** | C# analyzers |
| **StyleCop** | Code style |
| **dotnet format** | Code formatting |

---

## 22. NuGet Packages (Game Dev)

| Package | Purpose |
|---------|----------|
| **Newtonsoft.Json** | JSON serialization |
| **YamlDotNet** | YAML serialization |
| **NJsonSchema** | Schema validation |
| **Serilog** | Structured logging |
| **Autofac** | Dependency injection |
| **Polly** | Resilience patterns |

---

## 23. Mocking & Testing Libraries

| Tool | Purpose |
|------|----------|
| **Moq** | Mocking framework |
| **NSubstitute** | Mocking alternative |
| **FluentAssertions** | Assertion library |
| **Bogus** | Fake data generation |
| **AutoFixture** | Test data |

---

## 24. Unity DOTS Packages

| Package | Purpose |
|---------|----------|
| **Unity.Entities** | ECS framework |
| **Unity.Burst** | Performance compiler |
| **Unity.Collections** | Native collections |
| **Unity.Mathematics** | SIMD math |
| **Unity.Jobs** | Multi-threading |

---

## 11. Dependency Resolution

| Tool | Description | Status |
|------|-------------|--------|
| **NuGet** | Package management | Already using |
| **SemVer.NET** | Semantic versioning | Available |
| **NuGet.PackageResolver** | Dependency resolution | Available |

---

## 12. Logging & Observability

| Tool | Description | Status |
|------|-------------|--------|
| **Serilog** | Structured logging | Already using |
| **NLog** | Logging framework | Available |
| **Microsoft.Extensions.Logging** | Logging abstraction | Already using |

---

## 13. Game-Specific Tools

| Tool | Description | Source |
|------|-------------|--------|
| **Unity Profiler** | Performance profiling | Unity Editor |
| **Frame Debugger** | Rendering debugging | Unity Editor |
| **Network Profiler** | Network analysis | Unity Editor |

---

## 14. Mod Distribution

| Platform | Description |
|----------|-------------|
| **Thunderstore** | Unity mod distribution |
| **Nexus Mods** | General mod hosting |
| **Steam Workshop** | Integrated distribution |

---

## Key Takeaways

### Already Aligned with Best Practices:
- BepInEx for modding framework
- Newtonsoft.Json + YamlDotNet for config
- NJsonSchema for validation
- Named pipes for IPC (game bridge)
- xUnit + FluentAssertions for testing

### Potential Additions:
- AltTester for deeper UI automation
- thunderstore-cli for package publishing
- NSubstitute as alternative mocking

---

*Research Date: 2026-03-13*
*Sources: Unity Docs, BepInEx Docs, NuGet Gallery, GitHub, Claude Code Docs*
