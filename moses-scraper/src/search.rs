use anyhow::Result;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ModuleRef {
    pub number: i32,
    pub version: i32,
    pub title: String,
    pub detail_url: String,
}

/// Fetch all modules from the exported CSV file
pub async fn fetch_all_modules(_search_url: &str, limit: Option<usize>) -> Result<Vec<ModuleRef>> {
    let csv_path = "Modul_export.csv";

    eprintln!("Loading modules from {}...", csv_path);

    if !Path::new(csv_path).exists() {
        anyhow::bail!("CSV file not found: {}. Please export the module list from MOSES and place it in the project directory.", csv_path);
    }

    let mut reader = csv::Reader::from_path(csv_path)?;
    let mut modules = Vec::new();

    for result in reader.records() {
        let record = result?;

        // CSV format: "Nummer/Version","Modultitel","Sprache(n)","LP","Benotung","Verantwortliche Person","Zugehörigkeit"
        let nummer_version = record.get(0).unwrap_or("");
        let title = record.get(1).unwrap_or("").to_string();

        // Parse "#50830 v2" format
        let (number, version) = match parse_number_version(nummer_version) {
            Ok(nv) => nv,
            Err(_) => {
                eprintln!("Warning: Failed to parse: {}", nummer_version);
                continue;
            }
        };

        // Build detail URL
        let detail_url = format!(
            "https://moseskonto.tu-berlin.de/moses/modultransfersystem/bolognamodule/beschreibung/anzeigen.html?nummer={}&version={}&sprache=1",
            number, version
        );

        modules.push(ModuleRef {
            number,
            version,
            title,
            detail_url,
        });

        if let Some(limit) = limit {
            if modules.len() >= limit {
                break;
            }
        }
    }

    eprintln!("Loaded {} modules from CSV", modules.len());
    Ok(modules)
}

fn parse_number_version(text: &str) -> Result<(i32, i32)> {
    // Format is "#50830 v2"
    let text = text.trim();

    // Remove leading '#' if present
    let text = text.strip_prefix('#').unwrap_or(text);

    // Split by whitespace
    let parts: Vec<&str> = text.split_whitespace().collect();

    if parts.len() < 2 {
        anyhow::bail!("Invalid number/version format: {}", text);
    }

    // Extract number (first part)
    let number = parts[0].parse::<i32>()?;

    // Extract version (after 'v')
    let version_part = parts.iter()
        .find(|s| s.starts_with('v'))
        .ok_or_else(|| anyhow::anyhow!("Failed to find version"))?;

    let version_str = version_part.strip_prefix('v').unwrap_or(version_part);
    let version = version_str.parse::<i32>()?;

    Ok((number, version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number_version() {
        assert_eq!(parse_number_version("#50123 v5").unwrap(), (50123, 5));
        assert_eq!(parse_number_version("50123 v5").unwrap(), (50123, 5));
        assert_eq!(parse_number_version("  #50123  v5  ").unwrap(), (50123, 5));
    }
}
