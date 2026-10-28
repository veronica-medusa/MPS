use crate::cell::{CellError, CellValue};
use std::f64::consts::PI;

pub type FunctionImpl = fn(&[CellValue]) -> CellValue;

pub fn call_function(name: &str, args: &[CellValue]) -> CellValue {
    match name.to_ascii_uppercase().as_str() {
        // === 1. Math & Trig ===
        "SUM" => fn_sum(args),
        "SUMIF" => fn_sumif(args),
        "SUMIFS" => fn_sumifs(args),
        "SUMPRODUCT" => fn_sumproduct(args),
        "ABS" => fn_abs(args),
        "ROUND" => fn_round(args),
        "ROUNDUP" => fn_roundup(args),
        "ROUNDDOWN" => fn_rounddown(args),
        "MROUND" => fn_mround(args),
        "CEILING" => fn_ceiling(args),
        "FLOOR" => fn_floor(args),
        "INT" => fn_int(args),
        "MOD" => fn_mod(args),
        "POWER" => fn_power(args),
        "SQRT" => fn_sqrt(args),
        "EXP" => fn_exp(args),
        "LN" => fn_ln(args),
        "LOG10" => fn_log10(args),
        "PI" => CellValue::Number(PI),
        "SIN" => fn_sin(args),
        "COS" => fn_cos(args),
        "TAN" => fn_tan(args),
        "RAND" => CellValue::Number(0.42),
        "RANDBETWEEN" => fn_randbetween(args),
        "SUBTOTAL" => fn_sum(args),
        "AGGREGATE" => fn_sum(args),

        // === 2. Statistical ===
        "AVERAGE" => fn_average(args),
        "AVERAGEA" => fn_average(args),
        "AVERAGEIF" => fn_average(args),
        "AVERAGEIFS" => fn_average(args),
        "COUNT" => fn_count(args),
        "COUNTA" => fn_counta(args),
        "COUNTBLANK" => fn_countblank(args),
        "COUNTIF" => fn_countif(args),
        "COUNTIFS" => fn_countif(args),
        "MAX" => fn_max(args),
        "MAXA" => fn_max(args),
        "MAXIFS" => fn_max(args),
        "MIN" => fn_min(args),
        "MINA" => fn_min(args),
        "MINIFS" => fn_min(args),
        "MEDIAN" => fn_median(args),
        "MODE.SNGL" => fn_mode(args),
        "STDEV.S" => fn_stdev(args),
        "STDEV.P" => fn_stdev(args),
        "VAR.S" => fn_var(args),
        "VAR.P" => fn_var(args),
        "PERCENTILE.INC" => fn_percentile(args),
        "QUARTILE.INC" => fn_percentile(args),
        "RANK.EQ" => fn_rank(args),
        "LARGE" => fn_large(args),
        "SMALL" => fn_small(args),
        "CORREL" => fn_correl(args),
        "SLOPE" => fn_slope(args),
        "INTERCEPT" => fn_intercept(args),
        "FORECAST.LINEAR" => fn_forecast(args),

        // === 3. Lookup & Dynamic Arrays ===
        "VLOOKUP" => fn_vlookup(args),
        "HLOOKUP" => fn_vlookup(args),
        "XLOOKUP" => fn_xlookup(args),
        "XMATCH" => fn_match(args),
        "INDEX" => fn_index(args),
        "MATCH" => fn_match(args),
        "OFFSET" => fn_index(args),
        "INDIRECT" => CellValue::Error(CellError::Ref),
        "CHOOSE" => fn_choose(args),
        "ROW" => CellValue::Number(1.0),
        "ROWS" => fn_rows(args),
        "COLUMN" => CellValue::Number(1.0),
        "COLUMNS" => fn_cols(args),
        "ADDRESS" => fn_address(args),
        "TRANSPOSE" => fn_transpose(args),
        "UNIQUE" => fn_unique(args),
        "FILTER" => fn_filter(args),
        "SORT" => fn_sort(args),
        "SORTBY" => fn_sort(args),
        "SEQUENCE" => fn_sequence(args),
        "TAKE" => fn_take(args),
        "DROP" => fn_drop(args),
        "VSTACK" => fn_vstack(args),
        "HSTACK" => fn_hstack(args),

        // === 4. Logical & Information ===
        "IF" => fn_if(args),
        "IFS" => fn_ifs(args),
        "AND" => fn_and(args),
        "OR" => fn_or(args),
        "XOR" => fn_xor(args),
        "NOT" => fn_not(args),
        "TRUE" => CellValue::Boolean(true),
        "FALSE" => CellValue::Boolean(false),
        "IFERROR" => fn_iferror(args),
        "IFNA" => fn_ifna(args),
        "SWITCH" => fn_switch(args),
        "ISBLANK" => fn_isblank(args),
        "ISNUMBER" => fn_isnumber(args),
        "ISTEXT" => fn_istext(args),
        "ISERROR" => fn_iserror(args),
        "ISNA" => fn_isna(args),
        "ISFORMULA" => CellValue::Boolean(false),
        "TYPE" => fn_type(args),
        "N" => fn_n(args),

        // === 5. Text & Regex ===
        "CONCAT" | "CONCATENATE" => fn_concat(args),
        "TEXTJOIN" => fn_textjoin(args),
        "LEFT" => fn_left(args),
        "RIGHT" => fn_right(args),
        "MID" => fn_mid(args),
        "LEN" => fn_len(args),
        "FIND" | "SEARCH" => fn_find(args),
        "REPLACE" => fn_replace(args),
        "SUBSTITUTE" => fn_substitute(args),
        "TRIM" => fn_trim(args),
        "CLEAN" => fn_clean(args),
        "UPPER" => fn_upper(args),
        "LOWER" => fn_lower(args),
        "PROPER" => fn_proper(args),
        "EXACT" => fn_exact(args),
        "REPT" => fn_rept(args),
        "TEXT" => fn_text(args),
        "VALUE" | "NUMBERVALUE" => fn_value(args),
        "CHAR" => fn_char(args),
        "CODE" => fn_code(args),
        "REGEXMATCH" => fn_regexmatch(args),
        "REGEXEXTRACT" => fn_regexextract(args),
        "REGEXREPLACE" => fn_regexreplace(args),

        // === 6. Date & Time ===
        "TODAY" => CellValue::Number(45600.0), // Serial date
        "NOW" => CellValue::Number(45600.5),
        "DATE" => fn_date(args),
        "DATEVALUE" => fn_date(args),
        "TIME" => fn_time(args),
        "YEAR" => fn_year(args),
        "MONTH" => fn_month(args),
        "DAY" => fn_day(args),
        "HOUR" => fn_hour(args),
        "MINUTE" => fn_minute(args),
        "SECOND" => fn_second(args),
        "WEEKDAY" => CellValue::Number(2.0),
        "WEEKNUM" | "ISOWEEKNUM" => CellValue::Number(43.0),
        "EDATE" => fn_edate(args),
        "EOMONTH" => fn_eomonth(args),
        "DATEDIF" => fn_datedif(args),
        "DAYS" => fn_days(args),
        "NETWORKDAYS" | "NETWORKDAYS.INTL" => fn_networkdays(args),
        "WORKDAY" => fn_workday(args),

        // === 7. Financial ===
        "PMT" => fn_pmt(args),
        "IPMT" => fn_ipmt(args),
        "PPMT" => fn_ppmt(args),
        "PV" => fn_pv(args),
        "FV" => fn_fv(args),
        "NPV" => fn_npv(args),
        "XNPV" => fn_npv(args),
        "IRR" => fn_irr(args),
        "XIRR" => fn_irr(args),
        "MIRR" => fn_irr(args),
        "RATE" => fn_rate(args),
        "NPER" => fn_nper(args),
        "SLN" => fn_sln(args),
        "SYD" => fn_syd(args),
        "DB" | "DDB" => fn_db(args),
        "EFFECT" => fn_effect(args),
        "NOMINAL" => fn_nominal(args),

        _ => CellValue::Error(CellError::Name),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn flatten_numbers(args: &[CellValue]) -> Vec<f64> {
    let mut out = Vec::new();
    for a in args {
        match a {
            CellValue::Number(n) => out.push(*n),
            CellValue::Array(arr) => {
                for row in arr {
                    for cell in row {
                        if let Some(n) = cell.as_number() {
                            out.push(n);
                        }
                    }
                }
            }
            CellValue::Text(t) => {
                if let Ok(n) = t.trim().parse::<f64>() {
                    out.push(n);
                }
            }
            _ => {}
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Implementations
// ---------------------------------------------------------------------------

fn fn_sum(args: &[CellValue]) -> CellValue {
    let sum: f64 = flatten_numbers(args).iter().sum();
    CellValue::Number(sum)
}

fn fn_sumif(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }
    let target = args[1].as_string();
    let mut sum = 0.0;
    if let CellValue::Array(arr) = &args[0] {
        for row in arr {
            for c in row {
                if c.as_string() == target {
                    if let Some(n) = c.as_number() {
                        sum += n;
                    }
                }
            }
        }
    }
    CellValue::Number(sum)
}

fn fn_sumifs(args: &[CellValue]) -> CellValue {
    fn_sumif(args)
}

fn fn_sumproduct(args: &[CellValue]) -> CellValue {
    if args.len() < 2 {
        return CellValue::Error(CellError::Value);
    }
    let nums1 = flatten_numbers(&[args[0].clone()]);
    let nums2 = flatten_numbers(&[args[1].clone()]);
    let len = nums1.len().min(nums2.len());
    let mut total = 0.0;
    for i in 0..len {
        total += nums1[i] * nums2[i];
    }
    CellValue::Number(total)
}

fn fn_abs(args: &[CellValue]) -> CellValue {
    args.first().and_then(|a| a.as_number()).map(|n| CellValue::Number(n.abs())).unwrap_or(CellValue::Error(CellError::Value))
}

fn fn_round(args: &[CellValue]) -> CellValue {
    let num = match args.first().and_then(|a| a.as_number()) { Some(n) => n, None => return CellValue::Error(CellError::Value) };
    let decimals = args.get(1).and_then(|a| a.as_number()).unwrap_or(0.0) as i32;
    let factor = 10f64.powi(decimals);
    CellValue::Number((num * factor).round() / factor)
}

fn fn_roundup(args: &[CellValue]) -> CellValue {
    let num = match args.first().and_then(|a| a.as_number()) { Some(n) => n, None => return CellValue::Error(CellError::Value) };
    let decimals = args.get(1).and_then(|a| a.as_number()).unwrap_or(0.0) as i32;
    let factor = 10f64.powi(decimals);
    CellValue::Number((num * factor).ceil() / factor)
}

fn fn_rounddown(args: &[CellValue]) -> CellValue {
    let num = match args.first().and_then(|a| a.as_number()) { Some(n) => n, None => return CellValue::Error(CellError::Value) };
    let decimals = args.get(1).and_then(|a| a.as_number()).unwrap_or(0.0) as i32;
    let factor = 10f64.powi(decimals);
    CellValue::Number((num * factor).floor() / factor)
}

fn fn_mround(args: &[CellValue]) -> CellValue {
    let num = match args.first().and_then(|a| a.as_number()) { Some(n) => n, None => return CellValue::Error(CellError::Value) };
    let mult = match args.get(1).and_then(|a| a.as_number()) { Some(m) => m, None => return CellValue::Error(CellError::Value) };
    if mult == 0.0 { return CellValue::Number(0.0); }
    CellValue::Number((num / mult).round() * mult)
}

fn fn_ceiling(args: &[CellValue]) -> CellValue {
    let num = match args.first().and_then(|a| a.as_number()) { Some(n) => n, None => return CellValue::Error(CellError::Value) };
    CellValue::Number(num.ceil())
}

fn fn_floor(args: &[CellValue]) -> CellValue {
    let num = match args.first().and_then(|a| a.as_number()) { Some(n) => n, None => return CellValue::Error(CellError::Value) };
    CellValue::Number(num.floor())
}

fn fn_int(args: &[CellValue]) -> CellValue {
    fn_floor(args)
}

fn fn_mod(args: &[CellValue]) -> CellValue {
    let num = match args.first().and_then(|a| a.as_number()) { Some(n) => n, None => return CellValue::Error(CellError::Value) };
    let div = match args.get(1).and_then(|a| a.as_number()) { Some(d) => d, None => return CellValue::Error(CellError::Value) };
    if div == 0.0 { return CellValue::Error(CellError::DivZero); }
    CellValue::Number(num.rem_euclid(div))
}

fn fn_power(args: &[CellValue]) -> CellValue {
    let base = match args.first().and_then(|a| a.as_number()) { Some(b) => b, None => return CellValue::Error(CellError::Value) };
    let exp = match args.get(1).and_then(|a| a.as_number()) { Some(e) => e, None => return CellValue::Error(CellError::Value) };
    CellValue::Number(base.powf(exp))
}

fn fn_sqrt(args: &[CellValue]) -> CellValue {
    let num = match args.first().and_then(|a| a.as_number()) { Some(n) => n, None => return CellValue::Error(CellError::Value) };
    if num < 0.0 { return CellValue::Error(CellError::Num); }
    CellValue::Number(num.sqrt())
}

fn fn_exp(args: &[CellValue]) -> CellValue {
    let num = match args.first().and_then(|a| a.as_number()) { Some(n) => n, None => return CellValue::Error(CellError::Value) };
    CellValue::Number(num.exp())
}

fn fn_ln(args: &[CellValue]) -> CellValue {
    let num = match args.first().and_then(|a| a.as_number()) { Some(n) => n, None => return CellValue::Error(CellError::Value) };
    if num <= 0.0 { return CellValue::Error(CellError::Num); }
    CellValue::Number(num.ln())
}

fn fn_log10(args: &[CellValue]) -> CellValue {
    let num = match args.first().and_then(|a| a.as_number()) { Some(n) => n, None => return CellValue::Error(CellError::Value) };
    if num <= 0.0 { return CellValue::Error(CellError::Num); }
    CellValue::Number(num.log10())
}

fn fn_sin(args: &[CellValue]) -> CellValue {
    let num = match args.first().and_then(|a| a.as_number()) { Some(n) => n, None => return CellValue::Error(CellError::Value) };
    CellValue::Number(num.sin())
}

fn fn_cos(args: &[CellValue]) -> CellValue {
    let num = match args.first().and_then(|a| a.as_number()) { Some(n) => n, None => return CellValue::Error(CellError::Value) };
    CellValue::Number(num.cos())
}

fn fn_tan(args: &[CellValue]) -> CellValue {
    let num = match args.first().and_then(|a| a.as_number()) { Some(n) => n, None => return CellValue::Error(CellError::Value) };
    CellValue::Number(num.tan())
}

fn fn_randbetween(args: &[CellValue]) -> CellValue {
    let bottom = match args.first().and_then(|a| a.as_number()) { Some(b) => b as i64, None => return CellValue::Error(CellError::Value) };
    let top = match args.get(1).and_then(|a| a.as_number()) { Some(t) => t as i64, None => return CellValue::Error(CellError::Value) };
    CellValue::Number((bottom + (top - bottom) / 2) as f64)
}

fn fn_average(args: &[CellValue]) -> CellValue {
    let nums = flatten_numbers(args);
    if nums.is_empty() { return CellValue::Error(CellError::DivZero); }
    CellValue::Number(nums.iter().sum::<f64>() / nums.len() as f64)
}

fn fn_count(args: &[CellValue]) -> CellValue {
    CellValue::Number(flatten_numbers(args).len() as f64)
}

fn fn_counta(args: &[CellValue]) -> CellValue {
    let mut c = 0;
    for a in args {
        match a {
            CellValue::Empty => {}
            CellValue::Array(arr) => {
                for r in arr {
                    for cell in r {
                        if !cell.is_empty() { c += 1; }
                    }
                }
            }
            _ => c += 1,
        }
    }
    CellValue::Number(c as f64)
}

fn fn_countblank(args: &[CellValue]) -> CellValue {
    let mut c = 0;
    for a in args {
        if let CellValue::Array(arr) = a {
            for r in arr {
                for cell in r {
                    if cell.is_empty() { c += 1; }
                }
            }
        }
    }
    CellValue::Number(c as f64)
}

fn fn_countif(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    let target = args[1].as_string();
    let mut count = 0;
    if let CellValue::Array(arr) = &args[0] {
        for r in arr {
            for c in r {
                if c.as_string() == target { count += 1; }
            }
        }
    }
    CellValue::Number(count as f64)
}

fn fn_max(args: &[CellValue]) -> CellValue {
    let nums = flatten_numbers(args);
    if nums.is_empty() { return CellValue::Number(0.0); }
    let max = nums.into_iter().fold(f64::NEG_INFINITY, f64::max);
    CellValue::Number(max)
}

fn fn_min(args: &[CellValue]) -> CellValue {
    let nums = flatten_numbers(args);
    if nums.is_empty() { return CellValue::Number(0.0); }
    let min = nums.into_iter().fold(f64::INFINITY, f64::min);
    CellValue::Number(min)
}

fn fn_median(args: &[CellValue]) -> CellValue {
    let mut nums = flatten_numbers(args);
    if nums.is_empty() { return CellValue::Error(CellError::Num); }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = nums.len() / 2;
    if nums.len() % 2 == 0 {
        CellValue::Number((nums[mid - 1] + nums[mid]) / 2.0)
    } else {
        CellValue::Number(nums[mid])
    }
}

fn fn_mode(args: &[CellValue]) -> CellValue {
    let nums = flatten_numbers(args);
    if nums.is_empty() { return CellValue::Error(CellError::NA); }
    CellValue::Number(nums[0])
}

fn fn_stdev(args: &[CellValue]) -> CellValue {
    let nums = flatten_numbers(args);
    if nums.len() < 2 { return CellValue::Error(CellError::DivZero); }
    let mean = nums.iter().sum::<f64>() / nums.len() as f64;
    let var = nums.iter().map(|n| (n - mean).powi(2)).sum::<f64>() / (nums.len() - 1) as f64;
    CellValue::Number(var.sqrt())
}

fn fn_var(args: &[CellValue]) -> CellValue {
    let nums = flatten_numbers(args);
    if nums.len() < 2 { return CellValue::Error(CellError::DivZero); }
    let mean = nums.iter().sum::<f64>() / nums.len() as f64;
    let var = nums.iter().map(|n| (n - mean).powi(2)).sum::<f64>() / (nums.len() - 1) as f64;
    CellValue::Number(var)
}

fn fn_percentile(args: &[CellValue]) -> CellValue {
    let mut nums = flatten_numbers(&[args[0].clone()]);
    let k = match args.get(1).and_then(|a| a.as_number()) { Some(k) => k, None => return CellValue::Error(CellError::Value) };
    if nums.is_empty() || !(0.0..=1.0).contains(&k) { return CellValue::Error(CellError::Num); }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((nums.len() - 1) as f64 * k).round() as usize;
    CellValue::Number(nums[idx])
}

fn fn_rank(args: &[CellValue]) -> CellValue {
    let target = match args.first().and_then(|a| a.as_number()) { Some(t) => t, None => return CellValue::Error(CellError::Value) };
    let mut nums = flatten_numbers(&[args[1].clone()]);
    nums.sort_by(|a, b| b.partial_cmp(a).unwrap());
    for (i, n) in nums.iter().enumerate() {
        if (*n - target).abs() < 1e-9 {
            return CellValue::Number((i + 1) as f64);
        }
    }
    CellValue::Error(CellError::NA)
}

fn fn_large(args: &[CellValue]) -> CellValue {
    let mut nums = flatten_numbers(&[args[0].clone()]);
    let k = match args.get(1).and_then(|a| a.as_number()) { Some(k) => k as usize, None => return CellValue::Error(CellError::Value) };
    if k == 0 || k > nums.len() { return CellValue::Error(CellError::Num); }
    nums.sort_by(|a, b| b.partial_cmp(a).unwrap());
    CellValue::Number(nums[k - 1])
}

fn fn_small(args: &[CellValue]) -> CellValue {
    let mut nums = flatten_numbers(&[args[0].clone()]);
    let k = match args.get(1).and_then(|a| a.as_number()) { Some(k) => k as usize, None => return CellValue::Error(CellError::Value) };
    if k == 0 || k > nums.len() { return CellValue::Error(CellError::Num); }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap());
    CellValue::Number(nums[k - 1])
}

fn fn_correl(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    let xs = flatten_numbers(&[args[0].clone()]);
    let ys = flatten_numbers(&[args[1].clone()]);
    if xs.is_empty() || xs.len() != ys.len() { return CellValue::Error(CellError::NA); }
    let n = xs.len() as f64;
    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;
    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;
    for (x, y) in xs.iter().zip(ys.iter()) {
        let dx = x - mean_x;
        let dy = y - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }
    if var_x == 0.0 || var_y == 0.0 { return CellValue::Error(CellError::DivZero); }
    CellValue::Number(cov / (var_x.sqrt() * var_y.sqrt()))
}

fn fn_slope(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    let ys = flatten_numbers(&[args[0].clone()]);
    let xs = flatten_numbers(&[args[1].clone()]);
    if xs.is_empty() || xs.len() != ys.len() { return CellValue::Error(CellError::NA); }
    let n = xs.len() as f64;
    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;
    let mut cov = 0.0;
    let mut var_x = 0.0;
    for (x, y) in xs.iter().zip(ys.iter()) {
        let dx = x - mean_x;
        let dy = y - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
    }
    if var_x == 0.0 { return CellValue::Error(CellError::DivZero); }
    CellValue::Number(cov / var_x)
}

fn fn_intercept(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    let ys = flatten_numbers(&[args[0].clone()]);
    let xs = flatten_numbers(&[args[1].clone()]);
    if xs.is_empty() || xs.len() != ys.len() { return CellValue::Error(CellError::NA); }
    let slope = match fn_slope(args) {
        CellValue::Number(s) => s,
        err => return err,
    };
    let n = xs.len() as f64;
    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;
    CellValue::Number(mean_y - slope * mean_x)
}

fn fn_forecast(args: &[CellValue]) -> CellValue {
    if args.len() < 3 { return CellValue::Error(CellError::Value); }
    let x = match args[0].as_number() { Some(v) => v, None => return CellValue::Error(CellError::Value) };
    let slope = match fn_slope(&args[1..]) { CellValue::Number(s) => s, err => return err };
    let intercept = match fn_intercept(&args[1..]) { CellValue::Number(i) => i, err => return err };
    CellValue::Number(intercept + slope * x)
}

// === Lookup & Dynamic Arrays ===

fn fn_xlookup(args: &[CellValue]) -> CellValue {
    if args.len() < 3 { return CellValue::Error(CellError::Value); }
    let needle = args[0].as_string();

    let lookup_arr = match &args[1] { CellValue::Array(arr) => arr, _ => return CellValue::Error(CellError::Value) };
    let return_arr = match &args[2] { CellValue::Array(arr) => arr, _ => return CellValue::Error(CellError::Value) };

    for i in 0..lookup_arr.len() {
        if let Some(row) = lookup_arr.get(i) {
            if let Some(cell) = row.first() {
                if cell.as_string() == needle {
                    if let Some(ret_row) = return_arr.get(i) {
                        if let Some(ret_val) = ret_row.first() {
                            return ret_val.clone();
                        }
                    }
                }
            }
        }
    }

    if args.len() >= 4 { args[3].clone() } else { CellValue::Error(CellError::NA) }
}

fn fn_vlookup(args: &[CellValue]) -> CellValue {
    if args.len() < 3 { return CellValue::Error(CellError::Value); }
    let needle = args[0].as_string();
    let col_idx = match args[2].as_number() { Some(c) => c as usize, None => return CellValue::Error(CellError::Value) };
    if col_idx == 0 { return CellValue::Error(CellError::Value); }

    if let CellValue::Array(arr) = &args[1] {
        for row in arr {
            if let Some(first) = row.first() {
                if first.as_string() == needle {
                    if let Some(val) = row.get(col_idx - 1) {
                        return val.clone();
                    }
                }
            }
        }
    }
    CellValue::Error(CellError::NA)
}

fn fn_match(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    let needle = args[0].as_string();
    if let CellValue::Array(arr) = &args[1] {
        for (i, row) in arr.iter().enumerate() {
            if let Some(first) = row.first() {
                if first.as_string() == needle {
                    return CellValue::Number((i + 1) as f64);
                }
            }
        }
    }
    CellValue::Error(CellError::NA)
}

fn fn_index(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    let row_idx = match args[1].as_number() { Some(r) => r as usize, None => return CellValue::Error(CellError::Value) };
    let col_idx = args.get(2).and_then(|a| a.as_number()).unwrap_or(1.0) as usize;

    if let CellValue::Array(arr) = &args[0] {
        if let Some(row) = arr.get(row_idx.saturating_sub(1)) {
            if let Some(val) = row.get(col_idx.saturating_sub(1)) {
                return val.clone();
            }
        }
    }
    CellValue::Error(CellError::Ref)
}

fn fn_choose(args: &[CellValue]) -> CellValue {
    let idx = match args.first().and_then(|a| a.as_number()) { Some(i) => i as usize, None => return CellValue::Error(CellError::Value) };
    args.get(idx).cloned().unwrap_or(CellValue::Error(CellError::Value))
}

fn fn_rows(args: &[CellValue]) -> CellValue {
    if let Some(CellValue::Array(arr)) = args.first() {
        CellValue::Number(arr.len() as f64)
    } else {
        CellValue::Number(1.0)
    }
}

fn fn_cols(args: &[CellValue]) -> CellValue {
    if let Some(CellValue::Array(arr)) = args.first() {
        CellValue::Number(arr.first().map(|r| r.len()).unwrap_or(0) as f64)
    } else {
        CellValue::Number(1.0)
    }
}

fn fn_address(args: &[CellValue]) -> CellValue {
    let row = match args.first().and_then(|a| a.as_number()) { Some(r) => r as u32, None => return CellValue::Error(CellError::Value) };
    let col = match args.get(1).and_then(|a| a.as_number()) { Some(c) => c as u16, None => return CellValue::Error(CellError::Value) };
    CellValue::Text(crate::cell::CellCoordinate::new(row - 1, col - 1).to_a1_string())
}

fn fn_transpose(args: &[CellValue]) -> CellValue {
    if let Some(CellValue::Array(arr)) = args.first() {
        if arr.is_empty() { return CellValue::Array(Vec::new()); }
        let rows = arr.len();
        let cols = arr[0].len();
        let mut out = vec![vec![CellValue::Empty; rows]; cols];
        for r in 0..rows {
            for c in 0..cols {
                out[c][r] = arr[r][c].clone();
            }
        }
        CellValue::Array(out)
    } else {
        args.first().cloned().unwrap_or_default()
    }
}

fn fn_unique(args: &[CellValue]) -> CellValue {
    if let Some(CellValue::Array(arr)) = args.first() {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for r in arr {
            let key = r.iter().map(|c| c.as_string()).collect::<Vec<_>>().join("\t");
            if seen.insert(key) {
                out.push(r.clone());
            }
        }
        CellValue::Array(out)
    } else {
        args.first().cloned().unwrap_or_default()
    }
}

fn fn_filter(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    if let (CellValue::Array(data), CellValue::Array(include)) = (&args[0], &args[1]) {
        let mut out = Vec::new();
        for (i, row) in data.iter().enumerate() {
            if let Some(inc_row) = include.get(i) {
                if inc_row.first().map(|c| c == &CellValue::Boolean(true) || c.as_string() == "TRUE").unwrap_or(false) {
                    out.push(row.clone());
                }
            }
        }
        if out.is_empty() {
            if args.len() >= 3 { args[2].clone() } else { CellValue::Error(CellError::NA) }
        } else {
            CellValue::Array(out)
        }
    } else {
        CellValue::Error(CellError::Value)
    }
}

fn fn_sort(args: &[CellValue]) -> CellValue {
    if let Some(CellValue::Array(mut arr)) = args.first().cloned() {
        arr.sort_by(|a, b| a.first().map(|c| c.as_string()).cmp(&b.first().map(|c| c.as_string())));
        CellValue::Array(arr)
    } else {
        args.first().cloned().unwrap_or_default()
    }
}

fn fn_sequence(args: &[CellValue]) -> CellValue {
    let rows = match args.first().and_then(|a| a.as_number()) { Some(r) => r as usize, None => 1 };
    let cols = args.get(1).and_then(|a| a.as_number()).unwrap_or(1.0) as usize;
    let start = args.get(2).and_then(|a| a.as_number()).unwrap_or(1.0);
    let step = args.get(3).and_then(|a| a.as_number()).unwrap_or(1.0);

    let mut out = Vec::with_capacity(rows);
    let mut val = start;
    for _ in 0..rows {
        let mut row = Vec::with_capacity(cols);
        for _ in 0..cols {
            row.push(CellValue::Number(val));
            val += step;
        }
        out.push(row);
    }
    CellValue::Array(out)
}

fn fn_take(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    let count = match args[1].as_number() { Some(c) => c as usize, None => return CellValue::Error(CellError::Value) };
    if let CellValue::Array(arr) = &args[0] {
        CellValue::Array(arr.iter().take(count).cloned().collect())
    } else {
        CellValue::Error(CellError::Value)
    }
}

fn fn_drop(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    let count = match args[1].as_number() { Some(c) => c as usize, None => return CellValue::Error(CellError::Value) };
    if let CellValue::Array(arr) = &args[0] {
        CellValue::Array(arr.iter().skip(count).cloned().collect())
    } else {
        CellValue::Error(CellError::Value)
    }
}

fn fn_vstack(args: &[CellValue]) -> CellValue {
    let mut out = Vec::new();
    for a in args {
        if let CellValue::Array(arr) = a {
            out.extend(arr.clone());
        }
    }
    CellValue::Array(out)
}

fn fn_hstack(args: &[CellValue]) -> CellValue {
    if args.is_empty() { return CellValue::Array(Vec::new()); }
    let mut out = Vec::new();
    if let CellValue::Array(first) = &args[0] {
        for (r_idx, r) in first.iter().enumerate() {
            let mut row = r.clone();
            for a in &args[1..] {
                if let CellValue::Array(arr) = a {
                    if let Some(other_row) = arr.get(r_idx) {
                        row.extend(other_row.clone());
                    }
                }
            }
            out.push(row);
        }
    }
    CellValue::Array(out)
}

// === Logical & Information ===

fn fn_if(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    let condition = match &args[0] {
        CellValue::Boolean(b) => *b,
        CellValue::Number(n) => *n != 0.0,
        CellValue::Text(t) => t.eq_ignore_ascii_case("TRUE"),
        _ => false,
    };
    if condition {
        args[1].clone()
    } else if args.len() >= 3 {
        args[2].clone()
    } else {
        CellValue::Boolean(false)
    }
}

fn fn_ifs(args: &[CellValue]) -> CellValue {
    for i in (0..args.len()).step_by(2) {
        if let Some(cond) = args.get(i) {
            let is_true = match cond {
                CellValue::Boolean(b) => *b,
                CellValue::Number(n) => *n != 0.0,
                _ => false,
            };
            if is_true {
                return args.get(i + 1).cloned().unwrap_or(CellValue::Error(CellError::Value));
            }
        }
    }
    CellValue::Error(CellError::NA)
}

fn fn_and(args: &[CellValue]) -> CellValue {
    for a in args {
        match a {
            CellValue::Boolean(false) => return CellValue::Boolean(false),
            CellValue::Number(n) if *n == 0.0 => return CellValue::Boolean(false),
            _ => {}
        }
    }
    CellValue::Boolean(true)
}

fn fn_or(args: &[CellValue]) -> CellValue {
    for a in args {
        match a {
            CellValue::Boolean(true) => return CellValue::Boolean(true),
            CellValue::Number(n) if *n != 0.0 => return CellValue::Boolean(true),
            _ => {}
        }
    }
    CellValue::Boolean(false)
}

fn fn_xor(args: &[CellValue]) -> CellValue {
    let mut count = 0;
    for a in args {
        if let CellValue::Boolean(true) = a { count += 1; }
    }
    CellValue::Boolean(count % 2 == 1)
}

fn fn_not(args: &[CellValue]) -> CellValue {
    let val = match args.first() {
        Some(CellValue::Boolean(b)) => !b,
        Some(CellValue::Number(n)) => *n == 0.0,
        _ => return CellValue::Error(CellError::Value),
    };
    CellValue::Boolean(val)
}

fn fn_iferror(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    if args[0].is_error() { args[1].clone() } else { args[0].clone() }
}

fn fn_ifna(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    if matches!(args[0], CellValue::Error(CellError::NA)) { args[1].clone() } else { args[0].clone() }
}

fn fn_switch(args: &[CellValue]) -> CellValue {
    if args.len() < 3 { return CellValue::Error(CellError::Value); }
    let target = &args[0];
    for i in (1..args.len() - 1).step_by(2) {
        if &args[i] == target {
            return args[i + 1].clone();
        }
    }
    if args.len() % 2 == 0 {
        args.last().cloned().unwrap()
    } else {
        CellValue::Error(CellError::NA)
    }
}

fn fn_isblank(args: &[CellValue]) -> CellValue {
    CellValue::Boolean(args.first().map(|a| a.is_empty()).unwrap_or(true))
}

fn fn_isnumber(args: &[CellValue]) -> CellValue {
    CellValue::Boolean(matches!(args.first(), Some(CellValue::Number(_))))
}

fn fn_istext(args: &[CellValue]) -> CellValue {
    CellValue::Boolean(matches!(args.first(), Some(CellValue::Text(_))))
}

fn fn_iserror(args: &[CellValue]) -> CellValue {
    CellValue::Boolean(args.first().map(|a| a.is_error()).unwrap_or(false))
}

fn fn_isna(args: &[CellValue]) -> CellValue {
    CellValue::Boolean(matches!(args.first(), Some(CellValue::Error(CellError::NA))))
}

fn fn_type(args: &[CellValue]) -> CellValue {
    let t = match args.first() {
        Some(CellValue::Number(_)) => 1.0,
        Some(CellValue::Text(_)) => 2.0,
        Some(CellValue::Boolean(_)) => 4.0,
        Some(CellValue::Error(_)) => 16.0,
        Some(CellValue::Array(_)) => 64.0,
        _ => 1.0,
    };
    CellValue::Number(t)
}

fn fn_n(args: &[CellValue]) -> CellValue {
    CellValue::Number(args.first().and_then(|a| a.as_number()).unwrap_or(0.0))
}

// === Text & Regex ===

fn fn_concat(args: &[CellValue]) -> CellValue {
    let mut s = String::new();
    for a in args { s.push_str(&a.as_string()); }
    CellValue::Text(s)
}

fn fn_textjoin(args: &[CellValue]) -> CellValue {
    if args.len() < 3 { return CellValue::Error(CellError::Value); }
    let delim = args[0].as_string();
    let ignore_empty = args[1] == CellValue::Boolean(true);
    let mut parts = Vec::new();
    for a in &args[2..] {
        let text = a.as_string();
        if !ignore_empty || !text.is_empty() {
            parts.push(text);
        }
    }
    CellValue::Text(parts.join(&delim))
}

fn fn_left(args: &[CellValue]) -> CellValue {
    let text = match args.first() { Some(t) => t.as_string(), None => return CellValue::Error(CellError::Value) };
    let count = args.get(1).and_then(|a| a.as_number()).unwrap_or(1.0) as usize;
    CellValue::Text(text.chars().take(count).collect())
}

fn fn_right(args: &[CellValue]) -> CellValue {
    let text = match args.first() { Some(t) => t.as_string(), None => return CellValue::Error(CellError::Value) };
    let count = args.get(1).and_then(|a| a.as_number()).unwrap_or(1.0) as usize;
    let total = text.chars().count();
    let skip = total.saturating_sub(count);
    CellValue::Text(text.chars().skip(skip).collect())
}

fn fn_mid(args: &[CellValue]) -> CellValue {
    if args.len() < 3 { return CellValue::Error(CellError::Value); }
    let text = args[0].as_string();
    let start = match args[1].as_number() { Some(s) => (s as usize).saturating_sub(1), None => return CellValue::Error(CellError::Value) };
    let count = match args[2].as_number() { Some(c) => c as usize, None => return CellValue::Error(CellError::Value) };
    CellValue::Text(text.chars().skip(start).take(count).collect())
}

fn fn_len(args: &[CellValue]) -> CellValue {
    let text = args.first().map(|a| a.as_string()).unwrap_or_default();
    CellValue::Number(text.chars().count() as f64)
}

fn fn_find(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    let find = args[0].as_string();
    let within = args[1].as_string();
    if let Some(pos) = within.find(&find) {
        CellValue::Number((pos + 1) as f64)
    } else {
        CellValue::Error(CellError::Value)
    }
}

fn fn_replace(args: &[CellValue]) -> CellValue {
    if args.len() < 4 { return CellValue::Error(CellError::Value); }
    let text = args[0].as_string();
    let start = match args[1].as_number() { Some(s) => (s as usize).saturating_sub(1), None => return CellValue::Error(CellError::Value) };
    let num_chars = match args[2].as_number() { Some(c) => c as usize, None => return CellValue::Error(CellError::Value) };
    let new_text = args[3].as_string();

    let prefix: String = text.chars().take(start).collect();
    let suffix: String = text.chars().skip(start + num_chars).collect();
    CellValue::Text(format!("{}{}{}", prefix, new_text, suffix))
}

fn fn_substitute(args: &[CellValue]) -> CellValue {
    if args.len() < 3 { return CellValue::Error(CellError::Value); }
    let text = args[0].as_string();
    let old_text = args[1].as_string();
    let new_text = args[2].as_string();
    CellValue::Text(text.replace(&old_text, &new_text))
}

fn fn_trim(args: &[CellValue]) -> CellValue {
    let text = args.first().map(|a| a.as_string()).unwrap_or_default();
    CellValue::Text(text.trim().to_string())
}

fn fn_clean(args: &[CellValue]) -> CellValue {
    let text = args.first().map(|a| a.as_string()).unwrap_or_default();
    CellValue::Text(text.chars().filter(|c| !c.is_control()).collect())
}

fn fn_upper(args: &[CellValue]) -> CellValue {
    let text = args.first().map(|a| a.as_string()).unwrap_or_default();
    CellValue::Text(text.to_uppercase())
}

fn fn_lower(args: &[CellValue]) -> CellValue {
    let text = args.first().map(|a| a.as_string()).unwrap_or_default();
    CellValue::Text(text.to_lowercase())
}

fn fn_proper(args: &[CellValue]) -> CellValue {
    let text = args.first().map(|a| a.as_string()).unwrap_or_default();
    let mut res = String::new();
    let mut capitalize = true;
    for c in text.chars() {
        if c.is_alphabetic() {
            if capitalize {
                res.extend(c.to_uppercase());
                capitalize = false;
            } else {
                res.extend(c.to_lowercase());
            }
        } else {
            res.push(c);
            capitalize = true;
        }
    }
    CellValue::Text(res)
}

fn fn_exact(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    CellValue::Boolean(args[0].as_string() == args[1].as_string())
}

fn fn_rept(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    let text = args[0].as_string();
    let count = match args[1].as_number() { Some(c) => c as usize, None => return CellValue::Error(CellError::Value) };
    CellValue::Text(text.repeat(count))
}

fn fn_text(args: &[CellValue]) -> CellValue {
    args.first().map(|a| CellValue::Text(a.as_string())).unwrap_or_default()
}

fn fn_value(args: &[CellValue]) -> CellValue {
    let text = args.first().map(|a| a.as_string()).unwrap_or_default();
    match text.trim().parse::<f64>() {
        Ok(n) => CellValue::Number(n),
        Err(_) => CellValue::Error(CellError::Value),
    }
}

fn fn_char(args: &[CellValue]) -> CellValue {
    let code = match args.first().and_then(|a| a.as_number()) { Some(c) => c as u32, None => return CellValue::Error(CellError::Value) };
    match char::from_u32(code) {
        Some(c) => CellValue::Text(c.to_string()),
        None => CellValue::Error(CellError::Value),
    }
}

fn fn_code(args: &[CellValue]) -> CellValue {
    let text = args.first().map(|a| a.as_string()).unwrap_or_default();
    match text.chars().next() {
        Some(c) => CellValue::Number(c as u32 as f64),
        None => CellValue::Error(CellError::Value),
    }
}

fn fn_regexmatch(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    let text = args[0].as_string();
    let pat = args[1].as_string();
    match regex::Regex::new(&pat) {
        Ok(re) => CellValue::Boolean(re.is_match(&text)),
        Err(_) => CellValue::Error(CellError::Value),
    }
}

fn fn_regexextract(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    let text = args[0].as_string();
    let pat = args[1].as_string();
    match regex::Regex::new(&pat) {
        Ok(re) => {
            if let Some(m) = re.find(&text) {
                CellValue::Text(m.as_str().to_string())
            } else {
                CellValue::Error(CellError::NA)
            }
        }
        Err(_) => CellValue::Error(CellError::Value),
    }
}

fn fn_regexreplace(args: &[CellValue]) -> CellValue {
    if args.len() < 3 { return CellValue::Error(CellError::Value); }
    let text = args[0].as_string();
    let pat = args[1].as_string();
    let rep = args[2].as_string();
    match regex::Regex::new(&pat) {
        Ok(re) => CellValue::Text(re.replace_all(&text, rep.as_str()).to_string()),
        Err(_) => CellValue::Error(CellError::Value),
    }
}

// === Date & Time ===

fn serial_to_ymd(serial: f64) -> (i32, u32, u32) {
    let days = serial.floor() as i64;
    let adj_days = if days > 60 { days - 1 } else { days };
    let mut year = 1900;
    let mut d = adj_days - 1;
    while d >= if is_leap_year(year) { 366 } else { 365 } {
        d -= if is_leap_year(year) { 366 } else { 365 };
        year += 1;
    }
    let leap = is_leap_year(year);
    let month_days = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1;
    for &md in &month_days {
        if d < md { break; }
        d -= md;
        month += 1;
    }
    let day = (d + 1) as u32;
    (year as i32, month as u32, day)
}

fn ymd_to_serial(year: i32, month: u32, day: u32) -> f64 {
    let mut total_days: i64 = 0;
    for y in 1900..year {
        total_days += if is_leap_year(y as i64) { 366 } else { 365 };
    }
    let leap = is_leap_year(year as i64);
    let month_days = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 1..month {
        total_days += month_days[(m - 1) as usize];
    }
    total_days += day as i64;
    if total_days >= 60 {
        total_days += 1; // 1900 leap year bug
    }
    total_days as f64
}

fn is_leap_year(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

fn fn_date(args: &[CellValue]) -> CellValue {
    if args.len() < 3 { return CellValue::Error(CellError::Value); }
    let y = match args[0].as_number() { Some(y) => y as i32, None => return CellValue::Error(CellError::Value) };
    let m = match args[1].as_number() { Some(m) => m as u32, None => return CellValue::Error(CellError::Value) };
    let d = match args[2].as_number() { Some(d) => d as u32, None => return CellValue::Error(CellError::Value) };
    CellValue::Number(ymd_to_serial(y, m, d))
}

fn fn_year(args: &[CellValue]) -> CellValue {
    let serial = match args.first().and_then(|a| a.as_number()) { Some(s) => s, None => return CellValue::Error(CellError::Value) };
    let (y, _, _) = serial_to_ymd(serial);
    CellValue::Number(y as f64)
}

fn fn_month(args: &[CellValue]) -> CellValue {
    let serial = match args.first().and_then(|a| a.as_number()) { Some(s) => s, None => return CellValue::Error(CellError::Value) };
    let (_, m, _) = serial_to_ymd(serial);
    CellValue::Number(m as f64)
}

fn fn_day(args: &[CellValue]) -> CellValue {
    let serial = match args.first().and_then(|a| a.as_number()) { Some(s) => s, None => return CellValue::Error(CellError::Value) };
    let (_, _, d) = serial_to_ymd(serial);
    CellValue::Number(d as f64)
}

fn fn_hour(args: &[CellValue]) -> CellValue {
    let serial = match args.first().and_then(|a| a.as_number()) { Some(s) => s, None => return CellValue::Error(CellError::Value) };
    let secs = (serial.fract().abs() * 86400.0).round() as u64;
    CellValue::Number((secs / 3600) as f64)
}

fn fn_minute(args: &[CellValue]) -> CellValue {
    let serial = match args.first().and_then(|a| a.as_number()) { Some(s) => s, None => return CellValue::Error(CellError::Value) };
    let secs = (serial.fract().abs() * 86400.0).round() as u64;
    CellValue::Number(((secs % 3600) / 60) as f64)
}

fn fn_second(args: &[CellValue]) -> CellValue {
    let serial = match args.first().and_then(|a| a.as_number()) { Some(s) => s, None => return CellValue::Error(CellError::Value) };
    let secs = (serial.fract().abs() * 86400.0).round() as u64;
    CellValue::Number((secs % 60) as f64)
}

fn fn_edate(args: &[CellValue]) -> CellValue {
    let s = match args.first().and_then(|a| a.as_number()) { Some(v) => v, None => return CellValue::Error(CellError::Value) };
    let months_to_add = match args.get(1).and_then(|a| a.as_number()) { Some(v) => v as i32, None => return CellValue::Error(CellError::Value) };
    let (mut y, mut m, d) = serial_to_ymd(s);
    let total_months = (y * 12 + (m as i32 - 1)) + months_to_add;
    y = total_months / 12;
    m = (total_months % 12 + 1) as u32;
    let leap = is_leap_year(y as i64);
    let max_days = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][(m - 1) as usize];
    let adj_d = d.min(max_days);
    CellValue::Number(ymd_to_serial(y, m, adj_d))
}

fn fn_eomonth(args: &[CellValue]) -> CellValue {
    let s = match args.first().and_then(|a| a.as_number()) { Some(v) => v, None => return CellValue::Error(CellError::Value) };
    let months_to_add = match args.get(1).and_then(|a| a.as_number()) { Some(v) => v as i32, None => return CellValue::Error(CellError::Value) };
    let (mut y, mut m, _) = serial_to_ymd(s);
    let total_months = (y * 12 + (m as i32 - 1)) + months_to_add;
    y = total_months / 12;
    m = (total_months % 12 + 1) as u32;
    let leap = is_leap_year(y as i64);
    let last_day = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][(m - 1) as usize];
    CellValue::Number(ymd_to_serial(y, m, last_day))
}

fn fn_networkdays(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    let s1 = match args[0].as_number() { Some(s) => s.floor() as i64, None => return CellValue::Error(CellError::Value) };
    let s2 = match args[1].as_number() { Some(s) => s.floor() as i64, None => return CellValue::Error(CellError::Value) };
    let holidays: Vec<i64> = if args.len() > 2 {
        flatten_numbers(&args[2..]).into_iter().map(|h| h.floor() as i64).collect()
    } else {
        Vec::new()
    };

    let start = s1.min(s2);
    let end = s1.max(s2);
    let mut count = 0;
    for day in start..=end {
        let weekday = (day + 6) % 7; // 0=Mon, 5=Sat, 6=Sun
        if weekday < 5 && !holidays.contains(&day) {
            count += 1;
        }
    }
    let res = if s1 <= s2 { count } else { -count };
    CellValue::Number(res as f64)
}

fn fn_workday(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    let mut current = match args[0].as_number() { Some(s) => s.floor() as i64, None => return CellValue::Error(CellError::Value) };
    let days_to_add = match args[1].as_number() { Some(d) => d as i64, None => return CellValue::Error(CellError::Value) };
    let holidays: Vec<i64> = if args.len() > 2 {
        flatten_numbers(&args[2..]).into_iter().map(|h| h.floor() as i64).collect()
    } else {
        Vec::new()
    };

    let step = if days_to_add >= 0 { 1 } else { -1 };
    let mut remaining = days_to_add.abs();
    while remaining > 0 {
        current += step;
        let weekday = (current + 6) % 7;
        if weekday < 5 && !holidays.contains(&current) {
            remaining -= 1;
        }
    }
    CellValue::Number(current as f64)
}

fn fn_irr(args: &[CellValue]) -> CellValue {
    if args.is_empty() { return CellValue::Error(CellError::Value); }
    let flows = flatten_numbers(args);
    if flows.len() < 2 { return CellValue::Error(CellError::Value); }
    let mut r = 0.1;
    for _ in 0..100 {
        let mut npv = 0.0;
        let mut d_npv = 0.0;
        for (t, &cf) in flows.iter().enumerate() {
            let denom = (1.0_f64 + r).powi(t as i32);
            npv += cf / denom;
            if t > 0 {
                d_npv -= (t as f64 * cf) / ((1.0_f64 + r).powi((t + 1) as i32));
            }
        }
        if npv.abs() < 1e-7 {
            return CellValue::Number(r);
        }
        if d_npv.abs() < 1e-12 {
            break;
        }
        let next_r = r - npv / d_npv;
        if (next_r - r).abs() < 1e-9 {
            return CellValue::Number(next_r);
        }
        r = next_r;
    }
    if r.is_nan() || r.is_infinite() {
        CellValue::Error(CellError::Num)
    } else {
        CellValue::Number(r)
    }
}

fn fn_rate(args: &[CellValue]) -> CellValue {
    if args.len() < 3 { return CellValue::Error(CellError::Value); }
    let nper = match args[0].as_number() { Some(n) => n, None => return CellValue::Error(CellError::Value) };
    let pmt = match args[1].as_number() { Some(p) => p, None => return CellValue::Error(CellError::Value) };
    let pv = match args[2].as_number() { Some(v) => v, None => return CellValue::Error(CellError::Value) };
    let fv = args.get(3).and_then(|a| a.as_number()).unwrap_or(0.0);
    let p_type = args.get(4).and_then(|a| a.as_number()).unwrap_or(0.0);
    let mut r = args.get(5).and_then(|a| a.as_number()).unwrap_or(0.1);

    for _ in 0..100 {
        let f = if r == 0.0 {
            pv + pmt * nper + fv
        } else {
            pv * (1.0 + r).powf(nper) + pmt * (1.0 + r * p_type) * ((1.0 + r).powf(nper) - 1.0) / r + fv
        };
        if f.abs() < 1e-7 {
            return CellValue::Number(r);
        }
        let dr = 1e-5;
        let r_hi = r + dr;
        let f_hi = pv * (1.0 + r_hi).powf(nper) + pmt * (1.0 + r_hi * p_type) * ((1.0 + r_hi).powf(nper) - 1.0) / r_hi + fv;
        let df = (f_hi - f) / dr;
        if df.abs() < 1e-12 { break; }
        let next_r = r - f / df;
        if (next_r - r).abs() < 1e-9 {
            return CellValue::Number(next_r);
        }
        r = next_r;
    }
    CellValue::Number(r)
}

fn fn_nper(args: &[CellValue]) -> CellValue {
    if args.len() < 3 { return CellValue::Error(CellError::Value); }
    let rate = match args[0].as_number() { Some(r) => r, None => return CellValue::Error(CellError::Value) };
    let pmt = match args[1].as_number() { Some(p) => p, None => return CellValue::Error(CellError::Value) };
    let pv = match args[2].as_number() { Some(v) => v, None => return CellValue::Error(CellError::Value) };
    let fv = args.get(3).and_then(|a| a.as_number()).unwrap_or(0.0);
    let p_type = args.get(4).and_then(|a| a.as_number()).unwrap_or(0.0);

    if rate == 0.0 {
        if pmt == 0.0 { return CellValue::Error(CellError::DivZero); }
        return CellValue::Number(-(pv + fv) / pmt);
    }
    let num = pmt * (1.0 + rate * p_type) - fv * rate;
    let den = pmt * (1.0 + rate * p_type) + pv * rate;
    if den == 0.0 || (num / den) <= 0.0 { return CellValue::Error(CellError::Num); }
    CellValue::Number((num / den).ln() / (1.0 + rate).ln())
}

fn fn_effect(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    let nom = match args[0].as_number() { Some(n) => n, None => return CellValue::Error(CellError::Value) };
    let npery = match args[1].as_number() { Some(p) => p, None => return CellValue::Error(CellError::Value) };
    if nom <= 0.0 || npery < 1.0 { return CellValue::Error(CellError::Num); }
    CellValue::Number((1.0 + nom / npery).powf(npery) - 1.0)
}

fn fn_nominal(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    let eff = match args[0].as_number() { Some(e) => e, None => return CellValue::Error(CellError::Value) };
    let npery = match args[1].as_number() { Some(p) => p, None => return CellValue::Error(CellError::Value) };
    if eff <= 0.0 || npery < 1.0 { return CellValue::Error(CellError::Num); }
    CellValue::Number(npery * ((1.0 + eff).powf(1.0 / npery) - 1.0))
}

fn fn_time(args: &[CellValue]) -> CellValue {
    if args.len() < 3 { return CellValue::Error(CellError::Value); }
    let h = match args[0].as_number() { Some(h) => h, None => return CellValue::Error(CellError::Value) };
    let m = match args[1].as_number() { Some(m) => m, None => return CellValue::Error(CellError::Value) };
    let s = match args[2].as_number() { Some(s) => s, None => return CellValue::Error(CellError::Value) };
    CellValue::Number((h * 3600.0 + m * 60.0 + s) / 86400.0)
}

fn fn_datedif(args: &[CellValue]) -> CellValue {
    let d1 = args.first().and_then(|a| a.as_number()).unwrap_or(0.0);
    let d2 = args.get(1).and_then(|a| a.as_number()).unwrap_or(0.0);
    CellValue::Number((d2 - d1).abs())
}

fn fn_days(args: &[CellValue]) -> CellValue {
    fn_datedif(args)
}

fn fn_pmt(args: &[CellValue]) -> CellValue {
    if args.len() < 3 { return CellValue::Error(CellError::Value); }
    let rate = match args[0].as_number() { Some(r) => r, None => return CellValue::Error(CellError::Value) };
    let nper = match args[1].as_number() { Some(n) => n, None => return CellValue::Error(CellError::Value) };
    let pv = match args[2].as_number() { Some(p) => p, None => return CellValue::Error(CellError::Value) };
    if rate == 0.0 { return CellValue::Number(-pv / nper); }
    let pmt = (rate * pv) / (1.0 - (1.0 + rate).powf(-nper));
    CellValue::Number(-pmt)
}

fn fn_ipmt(args: &[CellValue]) -> CellValue { fn_pmt(args) }
fn fn_ppmt(args: &[CellValue]) -> CellValue { fn_pmt(args) }

fn fn_pv(args: &[CellValue]) -> CellValue {
    if args.len() < 3 { return CellValue::Error(CellError::Value); }
    let rate = match args[0].as_number() { Some(r) => r, None => return CellValue::Error(CellError::Value) };
    let nper = match args[1].as_number() { Some(n) => n, None => return CellValue::Error(CellError::Value) };
    let pmt = match args[2].as_number() { Some(p) => p, None => return CellValue::Error(CellError::Value) };
    if rate == 0.0 { return CellValue::Number(-pmt * nper); }
    let pv = (pmt * (1.0 - (1.0 + rate).powf(-nper))) / rate;
    CellValue::Number(-pv)
}

fn fn_fv(args: &[CellValue]) -> CellValue {
    if args.len() < 3 { return CellValue::Error(CellError::Value); }
    let rate = match args[0].as_number() { Some(r) => r, None => return CellValue::Error(CellError::Value) };
    let nper = match args[1].as_number() { Some(n) => n, None => return CellValue::Error(CellError::Value) };
    let pmt = match args[2].as_number() { Some(p) => p, None => return CellValue::Error(CellError::Value) };
    if rate == 0.0 { return CellValue::Number(-pmt * nper); }
    let fv = (pmt * ((1.0 + rate).powf(nper) - 1.0)) / rate;
    CellValue::Number(-fv)
}

fn fn_npv(args: &[CellValue]) -> CellValue {
    if args.len() < 2 { return CellValue::Error(CellError::Value); }
    let rate = match args[0].as_number() { Some(r) => r, None => return CellValue::Error(CellError::Value) };
    let flows = flatten_numbers(&args[1..]);
    let mut total = 0.0;
    for (i, f) in flows.iter().enumerate() {
        total += f / (1.0 + rate).powi((i + 1) as i32);
    }
    CellValue::Number(total)
}

fn fn_sln(args: &[CellValue]) -> CellValue {
    if args.len() < 3 { return CellValue::Error(CellError::Value); }
    let cost = match args[0].as_number() { Some(c) => c, None => return CellValue::Error(CellError::Value) };
    let salvage = match args[1].as_number() { Some(s) => s, None => return CellValue::Error(CellError::Value) };
    let life = match args[2].as_number() { Some(l) => l, None => return CellValue::Error(CellError::Value) };
    if life == 0.0 { return CellValue::Error(CellError::DivZero); }
    CellValue::Number((cost - salvage) / life)
}

fn fn_syd(args: &[CellValue]) -> CellValue { fn_sln(args) }
fn fn_db(args: &[CellValue]) -> CellValue { fn_sln(args) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_math_functions() {
        let res = call_function("SUM", &[CellValue::Number(10.0), CellValue::Number(20.0), CellValue::Number(12.0)]);
        assert_eq!(res, CellValue::Number(42.0));

        let abs = call_function("ABS", &[CellValue::Number(-42.0)]);
        assert_eq!(abs, CellValue::Number(42.0));

        let mround = call_function("MROUND", &[CellValue::Number(13.0), CellValue::Number(5.0)]);
        assert_eq!(mround, CellValue::Number(15.0));
    }

    #[test]
    fn test_lookup_functions() {
        let lookup_range = CellValue::Array(vec![
            vec![CellValue::Text("P1".into())],
            vec![CellValue::Text("P2".into())],
        ]);
        let return_range = CellValue::Array(vec![
            vec![CellValue::Number(100.0)],
            vec![CellValue::Number(200.0)],
        ]);

        let res = call_function("XLOOKUP", &[
            CellValue::Text("P2".into()),
            lookup_range,
            return_range,
        ]);
        assert_eq!(res, CellValue::Number(200.0));
    }

    #[test]
    fn test_financial_pmt() {
        // PMT(0.05/12, 60, 20000)
        let rate = 0.05 / 12.0;
        let pmt = call_function("PMT", &[
            CellValue::Number(rate),
            CellValue::Number(60.0),
            CellValue::Number(20000.0),
        ]);
        if let CellValue::Number(n) = pmt {
            assert!((n.abs() - 377.42).abs() < 1.0);
        } else {
            panic!("Expected Number");
        }
    }
}
