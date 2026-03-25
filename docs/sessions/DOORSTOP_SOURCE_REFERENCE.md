# UnityDoorstop Source Code Reference

Complete source code comparison from both UnityDoorstop forks for porting to DINOForge.

---

## 1. Configuration Header (NeighTools vs devopsdinosaur)

### NeighTools - `src/config/config.h`

```c
#ifndef CONFIG_H
#define CONFIG_H

typedef struct {
    // Core activation
    bool_t enabled;
    bool_t redirect_output_log;
    bool_t ignore_disabled_env;

    // Assembly loading (arrays for multiple assemblies)
    char_t **target_assemblies;
    size_t num_assemblies;
    size_t assembly_index;

    // Boot configuration
    char_t *boot_config_override;

    // Mono runtime paths
    char_t *mono_dll_search_path_override;
    bool_t mono_debug_enabled;
    bool_t mono_debug_suspend;
    char_t *mono_debug_address;

    // CoreCLR paths
    char_t *clr_runtime_coreclr_path;
    char_t *clr_corlib_dir;
} Config;

extern Config config;

void load_config(void);
void init_config_defaults(void);
void cleanup_config(void);
void parse_target_assembly_string(const char_t *string);

#endif
```

### devopsdinosaur - `src/config/config.h`

**Differences from NeighTools:**
- Single `target_assembly` string instead of array-based `target_assemblies[]`
- No `num_assemblies` or `assembly_index` fields
- No `parse_target_assembly_string()` function declared
- Same boolean flags and path overrides

---

## 2. Configuration Common Implementation (NeighTools vs devopsdinosaur)

### NeighTools - `src/config/common.c`

```c
#include "config.h"
#include "../crt.h"
#include "../util/util.h"

Config config;

void cleanup_config(void) {
    // Cleanup assembly array
    if (config.target_assemblies) {
        for (size_t i = 0; i < config.num_assemblies; i++) {
            if (config.target_assemblies[i]) {
                free(config.target_assemblies[i]);
            }
        }
        free(config.target_assemblies);
        config.target_assemblies = NULL;
    }

    // Cleanup string fields
    if (config.boot_config_override) {
        free(config.boot_config_override);
        config.boot_config_override = NULL;
    }
    if (config.mono_dll_search_path_override) {
        free(config.mono_dll_search_path_override);
        config.mono_dll_search_path_override = NULL;
    }
    if (config.mono_debug_address) {
        free(config.mono_debug_address);
        config.mono_debug_address = NULL;
    }
    if (config.clr_runtime_coreclr_path) {
        free(config.clr_runtime_coreclr_path);
        config.clr_runtime_coreclr_path = NULL;
    }
    if (config.clr_corlib_dir) {
        free(config.clr_corlib_dir);
        config.clr_corlib_dir = NULL;
    }
}

void init_config_defaults(void) {
    config.enabled = FALSE;
    config.redirect_output_log = FALSE;
    config.ignore_disabled_env = FALSE;
    config.target_assemblies = NULL;
    config.num_assemblies = 0;
    config.assembly_index = 0;
    config.boot_config_override = NULL;
    config.mono_dll_search_path_override = NULL;
    config.mono_debug_enabled = FALSE;
    config.mono_debug_suspend = FALSE;
    config.mono_debug_address = NULL;
    config.clr_runtime_coreclr_path = NULL;
    config.clr_corlib_dir = NULL;
}
```

### devopsdinosaur - `src/config/common.c`

**Differences:**
- Cleanup only handles single `target_assembly` string
- No assembly array cleanup
- Otherwise identical pattern

---

## 3. Windows Configuration Loader (NeighTools)

### Key Functions - `src/windows/config.c` (NeighTools)

**`load_config()` - Main orchestration:**
```c
void load_config(void) {
    init_config_defaults();
    init_config_file();
    init_cmd_args();
    init_env_vars();
}
```

**`init_config_file()` - INI file loading:**
- Reads `doorstop_config.ini` from game directory
- Sections: `[General]`, `[UnityMono]`, `[Il2Cpp]`
- Uses Windows API `GetPrivateProfileString()`

**`init_cmd_args()` - Command-line argument parsing:**
- Parses `--doorstop-enabled`, `--doorstop-target-assembly`, etc.
- Uses `CommandLineToArgv()` to split command line
- Pattern: `--doorstop-key=value` or `--doorstop-key value`

**`init_env_vars()` - Environment variable checks:**
- Checks `DOORSTOP_DISABLE` environment variable
- Can override all other settings unless `ignore_disabled_env` is set

**`parse_target_assembly_string()` - Assembly path resolution (NeighTools extension):**
- Handles semicolon-delimited paths (e.g., `path1;path2`)
- Expands directories to find `.dll` files
- Uses Windows `FindFirstFile()` / `FindNextFile()` API
- Resolves relative paths to absolute

### Key Functions - `src/windows/config.c` (devopsdinosaur)

**Same functions, different assembly handling:**
- Single string target_assembly instead of array
- No directory expansion or multiple assembly support
- Simpler path resolution logic

---

## 4. Bootstrap Header

### `src/bootstrap.h` (Common to both forks)

```c
#ifndef BOOTSTRAP_H
#define BOOTSTRAP_H

#include "runtimes/il2cpp.h"
#include "runtimes/mono.h"
#include "util/util.h"

// Initialize Mono runtime and bootstrap managed code
void *init_mono(const char *root_domain_name, const char *runtime_version);

// Initialize IL2CPP runtime
int init_il2cpp(const char *domain_name);

// Image loading hook for search path overrides
void *hook_mono_image_open_from_data_with_name(
    void *data,
    unsigned long data_len,
    int need_copy,
    MonoImageOpenStatus *status,
    int refonly,
    const char *name);

// JIT option parsing hook
void hook_mono_jit_parse_options(int argc, char **argv);

// Debug init hook
void hook_mono_debug_init(MonoDebugFormat format);

#endif
```

---

## 5. Bootstrap Implementation (NeighTools)

### `src/bootstrap.c` - Core Functions

**`mono_doorstop_bootstrap(void *mono_domain)` - Main Mono bootstrap:**

1. **Initialization Check**
   ```c
   if (getenv(TEXT("DOORSTOP_INITIALIZED"))) {
       LOG("DOORSTOP_INITIALIZED is set! Skipping!");
       return;
   }
   setenv(TEXT("DOORSTOP_INITIALIZED"), TEXT("TRUE"), TRUE);
   ```

2. **Domain Configuration**
   ```c
   if (mono.domain_set_config) {
       // Set domain config paths for .config file loading
       mono.domain_set_config(mono_domain, folder_path, config_path);
   }
   ```

3. **Environment Variables Setup**
   ```c
   setenv(TEXT("DOORSTOP_INVOKE_DLL_PATH"), config.target_assemblies[0], TRUE);
   setenv(TEXT("DOORSTOP_PROCESS_PATH"), app_path, TRUE);
   setenv(TEXT("DOORSTOP_MANAGED_FOLDER_DIR"), assembly_root, TRUE);
   ```

4. **Assembly Loading Loop** (supports multiple assemblies)
   ```c
   for (config.assembly_index = 0;
        config.assembly_index < config.num_assemblies;
        config.assembly_index++) {
       current_assembly = config.target_assemblies[config.assembly_index];

       // Open file and read bytes
       void *file = fopen(current_assembly, "r");
       size_t size = get_file_size(file);
       void *data = malloc(size);
       fread(data, size, 1, file);
       fclose(file);

       // Open as Mono image
       void *image = mono.image_open_from_data_with_name(
           data, size, TRUE, &status, FALSE, dll_path);

       // Load assembly from image
       void *assembly = mono.assembly_load_from_full(
           image, dll_path, &status, FALSE);

       // Find Doorstop.Entrypoint:Start method
       void *desc = mono.method_desc_new("Doorstop.Entrypoint:Start", TRUE);
       void *method = mono.method_desc_search_in_image(desc, image);

       // Invoke with exception handling
       void *exc = NULL;
       mono.runtime_invoke(method, NULL, NULL, &exc);
       if (exc != NULL) {
           // Log exception details
       }
   }
   ```

**`init_mono()` - Mono runtime initialization:**

1. **Assembly search path override**
   ```c
   char_t *mono_search_path = // build path
   mono.set_assemblies_path(mono_search_path_n);
   setenv(TEXT("DOORSTOP_DLL_SEARCH_DIRS"), mono_search_path, TRUE);
   ```

2. **Debugger initialization** (conditional)
   ```c
   if (config.mono_debug_enabled && !debugger_already_enabled) {
       if (mono_is_net35) {
           mono.debug_init(MONO_DEBUG_FORMAT_MONO);
       }
       domain = mono.jit_init_version(root_domain_name, runtime_version);
       mono.debug_domain_create(domain);
   }
   ```

3. **Bootstrap invocation**
   ```c
   mono_doorstop_bootstrap(domain);
   return domain;
   ```

**`il2cpp_doorstop_bootstrap()` - CoreCLR initialization:**

1. **Path validation**
   ```c
   if (!config.clr_corlib_dir || !config.clr_runtime_coreclr_path) {
       LOG("No CoreCLR paths set, skipping loading");
       return;
   }
   ```

2. **Load CoreCLR**
   ```c
   void *coreclr_module = dlopen(config.clr_runtime_coreclr_path, RTLD_LAZY);
   load_coreclr_funcs(coreclr_module);
   ```

3. **Initialize CoreCLR with properties**
   ```c
   int result = coreclr.initialize(
       app_path_n, "Doorstop Domain", 1, &props, &app_paths_env_n,
       &host, &domain_id);
   ```

4. **Create and invoke delegate**
   ```c
   void (*startup)() = NULL;
   result = coreclr.create_delegate(
       host, domain_id, target_name_n,
       "Doorstop.Entrypoint", "Start",
       (void **)&startup);
   startup();
   ```

**Hook Functions:**

```c
// Configures debugger options from environment variables
void hook_mono_jit_parse_options(int argc, char **argv) {
    char_t *debug_options = getenv(TEXT("DNSPY_UNITY_DBG2"));
    if (debug_options || config.mono_debug_enabled) {
        // Build debug argument string
        // --debugger-agent=transport=dt_socket,server=y,address=<addr>[,suspend=n]
        char *debug_args = calloc(debug_args_len + 1, sizeof(char_t));
        strcat(debug_args, MONO_DEBUG_ARG_START);
        strcat(debug_args, config.mono_debug_address);
        if (!config.mono_debug_suspend) {
            strcat(debug_args, mono_is_net35 ?
                MONO_DEBUG_NO_SUSPEND_NET35 : MONO_DEBUG_NO_SUSPEND);
        }
        // Add to argv for JIT parsing
    }
    mono.jit_parse_options(argc, argv);
}

// Intercepts assembly loading to apply search path override
void *hook_mono_image_open_from_data_with_name(
    void *data, unsigned long data_len, int need_copy,
    MonoImageOpenStatus *status, int refonly, const char *name) {

    void *result = NULL;
    if (config.mono_dll_search_path_override) {
        // Check if assembly exists in override directory
        char_t *new_full_path = // build path in override dir
        if (file_exists(new_full_path)) {
            // Load from override directory instead
            void *file = fopen(new_full_path, "r");
            // ... read and return
        }
    }

    if (!result) {
        // Fall back to original loading
        result = mono.image_open_from_data_with_name(
            data, data_len, need_copy, status, refonly, name);
    }
    return result;
}

// Tracks debugger initialization
void hook_mono_debug_init(MonoDebugFormat format) {
    mono_debug_init_called = TRUE;
    mono.debug_init(format);
}
```

---

## 6. CRT Header (Platform Abstraction)

### `src/crt.h` (Common to both forks)

```c
#ifndef CRT_H
#define CRT_H

#ifdef _WIN32
    // Windows configuration
    #include <windows.h>
    #define MAX_PATH 1024
    #ifdef _WIN64
        #define ENV64
    #else
        #define ENV32
    #endif
#else
    // Unix-like (Linux/macOS)
    #define _GNU_SOURCE
    #include <dlfcn.h>
    #include <libgen.h>
    #include <limits.h>
    #include <stdarg.h>
    #include <stddef.h>
    #include <stdio.h>
    #include <stdlib.h>
    #include <string.h>
    #include <sys/stat.h>
    #include <unistd.h>

    #ifdef __APPLE__
        #include <mach-o/dyld.h>
    #endif

    #ifdef __x86_64__
        #define ENV64
    #elif defined(__ppc64__)
        #define ENV64
    #else
        #define ENV32
    #endif

    #define MAX_PATH PATH_MAX
    #define shutenv
#endif

#endif
```

---

## Key Differences for DINOForge Port

### NeighTools Version (RECOMMENDED for DINOForge)

**Advantages:**
1. **Multiple Assembly Support** - Can load and invoke multiple managed DLLs in sequence
2. **Directory Expansion** - Can specify directories that get scanned for `.dll` files
3. **More Flexible Configuration** - Better suited for mod pack scenarios where multiple plugins need loading
4. **Better INI Section Organization** - Supports General, UnityMono, Il2Cpp sections

**Ideal for DINOForge because:**
- DINOForge is a mod platform that may need to load multiple domain plugins (Warfare, Economy, Scenario, UI)
- Packs are declarative; directory scanning fits the pack-discovery pattern
- Configuration hierarchy (file → args → env) matches agent-friendly configuration patterns

### devopsdinosaur Version

**Advantages:**
1. **Simpler** - Single assembly target is easier to understand
2. **Smaller codebase** - Less code to maintain

**Disadvantages:**
- Only supports one managed assembly
- No directory expansion
- Less flexible for complex mod scenarios

---

## Import Strategy for DINOForge

**Recommended approach:**

```
src/Runtime/Doorstop/
  ├── config/
  │   ├── config.h              (from NeighTools - with array support)
  │   └── common.c              (from NeighTools)
  ├── windows/
  │   ├── config.c              (from NeighTools - with dir expansion)
  │   └── <other platform files>
  ├── bootstrap.h               (common)
  ├── bootstrap.c               (from NeighTools - multi-assembly support)
  ├── crt.h                      (common)
  └── <runtime hooks, etc.>
```

This provides:
- Multi-assembly loading for future expansion (Economy, Scenario, UI domains)
- Directory scanning for pack discovery
- Flexible configuration matching DINOForge's declarative philosophy
- Better testability (easier to inject mock assemblies)

---

## Critical Implementation Notes

### Assembly Loading Sequence

```
1. Read config from file/args/env
2. Set DOORSTOP_INITIALIZED to prevent re-entry
3. Configure Mono domain paths (.config file location)
4. Loop through target_assemblies array:
   a. Read assembly file to memory
   b. Open as Mono image
   c. Load assembly from image
   d. Find Doorstop.Entrypoint:Start method
   e. Invoke with exception handling
   f. Log any exceptions
5. Return control to game
```

### Environment Variables Set by Doorstop

```
DOORSTOP_INITIALIZED       Set to "TRUE" to prevent re-entry
DOORSTOP_INVOKE_DLL_PATH   Path to DLL being loaded (useful for managed code)
DOORSTOP_PROCESS_PATH      Game executable path
DOORSTOP_MANAGED_FOLDER_DIR Root assembly directory
DOORSTOP_DLL_SEARCH_DIRS   Search paths for assemblies
```

### Hook Points

1. `hook_mono_jit_parse_options()` - Configure JIT debugger
2. `hook_mono_image_open_from_data_with_name()` - Override assembly search paths
3. `hook_mono_debug_init()` - Track debugger initialization state

These hooks are called during Mono initialization before the game's managed code runs.

---

## File Size Reference

| File | NeighTools | devopsdinosaur |
|------|-----------|----------------|
| bootstrap.c | 14,516 bytes | 15,504 bytes |
| config.h | 2,216 bytes | 2,629 bytes |
| windows/config.c | varies | varies |
| common.c | 1,182 bytes | 1,470 bytes |

All files are under 500 lines of C code - suitable for integration into DINOForge runtime layer.
