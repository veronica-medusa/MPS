use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NumberFormat {
    #[default]
    Decimal,
    LowerLetter,
    UpperLetter,
    LowerRoman,
    UpperRoman,
    Bullet,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumberingLevel {
    pub level: u8, // 0..=8
    pub format: NumberFormat,
    pub text_pattern: String, // e.g. "%1." or "%1.%2." or "•"
    pub start_at: u32,
    pub indent_twips: u32,
    pub hanging_twips: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbstractNumbering {
    pub id: u32,
    pub levels: HashMap<u8, NumberingLevel>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NumberingInstance {
    pub num_id: u32,
    pub abstract_num_id: u32,
    pub counters: HashMap<u8, u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NumberingRegistry {
    pub abstract_nums: HashMap<u32, AbstractNumbering>,
    pub instances: HashMap<u32, NumberingInstance>,
}

impl NumberingRegistry {
    pub fn new() -> Self {
        let mut reg = Self::default();
        reg.load_builtin_defaults();
        reg
    }

    pub fn load_builtin_defaults(&mut self) {
        // Bullet list (abstract 1)
        let mut bullet_levels = HashMap::new();
        for lvl in 0..=8 {
            let symbol = match lvl % 3 {
                0 => "•",
                1 => "◦",
                _ => "▪",
            };
            bullet_levels.insert(lvl, NumberingLevel {
                level: lvl,
                format: NumberFormat::Bullet,
                text_pattern: symbol.to_string(),
                start_at: 1,
                indent_twips: (lvl as u32 + 1) * 360,
                hanging_twips: 360,
            });
        }
        self.abstract_nums.insert(1, AbstractNumbering {
            id: 1,
            levels: bullet_levels,
        });
        self.instances.insert(1, NumberingInstance {
            num_id: 1,
            abstract_num_id: 1,
            counters: HashMap::new(),
        });

        // Decimal outline list: 1., 1.1., 1.1.1. (abstract 2)
        let mut decimal_levels = HashMap::new();
        for lvl in 0..=8 {
            let mut pattern = String::new();
            for i in 0..=lvl {
                pattern.push_str(&format!("%{}.", i + 1));
            }
            decimal_levels.insert(lvl, NumberingLevel {
                level: lvl,
                format: NumberFormat::Decimal,
                text_pattern: pattern,
                start_at: 1,
                indent_twips: (lvl as u32 + 1) * 360,
                hanging_twips: 360,
            });
        }
        self.abstract_nums.insert(2, AbstractNumbering {
            id: 2,
            levels: decimal_levels,
        });
        self.instances.insert(2, NumberingInstance {
            num_id: 2,
            abstract_num_id: 2,
            counters: HashMap::new(),
        });
    }

    pub fn next_label(&mut self, num_id: u32, level: u8) -> String {
        let instance = match self.instances.get_mut(&num_id) {
            Some(inst) => inst,
            None => return String::new(),
        };

        let abstract_id = instance.abstract_num_id;
        let abstract_num = match self.abstract_nums.get(&abstract_id) {
            Some(a) => a,
            None => return String::new(),
        };

        let level_def = match abstract_num.levels.get(&level) {
            Some(l) => l,
            None => return String::new(),
        };

        // Increment counter for this level
        let start = level_def.start_at;
        let counter = instance.counters.entry(level).or_insert(start.saturating_sub(1));
        *counter += 1;
        let _current_val = *counter;

        // Reset child levels (e.g. going from 1.2 to 2 resets level 1 counter)
        for child_lvl in (level + 1)..=8 {
            instance.counters.remove(&child_lvl);
        }

        if level_def.format == NumberFormat::Bullet {
            return level_def.text_pattern.clone();
        }

        // Format pattern like "%1.%2." with current values
        let mut label = level_def.text_pattern.clone();
        for lvl in 0..=level {
            let token = format!("%{}", lvl + 1);
            if label.contains(&token) {
                let lvl_val = instance.counters.get(&lvl).copied().unwrap_or(1);
                let formatted = Self::format_number(lvl_val, level_def.format);
                label = label.replace(&token, &formatted);
            }
        }

        label
    }

    fn format_number(val: u32, fmt: NumberFormat) -> String {
        match fmt {
            NumberFormat::Decimal => val.to_string(),
            NumberFormat::LowerLetter => Self::to_letter(val, false),
            NumberFormat::UpperLetter => Self::to_letter(val, true),
            NumberFormat::LowerRoman => Self::to_roman(val).to_lowercase(),
            NumberFormat::UpperRoman => Self::to_roman(val),
            NumberFormat::Bullet => "•".to_string(),
        }
    }

    fn to_letter(val: u32, upper: bool) -> String {
        let mut n = val;
        let mut res = String::new();
        while n > 0 {
            let rem = ((n - 1) % 26) as u8;
            let base = if upper { b'A' } else { b'a' };
            res.insert(0, (base + rem) as char);
            n = (n - 1) / 26;
        }
        if res.is_empty() {
            "a".to_string()
        } else {
            res
        }
    }

    fn to_roman(mut num: u32) -> String {
        let lookup: [(u32, &str); 13] = [
            (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"),
            (100, "C"), (90, "XC"), (50, "L"), (40, "XL"),
            (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I"),
        ];
        let mut roman = String::new();
        for (value, symbol) in lookup {
            while num >= value {
                roman.push_str(symbol);
                num -= value;
            }
        }
        if roman.is_empty() {
            "I".to_string()
        } else {
            roman
        }
    }
}
