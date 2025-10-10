use anyhow::{Context, Result};
use scraper::{Html, Selector};
use tokio::time::{sleep, Duration};

use crate::models::{ScrapedModule, ScrapedComponent, ScrapedWorkload, ScrapedStudyProgramUsage, ScrapedExam, ScrapedExamComponent};

pub async fn fetch_module_details(url: &str, retries: u32) -> Result<Option<ScrapedModule>> {
    let mut attempts = 0;
    let response = loop {
        match reqwest::get(url).await {
            Ok(resp) => {
                // Check for redirect to login page
                if resp.url().path().contains("login") || resp.url().path().contains("shibboleth") {
                    return Ok(None);
                }
                break resp;
            }
            Err(_e) if attempts < retries => {
                attempts += 1;
                sleep(Duration::from_secs(2_u64.pow(attempts))).await;
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    };

    let html = response.text().await.context("Failed to read response body")?;
    let document = Html::parse_document(&html);

    // Extract module number and version from URL
    let (number, version) = extract_number_version_from_url(url)?;

    let mut module = ScrapedModule {
        number,
        version,
        title: String::new(),
        credits: 0,
        languages: Vec::new(),
        valid_since: None,
        valid_until: None,
        faculty: None,
        institute: None,
        fachgebiet: None,
        responsible_person: None,
        examination_board: None,
        contact_email: None,
        contact_person: None,
        secretariat: None,
        website: None,
        learning_result: None,
        content: None,
        teaching_information: None,
        requirements: None,
        additional_info: None,
        registration: None,
        max_attendees: None,
        duration: None,
        components: Vec::new(),
        workload: Vec::new(),
        study_programs: Vec::new(),
        m_pord_nr: None,
        m_p_nr: None,
        mp_pord_nr: None,
        mp_p_nr: None,
        moses_link: url.to_string(),
        exam: None,
    };

    // Parse basic info from header
    parse_header(&document, &mut module)?;

    // Parse sections
    parse_zugehoerigkeit(&document, &mut module);
    parse_kontakt(&document, &mut module);
    parse_lernergebnisse(&document, &mut module);
    parse_lehrinhalte(&document, &mut module);
    parse_lehrformen(&document, &mut module);
    parse_voraussetzungen(&document, &mut module);
    parse_modulbestandteile(&document, &mut module);
    parse_arbeitsaufwand(&document, &mut module);
    parse_verwendung(&document, &mut module);
    parse_additional_info(&document, &mut module);
    parse_exam(&document, &mut module);

    Ok(Some(module))
}

fn extract_number_version_from_url(url: &str) -> Result<(i32, i32)> {
    let url_parts: Vec<&str> = url.split('?').collect();
    if url_parts.len() < 2 {
        anyhow::bail!("Invalid URL format");
    }

    let mut number = None;
    let mut version = None;

    for param in url_parts[1].split('&') {
        let kv: Vec<&str> = param.split('=').collect();
        if kv.len() == 2 {
            match kv[0] {
                "nummer" => number = Some(kv[1].parse::<i32>()?),
                "version" => version = Some(kv[1].parse::<i32>()?),
                _ => {}
            }
        }
    }

    Ok((
        number.context("Missing nummer parameter")?,
        version.context("Missing version parameter")?,
    ))
}

fn parse_header(document: &Html, module: &mut ScrapedModule) -> Result<()> {
    // Extract title from h1 tag
    let h1_selector = Selector::parse("h1").unwrap();
    if let Some(h1_elem) = document.select(&h1_selector).next() {
        let title = h1_elem.text().collect::<String>().trim().to_string();
        if !title.is_empty() {
            module.title = title;
        }
    }

    // Get all text content for credits extraction
    let body_selector = Selector::parse("body").unwrap();
    if let Some(body) = document.select(&body_selector).next() {
        let full_text = body.text().collect::<String>();

        // Extract credits - look for number before "LP"
        for line in full_text.lines() {
            if line.contains("Leistungspunkte") || line.contains(" LP") {
                // Extract number before "LP"
                if let Some(lp_pos) = line.find("LP") {
                    let before_lp = &line[..lp_pos];
                    // Get last sequence of digits
                    let digits: String = before_lp.chars()
                        .rev()
                        .take_while(|c| c.is_numeric() || c.is_whitespace())
                        .filter(|c| c.is_numeric())
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect();
                    if let Ok(credits) = digits.parse::<i32>() {
                        module.credits = credits;
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

fn parse_zugehoerigkeit(document: &Html, module: &mut ScrapedModule) {
    let body_selector = Selector::parse("body").unwrap();
    if let Some(body) = document.select(&body_selector).next() {
        let full_text = body.text().collect::<String>();

        // Extract faculty
        if let Some(fak_pos) = full_text.find("Fakultät") {
            let after = &full_text[fak_pos..];
            if let Some(line_end) = after.find('\n') {
                let line = &after[..line_end];
                module.faculty = Some(extract_value_after_label(line, "Fakultät"));
            }
        }

        // Extract institute
        if let Some(inst_pos) = full_text.find("Institut") {
            let after = &full_text[inst_pos..];
            if let Some(line_end) = after.find('\n') {
                let line = &after[..line_end];
                module.institute = Some(extract_value_after_label(line, "Institut"));
            }
        }

        // Extract Fachgebiet - look for lines with FG or numbered Fachgebiet
        for line in full_text.lines() {
            if (line.contains("FG ") || line.contains("Fachgebiet")) &&
               line.chars().any(|c| c.is_numeric()) {
                module.fachgebiet = Some(line.trim().to_string());
                break;
            }
        }

        // Extract examination board
        if let Some(pruef_pos) = full_text.find("Prüfungsausschuss") {
            let after = &full_text[pruef_pos..];
            if let Some(line_end) = after.find('\n') {
                let line = &after[..line_end];
                module.examination_board = Some(extract_value_after_label(line, "Prüfungsausschuss"));
            }
        }

        // Extract responsible person - look for "Modulverantwortliche*r"
        // Try multiple patterns
        for pattern in &["Modulverantwortliche*r", "Modulverantwortlicher", "Modulverantwortliche"] {
            if let Some(resp_pos) = full_text.find(pattern) {
                let after = &full_text[resp_pos + pattern.len()..];
                // Get the next line that looks like a name (contains a comma or has capitalized words)
                for line in after.lines().take(5) {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.contains("Modulverantwortliche") {
                        continue;
                    }
                    // Check if it looks like a name: has comma (LastName, FirstName) or starts with capital
                    if trimmed.contains(',') || (trimmed.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) && trimmed.len() > 3) {
                        module.responsible_person = Some(trimmed.to_string());
                        break;
                    }
                }
                if module.responsible_person.is_some() {
                    break;
                }
            }
        }
    }
}

fn parse_kontakt(document: &Html, module: &mut ScrapedModule) {
    // Parse contact information by looking for form-group divs with labels
    let form_group_selector = Selector::parse(".form-group").unwrap();

    for form_group in document.select(&form_group_selector) {
        let group_text = form_group.text().collect::<String>();

        // Check if this form group contains contact-related labels
        if group_text.contains("Sekretariat") {
            let value = group_text.replace("Sekretariat", "").trim().to_string();
            if !value.is_empty() && value != "Keine Angabe" {
                module.secretariat = Some(value);
            }
        } else if group_text.contains("Ansprechpartner") {
            let value = group_text
                .replace("Ansprechpartner*in", "")
                .replace("Ansprechpartnerin", "")
                .replace("Ansprechpartner", "")
                .trim()
                .to_string();
            if !value.is_empty() && value != "Keine Angabe" {
                module.contact_person = Some(value);
            }
        } else if group_text.contains("E-Mail") {
            let value = group_text.replace("E-Mail-Adresse", "").replace("E-Mail", "").trim().to_string();
            if !value.is_empty() && value != "Keine Angabe" {
                // Extract just the email address
                if let Some(at_pos) = value.find('@') {
                    let start = value[..at_pos].rfind(|c: char| c.is_whitespace() || !c.is_alphanumeric() && c != '.' && c != '-' && c != '_')
                        .map(|pos| pos + 1)
                        .unwrap_or(0);
                    let end = value[at_pos..].find(|c: char| c.is_whitespace())
                        .map(|pos| at_pos + pos)
                        .unwrap_or(value.len());
                    let email = value[start..end].trim().to_string();
                    module.contact_email = Some(email);
                }
            }
        } else if group_text.contains("Webseite") || group_text.contains("Website") {
            let value = group_text.replace("Webseite", "").replace("Website", "").trim().to_string();
            if !value.is_empty() && value != "Keine Angabe" && (value.contains("http") || value.contains("www")) {
                module.website = Some(value);
            }
        }
    }
}

fn parse_lernergebnisse(document: &Html, module: &mut ScrapedModule) {
    if let Some(text) = extract_section_content(document, "Lernergebnisse") {
        module.learning_result = Some(text);
    } else if let Some(text) = extract_section_content(document, "Qualifikationsziele") {
        module.learning_result = Some(text);
    }
}

fn parse_lehrinhalte(document: &Html, module: &mut ScrapedModule) {
    if let Some(text) = extract_section_content(document, "Lehrinhalte") {
        module.content = Some(text);
    }
}

fn parse_lehrformen(document: &Html, module: &mut ScrapedModule) {
    if let Some(text) = extract_section_content(document, "Lehrformen") {
        module.teaching_information = Some(text);
    } else if let Some(text) = extract_section_content(document, "Beschreibung der Lehr- und Lernformen") {
        module.teaching_information = Some(text);
    }
}

fn parse_voraussetzungen(document: &Html, module: &mut ScrapedModule) {
    if let Some(text) = extract_section_content(document, "Voraussetzungen") {
        module.requirements = Some(text);
    }
}

fn parse_modulbestandteile(document: &Html, module: &mut ScrapedModule) {
    let table_selector = Selector::parse("table").unwrap();
    let row_selector = Selector::parse("tr").unwrap();
    let cell_selector = Selector::parse("td, th").unwrap();

    for table in document.select(&table_selector) {
        let rows: Vec<_> = table.select(&row_selector).collect();
        if rows.is_empty() {
            continue;
        }

        // Get header row to identify columns
        let header_cells: Vec<String> = rows[0]
            .select(&cell_selector)
            .map(|cell| cell.text().collect::<String>().trim().to_lowercase())
            .collect();

        // Check if this is a component table
        if !header_cells.iter().any(|h| h.contains("art") || h.contains("typ")) {
            continue;
        }

        // Find column indices
        let art_idx = header_cells.iter().position(|h| h.contains("art") || h == "typ");
        let nummer_idx = header_cells.iter().position(|h| h.contains("nummer") || h.contains("nr"));
        let turnus_idx = header_cells.iter().position(|h| h.contains("turnus") || h.contains("rotation"));
        let sprache_idx = header_cells.iter().position(|h| h.contains("sprache") || h.contains("language"));
        let sws_idx = header_cells.iter().position(|h| h.contains("sws"));
        let name_idx = header_cells.iter().position(|h| h.contains("lehrveranstaltung") || h.contains("name") || h.contains("titel"));

        // Parse data rows
        for row in rows.iter().skip(1) {
            let cells: Vec<String> = row
                .select(&cell_selector)
                .map(|cell| cell.text().collect::<String>().trim().to_string())
                .collect();

            if cells.is_empty() {
                continue;
            }

            let component_type = art_idx.and_then(|i| cells.get(i)).map(|s| s.clone()).unwrap_or_default();
            if component_type.is_empty() || component_type.len() > 10 {
                continue; // Skip invalid rows
            }

            let name = name_idx.and_then(|i| cells.get(i)).map(|s| s.clone());
            let number = nummer_idx.and_then(|i| cells.get(i)).map(|s| s.clone()).unwrap_or_default();
            let rotation = turnus_idx
                .and_then(|i| cells.get(i))
                .map(|s| s.clone())
                .unwrap_or_else(|| "SoSe".to_string());
            let language = sprache_idx
                .and_then(|i| cells.get(i))
                .map(|s| s.clone())
                .unwrap_or_else(|| "de".to_string());
            let sws = sws_idx
                .and_then(|i| cells.get(i))
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);

            module.components.push(ScrapedComponent {
                name,
                component_type,
                number,
                rotation,
                sws,
                language,
            });
        }
    }
}

fn parse_arbeitsaufwand(document: &Html, module: &mut ScrapedModule) {
    let table_selector = Selector::parse("table").unwrap();
    let row_selector = Selector::parse("tr").unwrap();
    let cell_selector = Selector::parse("td, th").unwrap();

    for table in document.select(&table_selector) {
        let rows: Vec<_> = table.select(&row_selector).collect();
        if rows.is_empty() {
            continue;
        }

        // Get header row
        let header_cells: Vec<String> = rows[0]
            .select(&cell_selector)
            .map(|cell| cell.text().collect::<String>().trim().to_lowercase())
            .collect();

        // Check if this is a workload table
        let is_workload_table = header_cells.iter().any(|h| h.contains("aufwand") || h.contains("stunden"));

        if !is_workload_table {
            continue;
        }

        // Find column indices
        let desc_idx = header_cells.iter().position(|h|
            h.contains("beschreibung") || h.contains("aufwand")
        ).unwrap_or(0);
        let multiplikator_idx = header_cells.iter().position(|h| h.contains("multiplikator"));
        let stunden_idx = header_cells.iter().position(|h|
            h.contains("stunden") && !h.contains("gesamt")
        );
        let gesamt_idx = header_cells.iter().position(|h| h.contains("gesamt"))
            .or_else(|| Some(header_cells.len().saturating_sub(1)));

        // Parse data rows
        for row in rows.iter().skip(1) {
            let cells: Vec<String> = row
                .select(&cell_selector)
                .map(|cell| cell.text().collect::<String>().trim().to_string())
                .collect();

            if cells.is_empty() {
                continue;
            }

            let description = cells.get(desc_idx).map(|s| s.clone()).unwrap_or_default();
            if description.is_empty() {
                continue;
            }

            let factor = multiplikator_idx
                .and_then(|i| cells.get(i))
                .and_then(|s| {
                    let cleaned = s.replace(",", ".").replace("h", "");
                    cleaned.trim().parse::<f64>().ok()
                });

            let hours = stunden_idx
                .and_then(|i| cells.get(i))
                .and_then(|s| {
                    let cleaned = s.replace(",", ".").replace("h", "");
                    cleaned.trim().parse::<f64>().ok()
                });

            let total_hours = gesamt_idx
                .and_then(|i| cells.get(i))
                .and_then(|s| {
                    let cleaned = s.replace(",", ".").replace("h", "");
                    cleaned.trim().parse::<f64>().ok()
                })
                .unwrap_or(0.0);

            if total_hours > 0.0 {
                module.workload.push(ScrapedWorkload {
                    description,
                    factor,
                    hours,
                    total_hours,
                });
            }
        }
    }
}

fn parse_verwendung(document: &Html, module: &mut ScrapedModule) {
    // Parse "Verwendung in Studiengängen" table
    let table_selector = Selector::parse("table").unwrap();
    let row_selector = Selector::parse("tr").unwrap();
    let cell_selector = Selector::parse("td, th").unwrap();

    for table in document.select(&table_selector) {
        let rows: Vec<_> = table.select(&row_selector).collect();
        if rows.is_empty() {
            continue;
        }

        let header_text = rows[0].text().collect::<String>();
        if !header_text.contains("Studiengang") && !header_text.contains("Verwendung") {
            continue;
        }

        for row in rows.iter().skip(1) {
            let cells: Vec<_> = row.select(&cell_selector).collect();
            if cells.len() < 5 {
                continue;
            }

            // First cell is expand button, second cell contains the study program name
            let link_selector = Selector::parse("a").unwrap();
            let study_program = cells[1]
                .select(&link_selector)
                .next()
                .map(|a| a.text().collect::<String>().trim().to_string())
                .unwrap_or_else(|| cells[1].text().collect::<String>().trim().to_string());

            let stupo = cells.get(2).map(|c| c.text().collect::<String>().trim().to_string()).unwrap_or_default();
            let first_usage = cells.get(cells.len() - 2).map(|c| c.text().collect::<String>().trim().to_string()).unwrap_or_default();
            let last_usage = cells.last().map(|c| c.text().collect::<String>().trim().to_string()).unwrap_or_default();

            if !study_program.is_empty() && !study_program.starts_with("$(function") {
                module.study_programs.push(ScrapedStudyProgramUsage {
                    study_program_name: study_program,
                    study_program_link: None,
                    stupo_name: stupo,
                    stupo_link: None,
                    first_usage,
                    last_usage,
                });
            }
        }
    }
}

fn parse_additional_info(document: &Html, module: &mut ScrapedModule) {
    let body_selector = Selector::parse("body").unwrap();
    if let Some(body) = document.select(&body_selector).next() {
        let full_text = body.text().collect::<String>();

        // Extract validity period - look for "Gültigkeit" or "Seit" patterns
        for line in full_text.lines() {
            let line_lower = line.to_lowercase();
            if (line_lower.contains("gültig") || line_lower.contains("seit")) &&
               (line.contains("SS") || line.contains("WS") || line.contains("SoSe") || line.contains("WiSe")) {
                // Extract the validity info
                let cleaned = line
                    .replace("Gültigkeit", "")
                    .replace("Gültig seit", "")
                    .replace("Seit", "")
                    .trim()
                    .to_string();
                if !cleaned.is_empty() {
                    module.valid_since = Some(cleaned);
                    break;
                }
            }
        }

        // Extract max attendees from section
        if let Some(text) = extract_section_content(document, "Maximale teilnehmende Personen") {
            // Look for "beträgt" followed by a number
            if let Some(betraegt_pos) = text.find("beträgt") {
                let after = &text[betraegt_pos + "beträgt".len()..];
                // Find first sequence of digits
                for word in after.split_whitespace() {
                    let digits: String = word.chars().filter(|c| c.is_numeric()).collect();
                    if !digits.is_empty() {
                        if let Ok(num) = digits.parse::<i32>() {
                            module.max_attendees = Some(num);
                            break;
                        }
                    }
                }
            }
        }

        // Extract registration/Anmeldeformalitäten section
        if let Some(text) = extract_section_content(document, "Anmeldeformalitäten") {
            module.registration = Some(text);
        } else if let Some(text) = extract_section_content(document, "Anmeldemodalitäten") {
            module.registration = Some(text);
        } else if let Some(text) = extract_section_content(document, "Anmeldung") {
            module.registration = Some(text);
        }

        // Extract duration from "Dauer des Moduls" section - store the whole content
        module.duration = extract_section_content(document, "Dauer des Moduls")
            .or_else(|| extract_section_content(document, "Dauer"));

        // Extract languages from the page (not just from module list)
        for line in full_text.lines() {
            let line_lower = line.to_lowercase();
            if line_lower.contains("unterrichtssprache") || line_lower.contains("sprache") {
                if line.contains("Deutsch") || line.contains("de") {
                    if !module.languages.contains(&"de".to_string()) {
                        module.languages.push("de".to_string());
                    }
                }
                if line.contains("English") || line.contains("en") {
                    if !module.languages.contains(&"en".to_string()) {
                        module.languages.push("en".to_string());
                    }
                }
            }
        }
    }
}

fn extract_section_content(document: &Html, section_header: &str) -> Option<String> {
    let body_selector = Selector::parse("body").unwrap();
    if let Some(body) = document.select(&body_selector).next() {
        let full_text = body.text().collect::<String>();

        // Find the section header - look for it at the start of a line
        let mut start_pos = None;
        for (idx, line) in full_text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed == section_header || trimmed.starts_with(section_header) {
                // Calculate the byte position in the full text
                let lines_before: Vec<&str> = full_text.lines().take(idx).collect();
                let bytes_before: usize = lines_before.iter().map(|l| l.len() + 1).sum(); // +1 for newline
                start_pos = Some(bytes_before);
                break;
            }
        }

        if let Some(start_pos) = start_pos {
            // Skip past the header line
            let after_start = &full_text[start_pos..];
            let after_header = if let Some(newline_pos) = after_start.find('\n') {
                &after_start[newline_pos + 1..]
            } else {
                after_start
            };

            // List of potential next section headers to stop at
            let next_sections = [
                "Modulbestandteile",
                "Arbeitsaufwand",
                "Prüfungsform",
                "Verwendung in Studiengängen",
                "Lernergebnisse",
                "Lehrinhalte",
                "Lehrformen",
                "Voraussetzungen",
                "Literatur",
                "Anmerkungen",
                "Maximale teilnehmende Personen",
                "Anmeldeformalitäten",
                "Dauer des Moduls",
                "Abschluss des Moduls",
                "Kontakt",
                "Zugehörigkeit",
            ];

            // Find the nearest next section header (must be at line start)
            let mut end_pos = after_header.len();
            for line in after_header.lines() {
                let trimmed = line.trim();
                for next_section in &next_sections {
                    if trimmed == *next_section || trimmed.starts_with(next_section) {
                        // Found a section header - calculate position
                        if let Some(pos) = after_header.find(line) {
                            end_pos = end_pos.min(pos);
                            break;
                        }
                    }
                }
                if end_pos < after_header.len() {
                    break;
                }
            }

            // Extract and clean the content
            let content = after_header[..end_pos].trim();
            if !content.is_empty() && content.len() > 3 {
                return Some(content.to_string());
            }
        }
    }
    None
}

fn extract_value_after_label(line: &str, label: &str) -> String {
    // Remove the label and extract the value
    line.replace(label, "")
        .trim()
        .to_string()
}

fn find_section_text(document: &Html, headers: &[&str]) -> Option<String> {
    for header in headers {
        if let Some(text) = extract_section_content(document, header) {
            return Some(text);
        }
    }
    None
}

fn parse_exam(document: &Html, module: &mut ScrapedModule) {
    let body_selector = Selector::parse("body").unwrap();
    let Some(body) = document.select(&body_selector).next() else {
        return;
    };

    let full_text = body.text().collect::<String>();

    // Early return if no exam section
    let Some(start_pos) = full_text.find("Abschluss des Moduls") else {
        return;
    };

    let after_section = &full_text[start_pos..];

    // Parse graded status (Benotet vs Unbenotet)
    // Look for the "Benotung" field specifically
    let graded = if let Some(pos) = after_section.find("Benotung") {
        let after_benotung = &after_section[pos..];
        // Take the next few lines and check for "Benotet" or "Unbenotet"
        let mut found_benotet = false;
        let mut found_unbenotet = false;
        for line in after_benotung.lines().take(3) {
            let trimmed = line.trim();
            if trimmed == "Benotet" {
                found_benotet = true;
                break;
            } else if trimmed == "Unbenotet" || trimmed.contains("Unbenotet") {
                found_unbenotet = true;
                break;
            }
        }
        found_benotet || !found_unbenotet
    } else {
        true // default to graded
    };

    // Parse exam type
    let mut exam_type = None;
    if let Some(pos) = after_section.find("Prüfungsform") {
        let after_form = &after_section[pos + "Prüfungsform".len()..];

        // First try to extract from the same line (common case: "PrüfungsformMündliche Prüfung")
        if let Some(first_line) = after_form.lines().next() {
            let trimmed = first_line.trim();
            // Check if there's text immediately after "Prüfungsform" on the same line
            if !trimmed.is_empty()
                && !trimmed.contains("Benotung")
                && !trimmed.contains("Sprache")
                && !trimmed.contains("Art der") {
                // Extract until we hit another field or newline
                let extracted = if let Some(end_pos) = trimmed.find("Sprache") {
                    &trimmed[..end_pos]
                } else if let Some(end_pos) = trimmed.find("Art der") {
                    &trimmed[..end_pos]
                } else {
                    trimmed
                };

                let cleaned = extracted.trim();
                if !cleaned.is_empty() && cleaned.len() > 3 {
                    exam_type = Some(cleaned.to_string());
                }
            }
        }
    }

    // Parse language (optional, mainly for completeness)
    let mut language = None;
    if let Some(pos) = after_section.find("Sprache(n)") {
        let after_lang = &after_section[pos + "Sprache(n)".len()..];
        if let Some(first_line) = after_lang.lines().next() {
            let trimmed = first_line.trim();
            if !trimmed.is_empty() {
                // Extract until we hit another field
                let extracted = if let Some(end_pos) = trimmed.find("Dauer") {
                    &trimmed[..end_pos]
                } else if let Some(end_pos) = trimmed.find("Prüfungs") {
                    &trimmed[..end_pos]
                } else {
                    trimmed
                };

                let cleaned = extracted.trim();
                if !cleaned.is_empty() {
                    language = Some(cleaned.to_string());
                }
            }
        }
    }

    // Parse duration/scope
    let mut duration_scope = None;
    if let Some(pos) = after_section.find("Dauer/Umfang") {
        let after_duration = &after_section[pos + "Dauer/Umfang".len()..];
        if let Some(first_line) = after_duration.lines().next() {
            let trimmed = first_line.trim();
            if !trimmed.is_empty() {
                // Extract until we hit another field
                let extracted = if let Some(end_pos) = trimmed.find("Prüfungs") {
                    &trimmed[..end_pos]
                } else if let Some(end_pos) = trimmed.find("Notenschlüssel") {
                    &trimmed[..end_pos]
                } else {
                    trimmed
                };

                let cleaned = extracted.trim();
                if !cleaned.is_empty() {
                    duration_scope = Some(cleaned.to_string());
                }
            }
        }
    }

    // Parse description (Prüfungsbeschreibung)
    let description = if after_section.contains("Prüfungsbeschreibung") {
        extract_section_content(document, "Prüfungsbeschreibung")
    } else {
        None
    };

    // Parse Notenschlüssel (grading key) - simplified to just extract the section
    let clef = if after_section.contains("Notenschlüssel") {
        // Try to extract a simple text representation
        if let Some(pos) = after_section.find("Notenschlüssel") {
            let after_clef = &after_section[pos..];
            let mut clef_lines = Vec::new();
            for line in after_clef.lines().skip(1).take(30) {
                let trimmed = line.trim();
                if trimmed.contains("Prüfungsbeschreibung")
                    || trimmed.contains("Dauer des Moduls")
                    || trimmed.contains("Sonstiges") {
                    break;
                }
                if !trimmed.is_empty()
                    && trimmed != "Notenschlüssel"
                    && !trimmed.contains("fa-star") {
                    clef_lines.push(trimmed.to_string());
                }
            }
            if !clef_lines.is_empty() {
                Some(clef_lines.join(" | "))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    // Parse exam components (Prüfungselemente) - only for portfolio exams
    let mut components = Vec::new();
    if after_section.contains("Prüfungselemente") {
        let table_selector = Selector::parse("table").unwrap();
        let row_selector = Selector::parse("tr").unwrap();
        let cell_selector = Selector::parse("td, th").unwrap();

        // Search all tables in the document
        for table in document.select(&table_selector) {
            let rows: Vec<_> = table.select(&row_selector).collect();
            if rows.is_empty() {
                continue;
            }

            // Get header row to identify columns
            let header_cells: Vec<String> = rows[0]
                .select(&cell_selector)
                .map(|cell| cell.text().collect::<String>().trim().to_string())
                .collect();

            // Check if this is an exam components table by looking for specific column headers
            let has_name = header_cells.iter().any(|h| h.to_lowercase() == "name");
            let has_points = header_cells.iter().any(|h| h.to_lowercase() == "punkte");

            // Must have at least name and points columns to be a valid exam components table
            if !has_name || !has_points {
                continue;
            }

            // Find column indices (case-insensitive)
            let name_idx = header_cells.iter().position(|h| h.to_lowercase() == "name");
            let points_idx = header_cells.iter().position(|h| h.to_lowercase() == "punkte");
            let category_idx = header_cells.iter().position(|h| h.to_lowercase() == "kategorie");
            let scope_idx = header_cells.iter().position(|h| {
                let lower = h.to_lowercase();
                lower.contains("dauer") || lower.contains("umfang")
            });

            // Parse data rows (skip header row)
            for row in rows.iter().skip(1) {
                let cells: Vec<_> = row.select(&cell_selector).collect();

                if cells.is_empty() {
                    continue;
                }

                let name = name_idx
                    .and_then(|idx| cells.get(idx))
                    .map(|cell| cell.text().collect::<String>().trim().to_string());

                if let Some(name_value) = name {
                    // Skip empty names or table footers
                    if name_value.is_empty() || name_value == "Keine Angabe" {
                        continue;
                    }

                    let points = points_idx
                        .and_then(|idx| cells.get(idx))
                        .and_then(|cell| {
                            let text = cell.text().collect::<String>().trim().to_string();
                            // Extract just the number from the text
                            text.chars()
                                .filter(|c| c.is_numeric())
                                .collect::<String>()
                                .parse::<i32>()
                                .ok()
                        });

                    let category = category_idx
                        .and_then(|idx| cells.get(idx))
                        .map(|cell| cell.text().collect::<String>().trim().to_string())
                        .filter(|s| !s.is_empty());

                    let scope = scope_idx
                        .and_then(|idx| cells.get(idx))
                        .map(|cell| cell.text().collect::<String>().trim().to_string())
                        .filter(|s| !s.is_empty());

                    components.push(ScrapedExamComponent {
                        name: name_value,
                        points,
                        category,
                        scope,
                    });
                }
            }

            // If we found components, we're done - no need to check other tables
            if !components.is_empty() {
                break;
            }
        }
    }

    // Only create exam if we have at least an exam type
    if let Some(exam_type_value) = exam_type {
        module.exam = Some(ScrapedExam {
            graded,
            exam_type: exam_type_value,
            language,
            duration_scope,
            description,
            clef,
            components,
        });
    }
}
