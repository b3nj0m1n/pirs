use anyhow::Result;
use pirs_core::discover;
use std::path::Path;

pub fn run(cwd: &Path) -> Result<()> {
    let d = discover(cwd)?;
    println!("project_root: {}", d.root.display());
    println!("source: {:?}", d.source);
    println!("pir_dir: {}", d.config.pir_dir.display());
    println!(
        "resolved_pir_dir: {}",
        d.config.pir_path(&d.root).display()
    );
    println!(
        "templates.default: {:?}",
        d.config.templates.default
    );
    println!("templates.custom: {:?}", d.config.templates.custom);
    println!("privacy.redaction_patterns: {:?}", d.config.privacy.redaction_patterns);
    println!("privacy.sensitive_fields: {:?}", d.config.privacy.sensitive_fields);
    println!("mcp.http_enabled: {}", d.config.mcp.http_enabled);
    println!("mcp.http_bind: {:?}", d.config.mcp.http_bind);
    Ok(())
}
