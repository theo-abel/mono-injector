<p>
    <img src="gui/assets/mono-injector-banner.png" alt="mono-injector banner" width="100%">
</p>

<div align="left">
    <h1>mono-injector</h1>
    <p>
        mono-injector is a Rust-based Windows tool for injecting managed .NET assemblies into Mono/Unity processes.
    </p>
    <p>
        Built with precision and reliability in mind, mono-injector provides a robust solution for developers and researchers working with Mono runtime environments.
    </p>
    <div align="center">
        <img src="https://img.shields.io/badge/Built%20with%20Rust-grey?style=for-the-badge&logo=rust&color=%23282828">
    </div>
</div>

## Description

`mono-injector` is a Rust workspace for injecting managed .NET assemblies into Windows processes that host Mono, especially Unity games and tools.

The workspace is split into four crates:

| Crate | Purpose |
| --- | --- |
| `mono-injector` | Low-level Windows remote Mono API caller. |
| `mono-injector-core` | Reusable orchestration layer for profiles, process discovery, injection/ejection planning, metadata inspection, remembered handles, Steam launch, and readiness waits. |
| `mono-injector-cli` | Command-line frontend exposed as the `mono-injector` binary. |
| `mono-injector-gui` | Iced-based graphical frontend using the same core layer. |

The default managed entry point is `Loader::Init`. The default eject entry point is `Loader::Unload`. If no namespace is supplied, the core inspects the assembly metadata and prefers the namespace of the selected class before falling back to the most common non-empty namespace.

Use this on software you own, control, or are explicitly allowed to modify. Injection into protected or third-party processes can violate terms of service and can crash the target process.

## Features

Injection and ejection:

- Inject managed assemblies into Windows processes hosting Mono.
- Eject managed assemblies by invoking a cleanup method and unloading the remembered Mono assembly handle.
- Support x86 and x64 targets through architecture-specific remote stubs.
- Execute Mono API calls from outside the target process without injecting a native loader DLL.
- Configure namespace, class, inject method, and eject method per operation.
- Use practical defaults for common loaders: class `Loader`, inject method `Init`, eject method `Unload`.
- Run inject and eject dry-runs that resolve the plan without calling Mono in the target process.
- Use raw handle ejection for advanced/manual recovery, gated behind `--force`.

Process and runtime resolution:

- Resolve targets by PID or exact process name.
- List running processes and filter by process name, module name, Mono runtime modules, or Unity modules.
- Include matching or loaded module names in process listings.
- Select the Mono runtime module with a case-insensitive `--mono-module` hint.
- Configure remote-thread wait timeout with `--timeout-ms`.
- Configure the base directory passed to `mono_assembly_load_from_full` with `--base-dir`.
- Validate managed assemblies by reading .NET metadata.
- Infer the namespace from .NET metadata when it is omitted.

Profiles and remembered state:

- Store reusable TOML profiles for target, assembly, entry point, runtime, wait, settle, and Steam settings.
- Select profiles by positional name or `--profile` flag.
- Remember successful injection handles locally for later safe ejection.
- Protect against PID reuse by recording process start time.
- Guard ejection by matching remembered handles against the same process instance, namespace, and class.
- Require `--latest` or an explicit handle when several remembered injections match.
- Clean stale remembered records or clear all records manually.

Launch and readiness:

- Wait for a process before injecting.
- Wait for a module before injecting.
- Configure process/module wait timeout and poll interval.
- Configure a settle delay after readiness before injection.
- Launch Steam apps with `steam://rungameid/<APP_ID>`.
- Skip Steam launch when the target process is already running.
- Use Steam defaults of `d3d11.dll` readiness and `8000ms` settle delay unless overridden.

Frontends and tooling:

- CLI supports JSON output where useful through the global `--json` flag.
- CLI can run a post-injection command with target process, PID, handle, and assembly path environment variables.
- GUI supports inject, eject, active-injection status, process browsing, profile management, and logs.
- CLI and GUI share the same `mono-injector-core` planning, profile, state, and runtime behavior.
- Workspace uses Rust 2024 with `warnings = deny` and Clippy pedantic enabled.

## Docs

### Usage

The CLI binary is named `mono-injector`. From source, run it with:

```powershell
cargo run -p mono-injector-cli -- <command> [options]
```

After building, run the binary directly:

```powershell
target\debug\mono-injector.exe <command> [options]
```

Top-level usage:

```text
Usage: mono-injector.exe [OPTIONS] <COMMAND>

Commands:
  inject   Inject a managed assembly into a target process
  eject    Eject a previously injected assembly from a target process
  list     List running processes
  status   Show remembered injections
  clean    Remove stale remembered injections
  profile  Inspect profile configuration
  help     Print this message or the help of the given subcommand(s)

Options:
      --json     Emit machine-readable JSON output where supported
  -h, --help     Print help
  -V, --version  Print version
```

#### `inject`

Inject a managed assembly into a target process.

```text
Usage: mono-injector.exe inject [OPTIONS] [PROFILE] [-- <POST_COMMAND>...]

Arguments:
  [PROFILE]          Optional profile name
  [POST_COMMAND]...  Command to run after successful injection. Pass after `--`

Options:
      --json                            Emit machine-readable JSON output where supported
      --profile <PROFILE_ALIAS>         Profile name alias for scripts that prefer flags
  -p, --process <PROCESS>               Target process id or exact process name
  -a, --assembly <ASSEMBLY>             Managed assembly to load into the target process
  -n, --namespace <NAMESPACE>           Namespace containing the loader class
  -c, --class <CLASS_NAME>              Loader class name
  -m, --method <METHOD_NAME>            Loader method to invoke after loading the assembly
      --eject-method <EJECT_METHOD>     Cleanup method recorded for later default ejection
      --wait                            Wait for the target process before injecting
      --wait-timeout <WAIT_TIMEOUT>     Time to wait for process/module readiness, for example 120s or 2m [default: 120s]
      --poll-interval <POLL_INTERVAL>   Time between process/module readiness checks, for example 1000ms or 1s [default: 1000ms]
      --wait-module <WAIT_MODULE>       Wait for a loaded module before injecting, for example UnityPlayer.dll
      --no-wait-module                  Disable the default readiness-module wait used with --steam-app
      --settle-ms <SETTLE>              Extra time to wait after readiness before injecting. Use 0ms to disable
      --steam-app <STEAM_APP>           Launch a Steam app before waiting for the process
      --dry-run                         Resolve inputs without calling Mono in the target process
      --timeout-ms <TIMEOUT_MS>         Remote-thread wait timeout in milliseconds [default: 5000]
      --mono-module <MONO_MODULE_HINT>  Case-insensitive fragment used to find the target Mono module
      --base-dir <BASE_DIR>             Base directory passed to `mono_assembly_load_from_full`
  -h, --help                            Print help
  -V, --version                         Print version
```

Basic inject using defaults:

```powershell
mono-injector inject --process Game.exe --assembly C:\mods\MyMod.dll
```

Explicit entry point:

```powershell
mono-injector inject -p Game.exe -a C:\mods\MyMod.dll -n MyMod -c Loader -m Init --eject-method Unload
```

Dry-run an inject plan:

```powershell
mono-injector inject --dry-run -p Game.exe -a C:\mods\MyMod.dll
```

Wait for the process and a readiness module:

```powershell
mono-injector inject -p Game.exe -a C:\mods\MyMod.dll --wait --wait-module UnityPlayer.dll --settle-ms 3000ms
```

Launch a Steam app and inject after default Steam readiness behavior:

```powershell
mono-injector inject -p Game.exe -a C:\mods\MyMod.dll --steam-app 480
```

Run a command after successful injection:

```powershell
mono-injector inject -p Game.exe -a C:\mods\MyMod.dll -- powershell -NoProfile -Command "Write-Output $env:MONO_INJECTOR_HANDLE"
```

The post-command receives these environment variables:

| Variable | Value |
| --- | --- |
| `MONO_INJECTOR_PROCESS` | Target process name. |
| `MONO_INJECTOR_PID` | Target PID. |
| `MONO_INJECTOR_HANDLE` | Assembly handle returned by inject, or an empty string if absent. |
| `MONO_INJECTOR_ASSEMBLY` | Managed assembly path. |

#### `eject`

Eject a previously injected assembly from a target process.

```text
Usage: mono-injector.exe eject [OPTIONS] [PROFILE]

Arguments:
  [PROFILE]  Optional profile name

Options:
      --json                            Emit machine-readable JSON output where supported
      --profile <PROFILE_ALIAS>         Profile name alias for scripts that prefer flags
  -p, --process <PROCESS>               Target process id or exact process name
  -a, --assembly <HANDLE>               Assembly handle returned by inject. Defaults to a matching remembered injection
      --raw-handle <RAW_HANDLE>         Explicit unsafe handle mode; requires --force
  -n, --namespace <NAMESPACE>           Namespace containing the loader class
  -c, --class <CLASS_NAME>              Loader class name
  -m, --method <METHOD_NAME>            Cleanup method to invoke before closing the assembly
      --latest                          Use the latest matching remembered injection when several match
      --force                           Bypass the local injection-record guard for advanced/manual ejection
      --dry-run                         Resolve inputs without calling Mono in the target process
      --timeout-ms <TIMEOUT_MS>         Remote-thread wait timeout in milliseconds [default: 5000]
      --mono-module <MONO_MODULE_HINT>  Case-insensitive fragment used to find the target Mono module
      --base-dir <BASE_DIR>             Base directory passed to `mono_assembly_load_from_full`
  -h, --help                            Print help
  -V, --version                         Print version
```

Eject using the remembered matching handle:

```powershell
mono-injector eject --process Game.exe
```

Eject a specific remembered handle:

```powershell
mono-injector eject -p Game.exe -a 0x12345678
```

Resolve an eject plan without touching the process:

```powershell
mono-injector eject --dry-run -p Game.exe --latest
```

Use a raw handle manually:

```powershell
mono-injector eject -p Game.exe --raw-handle 0x12345678 --force -n MyMod -c Loader -m Unload
```

Safe eject behavior:

- Without `--force`, `eject` requires the handle to be remembered for the same PID, process start time, namespace, and class.
- If one remembered injection matches, the handle is selected automatically.
- If several remembered injections match, pass `--latest` or provide a specific `--assembly <HANDLE>`.
- `--raw-handle` is intentionally unsafe and requires `--force`.

#### `list`

List running processes.

```text
Usage: mono-injector.exe list [OPTIONS]

Options:
  -f, --filter <FILTER>  Case-insensitive substring used to filter process and module names
      --json             Emit machine-readable JSON output where supported
      --mono             Show only processes with a Mono runtime module loaded
      --unity            Show only Unity processes
      --modules          Include matching or loaded module names in the output
  -h, --help             Print help
  -V, --version          Print version
```

Examples:

```powershell
mono-injector list
mono-injector list --mono
mono-injector list --unity --modules
mono-injector list --filter Game --modules
mono-injector --json list --mono
```

#### `status`

Show remembered injections.

```text
Usage: mono-injector.exe status [OPTIONS] [PROFILE]

Arguments:
  [PROFILE]  Optional profile name to resolve the target process

Options:
      --json                     Emit machine-readable JSON output where supported
      --profile <PROFILE_ALIAS>  Profile name alias for scripts that prefer flags
  -p, --process <PROCESS>        Target process id or exact process name
  -h, --help                     Print help
  -V, --version                  Print version
```

Examples:

```powershell
mono-injector status
mono-injector status --process Game.exe
mono-injector status my-profile
mono-injector --json status
```

#### `clean`

Remove remembered injection records.

```text
Usage: mono-injector.exe clean [OPTIONS]

Options:
      --all      Remove all remembered injections, including live ones
      --json     Emit machine-readable JSON output where supported
  -h, --help     Print help
  -V, --version  Print version
```

Examples:

```powershell
mono-injector clean
mono-injector clean --all
mono-injector --json clean
```

Default `clean` removes stale records only. `clean --all` removes every remembered record, including records for live process instances.

#### `profile`

Inspect profile configuration.

```text
Usage: mono-injector.exe profile [OPTIONS] <COMMAND>

Commands:
  list  List configured profiles
  show  Show one configured profile
  path  Print the profiles file path
  help  Print this message or the help of the given subcommand(s)

Options:
      --json     Emit machine-readable JSON output where supported
  -h, --help     Print help
  -V, --version  Print version
```

Profile subcommands:

```text
Usage: mono-injector.exe profile list [OPTIONS]
Usage: mono-injector.exe profile show [OPTIONS] <NAME>
Usage: mono-injector.exe profile path [OPTIONS]
```

Examples:

```powershell
mono-injector profile path
mono-injector profile list
mono-injector profile show my-profile
mono-injector --json profile show my-profile
```

Profiles are TOML and live at the path printed by `mono-injector profile path`. A typical profile file looks like this:

```toml
[profiles.my-profile]
process = "Game.exe"
assembly = "C:\\mods\\MyMod.dll"
namespace = "MyMod"
class = "Loader"
inject_method = "Init"
eject_method = "Unload"
mono_module = "mono-2.0-bdwgc"
base_dir = "C:\\mods"
timeout_ms = 5000
wait_module = "UnityPlayer.dll"
settle_ms = 3000
steam_app = 480
```

All profile fields are optional, but an inject operation still needs enough data after combining CLI flags and profile values to resolve a target process and assembly. CLI flags override profile values. Remembered injection records are stored separately in the user local data directory as `mono-injector/injections.json`.

### Build

Requirements:

- Windows target environment.
- Rust toolchain compatible with workspace `rust-version = "1.95.0"` and edition 2024.
- `just` for the documented task runner commands.
- A target process with an embedded Mono runtime for real injection tests.

Build the whole workspace:

```powershell
just build
```

Build release artifacts:

```powershell
just build release
```

Build only the CLI:

```powershell
just build dev mono-injector-cli
```

Build only the GUI:

```powershell
just build dev mono-injector-gui
```

Equivalent Cargo commands:

```powershell
cargo build --workspace --all-targets
cargo build --release --workspace --all-targets
cargo build -p mono-injector-cli
cargo build -p mono-injector-gui
```

Run from source:

```powershell
just cli --help
just gui
cargo run -p mono-injector-cli -- --help
cargo run -p mono-injector-gui
```

Format the workspace:

```powershell
just format
```

### Test

Run the required final check:

```powershell
just check
```

`just check` aliases `just lint`, which runs:

```powershell
cargo clippy --all-targets --all-features --all
```

Run all tests:

```powershell
just test
```

Run tests for one crate:

```powershell
just test mono-injector-core
cargo test -p mono-injector-core
```

Useful CLI smoke tests:

```powershell
cargo run -p mono-injector-cli -- --help
cargo run -p mono-injector-cli -- inject --help
cargo run -p mono-injector-cli -- eject --help
cargo run -p mono-injector-cli -- profile path
```

Real injection tests require a live Windows process hosting Mono and a managed assembly with compatible static entry points:

```csharp
public static class Loader
{
    public static void Init()
    {
    }

    public static void Unload()
    {
    }
}
```

### How does it work

At a high level, the CLI and GUI collect user intent and pass it to `mono-injector-core`. The core resolves profiles, process identity, assembly metadata, runtime options, wait behavior, and remembered injection state. The low-level `mono-injector` crate performs the actual remote Mono calls.

Injection flow:

1. Resolve a target process from a PID or exact process name.
2. Resolve the managed assembly path from CLI flags, a profile, or a remembered prior injection.
3. Read the assembly bytes from disk.
4. Validate .NET metadata.
5. Resolve namespace, class, inject method, and eject method.
6. Optionally launch a Steam app if configured and the target is not already running.
7. Optionally wait for the target process.
8. Optionally wait for a readiness module.
9. Optionally sleep for the settle delay.
10. Find the target Mono module in the remote process.
11. Call Mono APIs in the remote process to open the image, load the assembly, resolve the class and method, and invoke the static inject method.
12. Read the resulting assembly handle.
13. Store a remembered injection record for safe later ejection.

Ejection flow:

1. Resolve the target process.
2. Resolve the assembly handle from CLI flags or remembered records.
3. Resolve namespace, class, and eject method from CLI flags, profile values, or remembered records.
4. Enforce the local record guard unless `--force` is used.
5. Call the configured static cleanup method in the target process.
6. Close/unload the assembly through Mono.
7. Remove the remembered injection record.

The low-level crate works by allocating memory in the target process, writing a small architecture-specific stub, and running that stub with a remote thread. The stub calls functions exported by the embedded Mono runtime. Arguments and return values are marshaled through remote process memory with Windows APIs such as `ReadProcessMemory`, `WriteProcessMemory`, `VirtualAllocEx`, `VirtualFreeEx`, `CreateRemoteThread`, and module enumeration APIs.

The managed inject and eject methods should be static and parameterless. They should return quickly. The eject method is responsible for cleaning up objects, hooks, threads, event handlers, and other resources created by the injected assembly.

## Credits

- Techniques: https://github.com/wh0am15533/SharpMonoInjector
- Original SharpMonoInjector project: https://github.com/warbler/SharpMonoInjector

## License

This project is licensed under `GPL-3.0-only`. See [`LICENSE`](LICENSE) for the full license text.
