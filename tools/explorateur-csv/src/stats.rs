//! Per-column statistics. Null policy: an empty or whitespace-only cell is null and never counted.
//! Accumulated while rows stream in, so the final dispatch of a large file only finalises totals.
use crate::parser::{Parser, Table};
use serde_json::{Value, json};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Number,
    Text,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnStats {
    pub name: String,
    pub kind: Kind,
    pub count: usize,
    pub nulls: usize,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub mean: Option<f64>,
}
pub fn is_null(cell: &str) -> bool {
    cell.trim().is_empty()
}
const POW10: [f64; 23] = [
    1e0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10, 1e11, 1e12, 1e13, 1e14, 1e15, 1e16,
    1e17, 1e18, 1e19, 1e20, 1e21, 1e22,
];
/// `Ok(None)` means the cell is not decimal syntax; overflow to infinity is an error, never a silent text fallback.
pub fn parse_number(cell: &str) -> Result<Option<f64>, &'static str> {
    let s = cell.trim();
    let (negative, body) = match s.as_bytes().first() {
        Some(b'-') => (true, &s[1..]),
        Some(b'+') => (false, &s[1..]),
        _ => (false, s),
    };
    let b = body.as_bytes();
    let mut i = 0;
    let mut mantissa = 0u64;
    let mut digits = 0usize;
    while i < b.len() && b[i].is_ascii_digit() {
        if digits < 19 {
            mantissa = mantissa * 10 + u64::from(b[i] - b'0');
        }
        digits += 1;
        i += 1;
    }
    let int = i;
    let mut frac = 0;
    if b.get(i) == Some(&b'.') {
        i += 1;
        let start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            if digits < 19 {
                mantissa = mantissa * 10 + u64::from(b[i] - b'0');
            }
            digits += 1;
            i += 1;
        }
        frac = i - start;
    }
    if int + frac == 0 {
        return Ok(None);
    }
    let exponent = matches!(b.get(i), Some(b'e' | b'E'));
    if exponent {
        i += 1;
        if matches!(b.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        let start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == start {
            return Ok(None);
        }
    }
    if i != b.len() {
        return Ok(None);
    }
    // Clinger fast path: an exactly representable mantissa divided by an exact power of ten is correctly rounded.
    if !exponent && digits <= 15 && frac <= 22 {
        let value = mantissa as f64 / POW10[frac];
        return Ok(Some(if value == 0.0 {
            0.0
        } else if negative {
            -value
        } else {
            value
        }));
    }
    let value: f64 = s.parse().map_err(|_| "INVALID_NUMBER")?;
    if !value.is_finite() {
        return Err("NON_FINITE_NUMBER");
    }
    Ok(Some(if value == 0.0 { 0.0 } else { value }))
}
#[derive(Clone, Copy)]
struct Accumulator {
    count: usize,
    nulls: usize,
    numeric: bool,
    sum: f64,
    min: f64,
    max: f64,
}
impl Accumulator {
    fn new() -> Self {
        Self {
            count: 0,
            nulls: 0,
            numeric: true,
            sum: 0.0,
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }
    fn add(&mut self, cell: &str) -> Result<(), &'static str> {
        if is_null(cell) {
            self.nulls += 1;
            return Ok(());
        }
        self.count += 1;
        if !self.numeric {
            return Ok(());
        }
        match parse_number(cell)? {
            Some(n) => {
                self.sum += n;
                if !self.sum.is_finite() {
                    return Err("NUMERIC_OVERFLOW");
                }
                if n < self.min {
                    self.min = n;
                }
                if n > self.max {
                    self.max = n;
                }
            }
            None => self.numeric = false,
        }
        Ok(())
    }
    fn finish(self, name: &str) -> ColumnStats {
        if self.numeric && self.count > 0 {
            ColumnStats {
                name: name.into(),
                kind: Kind::Number,
                count: self.count,
                nulls: self.nulls,
                min: Some(self.min),
                max: Some(self.max),
                mean: Some(self.sum / self.count as f64),
            }
        } else {
            ColumnStats {
                name: name.into(),
                kind: Kind::Text,
                count: self.count,
                nulls: self.nulls,
                min: None,
                max: None,
                mean: None,
            }
        }
    }
}
/// Streaming parser plus per-column accumulators; `feed` is cheap enough to run once per file chunk.
pub struct Analyzer {
    parser: Parser,
    accumulators: Vec<Accumulator>,
    done: usize,
}
impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}
impl Analyzer {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
            accumulators: Vec::new(),
            done: 0,
        }
    }
    fn absorb(&mut self) -> Result<(), &'static str> {
        if self.accumulators.is_empty() {
            self.accumulators = vec![Accumulator::new(); self.parser.columns()];
        }
        while self.done < self.parser.rows() {
            for c in 0..self.accumulators.len() {
                let cell = self.parser.cell(self.done, c);
                self.accumulators[c].add(cell)?;
            }
            self.done += 1;
        }
        Ok(())
    }
    pub fn feed(&mut self, bytes: &[u8]) -> Result<(), &'static str> {
        self.parser.feed(bytes)?;
        self.absorb()
    }
    pub fn finish(mut self) -> Result<(Table, Vec<ColumnStats>), &'static str> {
        self.absorb()?;
        let Analyzer {
            parser,
            mut accumulators,
            mut done,
        } = self;
        let table = parser.finish()?;
        if accumulators.is_empty() {
            accumulators = vec![Accumulator::new(); table.columns()];
        }
        while done < table.rows() {
            for c in 0..accumulators.len() {
                accumulators[c].add(table.cell(done, c))?;
            }
            done += 1;
        }
        let stats = table
            .headers
            .iter()
            .zip(accumulators)
            .map(|(name, acc)| acc.finish(name))
            .collect();
        Ok((table, stats))
    }
}
pub fn summarize(table: &Table) -> Result<Vec<ColumnStats>, &'static str> {
    let mut accumulators = vec![Accumulator::new(); table.columns()];
    for r in 0..table.rows() {
        for (c, acc) in accumulators.iter_mut().enumerate() {
            acc.add(table.cell(r, c))?;
        }
    }
    Ok(table
        .headers
        .iter()
        .zip(accumulators)
        .map(|(name, acc)| acc.finish(name))
        .collect())
}
pub fn to_json(stats: &[ColumnStats]) -> Value {
    Value::Array(stats.iter().map(|s|{let mut o=json!({"name":s.name,"kind":if s.kind==Kind::Number{"number"}else{"text"},"count":s.count,"nulls":s.nulls});for (k,v) in [("min",s.min),("max",s.max),("mean",s.mean)]{if let Some(v)=v{o[k]=json!(v);}}o}).collect())
}
fn quote(text: &str) -> String {
    if text.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text.into()
    }
}
/// Column names come from untrusted files: neutralise spreadsheet formula prefixes in the exported report.
fn safe_text(text: &str) -> String {
    if text.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{text}")
    } else {
        text.into()
    }
}
fn number(value: Option<f64>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
}
pub fn report_csv(stats: &[ColumnStats]) -> String {
    let mut out = String::from("colonne,type,valeurs,vides,min,max,moyenne\n");
    for s in stats {
        out.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            quote(&safe_text(&s.name)),
            if s.kind == Kind::Number {
                "nombre"
            } else {
                "texte"
            },
            s.count,
            s.nulls,
            number(s.min),
            number(s.max),
            number(s.mean)
        ));
    }
    out
}
