//! Minimal PDF reader/merger: no I/O, no system access, bounded memory.
//! Reads classic xref tables, xref streams and object streams, then writes the
//! pages of two documents into one new, unencrypted file.
use miniz_oxide::inflate::decompress_to_vec_zlib_with_limit;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const INFLATE_LIMIT: usize = 8 * 1024 * 1024;
const MAX_OBJECTS: u32 = 100_000;
const MAX_PAGES: usize = 2_000;
const MAX_DEPTH: u8 = 64;
const INHERITED: [&[u8]; 4] = [b"Resources", b"MediaBox", b"CropBox", b"Rotate"];

type Dict = BTreeMap<Vec<u8>, Obj>;
type Res<T> = Result<T, &'static str>;

#[derive(Clone, Debug)]
enum Obj {
    Null,
    Bool(bool),
    Num(String),
    Name(Vec<u8>),
    /// Literal or hexadecimal string, kept verbatim with its delimiters.
    Raw(Vec<u8>),
    Array(Vec<Obj>),
    Dict(Dict),
    Stream(Dict, Vec<u8>),
    Ref(u32),
}

#[derive(Clone, Copy)]
enum Loc {
    Free,
    Offset(usize),
    InStream(u32),
}

fn is_ws(c: u8) -> bool {
    matches!(c, 0 | 9 | 10 | 12 | 13 | 32)
}
fn is_delim(c: u8) -> bool {
    matches!(
        c,
        b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%'
    )
}
fn find(hay: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    hay.get(from..)?
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|p| p + from)
}
fn rfind(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).rposition(|w| w == needle)
}

struct Lexer<'a> {
    data: &'a [u8],
    pos: usize,
}
impl<'a> Lexer<'a> {
    fn skip_ws(&mut self) {
        while let Some(&c) = self.data.get(self.pos) {
            if is_ws(c) {
                self.pos += 1;
            } else if c == b'%' {
                while self
                    .data
                    .get(self.pos)
                    .is_some_and(|c| !matches!(c, 10 | 13))
                {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }
    fn token(&mut self) -> &'a [u8] {
        let start = self.pos;
        while self
            .data
            .get(self.pos)
            .is_some_and(|c| !is_ws(*c) && !is_delim(*c))
        {
            self.pos += 1;
        }
        &self.data[start..self.pos]
    }
    fn uint(&mut self) -> Res<u32> {
        self.skip_ws();
        let t = self.token();
        if t.is_empty() || !t.iter().all(u8::is_ascii_digit) {
            return Err("INVALID_PDF");
        }
        std::str::from_utf8(t)
            .ok()
            .and_then(|s| s.parse().ok())
            .ok_or("INVALID_PDF")
    }
    fn keyword(&mut self, word: &[u8]) -> Res<()> {
        self.skip_ws();
        if self.token() == word {
            Ok(())
        } else {
            Err("INVALID_PDF")
        }
    }
    fn object(&mut self, depth: u8) -> Res<Obj> {
        if depth > MAX_DEPTH {
            return Err("INVALID_PDF");
        }
        self.skip_ws();
        let c = *self.data.get(self.pos).ok_or("INVALID_PDF")?;
        match c {
            b'<' if self.data.get(self.pos + 1) == Some(&b'<') => {
                self.pos += 2;
                let mut dict = Dict::new();
                loop {
                    self.skip_ws();
                    match self.data.get(self.pos) {
                        Some(b'>') if self.data.get(self.pos + 1) == Some(&b'>') => {
                            self.pos += 2;
                            return Ok(Obj::Dict(dict));
                        }
                        Some(b'/') => {
                            let Obj::Name(key) = self.object(depth + 1)? else {
                                return Err("INVALID_PDF");
                            };
                            let value = self.object(depth + 1)?;
                            dict.insert(key, value);
                        }
                        _ => return Err("INVALID_PDF"),
                    }
                }
            }
            b'<' => {
                let end = find(self.data, b">", self.pos).ok_or("INVALID_PDF")?;
                let raw = self.data[self.pos..=end].to_vec();
                self.pos = end + 1;
                Ok(Obj::Raw(raw))
            }
            b'[' => {
                self.pos += 1;
                let mut items = Vec::new();
                loop {
                    self.skip_ws();
                    if self.data.get(self.pos) == Some(&b']') {
                        self.pos += 1;
                        return Ok(Obj::Array(items));
                    }
                    if items.len() >= 1_000_000 {
                        return Err("INVALID_PDF");
                    }
                    items.push(self.object(depth + 1)?);
                }
            }
            b'(' => {
                let start = self.pos;
                let mut nesting = 0usize;
                while let Some(&c) = self.data.get(self.pos) {
                    self.pos += 1;
                    match c {
                        b'\\' => self.pos += 1,
                        b'(' => nesting += 1,
                        b')' => {
                            nesting -= 1;
                            if nesting == 0 {
                                let end = self.pos.min(self.data.len());
                                return Ok(Obj::Raw(self.data[start..end].to_vec()));
                            }
                        }
                        _ => {}
                    }
                }
                Err("INVALID_PDF")
            }
            b'/' => {
                self.pos += 1;
                Ok(Obj::Name(self.token().to_vec()))
            }
            b')' | b'>' | b']' | b'{' | b'}' => Err("INVALID_PDF"),
            _ => {
                let t = self.token();
                match t {
                    b"true" => return Ok(Obj::Bool(true)),
                    b"false" => return Ok(Obj::Bool(false)),
                    b"null" => return Ok(Obj::Null),
                    _ => {}
                }
                let text = std::str::from_utf8(t).map_err(|_| "INVALID_PDF")?;
                if !text.is_empty() && text.bytes().all(|c| c.is_ascii_digit()) {
                    let save = self.pos;
                    if let Ok(id) = text.parse::<u32>()
                        && self.uint().is_ok()
                    {
                        self.skip_ws();
                        if self.data.get(self.pos) == Some(&b'R')
                            && self
                                .data
                                .get(self.pos + 1)
                                .is_none_or(|c| is_ws(*c) || is_delim(*c))
                        {
                            self.pos += 1;
                            return Ok(Obj::Ref(id));
                        }
                    }
                    self.pos = save;
                    return Ok(Obj::Num(text.into()));
                }
                let body = text.strip_prefix(['+', '-']).unwrap_or(text);
                if !body.is_empty()
                    && body != "."
                    && body.bytes().all(|c| c.is_ascii_digit() || c == b'.')
                    && body.bytes().filter(|c| *c == b'.').count() <= 1
                {
                    return Ok(Obj::Num(text.into()));
                }
                Err("INVALID_PDF")
            }
        }
    }
}

impl Obj {
    fn int(&self) -> Option<i64> {
        match self {
            Obj::Num(s) => s.parse().ok(),
            _ => None,
        }
    }
    fn dict(&self) -> Option<&Dict> {
        match self {
            Obj::Dict(d) | Obj::Stream(d, _) => Some(d),
            _ => None,
        }
    }
}

fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let (ia, ib, ic) = (a as i32, b as i32, c as i32);
    let p = ia + ib - ic;
    let (pa, pb, pc) = ((p - ia).abs(), (p - ib).abs(), (p - ic).abs());
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

fn png_unpredict(data: &[u8], columns: usize, colors: usize, bpc: usize) -> Res<Vec<u8>> {
    let bits = columns
        .checked_mul(colors)
        .and_then(|n| n.checked_mul(bpc))
        .ok_or("INVALID_PDF")?;
    let row = bits.div_ceil(8);
    let bpp = (colors * bpc).div_ceil(8).max(1);
    if row == 0 || data.len() % (row + 1) != 0 {
        return Err("INVALID_PDF");
    }
    let mut out = Vec::with_capacity(data.len());
    let mut prev = vec![0u8; row];
    for line in data.chunks(row + 1) {
        let mut cur = line[1..].to_vec();
        for i in 0..row {
            let left = if i >= bpp { cur[i - bpp] } else { 0 };
            let up = prev[i];
            let upleft = if i >= bpp { prev[i - bpp] } else { 0 };
            let add = match line[0] {
                0 => 0,
                1 => left,
                2 => up,
                3 => ((left as u16 + up as u16) / 2) as u8,
                4 => paeth(left, up, upleft),
                _ => return Err("INVALID_PDF"),
            };
            cur[i] = cur[i].wrapping_add(add);
        }
        out.extend_from_slice(&cur);
        prev = cur;
    }
    Ok(out)
}

/// Decode a stream whose only filter is FlateDecode (optionally with a PNG predictor).
fn decode(dict: &Dict, raw: &[u8]) -> Res<Vec<u8>> {
    let filters: Vec<&[u8]> = match dict.get(b"Filter".as_slice()) {
        None | Some(Obj::Null) => Vec::new(),
        Some(Obj::Name(n)) => vec![n.as_slice()],
        Some(Obj::Array(a)) => a
            .iter()
            .map(|f| match f {
                Obj::Name(n) => Ok(n.as_slice()),
                _ => Err("INVALID_PDF"),
            })
            .collect::<Res<_>>()?,
        _ => return Err("INVALID_PDF"),
    };
    match filters.as_slice() {
        [] => Ok(raw.to_vec()),
        [b"FlateDecode"] | [b"Fl"] => {
            let data =
                decompress_to_vec_zlib_with_limit(raw, INFLATE_LIMIT).map_err(|_| "INVALID_PDF")?;
            let params = match dict.get(b"DecodeParms".as_slice()) {
                Some(Obj::Dict(d)) => Some(d),
                Some(Obj::Array(a)) => match a.as_slice() {
                    [Obj::Dict(d)] => Some(d),
                    _ => None,
                },
                _ => None,
            };
            let get = |key: &[u8], default: i64| {
                params
                    .and_then(|p| p.get(key))
                    .and_then(Obj::int)
                    .unwrap_or(default)
            };
            match get(b"Predictor", 1) {
                1 => Ok(data),
                10..=15 => {
                    let size = |v: i64| usize::try_from(v).ok().filter(|n| *n > 0 && *n <= 1 << 20);
                    png_unpredict(
                        &data,
                        size(get(b"Columns", 1)).ok_or("INVALID_PDF")?,
                        size(get(b"Colors", 1)).ok_or("INVALID_PDF")?,
                        size(get(b"BitsPerComponent", 8)).ok_or("INVALID_PDF")?,
                    )
                }
                _ => Err("UNSUPPORTED_PDF"),
            }
        }
        _ => Err("UNSUPPORTED_PDF"),
    }
}

struct Doc<'a> {
    data: &'a [u8],
    xref: BTreeMap<u32, Loc>,
    trailer: Dict,
    streams: BTreeMap<u32, BTreeMap<u32, Obj>>,
    overrides: BTreeMap<u32, Obj>,
}

impl<'a> Doc<'a> {
    fn open(data: &'a [u8]) -> Res<Self> {
        if find(&data[..data.len().min(1024)], b"%PDF-", 0).is_none() {
            return Err("INVALID_PDF");
        }
        let tail = rfind(data, b"startxref").ok_or("INVALID_PDF")?;
        let mut lexer = Lexer {
            data,
            pos: tail + 9,
        };
        lexer.skip_ws();
        let first = lexer.uint()? as usize;
        let mut doc = Doc {
            data,
            xref: BTreeMap::new(),
            trailer: Dict::new(),
            streams: BTreeMap::new(),
            overrides: BTreeMap::new(),
        };
        let mut pending = VecDeque::from([first]);
        let mut seen = BTreeSet::new();
        while let Some(offset) = pending.pop_front() {
            if !seen.insert(offset) || seen.len() > 64 {
                continue;
            }
            let trailer = doc.read_section(offset)?;
            if doc.trailer.is_empty() {
                doc.trailer = trailer.clone();
            }
            if let Some(prev) = trailer.get(b"Prev".as_slice()).and_then(Obj::int) {
                pending.push_front(usize::try_from(prev).map_err(|_| "INVALID_PDF")?);
            }
            if let Some(hybrid) = trailer.get(b"XRefStm".as_slice()).and_then(Obj::int) {
                pending.push_front(usize::try_from(hybrid).map_err(|_| "INVALID_PDF")?);
            }
        }
        if doc.trailer.contains_key(b"Encrypt".as_slice()) {
            return Err("ENCRYPTED");
        }
        Ok(doc)
    }

    fn put(&mut self, id: u32, loc: Loc) -> Res<()> {
        if id >= MAX_OBJECTS {
            return Err("UNSUPPORTED_PDF");
        }
        self.xref.entry(id).or_insert(loc);
        Ok(())
    }

    fn read_section(&mut self, offset: usize) -> Res<Dict> {
        let mut lexer = Lexer {
            data: self.data,
            pos: offset,
        };
        lexer.skip_ws();
        if self.data.get(lexer.pos..lexer.pos + 4) == Some(b"xref") {
            lexer.pos += 4;
            loop {
                lexer.skip_ws();
                if self.data.get(lexer.pos..lexer.pos + 7) == Some(b"trailer") {
                    lexer.pos += 7;
                    return match lexer.object(0)? {
                        Obj::Dict(d) => Ok(d),
                        _ => Err("INVALID_PDF"),
                    };
                }
                let start = lexer.uint()?;
                let count = lexer.uint()?;
                if count > MAX_OBJECTS {
                    return Err("UNSUPPORTED_PDF");
                }
                for i in 0..count {
                    let off = lexer.uint()? as usize;
                    lexer.uint()?;
                    lexer.skip_ws();
                    let kind = lexer.token();
                    let loc = match kind {
                        b"n" => Loc::Offset(off),
                        b"f" => Loc::Free,
                        _ => return Err("INVALID_PDF"),
                    };
                    self.put(start.checked_add(i).ok_or("INVALID_PDF")?, loc)?;
                }
            }
        }
        let Obj::Stream(dict, raw) = self.read_at(offset, None, 0)? else {
            return Err("INVALID_PDF");
        };
        let bytes = decode(&dict, &raw)?;
        let widths: Vec<usize> = match dict.get(b"W".as_slice()) {
            Some(Obj::Array(a)) if a.len() == 3 => a
                .iter()
                .map(|n| {
                    n.int()
                        .and_then(|n| usize::try_from(n).ok())
                        .filter(|n| *n <= 8)
                })
                .collect::<Option<_>>()
                .ok_or("INVALID_PDF")?,
            _ => return Err("INVALID_PDF"),
        };
        let size = dict
            .get(b"Size".as_slice())
            .and_then(Obj::int)
            .ok_or("INVALID_PDF")?;
        let index: Vec<i64> = match dict.get(b"Index".as_slice()) {
            Some(Obj::Array(a)) => a
                .iter()
                .map(|n| n.int().ok_or("INVALID_PDF"))
                .collect::<Res<_>>()?,
            _ => vec![0, size],
        };
        let step: usize = widths.iter().sum();
        if step == 0 || index.len() % 2 != 0 {
            return Err("INVALID_PDF");
        }
        let field = |bytes: &[u8], at: usize, width: usize| {
            bytes[at..at + width]
                .iter()
                .fold(0u64, |n, b| (n << 8) | *b as u64)
        };
        let mut at = 0;
        for pair in index.chunks(2) {
            let (start, count) = (
                u32::try_from(pair[0]).map_err(|_| "INVALID_PDF")?,
                u32::try_from(pair[1]).map_err(|_| "INVALID_PDF")?,
            );
            for i in 0..count {
                if at + step > bytes.len() {
                    return Err("INVALID_PDF");
                }
                let kind = if widths[0] == 0 {
                    1
                } else {
                    field(&bytes, at, widths[0])
                };
                let second = field(&bytes, at + widths[0], widths[1]);
                let id = start.checked_add(i).ok_or("INVALID_PDF")?;
                let loc = match kind {
                    0 => Loc::Free,
                    1 => Loc::Offset(usize::try_from(second).map_err(|_| "INVALID_PDF")?),
                    2 => Loc::InStream(u32::try_from(second).map_err(|_| "INVALID_PDF")?),
                    _ => Loc::Free,
                };
                self.put(id, loc)?;
                at += step;
            }
        }
        Ok(dict)
    }

    /// Parse the indirect object at `offset`. `depth` bounds indirect /Length chains.
    fn read_at(&self, offset: usize, expect: Option<u32>, depth: u8) -> Res<Obj> {
        let mut lexer = Lexer {
            data: self.data,
            pos: offset,
        };
        let id = lexer.uint()?;
        lexer.uint()?;
        lexer.keyword(b"obj")?;
        if expect.is_some_and(|e| e != id) {
            return Err("INVALID_PDF");
        }
        let object = lexer.object(0)?;
        lexer.skip_ws();
        let Obj::Dict(dict) = object else {
            return Ok(object);
        };
        if self.data.get(lexer.pos..lexer.pos + 6) != Some(b"stream") {
            return Ok(Obj::Dict(dict));
        }
        let mut start = lexer.pos + 6;
        if self.data.get(start) == Some(&b'\r') {
            start += 1;
        }
        if self.data.get(start) == Some(&b'\n') {
            start += 1;
        }
        let declared = match dict.get(b"Length".as_slice()) {
            Some(Obj::Ref(r)) if depth < 2 => match self.xref.get(r) {
                Some(Loc::Offset(o)) => self.read_at(*o, Some(*r), depth + 1)?.int(),
                _ => None,
            },
            Some(other) => other.int(),
            None => None,
        };
        let valid = declared.and_then(|n| usize::try_from(n).ok()).filter(|n| {
            start.checked_add(*n).is_some_and(|end| {
                let mut probe = Lexer {
                    data: self.data,
                    pos: end,
                };
                probe.skip_ws();
                self.data.get(probe.pos..probe.pos + 9) == Some(b"endstream")
            })
        });
        let len = match valid {
            Some(n) => n,
            None => {
                let mut end = find(self.data, b"endstream", start).ok_or("INVALID_PDF")?;
                if end > start && self.data[end - 1] == b'\n' {
                    end -= 1;
                }
                if end > start && self.data[end - 1] == b'\r' {
                    end -= 1;
                }
                end - start
            }
        };
        Ok(Obj::Stream(dict, self.data[start..start + len].to_vec()))
    }

    fn exists(&self, id: u32) -> bool {
        self.overrides.contains_key(&id)
            || matches!(self.xref.get(&id), Some(Loc::Offset(_) | Loc::InStream(_)))
    }

    fn get(&mut self, id: u32) -> Res<Obj> {
        if let Some(o) = self.overrides.get(&id) {
            return Ok(o.clone());
        }
        match self.xref.get(&id).copied() {
            Some(Loc::Offset(offset)) => self.read_at(offset, Some(id), 0),
            Some(Loc::InStream(holder)) => {
                if !self.streams.contains_key(&holder) {
                    let members = self.parse_object_stream(holder)?;
                    self.streams.insert(holder, members);
                }
                self.streams[&holder].get(&id).cloned().ok_or("INVALID_PDF")
            }
            _ => Ok(Obj::Null),
        }
    }

    fn parse_object_stream(&self, holder: u32) -> Res<BTreeMap<u32, Obj>> {
        let Some(Loc::Offset(offset)) = self.xref.get(&holder).copied() else {
            return Err("INVALID_PDF");
        };
        let Obj::Stream(dict, raw) = self.read_at(offset, Some(holder), 0)? else {
            return Err("INVALID_PDF");
        };
        let bytes = decode(&dict, &raw)?;
        let count = dict
            .get(b"N".as_slice())
            .and_then(Obj::int)
            .ok_or("INVALID_PDF")?;
        let first = dict
            .get(b"First".as_slice())
            .and_then(Obj::int)
            .ok_or("INVALID_PDF")?;
        let (count, first) = (
            usize::try_from(count).map_err(|_| "INVALID_PDF")?,
            usize::try_from(first).map_err(|_| "INVALID_PDF")?,
        );
        if count > MAX_OBJECTS as usize || first > bytes.len() {
            return Err("INVALID_PDF");
        }
        let mut header = Lexer {
            data: &bytes,
            pos: 0,
        };
        let mut members = BTreeMap::new();
        for _ in 0..count {
            let id = header.uint()?;
            let at = first
                .checked_add(header.uint()? as usize)
                .ok_or("INVALID_PDF")?;
            let mut body = Lexer {
                data: &bytes,
                pos: at,
            };
            members.insert(id, body.object(0)?);
        }
        Ok(members)
    }

    /// Leaves of the page tree in reading order, as page dictionaries that carry
    /// their inherited attributes. They are registered as overrides.
    fn pages(&mut self) -> Res<Vec<u32>> {
        let Some(Obj::Ref(root)) = self.trailer.get(b"Root".as_slice()).cloned() else {
            return Err("INVALID_PDF");
        };
        let catalog = self.get(root)?;
        let Some(Obj::Ref(top)) = catalog
            .dict()
            .and_then(|d| d.get(b"Pages".as_slice()))
            .cloned()
        else {
            return Err("INVALID_PDF");
        };
        let mut leaves = Vec::new();
        let mut visited = BTreeSet::new();
        self.walk(top, &Dict::new(), &mut visited, &mut leaves, 0)?;
        if leaves.is_empty() {
            return Err("NO_PAGES");
        }
        Ok(leaves)
    }

    fn walk(
        &mut self,
        id: u32,
        inherited: &Dict,
        visited: &mut BTreeSet<u32>,
        leaves: &mut Vec<u32>,
        depth: u8,
    ) -> Res<()> {
        if depth > MAX_DEPTH || !visited.insert(id) {
            return Err("INVALID_PDF");
        }
        let node = self.get(id)?;
        let Obj::Dict(mut dict) = node else {
            return Err("INVALID_PDF");
        };
        let mut scope = inherited.clone();
        for key in INHERITED {
            if let Some(v) = dict.get(key) {
                scope.insert(key.to_vec(), v.clone());
            }
        }
        let kids = match dict.get(b"Kids".as_slice()) {
            Some(Obj::Array(kids)) => Some(kids.clone()),
            Some(_) => return Err("INVALID_PDF"),
            None => None,
        };
        match kids {
            Some(kids) => {
                for kid in kids {
                    let Obj::Ref(kid) = kid else {
                        return Err("INVALID_PDF");
                    };
                    self.walk(kid, &scope, visited, leaves, depth + 1)?;
                }
            }
            None => {
                if leaves.len() >= MAX_PAGES {
                    return Err("TOO_LARGE");
                }
                for (key, value) in scope {
                    dict.entry(key).or_insert(value);
                }
                dict.remove(b"Parent".as_slice());
                self.overrides.insert(id, Obj::Dict(dict));
                leaves.push(id);
            }
        }
        Ok(())
    }
}

fn remap(
    obj: Obj,
    doc: &Doc,
    map: &mut BTreeMap<u32, u32>,
    next: &mut u32,
    todo: &mut Vec<u32>,
) -> Obj {
    match obj {
        Obj::Ref(id) => {
            if !doc.exists(id) {
                return Obj::Null;
            }
            let new = *map.entry(id).or_insert_with(|| {
                let n = *next;
                *next += 1;
                todo.push(id);
                n
            });
            Obj::Ref(new)
        }
        Obj::Array(items) => Obj::Array(
            items
                .into_iter()
                .map(|o| remap(o, doc, map, next, todo))
                .collect(),
        ),
        Obj::Dict(d) => Obj::Dict(
            d.into_iter()
                .map(|(k, v)| (k, remap(v, doc, map, next, todo)))
                .collect(),
        ),
        Obj::Stream(d, raw) => {
            let d = d
                .into_iter()
                .map(|(k, v)| (k, remap(v, doc, map, next, todo)))
                .collect();
            Obj::Stream(d, raw)
        }
        other => other,
    }
}

fn write_obj(out: &mut Vec<u8>, obj: &Obj) {
    match obj {
        Obj::Null => out.extend_from_slice(b"null"),
        Obj::Bool(b) => out.extend_from_slice(if *b { b"true" } else { b"false" }),
        Obj::Num(n) => out.extend_from_slice(n.as_bytes()),
        Obj::Name(n) => {
            out.push(b'/');
            out.extend_from_slice(n);
        }
        Obj::Raw(r) => out.extend_from_slice(r),
        Obj::Array(items) => {
            out.push(b'[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(b' ');
                }
                write_obj(out, item);
            }
            out.push(b']');
        }
        Obj::Dict(d) => write_dict(out, d),
        Obj::Stream(d, raw) => {
            let mut d = d.clone();
            d.insert(b"Length".to_vec(), Obj::Num(raw.len().to_string()));
            write_dict(out, &d);
            out.extend_from_slice(b"\nstream\n");
            out.extend_from_slice(raw);
            out.extend_from_slice(b"\nendstream");
        }
        Obj::Ref(id) => out.extend_from_slice(format!("{id} 0 R").as_bytes()),
    }
}
fn write_dict(out: &mut Vec<u8>, dict: &Dict) {
    out.extend_from_slice(b"<<");
    for (key, value) in dict {
        out.push(b'/');
        out.extend_from_slice(key);
        out.push(b' ');
        write_obj(out, value);
        out.push(b' ');
    }
    out.extend_from_slice(b">>");
}

/// Merge `first` then `second` into one PDF. Returns the file and its page count.
/// Document-level extras (outlines, forms, named destinations) are not carried over.
pub fn merge(first: &[u8], second: &[u8], max_output: usize) -> Res<(Vec<u8>, usize)> {
    let mut objects: BTreeMap<u32, Obj> = BTreeMap::new();
    let mut page_ids = Vec::new();
    let mut next = 3u32;
    for bytes in [first, second] {
        let mut doc = Doc::open(bytes)?;
        let leaves = doc.pages()?;
        let mut map = BTreeMap::new();
        let mut todo = Vec::new();
        for leaf in &leaves {
            let mut pending = Vec::new();
            let new = *map.entry(*leaf).or_insert_with(|| {
                let n = next;
                next += 1;
                pending.push(*leaf);
                n
            });
            todo.extend(pending);
            page_ids.push(new);
        }
        while let Some(id) = todo.pop() {
            let new = map[&id];
            let object = doc.get(id)?;
            let object = remap(object, &doc, &mut map, &mut next, &mut todo);
            if next > MAX_OBJECTS {
                return Err("TOO_LARGE");
            }
            objects.insert(new, object);
        }
        if page_ids.len() > MAX_PAGES {
            return Err("TOO_LARGE");
        }
    }
    for id in &page_ids {
        if let Some(Obj::Dict(page)) = objects.get_mut(id) {
            page.insert(b"Parent".to_vec(), Obj::Ref(2));
        }
    }
    let catalog = Dict::from([
        (b"Type".to_vec(), Obj::Name(b"Catalog".to_vec())),
        (b"Pages".to_vec(), Obj::Ref(2)),
    ]);
    let pages = Dict::from([
        (b"Type".to_vec(), Obj::Name(b"Pages".to_vec())),
        (
            b"Kids".to_vec(),
            Obj::Array(page_ids.iter().map(|id| Obj::Ref(*id)).collect()),
        ),
        (b"Count".to_vec(), Obj::Num(page_ids.len().to_string())),
    ]);
    objects.insert(1, Obj::Dict(catalog));
    objects.insert(2, Obj::Dict(pages));

    let mut out = b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::with_capacity(objects.len());
    for expected in 1..next {
        let object = objects.get(&expected).ok_or("INVALID_PDF")?;
        offsets.push(out.len());
        out.extend_from_slice(format!("{expected} 0 obj\n").as_bytes());
        write_obj(&mut out, object);
        out.extend_from_slice(b"\nendobj\n");
        if out.len() > max_output {
            return Err("TOO_LARGE");
        }
    }
    let table = out.len();
    out.extend_from_slice(format!("xref\n0 {next}\n0000000000 65535 f \n").as_bytes());
    for offset in offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!("trailer\n<< /Size {next} /Root 1 0 R >>\nstartxref\n{table}\n%%EOF\n").as_bytes(),
    );
    if out.len() > max_output {
        return Err("TOO_LARGE");
    }
    Ok((out, page_ids.len()))
}

/// Number of pages of a readable, unencrypted PDF.
pub fn page_count(bytes: &[u8]) -> Res<usize> {
    Ok(Doc::open(bytes)?.pages()?.len())
}
