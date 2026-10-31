use crate::cell::CellValue;
use wps_core::color::Color;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionOp {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SectionCondition {
    pub op: ConditionOp,
    pub target: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormatSection {
    pub condition: Option<SectionCondition>,
    pub color: Option<Color>,
    pub pattern: String,
    pub is_text_section: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberFormatter {
    pub raw_code: String,
    pub sections: Vec<FormatSection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormattedCell {
    pub text: String,
    pub color: Option<Color>,
}

impl NumberFormatter {
    /// Parses an Excel/WPS format code string (e.g. "$#,##0.00;[Red]($#,##0.00);\"-\";@")
    pub fn new(code: &str) -> Self {
        let trimmed = code.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("general") {
            return Self {
                raw_code: "General".to_string(),
                sections: vec![FormatSection {
                    condition: None,
                    color: None,
                    pattern: "General".to_string(),
                    is_text_section: false,
                }],
            };
        }

        let raw_sections: Vec<&str> = trimmed.split(';').collect();
        let mut sections = Vec::new();

        for (idx, raw) in raw_sections.iter().enumerate() {
            let mut sec_str = raw.trim().to_string();
            let mut color = None;
            let mut condition = None;

            // Extract [Color] tags
            if let Some(start) = sec_str.find('[') {
                if let Some(end) = sec_str[start..].find(']') {
                    let tag = &sec_str[start + 1..start + end];
                    let tag_lower = tag.to_lowercase();
                    match tag_lower.as_str() {
                        "red" => color = Some(Color::rgb(224, 49, 49)),
                        "green" => color = Some(Color::rgb(12, 166, 120)),
                        "blue" => color = Some(Color::rgb(59, 91, 219)),
                        "yellow" => color = Some(Color::rgb(245, 159, 0)),
                        "cyan" => color = Some(Color::rgb(21, 170, 191)),
                        "magenta" => color = Some(Color::rgb(230, 73, 128)),
                        "black" => color = Some(Color::BLACK),
                        "white" => color = Some(Color::WHITE),
                        _ => {
                            // Check comparison condition e.g. [>100], [<0]
                            if let Some(cond) = parse_condition(tag) {
                                condition = Some(cond);
                            }
                        }
                    }
                    if color.is_some() || condition.is_some() {
                        sec_str = format!("{}{}", &sec_str[..start], &sec_str[start + end + 1..]);
                    }
                }
            }

            // Check if second bracket exists (e.g. [Red][<0])
            if let Some(start) = sec_str.find('[') {
                if let Some(end) = sec_str[start..].find(']') {
                    let tag = &sec_str[start + 1..start + end];
                    if let Some(cond) = parse_condition(tag) {
                        condition = Some(cond);
                        sec_str = format!("{}{}", &sec_str[..start], &sec_str[start + end + 1..]);
                    }
                }
            }

            let is_text_section = idx == 3 || sec_str.contains('@');
            sections.push(FormatSection {
                condition,
                color,
                pattern: sec_str.trim().to_string(),
                is_text_section,
            });
        }

        Self {
            raw_code: trimmed.to_string(),
            sections,
        }
    }

    /// Formats a CellValue into a formatted display string and optional color
    pub fn format(&self, value: &CellValue) -> FormattedCell {
        match value {
            CellValue::Empty => FormattedCell {
                text: String::new(),
                color: None,
            },
            CellValue::Error(e) => FormattedCell {
                text: e.as_str().to_string(),
                color: Some(Color::rgb(224, 49, 49)),
            },
            CellValue::Boolean(b) => FormattedCell {
                text: if *b { "TRUE".to_string() } else { "FALSE".to_string() },
                color: None,
            },
            CellValue::Text(t) => {
                // If 4th section or text section exists, format with it
                if let Some(sec) = self.sections.iter().find(|s| s.is_text_section) {
                    let formatted = sec.pattern.replace('@', t);
                    FormattedCell {
                        text: if formatted.is_empty() { t.clone() } else { formatted },
                        color: sec.color,
                    }
                } else {
                    FormattedCell {
                        text: t.clone(),
                        color: None,
                    }
                }
            }
            CellValue::Number(num) => self.format_number(*num),
            CellValue::Array(matrix) => {
                if let Some(row) = matrix.first() {
                    if let Some(first_val) = row.first() {
                        return self.format(first_val);
                    }
                }
                FormattedCell {
                    text: "#VALUE!".to_string(),
                    color: Some(Color::rgb(224, 49, 49)),
                }
            }
        }
    }

    fn format_number(&self, num: f64) -> FormattedCell {
        if self.sections.is_empty() || self.raw_code.eq_ignore_ascii_case("general") {
            return FormattedCell {
                text: format_general_number(num),
                color: None,
            };
        }

        // Determine matching section based on conditions, or sign
        let matching_section = self.select_section_for_number(num);

        let pattern = &matching_section.pattern;
        let color = matching_section.color;

        // Check date/time formats
        if pattern.contains("yy") || pattern.contains("yyyy") || pattern.contains("mm/dd") || pattern.contains("hh:") || pattern.contains("[h]:") {
            let date_str = format_serial_date(num, pattern);
            return FormattedCell {
                text: date_str,
                color,
            };
        }

        // Check percentage
        if pattern.contains('%') {
            let pct_val = num * 100.0;
            let decimals = count_decimal_zeros(pattern);
            let s = format!("{:.*}%", decimals, pct_val);
            return FormattedCell { text: s, color };
        }

        // Check currency / thousand comma separator / fixed decimal
        let formatted_num = format_numeric_pattern(num, pattern);
        FormattedCell {
            text: formatted_num,
            color,
        }
    }

    fn select_section_for_number(&self, num: f64) -> &FormatSection {
        // First check explicit conditions [>100], etc.
        for sec in &self.sections {
            if let Some(ref cond) = sec.condition {
                let matched = match cond.op {
                    ConditionOp::GreaterThan => num > cond.target,
                    ConditionOp::GreaterThanOrEqual => num >= cond.target,
                    ConditionOp::LessThan => num < cond.target,
                    ConditionOp::LessThanOrEqual => num <= cond.target,
                    ConditionOp::Equal => (num - cond.target).abs() < 1e-9,
                    ConditionOp::NotEqual => (num - cond.target).abs() >= 1e-9,
                };
                if matched {
                    return sec;
                }
            }
        }

        // Standard 4-section fallback: positive ; negative ; zero ; text
        if num > 0.0 {
            &self.sections[0]
        } else if num < 0.0 {
            if self.sections.len() > 1 {
                &self.sections[1]
            } else {
                &self.sections[0]
            }
        } else {
            if self.sections.len() > 2 {
                &self.sections[2]
            } else {
                &self.sections[0]
            }
        }
    }
}

fn parse_condition(tag: &str) -> Option<SectionCondition> {
    let clean = tag.trim();
    if let Some(rest) = clean.strip_prefix(">=") {
        let target = rest.trim().parse::<f64>().ok()?;
        Some(SectionCondition { op: ConditionOp::GreaterThanOrEqual, target })
    } else if let Some(rest) = clean.strip_prefix('>') {
        let target = rest.trim().parse::<f64>().ok()?;
        Some(SectionCondition { op: ConditionOp::GreaterThan, target })
    } else if let Some(rest) = clean.strip_prefix("<=") {
        let target = rest.trim().parse::<f64>().ok()?;
        Some(SectionCondition { op: ConditionOp::LessThanOrEqual, target })
    } else if let Some(rest) = clean.strip_prefix('<') {
        let target = rest.trim().parse::<f64>().ok()?;
        Some(SectionCondition { op: ConditionOp::LessThan, target })
    } else if let Some(rest) = clean.strip_prefix("==").or_else(|| clean.strip_prefix('=')) {
        let target = rest.trim().parse::<f64>().ok()?;
        Some(SectionCondition { op: ConditionOp::Equal, target })
    } else if let Some(rest) = clean.strip_prefix("<>").or_else(|| clean.strip_prefix("!=")) {
        let target = rest.trim().parse::<f64>().ok()?;
        Some(SectionCondition { op: ConditionOp::NotEqual, target })
    } else {
        None
    }
}

fn count_decimal_zeros(pattern: &str) -> usize {
    if let Some(dot_pos) = pattern.find('.') {
        pattern[dot_pos + 1..]
            .chars()
            .take_while(|c| *c == '0' || *c == '#')
            .count()
    } else {
        0
    }
}

fn format_general_number(num: f64) -> String {
    if num.is_nan() {
        return "#NUM!".to_string();
    }
    if num.is_infinite() {
        return if num > 0.0 { "#NUM!" } else { "-#NUM!" }.to_string();
    }
    if num.fract() == 0.0 && num.abs() < 1e12 {
        format!("{:.0}", num)
    } else if num.abs() >= 1e11 || (num.abs() < 1e-4 && num != 0.0) {
        format!("{:E}", num)
    } else {
        let s = format!("{:.6}", num);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn format_numeric_pattern(num: f64, pattern: &str) -> String {
    let abs_val = num.abs();
    let has_commas = pattern.contains(',');
    let has_currency = pattern.contains('$') || pattern.contains('¥') || pattern.contains('€') || pattern.contains('£') || pattern.contains('₹');
    let currency_char = if pattern.contains('$') {
        "$"
    } else if pattern.contains('¥') {
        "¥"
    } else if pattern.contains('€') {
        "€"
    } else if pattern.contains('£') {
        "£"
    } else if pattern.contains('₹') {
        "₹"
    } else {
        ""
    };

    let decimals = count_decimal_zeros(pattern);
    let is_parentheses_neg = pattern.contains('(') && pattern.contains(')');

    let base_str = if decimals > 0 {
        format!("{:.*}", decimals, abs_val)
    } else {
        format!("{:.0}", abs_val)
    };

    let parts: Vec<&str> = base_str.split('.').collect();
    let mut int_part = parts[0].to_string();
    let dec_part = parts.get(1).copied().unwrap_or("");

    if has_commas {
        int_part = add_thousands_separators(&int_part);
    }

    let final_num = if decimals > 0 {
        format!("{}.{}", int_part, dec_part)
    } else {
        int_part
    };

    let formatted = if has_currency {
        format!("{}{}", currency_char, final_num)
    } else {
        final_num
    };

    if num < 0.0 {
        if is_parentheses_neg {
            format!("({})", formatted)
        } else {
            format!("-{}", formatted)
        }
    } else {
        formatted
    }
}

fn add_thousands_separators(s: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for (i, &ch) in chars.iter().enumerate() {
        result.push(ch);
        let remaining = len - i - 1;
        if remaining > 0 && remaining % 3 == 0 {
            result.push(',');
        }
    }
    result
}

/// Converts Excel 1900 serial date into yyyy-mm-dd / custom date pattern
pub fn format_serial_date(serial: f64, pattern: &str) -> String {
    let days = serial.floor() as i64;
    let frac = serial - serial.floor();

    // 1900 Epoch calculations with leap year bug adjustment (day 60 = 1900-02-29)
    let adj_days = if days > 60 { days - 1 } else { days };

    let mut year = 1900;
    let mut d = adj_days - 1;

    while d >= if is_leap(year) { 366 } else { 365 } {
        d -= if is_leap(year) { 366 } else { 365 };
        year += 1;
    }

    let leap = is_leap(year);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];

    let mut month = 1;
    for &md in &month_days {
        if d < md {
            break;
        }
        d -= md;
        month += 1;
    }
    let day = d + 1;

    let total_secs = (frac * 86400.0).round() as u64;
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    let month_names = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let mut res = pattern.to_string();
    res = res.replace("yyyy", &format!("{:04}", year));
    res = res.replace("yy", &format!("{:02}", year % 100));
    res = res.replace("mmm", month_names[(month - 1).min(11) as usize]);
    res = res.replace(":mm", &format!(":{:02}", mins));
    res = res.replace("mm:", &format!("{:02}:", mins));
    res = res.replace("mm", &format!("{:02}", month));
    res = res.replace("dd", &format!("{:02}", day));
    res = res.replace("d", &format!("{}", day));
    res = res.replace("hh", &format!("{:02}", hours));
    res = res.replace("h", &format!("{}", hours));
    res = res.replace("ss", &format!("{:02}", secs));

    // Handle elapsed hours e.g. [h]:mm:ss
    if pattern.contains("[h]") {
        let elapsed_h = (serial * 24.0).floor() as i64;
        res = res.replace("[h]", &format!("{}", elapsed_h));
    }

    res
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}
