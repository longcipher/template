use std::path::Path;

use console::{Emoji, style};

// Emoji constants
pub static CHECKMARK: Emoji<'_, '_> = Emoji("✓ ", "√ ");
#[allow(dead_code)]
pub static CROSS: Emoji<'_, '_> = Emoji("✗ ", "x ");
pub static SKIP: Emoji<'_, '_> = Emoji("⊘ ", "o ");
pub static FOLDER: Emoji<'_, '_> = Emoji("📁 ", "");
pub static SPINNER: Emoji<'_, '_> = Emoji("⠋ ", "* ");

/// Print a success message in green
#[allow(dead_code)]
pub fn success(msg: &str) {
    println!("{} {}", CHECKMARK, style(msg).green());
}

/// Print an error message in red
#[allow(dead_code)]
pub fn error(msg: &str) {
    eprintln!("{} {}", CROSS, style(msg).red());
}

/// Print info about a cloning operation
pub fn cloning(owner: &str, repo: &str) {
    println!("{} Cloning {}/{}...", SPINNER, style(owner).cyan(), style(repo).cyan());
}

/// Print success after cloning
pub fn cloned() {
    println!("{} {}\n", CHECKMARK, style("Cloned successfully").green());
}

/// Print info about using a local template
pub fn local_template(path: &Path) {
    println!("{} Using local template: {}\n", FOLDER, style(path.display()).cyan());
}

/// Print info about applying template
pub fn applying(dest: &Path) {
    println!("Applying template to {}...", style(dest.display()).cyan());
}

/// Print a file that was applied
pub fn file_applied(path: &Path) {
    println!("  {} {}", CHECKMARK, style(path.display()).green());
}

/// Print a file that was skipped
pub fn file_skipped(path: &Path) {
    println!("  {} {} {}", SKIP, style(path.display()).yellow(), style("(exists)").dim());
}

/// Print a directory that was created (verbose mode)
pub fn dir_created(path: &Path) {
    println!("  {} Created: {}", FOLDER, style(path.display()).dim());
}

/// Print the final summary
pub fn summary(applied: usize, skipped: usize) {
    println!("\n{}", style("━".repeat(40)).dim());
    println!("{} {}", CHECKMARK, style("Done!").green().bold());
    println!("  Applied: {} files", style(applied).green());
    if skipped > 0 {
        println!("  Skipped: {} files {}", style(skipped).yellow(), style("(already exist)").dim());
    }
}
