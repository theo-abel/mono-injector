use std::sync::OnceLock;
use std::time::Duration;

use console::Style;
use indicatif::{ProgressBar, ProgressStyle};

struct Palette {
    success: Style,
    info: Style,
    warn: Style,
    error: Style,
    label: Style,
    value: Style,
    muted: Style,
}

static PALETTE: OnceLock<Palette> = OnceLock::new();

fn palette() -> &'static Palette {
    PALETTE.get_or_init(|| Palette {
        success: Style::new().green().bold(),
        info: Style::new().cyan(),
        warn: Style::new().yellow(),
        error: Style::new().red().bold(),
        label: Style::new().bold(),
        value: Style::new().yellow(),
        muted: Style::new().dim(),
    })
}

pub fn success(msg: &str) {
    eprintln!("{} {msg}", palette().success.apply_to("[✓]"));
}

pub fn info(msg: &str) {
    eprintln!("{} {msg}", palette().info.apply_to("[i]"));
}

pub fn warn(msg: &str) {
    eprintln!("{} {msg}", palette().warn.apply_to("[!]"));
}

pub fn error(msg: &str) {
    eprintln!("{} {msg}", palette().error.apply_to("[x]"));
}

pub fn label_value(label: &str, value: &str) {
    eprintln!(
        "{} {}",
        palette().label.apply_to(label),
        palette().value.apply_to(value),
    );
}

pub fn muted(msg: &str) {
    eprintln!("{}", palette().muted.apply_to(msg));
}

/// Creates a pre-styled spinner for long-running operations.
///
/// Call [`ProgressBar::set_message`] to update the label and
/// [`ProgressBar::finish_and_clear`] when the operation completes.
pub fn spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .expect("valid progress style template")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_message("starting...");
    pb
}
