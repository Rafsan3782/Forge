//! QuickJS-NG runtime embedding, sandboxing, and script isolation.

use crate::error::ScriptExecutionError;
use crate::sourcemap::ScriptSourceMapper;
use crate::transpiler::transpile_typescript;
use rquickjs::{Context, Function, Module, Object, Persistent, Runtime};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Configuration for script sandboxing and resource limits.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Maximum memory in bytes (default 16MB).
    pub memory_limit_bytes: usize,
    /// Maximum execution time per script per frame (default 5ms).
    pub max_duration_per_frame: Duration,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            memory_limit_bytes: 16 * 1024 * 1024,
            max_duration_per_frame: Duration::from_millis(5),
        }
    }
}

/// Compiled and loaded script module with lifecycle hooks.
#[derive(Clone)]
pub struct LoadedScript {
    pub script_path: String,
    pub source_mapper: ScriptSourceMapper,
    pub module_name: String,
    pub transpiled_code: String,
    pub namespace: Persistent<Object<'static>>,
    pub has_on_start: bool,
    pub has_on_update: bool,
    pub has_on_fixed_update: bool,
    pub has_on_collision_enter: bool,
    pub has_on_collision_exit: bool,
    pub has_on_trigger_enter: bool,
    pub has_on_destroy: bool,
    pub has_serialize: bool,
    pub has_deserialize: bool,
}

/// Execution controller for interrupting scripts that run past frame time limits.
#[derive(Clone)]
pub struct InterruptController {
    deadline: Arc<std::sync::Mutex<Option<Instant>>>,
    timed_out: Arc<AtomicBool>,
}

impl InterruptController {
    pub fn new() -> Self {
        Self {
            deadline: Arc::new(std::sync::Mutex::new(None)),
            timed_out: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start_execution(&self, max_duration: Duration) {
        self.timed_out.store(false, Ordering::SeqCst);
        let mut d = self.deadline.lock().unwrap();
        *d = Some(Instant::now() + max_duration);
    }

    pub fn stop_execution(&self) -> bool {
        let mut d = self.deadline.lock().unwrap();
        *d = None;
        self.timed_out.load(Ordering::SeqCst)
    }

    pub fn is_expired(&self) -> bool {
        if let Ok(guard) = self.deadline.try_lock() {
            if let Some(deadline) = *guard {
                if Instant::now() > deadline {
                    self.timed_out.store(true, Ordering::SeqCst);
                    return true;
                }
            }
        }
        false
    }
}

impl Default for InterruptController {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper around QuickJS Runtime and Context.
pub struct ScriptRuntime {
    #[allow(dead_code)]
    runtime: Runtime,
    context: Context,
    config: SandboxConfig,
    interrupt_controller: InterruptController,
}

impl ScriptRuntime {
    /// Creates a new sandboxed ScriptRuntime.
    pub fn new(config: SandboxConfig) -> Result<Self, String> {
        let runtime =
            Runtime::new().map_err(|e| format!("Failed to create QuickJS runtime: {}", e))?;
        runtime.set_memory_limit(config.memory_limit_bytes);

        let interrupt_controller = InterruptController::new();
        let controller_clone = interrupt_controller.clone();

        runtime.set_interrupt_handler(Some(Box::new(move || controller_clone.is_expired())));

        let context = Context::full(&runtime)
            .map_err(|e| format!("Failed to create QuickJS context: {}", e))?;

        // Initialize sandbox by removing/disallowing any dangerous globals
        context.with(|ctx| {
            let global = ctx.globals();
            // Delete eval from global object to enforce sandbox
            let _ = global.remove("eval");
        });

        Ok(Self {
            runtime,
            context,
            config,
            interrupt_controller,
        })
    }

    /// Borrows the inner QuickJS context.
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Loads and transpiles a TypeScript script.
    pub fn load_typescript_module(
        &mut self,
        path: &str,
        source_ts: &str,
    ) -> Result<LoadedScript, ScriptExecutionError> {
        let transpiled =
            transpile_typescript(path, source_ts).map_err(|e| ScriptExecutionError {
                entity_id: String::new(),
                script_path: path.to_string(),
                message: e,
                stack: None,
                file: Some(path.to_string()),
                line: None,
                column: None,
            })?;

        static SCRIPT_RELOAD_COUNTER: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(1);
        let version = SCRIPT_RELOAD_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let source_mapper = ScriptSourceMapper::new(path, transpiled.source_map_json.as_deref());
        let module_name = format!("{}_{}", path.replace('\\', "/"), version);

        let (
            ns_persistent,
            has_start,
            has_update,
            has_fixed,
            has_col_enter,
            has_col_exit,
            has_trig_enter,
            has_destroy,
            has_serialize,
            has_deserialize,
        ) = self
            .context
            .with(|ctx| -> Result<_, ScriptExecutionError> {
                let module =
                    Module::declare(ctx.clone(), module_name.clone(), transpiled.code.clone())
                        .map_err(|e| {
                            self.format_quickjs_error(ctx.clone(), path, "", &source_mapper, e)
                        })?;

                let (eval_module, promise) = module.eval().map_err(|e| {
                    self.format_quickjs_error(ctx.clone(), path, "", &source_mapper, e)
                })?;

                promise.finish::<()>().map_err(|e| {
                    self.format_quickjs_error(ctx.clone(), path, "", &source_mapper, e)
                })?;

                let ns = eval_module.namespace().map_err(|e| {
                    self.format_quickjs_error(ctx.clone(), path, "", &source_mapper, e)
                })?;

                let has_start = ns.get::<_, Function>("onStart").is_ok();
                let has_update = ns.get::<_, Function>("onUpdate").is_ok();
                let has_fixed = ns.get::<_, Function>("onFixedUpdate").is_ok();
                let has_col_enter = ns.get::<_, Function>("onCollisionEnter").is_ok();
                let has_col_exit = ns.get::<_, Function>("onCollisionExit").is_ok();
                let has_trig_enter = ns.get::<_, Function>("onTriggerEnter").is_ok();
                let has_destroy = ns.get::<_, Function>("onDestroy").is_ok();
                let has_serialize = ns.get::<_, Function>("serialize").is_ok();
                let has_deserialize = ns.get::<_, Function>("deserialize").is_ok();
                let ns_persistent = Persistent::save(&ctx, ns);

                Ok((
                    ns_persistent,
                    has_start,
                    has_update,
                    has_fixed,
                    has_col_enter,
                    has_col_exit,
                    has_trig_enter,
                    has_destroy,
                    has_serialize,
                    has_deserialize,
                ))
            })?;

        Ok(LoadedScript {
            script_path: path.to_string(),
            source_mapper,
            module_name,
            transpiled_code: transpiled.code,
            namespace: ns_persistent,
            has_on_start: has_start,
            has_on_update: has_update,
            has_on_fixed_update: has_fixed,
            has_on_collision_enter: has_col_enter,
            has_on_collision_exit: has_col_exit,
            has_on_trigger_enter: has_trig_enter,
            has_on_destroy: has_destroy,
            has_serialize,
            has_deserialize,
        })
    }

    /// Invokes a named lifecycle function on a loaded script within the time budget.
    pub fn invoke_function<A, R>(
        &mut self,
        entity_id: &str,
        script: &LoadedScript,
        fn_name: &str,
        args: A,
    ) -> Result<R, ScriptExecutionError>
    where
        A: for<'js> rquickjs::function::IntoArgs<'js>,
        R: for<'js> rquickjs::FromJs<'js>,
    {
        self.interrupt_controller
            .start_execution(self.config.max_duration_per_frame);

        let result = self.context.with(|ctx| {
            let ns: Object = script.namespace.clone().restore(&ctx).map_err(|e| {
                self.format_quickjs_error(
                    ctx.clone(),
                    &script.script_path,
                    entity_id,
                    &script.source_mapper,
                    e,
                )
            })?;

            let func: Function = ns.get(fn_name).map_err(|e| {
                self.format_quickjs_error(
                    ctx.clone(),
                    &script.script_path,
                    entity_id,
                    &script.source_mapper,
                    e,
                )
            })?;

            let ret: R = func.call(args).map_err(|e| {
                self.format_quickjs_error(
                    ctx.clone(),
                    &script.script_path,
                    entity_id,
                    &script.source_mapper,
                    e,
                )
            })?;

            Ok(ret)
        });

        let timed_out = self.interrupt_controller.stop_execution();
        if timed_out {
            return Err(ScriptExecutionError {
                entity_id: entity_id.to_string(),
                script_path: script.script_path.clone(),
                message: format!(
                    "Script execution time exceeded maximum allowed frame duration ({} ms)",
                    self.config.max_duration_per_frame.as_millis()
                ),
                stack: None,
                file: Some(script.script_path.clone()),
                line: None,
                column: None,
            });
        }

        result
    }

    /// Formats a QuickJS error and maps line numbers using the script's source mapper.
    pub fn format_quickjs_error(
        &self,
        ctx: rquickjs::Ctx<'_>,
        script_path: &str,
        entity_id: &str,
        source_mapper: &ScriptSourceMapper,
        err: rquickjs::Error,
    ) -> ScriptExecutionError {
        let (message, stack, raw_line, raw_col) = match err {
            rquickjs::Error::Exception => {
                let val = ctx.catch();
                let mut stack_str = None;
                let mut line = None;
                let mut col = None;

                let msg = if let Some(exc) = val.as_exception() {
                    let m = exc
                        .message()
                        .unwrap_or_else(|| "Unknown script exception".to_string());
                    if let Some(stack) = exc.stack() {
                        for line_str in stack.lines() {
                            if let Some(pos) = line_str.rfind(':') {
                                if let Ok(c) =
                                    line_str[pos + 1..].trim_end_matches(')').parse::<u32>()
                                {
                                    if let Some(prev_pos) = line_str[..pos].rfind(':') {
                                        if let Ok(l) = line_str[prev_pos + 1..pos].parse::<u32>() {
                                            line = Some(l);
                                            col = Some(c);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        stack_str = Some(stack);
                    }
                    m
                } else if let Some(s) = val.as_string() {
                    s.to_string().unwrap_or_else(|_| format!("{:?}", val))
                } else {
                    format!("{:?}", val)
                };

                (msg, stack_str, line, col)
            }
            other => (other.to_string(), None, None, None),
        };

        let (mapped_file, mapped_line, mapped_col) = match (raw_line, raw_col) {
            (Some(l), Some(c)) => {
                let loc = source_mapper.lookup(l, c);
                (
                    Some(loc.file),
                    Some(loc.line as i64),
                    Some(loc.column as i64),
                )
            }
            (Some(l), None) => {
                let loc = source_mapper.lookup(l, 1);
                (Some(loc.file), Some(loc.line as i64), None)
            }
            _ => (Some(script_path.to_string()), None, None),
        };

        ScriptExecutionError {
            entity_id: entity_id.to_string(),
            script_path: script_path.to_string(),
            message,
            stack,
            file: mapped_file,
            line: mapped_line,
            column: mapped_col,
        }
    }
}
