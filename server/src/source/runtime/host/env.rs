use wasmer::FunctionEnvMut;

use super::HostState;

pub(super) fn print(env: FunctionEnvMut<HostState>, pointer: i32, length: i32) {
    if pointer < 0 || length <= 0 {
        return;
    }
    match env.data().read_string(&env, pointer, length) {
        Ok(message) => tracing::debug!(source = %env.data().source_id, %message, "漫画源日志"),
        Err(error) => tracing::warn!(source = %env.data().source_id, %error, "无法读取漫画源日志"),
    }
}

pub(super) fn abort(env: FunctionEnvMut<HostState>) {
    tracing::warn!(source = %env.data().source_id, "漫画源主动终止执行");
}

pub(super) fn send_partial_result(mut env: FunctionEnvMut<HostState>, pointer: i32) {
    if pointer < 0 {
        return;
    }
    if let Ok((is_error, bytes)) = env.data().read_result(&env, pointer as u32) {
        if !is_error {
            env.data_mut().partial_results.push(bytes);
        }
    }
}
