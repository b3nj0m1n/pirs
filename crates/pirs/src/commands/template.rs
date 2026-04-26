use anyhow::Result;

pub fn list() -> Result<()> {
    for name in pirs_core::template::BUILTIN {
        println!("{name}");
    }
    Ok(())
}

pub fn show(name: &str) -> Result<()> {
    use pirs_core::{IncidentSeverity, IncidentType, Pir, template};
    // Render a placeholder PIR to give a preview.
    let mut p = Pir::new(0, "<title>");
    p.problem_statement = "<problem_statement>".into();
    p.severity = IncidentSeverity::Low;
    p.incident_type = match name {
        "production" => IncidentType::Production,
        "security" => IncidentType::Security,
        "process" => IncidentType::Process,
        _ => IncidentType::Development,
    };
    let body = template::render(&p, name)?;
    println!("Variables: number, title, problem_statement, severity, incident_type\n");
    println!("---\n{body}");
    Ok(())
}
