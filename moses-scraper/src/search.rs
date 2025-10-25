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

    let content = std::fs::read_to_string(csv_path)?;
    parse_csv_content(&content, limit, None)
}

/// Parse CSV content from a string (for web interface uploads)
pub fn parse_csv_content(
    content: &str,
    limit: Option<usize>,
    url_pattern: Option<&str>,
) -> Result<Vec<ModuleRef>> {
    let mut reader = csv::Reader::from_reader(content.as_bytes());
    let mut modules = Vec::new();

    let default_pattern = "https://moseskonto.tu-berlin.de/moses/modultransfersystem/bolognamodule/beschreibung/anzeigen.html?nummer={number}&version={version}&sprache=1";
    let pattern = url_pattern.unwrap_or(default_pattern);

    for result in reader.records() {
        let record = result?;

        // CSV format: "Nummer/Version","Modultitel","Sprache(n)","LP","Benotung","Verantwortliche Person","Zugehörigkeit"
        let nummer_version = record.get(0).unwrap_or("");
        let title = record.get(1).unwrap_or("").to_string();

        // Parse "#50830 v2" format
        let (number, version) = match parse_number_version(nummer_version) {
            Ok(nv) => nv,
            Err(_) => {
                tracing::warn!("Failed to parse: {}", nummer_version);
                continue;
            }
        };

        // Build detail URL from pattern
        let detail_url = pattern
            .replace("{number}", &number.to_string())
            .replace("{version}", &version.to_string());

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

    Ok(modules)
}

/// Validate CSV content without parsing all modules
pub fn validate_csv_content(content: &str) -> Result<CsvValidationResult> {
    let mut reader = csv::Reader::from_reader(content.as_bytes());

    // Check headers
    let headers = reader.headers()?;
    if headers.len() < 7 {
        anyhow::bail!("Invalid CSV format: expected at least 7 columns, got {}", headers.len());
    }

    // Expected headers
    let expected = vec![
        "Nummer/Version",
        "Modultitel",
        "Sprache(n)",
        "LP",
        "Benotung",
        "Verantwortliche Person",
        "Zugehörigkeit"
    ];

    for (i, expected_header) in expected.iter().enumerate() {
        if let Some(actual) = headers.get(i) {
            if actual != *expected_header {
                anyhow::bail!(
                    "Invalid CSV header at column {}: expected '{}', got '{}'",
                    i + 1,
                    expected_header,
                    actual
                );
            }
        }
    }

    // Count valid modules
    let mut total_count = 0;
    let mut valid_count = 0;
    let mut invalid_count = 0;

    for result in reader.records() {
        total_count += 1;
        let record = result?;

        let nummer_version = record.get(0).unwrap_or("");
        if parse_number_version(nummer_version).is_ok() {
            valid_count += 1;
        } else {
            invalid_count += 1;
        }
    }

    Ok(CsvValidationResult {
        total_rows: total_count,
        valid_modules: valid_count,
        invalid_rows: invalid_count,
    })
}

#[derive(Debug, Clone)]
pub struct CsvValidationResult {
    pub total_rows: usize,
    pub valid_modules: usize,
    pub invalid_rows: usize,
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
