use anyhow::Result;
use crossterm::terminal;
use std::io::{self, Write};

/// 跨平台的原始模式安全打印函数
pub fn raw_println(text: &str) -> Result<()> {
    let mut stdout = io::stdout();

    // 在原始模式下，需要手动处理换行
    // Windows 和 Unix 系统在原始模式下都需要 \r\n
    write!(stdout, "{}\r\n", text)?;
    stdout.flush()?;
    Ok(())
}

/// 跨平台的原始模式安全打印函数（不换行）
pub fn raw_print(text: &str) -> Result<()> {
    let mut stdout = io::stdout();
    write!(stdout, "{}", text)?;
    stdout.flush()?;
    Ok(())
}

/// 临时禁用原始模式执行函数，用于处理第三方库的输出
pub fn with_normal_mode<F, R>(f: F) -> Result<R>
where
    F: FnOnce() -> Result<R>,
{
    // 临时禁用原始模式
    terminal::disable_raw_mode()?;

    // 执行函数
    let result = f();

    // 重新启用原始模式
    terminal::enable_raw_mode()?;

    result
}

/// 异步版本的临时禁用原始模式
pub async fn with_normal_mode_async<F, Fut, R, E>(f: F) -> Result<R>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<R, E>>,
    E: Into<anyhow::Error>,
{
    // 临时禁用原始模式
    terminal::disable_raw_mode()?;

    // 执行异步函数
    let result = f().await.map_err(|e| e.into());

    // 重新启用原始模式
    terminal::enable_raw_mode()?;

    result
}

/// 格式化打印宏，类似 println! 但在原始模式下安全
#[macro_export]
macro_rules! raw_println {
    () => {
        $crate::kota_cli::utils::raw_println("")
    };
    ($($arg:tt)*) => {
        $crate::kota_cli::utils::raw_println(&format!($($arg)*))
    };
}

/// 格式化打印宏，类似 print! 但在原始模式下安全
#[macro_export]
macro_rules! raw_print {
    ($($arg:tt)*) => {
        $crate::kota_cli::utils::raw_print(&format!($($arg)*))
    };
}
