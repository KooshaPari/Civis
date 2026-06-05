use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use tokio::fs;
use tokio::process::Command;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars::JsonSchema,
    tool,
    tool_handler,
    tool_router,
    Json,
    ServiceExt,
    transport::stdio,
};

#[derive(Clone, Default)]
struct CivisMcpServer {
    tool_router: ToolRouter<Self>,
}

impl CivisMcpServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
struct BuildArgs {
    #[schemars(description = "Optional civis-cli target directory")]
    target_dir: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ScreenshotArgs {
    #[schemars(description = "Output path for the generated screenshot")]
    out: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct VerifyArgs {
    #[schemars(description = "Optional explicit output path")]
    out: Option<String>,
    #[schemars(description = "Optional civis-cli target directory")]
    target_dir: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct SpawnArgs {
    kind: String,
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct BrushArgs {
    material: String,
    x: i32,
    y: i32,
    z: i32,
    radius: u32,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct BuildResult {
    exit_code: i32,
    log_path: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct ScreenshotResult {
    path: String,
    bytes: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct VerifyResult {
    screenshot: Option<Value>,
    bytes: Option<Vec<u8>>,
    panicked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    panic_tail: Option<Vec<String>>,
    census: Value,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct StubResult {
    ok: bool,
    message: String,
    kind: String,
    params: Value,
}

#[tool_router]
impl CivisMcpServer {
    #[tool(name = "civis_build", description = "Run civis build for civ-standalone")]
    async fn civis_build(
        &self,
        Parameters(BuildArgs { target_dir }): Parameters<BuildArgs>,
    ) -> Result<Json<BuildResult>, String> {
        let args = build_cli_args("build", target_dir);
        let raw: Value = run_cli_json("civis_build", &args).await?;
        let exit_code = raw
            .get("exit")
            .and_then(Value::as_i64)
            .and_then(|v| i32::try_from(v).ok())
            .unwrap_or(0);
        let log_path = raw
            .get("log_path")
            .and_then(Value::as_str)
            .unwrap_or("civis-build.log")
            .to_string();

        Ok(Json(BuildResult { exit_code, log_path }))
    }

    #[tool(name = "civis_screenshot", description = "Capture a screenshot from the running civis world")]
    async fn civis_screenshot(
        &self,
        Parameters(ScreenshotArgs { out }): Parameters<ScreenshotArgs>,
    ) -> Result<Json<ScreenshotResult>, String> {
        let _ = run_cli_json::<Value>(
            "civis_screenshot",
            &[
                "screenshot".into(),
                "--out".into(),
                out.clone(),
            ],
        )
        .await?;

        let bytes = fs::read(&out)
            .await
            .map_err(|err| err.to_string())?;

        Ok(Json(ScreenshotResult { path: out, bytes }))
    }

    #[tool(name = "civis_census", description = "Read latest parsed census JSON")]
    async fn civis_census(
        &self,
        _params: Parameters<serde_json::Value>,
    ) -> Result<Json<Value>, String> {
        run_cli_json::<Value>("civis_census", &["census".into()]).await.map(Json)
    }

    #[tool(name = "civis_verify", description = "Run full verify loop and return screenshot, census and panic info")]
    async fn civis_verify(
        &self,
        Parameters(VerifyArgs { out, target_dir }): Parameters<VerifyArgs>,
    ) -> Result<Json<VerifyResult>, String> {
        let args = build_verify_cli_args(out, target_dir);
        let raw = run_cli_json::<Value>("civis_verify", &args).await?;

        let panicked = raw
            .get("panicked")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let panic_tail = raw
            .get("panic_tail")
            .and_then(Value::as_array)
            .map(|tail| {
                tail.iter()
                    .filter_map(Value::as_str)
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            });
        let census = raw.get("census").cloned().unwrap_or(Value::Null);
        let screenshot = raw.get("screenshot").cloned();

        let bytes = match screenshot.as_ref().and_then(|val| val.get("path")).and_then(Value::as_str) {
            Some(path) => Some(
                fs::read(path)
                    .await
                    .map_err(|err| format!("failed to read screenshot {path}: {err}"))?,
            ),
            None => None,
        };

        Ok(Json(VerifyResult {
            screenshot,
            bytes,
            panicked,
            panic_tail,
            census,
        }))
    }

    #[tool(name = "civis_spawn", description = "Spawn an object at the requested coordinates")]
    async fn civis_spawn(
        &self,
        Parameters(params): Parameters<SpawnArgs>,
    ) -> Result<Json<StubResult>, String> {
        // TODO: implement civis-spawn by wiring this handler to the running world bridge.
        let result = StubResult {
            ok: false,
            kind: "not yet wired".to_string(),
            message: "TODO: route to running instance via <mechanism> (spawn command bridge)."
                .to_string(),
            params: serde_json::to_value(params)
                .map_err(|err| err.to_string())?,
        };
        Ok(Json(result))
    }

    #[tool(name = "civis_brush_paint", description = "Paint a brush at the given point")]
    async fn civis_brush_paint(
        &self,
        Parameters(params): Parameters<BrushArgs>,
    ) -> Result<Json<StubResult>, String> {
        // TODO: implement brush paint by forwarding to the running instance bridge.
        let result = StubResult {
            ok: false,
            kind: "not yet wired".to_string(),
            message: "TODO: route to running instance via <mechanism> (brush paint command bridge)."
                .to_string(),
            params: serde_json::to_value(params)
                .map_err(|err| err.to_string())?,
        };
        Ok(Json(result))
    }
}

#[tool_handler]
impl rmcp::ServerHandler for CivisMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

fn build_cli_args(cmd: &str, target_dir: Option<String>) -> Vec<String> {
    let mut args = vec![cmd.to_string()];
    if let Some(target_dir) = target_dir {
        args.push("--target-dir".to_string());
        args.push(target_dir);
    }
    args
}

fn build_verify_cli_args(out: Option<String>, target_dir: Option<String>) -> Vec<String> {
    let mut args = vec!["verify".to_string()];
    if let Some(out) = out {
        args.push("--out".to_string());
        args.push(out);
    }
    if let Some(target_dir) = target_dir {
        args.push("--target-dir".to_string());
        args.push(target_dir);
    }
    args
}

async fn run_cli_json<T>(tool_name: &str, args: &[String]) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let output = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("civis-cli")
        .arg("--")
        .args(args)
        .output()
        .await
        .map_err(|err| format!("[{tool_name}] failed to spawn civis-cli: {err}"))?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "[{tool_name}] civis-cli failed: code={:?}\nstdout={stdout}\nstderr={stderr}",
            output.status.code()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        return Err(format!("[{tool_name}] civis-cli produced no stdout"));
    }

    serde_json::from_str(&stdout)
        .map_err(|err| format!("[{tool_name}] invalid JSON stdout: {err}. stdout={stdout}"))
}

#[tokio::main]
async fn main() {
    // TODO: switch all operations from CLI-shell fallback to civis-cli pub library
    // functions once signatures are finalized in `crates/civis-cli`.
    let _ = civis_cli::workspace_root();

    let transport = stdio();
    let service = CivisMcpServer::new().serve(transport).await;
    match service {
        Ok(service) => {
            if let Err(err) = service.waiting().await {
                eprintln!("civis-mcp server terminated: {err}");
            }
        }
        Err(err) => {
            eprintln!("civis-mcp failed to start: {err}");
            std::process::exit(1);
        }
    }
}
