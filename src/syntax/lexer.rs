//! Tokenizer for the Prolog-like surface syntax.

use super::{Args, F64, Name, Term};
use std::fmt;

/// A name's place in the lexer's table ([`Lexer::name`]). A token carries
/// its name's place and nothing it would have to drop, so that moving
/// tokens through the lookahead is cheap.
pub(crate) type NameId = u32;

/// The names the parser tests tokens for, in the first places of every
/// lexer's table: a test is a comparison of places.
pub(crate) mod kw {
    use super::NameId;

    pub(crate) const QUERY: NameId = 0;
    pub(crate) const EVIDENCE: NameId = 1;
    pub(crate) const T: NameId = 2;
    pub(crate) const NOT: NameId = 3;
    pub(crate) const IS: NameId = 4;
    pub(crate) const MOD: NameId = 5;
    pub(crate) const REM: NameId = 6;
    pub(crate) const XOR: NameId = 7;
    /// The names, each at its place.
    pub(super) const ALL: [&str; 8] = ["query", "evidence", "t", "not", "is", "mod", "rem", "xor"];
}

/// An operator written with symbols: a comparison or an arithmetic one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Op {
    Eq,
    Ne,
    Identical,
    NotIdentical,
    Lt,
    Le,
    Gt,
    Ge,
    ArithEq,
    ArithNe,
    TermLt,
    TermLe,
    TermGt,
    TermGe,
    Plus,
    Minus,
    Star,
    Slash,
    IntDiv,
    Pow,
    Caret,
    BitAnd,
    BitOr,
    Shl,
    Shr,
    Backslash,
}

impl Op {
    pub(crate) fn text(self) -> &'static str {
        match self {
            Op::Eq => "=",
            Op::Ne => "\\=",
            Op::Identical => "==",
            Op::NotIdentical => "\\==",
            Op::Lt => "<",
            Op::Le => "=<",
            Op::Gt => ">",
            Op::Ge => ">=",
            Op::ArithEq => "=:=",
            Op::ArithNe => "=\\=",
            Op::TermLt => "@<",
            Op::TermLe => "@=<",
            Op::TermGt => "@>",
            Op::TermGe => "@>=",
            Op::Plus => "+",
            Op::Minus => "-",
            Op::Star => "*",
            Op::Slash => "/",
            Op::IntDiv => "//",
            Op::Pow => "**",
            Op::Caret => "^",
            Op::BitAnd => "/\\",
            Op::BitOr => "\\/",
            Op::Shl => "<<",
            Op::Shr => ">>",
            Op::Backslash => "\\",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Tok<'a> {
    /// Lowercase-initial identifier or quoted atom.
    Name(NameId),
    /// Uppercase or `_`-initial identifier, as written.
    Var(&'a str),
    Int(i64),
    Float(f64),
    /// `(`
    LParen,
    RParen,
    /// `[`, `]` and `|`, of a list.
    LBracket,
    RBracket,
    Bar,
    Comma,
    Semi,
    /// `.` terminating a clause
    Dot,
    /// `:-`
    Neck,
    /// `::`
    DoubleColon,
    /// `\+`
    Naf,
    Op(Op),
    Eof,
}

/// What [`Lexer::common`] found: a token, or the byte where a name or
/// another kind of token starts.
enum Common<'a> {
    Tok(Tok<'a>),
    Name(usize),
    Other,
}

/// A token as error messages name it ([`Lexer::show`]).
pub(crate) struct Shown<'l, 't> {
    tok: Tok<'t>,
    names: &'l [Name],
}

impl fmt::Display for Shown<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.tok {
            Tok::Name(id) => write!(f, "name `{}`", self.names[id as usize]),
            Tok::Var(s) => write!(f, "variable `{s}`"),
            Tok::Int(i) => write!(f, "integer `{i}`"),
            Tok::Float(x) => write!(f, "number `{x}`"),
            Tok::LParen => f.write_str("`(`"),
            Tok::RParen => f.write_str("`)`"),
            Tok::LBracket => f.write_str("`[`"),
            Tok::RBracket => f.write_str("`]`"),
            Tok::Bar => f.write_str("`|`"),
            Tok::Comma => f.write_str("`,`"),
            Tok::Semi => f.write_str("`;`"),
            Tok::Dot => f.write_str("`.`"),
            Tok::Neck => f.write_str("`:-`"),
            Tok::DoubleColon => f.write_str("`::`"),
            Tok::Naf => f.write_str("`\\+`"),
            Tok::Op(op) => write!(f, "operator `{}`", op.text()),
            Tok::Eof => f.write_str("end of input"),
        }
    }
}

/// A token with its 1-based line number.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Spanned<'a> {
    pub(crate) tok: Tok<'a>,
    pub(crate) line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LexError {
    pub(crate) line: usize,
    pub(crate) msg: String,
}

/// The character at byte `i` of `src`, and its length in bytes.
fn char_at(src: &str, i: usize) -> (char, usize) {
    let c = src[i..]
        .chars()
        .next()
        .expect("i is a char boundary inside src");
    (c, c.len_utf8())
}

/// Whether an ASCII byte continues an identifier: a letter, a digit or `_`.
static IDENT: [bool; 256] = {
    let mut t = [false; 256];
    let mut c = 0;
    while c < 128 {
        t[c] = (c as u8).is_ascii_alphanumeric() || c == b'_' as usize;
        c += 1;
    }
    t
};

/// The end of an identifier whose first character ends before `i`: ASCII
/// letters, digits and `_`, and any other alphanumeric character.
fn ident_end(src: &str, mut i: usize) -> usize {
    let b = src.as_bytes();
    loop {
        match b.get(i) {
            Some(&d) if IDENT[d as usize] => i += 1,
            Some(&d) if d >= 0x80 => {
                let (ch, len) = char_at(src, i);
                if !ch.is_alphanumeric() {
                    return i;
                }
                i += len;
            }
            _ => return i,
        }
    }
}

/// `10^k` for the exponents a double holds exactly.
const POW10: [f64; 23] = [
    1e0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10, 1e11, 1e12, 1e13, 1e14, 1e15, 1e16,
    1e17, 1e18, 1e19, 1e20, 1e21, 1e22,
];

/// A number read from its first digit: where it ends, whether it is a
/// float, `d+(.d+)?([eE][+-]?d+)?` with a fraction or an exponent, and its
/// digits as an integer `w`, with how many there are, and its value
/// `w·10^k`. A `.` followed by a digit continues a number; a lone `.` ends
/// the clause. Read in one pass.
struct Number {
    end: usize,
    float: bool,
    w: u64,
    digits: usize,
    k: i64,
}

fn number(b: &[u8], start: usize) -> Number {
    let n = b.len();
    let digit = |i: usize| (i < n && b[i].is_ascii_digit()).then(|| u64::from(b[i] - b'0'));
    let (mut i, mut w, mut digits, mut k, mut float) = (start, 0u64, 0, 0i64, false);
    while let Some(d) = digit(i) {
        w = w.wrapping_mul(10).wrapping_add(d);
        (digits, i) = (digits + 1, i + 1);
    }
    if i + 1 < n && b[i] == b'.' && b[i + 1].is_ascii_digit() {
        float = true;
        i += 1;
        while let Some(d) = digit(i) {
            w = w.wrapping_mul(10).wrapping_add(d);
            (digits, k, i) = (digits + 1, k - 1, i + 1);
        }
    }
    if i < n && (b[i] == b'e' || b[i] == b'E') {
        let negative = i + 1 < n && b[i + 1] == b'-';
        let j = if i + 1 < n && (b[i + 1] == b'+' || b[i + 1] == b'-') {
            i + 2
        } else {
            i + 1
        };
        if digit(j).is_some() {
            float = true;
            i = j;
            let mut e: i64 = 0;
            while let Some(d) = digit(i) {
                e = e.saturating_mul(10).saturating_add(d as i64);
                i += 1;
            }
            k = if negative {
                k.saturating_sub(e)
            } else {
                k.saturating_add(e)
            };
        }
    }
    Number {
        end: i,
        float,
        w,
        digits,
        k,
    }
}

/// The value of a decimal whose digits make an integer `w` of at most 2^53
/// and whose value is `w·10^k` with `|k| ≤ 22`; `None` for any other, or
/// for more digits than `w` holds. Both `w` and `10^|k|` are exact doubles,
/// so one multiplication or division rounds the decimal's value once, to
/// nearest: the value `str::parse` gives, which takes this path itself for
/// such decimals after a general scan of the text. Most of a network's
/// probabilities are such decimals.
fn decimal(&Number { w, digits, k, .. }: &Number) -> Option<f64> {
    if digits > 19 || w > 1 << 53 || !(-22..=22).contains(&k) {
        return None;
    }
    let x = w as f64;
    Some(if k < 0 {
        x / POW10[k.unsigned_abs() as usize]
    } else {
        x * POW10[k as usize]
    })
}

/// The tokens of a source, one at a time. The scan is over bytes: every
/// token but a name starts with an ASCII byte, and a byte of 0x80 or more
/// is decoded as the char it starts. Neither the chars nor the tokens of
/// the whole source are collected first.
pub(crate) struct Lexer<'a> {
    src: &'a str,
    i: usize,
    line: usize,
    /// One [`Name`] per distinct name, at its place, shared by every token
    /// that spells it; [`kw::ALL`] first.
    names: Vec<Name>,
    /// The places by their names' hash ([`text_hash`]), probed in turn from
    /// the slot of the hash's top bits, the hash shifted right by `shift`.
    slots: Vec<Slot>,
    shift: u32,
    /// A quoted atom's text with its escapes read, kept for its capacity.
    quoted: String,
    /// The first error, where the scan stopped: from there on the tokens
    /// are the end of the input, on the error's line.
    error: Option<LexError>,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(src: &'a str) -> Self {
        let mut lexer = Lexer {
            src,
            i: 0,
            line: 1,
            names: Vec::new(),
            slots: vec![Slot::default(); 64],
            shift: 64 - 6,
            quoted: String::new(),
            error: None,
        };
        for word in kw::ALL {
            lexer.place(word.as_bytes());
        }
        lexer
    }

    /// Where the lexer stands, for [`Lexer::rewind`].
    pub(crate) fn mark(&self) -> (usize, usize, bool) {
        (self.i, self.line, self.error.is_some())
    }

    /// Back to a [`Lexer::mark`]: an error met since is forgotten, and met
    /// again if the scan comes back to it.
    pub(crate) fn rewind(&mut self, (i, line, erred): (usize, usize, bool)) {
        self.i = i;
        self.line = line;
        if !erred {
            self.error = None;
        }
    }

    /// The first error met, if any.
    pub(crate) fn error(&mut self) -> Option<LexError> {
        self.error.take()
    }

    /// The shared name at `id`.
    pub(crate) fn name(&self, id: NameId) -> &Name {
        &self.names[id as usize]
    }

    /// The name of text `s`, shared with the names the source spells so:
    /// an operator's or a list's, which the parser names.
    pub(crate) fn intern(&mut self, s: &str) -> Name {
        let id = self.place(s.as_bytes());
        self.names[id as usize].clone()
    }

    /// `tok` as error messages name it.
    pub(crate) fn show<'t>(&self, tok: Tok<'t>) -> Shown<'_, 't> {
        Shown {
            tok,
            names: &self.names,
        }
    }

    /// The place of the name of text `s`, a place made for it if it has
    /// none. A name read from the source as `str` is not checked again to
    /// start and end at a char boundary, which it does.
    fn place(&mut self, s: &[u8]) -> NameId {
        let key = text_key(s);
        self.find(s, key, text_hash(s, key))
    }

    /// [`Lexer::place`] of `s`, of key `key` ([`text_key`]) and hash
    /// `hash`. Inlined where a name is read, with the making of a place
    /// out of line.
    #[inline(always)]
    fn find(&mut self, s: &[u8], key: u64, hash: u64) -> NameId {
        let mask = self.slots.len() - 1;
        let mut i = (hash >> self.shift) as usize & mask;
        loop {
            let slot = self.slots[i];
            if slot.place == 0 {
                return self.add(s, key, i);
            }
            // A text shorter than eight bytes is its key and its length.
            if slot.key == key
                && slot.len as usize == s.len()
                && (s.len() < 8
                    || same_long_text(self.names[slot.place as usize - 1].as_bytes(), s))
            {
                return slot.place - 1;
            }
            i = (i + 1) & mask;
        }
    }

    /// A place for the name of text `s` and key `key`, at the empty slot
    /// `i` its probe ended on.
    #[cold]
    #[inline(never)]
    fn add(&mut self, s: &[u8], key: u64, i: usize) -> NameId {
        let id = NameId::try_from(self.names.len())
            .ok()
            .filter(|&id| id < NameId::MAX)
            .expect("fewer than 2^32 - 1 names");
        let len = u32::try_from(s.len()).expect("a name shorter than 4 GiB");
        // A name read from the source, which `s` then lies in, is its text
        // there, not checked again as UTF-8; a keyword's or a quoted
        // atom's is checked.
        let source = self.src.as_bytes().as_ptr_range();
        let text = if source.contains(&s.as_ptr()) {
            let at = s.as_ptr() as usize - source.start as usize;
            &self.src[at..at + s.len()]
        } else {
            std::str::from_utf8(s).expect("a name read from text")
        };
        self.names.push(text.into());
        self.slots[i] = Slot {
            key,
            len,
            place: id + 1,
        };
        if 2 * self.names.len() > self.slots.len() {
            self.grow();
        }
        id
    }

    /// Twice the slots, every name placed again.
    #[cold]
    #[inline(never)]
    fn grow(&mut self) {
        let mut slots = vec![Slot::default(); 2 * self.slots.len()];
        let (mask, shift) = (slots.len() - 1, self.shift - 1);
        for slot in &self.slots {
            if slot.place == 0 {
                continue;
            }
            let hash = text_hash(self.names[slot.place as usize - 1].as_bytes(), slot.key);
            let mut i = (hash >> shift) as usize;
            while slots[i].place != 0 {
                i = (i + 1) & mask;
            }
            slots[i] = *slot;
        }
        self.slots = slots;
        self.shift = shift;
    }

    /// The next token, and [`Tok::Eof`] at the end, again and again; after
    /// an error ([`Lexer::error`]), the end of the input on its line.
    pub(crate) fn token(&mut self) -> Spanned<'a> {
        let tok = self.scan();
        Spanned {
            tok,
            line: self.line,
        }
    }

    /// The parser's fast path for the arguments of an atom, the common
    /// compound: called with the scan past the compound's `(`, it reads
    /// the arguments and the `)` here, where each argument is a flat term,
    /// a name, a variable or a number, as the parser's flat terms read
    /// them (`_` a fresh variable `_G<n>`, `n` counted in `anon`): one held
    /// in place, and more gathered on `terms` and moved from there. Where a
    /// token is of another kind, `None`, and the scan, `terms` and `anon`
    /// back where they were: the parser reads the tokens as usual, with its
    /// errors. Read by the parser, an argument would move two tokens
    /// through its lookahead and be tested there twice.
    pub(crate) fn flat_arguments(
        &mut self,
        terms: &mut Vec<Term>,
        anon: &mut usize,
    ) -> Option<Args> {
        let (mark, start, fresh) = (self.mark(), terms.len(), *anon);
        loop {
            let term = match self.scan() {
                Tok::Name(id) => Term::Const(self.names[id as usize].clone()),
                Tok::Var("_") => {
                    *anon += 1;
                    Term::Var(format!("_G{}", *anon))
                }
                Tok::Var(v) => Term::Var(v.to_owned()),
                Tok::Int(i) => Term::Int(i),
                Tok::Float(x) => Term::Float(F64::rounded(x)),
                Tok::Op(Op::Minus) => match self.scan() {
                    Tok::Int(i) => Term::Int(-i),
                    Tok::Float(x) => Term::Float(F64::rounded(-x)),
                    _ => break,
                },
                _ => break,
            };
            match self.scan() {
                Tok::Comma => terms.push(term),
                Tok::RParen if terms.len() == start => return Some(Args::One(term)),
                Tok::RParen => {
                    terms.push(term);
                    return Some(Args::Many(terms.split_off(start)));
                }
                _ => break,
            }
        }
        self.rewind(mark);
        terms.truncate(start);
        *anon = fresh;
        None
    }

    /// The error `msg` on `line`, where the scan stops: the end of the
    /// input from here on, on that line.
    #[cold]
    #[inline(never)]
    fn fail(&mut self, line: usize, msg: String) -> Tok<'a> {
        self.i = self.src.len();
        self.line = line;
        self.error = Some(LexError { line, msg });
        Tok::Eof
    }

    /// The next token. The blanks of a line, line ends, the marks that are
    /// a token alone and the end are read here ([`Lexer::common`]), and a
    /// name, the other common token, and everything else are calls the
    /// scan ends on, so that it saves one register rather than the six of
    /// one loop over every kind.
    fn scan(&mut self) -> Tok<'a> {
        match self.common() {
            Common::Tok(tok) => tok,
            Common::Name(i) => self.word(i),
            Common::Other => self.uncommon(),
        }
    }

    /// The blanks of a line and line ends skipped, the next token where it
    /// is a mark that is a token alone or the end, or where the next token
    /// starts if it is a name or anything else, the scan on it.
    #[inline(always)]
    fn common(&mut self) -> Common<'a> {
        let b = self.src.as_bytes();
        let mut i = self.i;
        loop {
            let tok = match b.get(i) {
                None => {
                    self.i = i;
                    return Common::Tok(Tok::Eof);
                }
                Some(b' ') => {
                    i += 1;
                    continue;
                }
                Some(b'\n') => {
                    self.line += 1;
                    i += 1;
                    continue;
                }
                Some(b'(') => Tok::LParen,
                Some(b')') => Tok::RParen,
                Some(b',') => Tok::Comma,
                Some(b';') => Tok::Semi,
                Some(b'.') => Tok::Dot,
                Some(b'a'..=b'z') => return Common::Name(i),
                Some(_) => {
                    self.i = i;
                    return Common::Other;
                }
            };
            self.i = i + 1;
            return Common::Tok(tok);
        }
    }

    /// The name that starts at byte `i`, a lowercase letter, read and
    /// placed among the names, the scan moved past it. A name of fewer
    /// than eight ASCII bytes, the common one, is read from the eight bytes
    /// at `i`, which are its key ([`text_key`]) once the bytes past it are
    /// masked off; any other in [`Lexer::long_word`].
    #[inline(never)]
    fn word(&mut self, i: usize) -> Tok<'a> {
        let b = self.src.as_bytes();
        let Some(chunk) = b.get(i..).and_then(<[u8]>::first_chunk::<8>) else {
            return self.long_word(i, 1);
        };
        let mut n = 1;
        while n < 8 && IDENT[chunk[n] as usize] {
            n += 1;
        }
        if n == 8 || chunk[n] >= 0x80 {
            return self.long_word(i, n);
        }
        self.i = i + n;
        let key = u64::from_le_bytes(*chunk) & (u64::MAX >> (64 - 8 * n));
        Tok::Name(self.find(&chunk[..n], key, short_hash(key)))
    }

    /// [`Lexer::word`] of the name at byte `i`, whose first `n` bytes are
    /// ASCII and continue it.
    #[inline(never)]
    fn long_word(&mut self, i: usize, n: usize) -> Tok<'a> {
        let src = self.src;
        let end = ident_end(src, i + n);
        self.i = end;
        Tok::Name(self.place(&src.as_bytes()[i..end]))
    }

    /// The next token from a byte [`Lexer::common`] stops on: the blanks
    /// and comments of [`Lexer::other`] skipped, until a token of either.
    #[inline(never)]
    fn uncommon(&mut self) -> Tok<'a> {
        loop {
            if let Some(tok) = self.other(self.i) {
                return tok;
            }
            match self.common() {
                Common::Tok(tok) => return tok,
                Common::Name(i) => return self.word(i),
                Common::Other => {}
            }
        }
    }

    /// The token, blanks or comment at byte `i`, of the kinds
    /// [`Lexer::common`] leaves, read, the scan moved past them:
    /// `None` for blanks or a comment. A byte of 0x80 or more is decoded
    /// as the char it starts.
    fn other(&mut self, mut i: usize) -> Option<Tok<'a>> {
        let src = self.src;
        let b = src.as_bytes();
        let n = b.len();
        let at = |i: usize| b.get(i).copied();
        let mut line = self.line;
        let c = b[i];
        // The other blanks `char::is_whitespace` takes among the ASCII
        // bytes, identifiers, numbers, comments, quoted atoms, chars of
        // 0x80 or more, punctuation and operators.
        let tok = match c {
            b'\t' | b'\r' | 0x0b | 0x0c => {
                i += 1;
                None
            }
            b'A'..=b'Z' | b'_' => {
                let start = i;
                i = ident_end(src, i + 1);
                Some(Tok::Var(&src[start..i]))
            }
            b'0'..=b'9' => {
                let start = i;
                let number = number(b, i);
                i = number.end;
                if number.float {
                    let x = match decimal(&number) {
                        Some(x) => x,
                        None => match src[start..i].parse() {
                            Ok(x) => x,
                            Err(_) => {
                                return Some(
                                    self.fail(line, format!("bad number `{}`", &src[start..i])),
                                );
                            }
                        },
                    };
                    Some(Tok::Float(x))
                } else {
                    // Eighteen digits or fewer make an integer that fits.
                    let v = if number.digits <= 18 {
                        Some(number.w as i64)
                    } else {
                        b[start..i].iter().try_fold(0i64, |v, &d| {
                            v.checked_mul(10)?.checked_add(i64::from(d - b'0'))
                        })
                    };
                    match v {
                        Some(v) => Some(Tok::Int(v)),
                        None => {
                            return Some(
                                self.fail(line, format!("bad integer `{}`", &src[start..i])),
                            );
                        }
                    }
                }
            }
            b'%' => {
                while i < n && b[i] != b'\n' {
                    i += 1;
                }
                None
            }
            b'/' if at(i + 1) == Some(b'*') => {
                i += 2;
                loop {
                    if i >= n {
                        return Some(self.fail(line, "unterminated block comment".into()));
                    }
                    if b[i] == b'*' && at(i + 1) == Some(b'/') {
                        i += 2;
                        break;
                    }
                    if b[i] == b'\n' {
                        line += 1;
                    }
                    i += 1;
                }
                None
            }
            b'\'' | b'"' => {
                let quote = c;
                i += 1;
                let mut s = std::mem::take(&mut self.quoted);
                s.clear();
                loop {
                    if i >= n {
                        return Some(self.fail(line, "unterminated quoted atom".into()));
                    }
                    let d = b[i];
                    if d == quote {
                        // doubled quote is an escaped quote
                        if at(i + 1) == Some(quote) {
                            s.push(quote as char);
                            i += 2;
                            continue;
                        }
                        i += 1;
                        break;
                    }
                    if d == b'\\' && i + 1 < n {
                        let (e, len) = char_at(src, i + 1);
                        s.push(match e {
                            'n' => '\n',
                            't' => '\t',
                            other => other,
                        });
                        i += 1 + len;
                        continue;
                    }
                    if d == b'\n' {
                        line += 1;
                    }
                    let (ch, len) = char_at(src, i);
                    s.push(ch);
                    i += len;
                }
                let id = self.place(s.as_bytes());
                self.quoted = s;
                Some(Tok::Name(id))
            }
            0x80.. => {
                let (ch, len) = char_at(src, i);
                if ch.is_whitespace() {
                    i += len;
                    None
                } else if ch.is_alphabetic() {
                    let start = i;
                    i = ident_end(src, i + len);
                    let text = &src[start..i];
                    Some(if ch.is_uppercase() {
                        Tok::Var(text)
                    } else {
                        Tok::Name(self.place(text.as_bytes()))
                    })
                } else {
                    return Some(self.fail(line, format!("unexpected character `{ch}`")));
                }
            }
            // punctuation and operators
            _ => {
                let (tok, len) = match (c, at(i + 1), at(i + 2)) {
                    (b':', Some(b'-'), _) | (b'<', Some(b'-'), _) => (Tok::Neck, 2),
                    (b':', Some(b':'), _) => (Tok::DoubleColon, 2),
                    (b'\\', Some(b'='), Some(b'=')) => (Tok::Op(Op::NotIdentical), 3),
                    (b'\\', Some(b'='), _) => (Tok::Op(Op::Ne), 2),
                    (b'\\', Some(b'+'), _) => (Tok::Naf, 2),
                    (b'=', Some(b'='), _) => (Tok::Op(Op::Identical), 2),
                    (b'=', Some(b'<'), _) => (Tok::Op(Op::Le), 2),
                    (b'>', Some(b'='), _) => (Tok::Op(Op::Ge), 2),
                    (b'=', Some(b':'), Some(b'=')) => (Tok::Op(Op::ArithEq), 3),
                    (b'=', Some(b'\\'), Some(b'=')) => (Tok::Op(Op::ArithNe), 3),
                    (b'@', Some(b'='), Some(b'<')) => (Tok::Op(Op::TermLe), 3),
                    (b'@', Some(b'>'), Some(b'=')) => (Tok::Op(Op::TermGe), 3),
                    (b'@', Some(b'<'), _) => (Tok::Op(Op::TermLt), 2),
                    (b'@', Some(b'>'), _) => (Tok::Op(Op::TermGt), 2),
                    (b'<', Some(b'<'), _) => (Tok::Op(Op::Shl), 2),
                    (b'>', Some(b'>'), _) => (Tok::Op(Op::Shr), 2),
                    (b'/', Some(b'/'), _) => (Tok::Op(Op::IntDiv), 2),
                    (b'/', Some(b'\\'), _) => (Tok::Op(Op::BitAnd), 2),
                    (b'\\', Some(b'/'), _) => (Tok::Op(Op::BitOr), 2),
                    (b'*', Some(b'*'), _) => (Tok::Op(Op::Pow), 2),
                    (b'=', _, _) => (Tok::Op(Op::Eq), 1),
                    (b'<', _, _) => (Tok::Op(Op::Lt), 1),
                    (b'>', _, _) => (Tok::Op(Op::Gt), 1),
                    (b'+', _, _) => (Tok::Op(Op::Plus), 1),
                    (b'-', _, _) => (Tok::Op(Op::Minus), 1),
                    (b'*', _, _) => (Tok::Op(Op::Star), 1),
                    (b'/', _, _) => (Tok::Op(Op::Slash), 1),
                    (b'^', _, _) => (Tok::Op(Op::Caret), 1),
                    (b'\\', _, _) => (Tok::Op(Op::Backslash), 1),
                    (b'[', _, _) => (Tok::LBracket, 1),
                    (b']', _, _) => (Tok::RBracket, 1),
                    (b'|', _, _) => (Tok::Bar, 1),
                    _ => {
                        return Some(
                            self.fail(line, format!("unexpected character `{}`", c as char)),
                        );
                    }
                };
                i += len;
                Some(tok)
            }
        };
        self.i = i;
        self.line = line;
        tok
    }
}

/// A slot of [`Lexer::slots`]: the key ([`text_key`]) and the length of
/// the name it holds, and its place plus 1, 0 for none.
#[derive(Clone, Copy, Default)]
struct Slot {
    key: u64,
    len: u32,
    place: u32,
}

/// A name's key: its first eight bytes, little-endian, zero past its end.
/// A text shorter than eight bytes is its key and its length.
fn text_key(s: &[u8]) -> u64 {
    let mut b = [0; 8];
    let n = s.len().min(8);
    b[..n].copy_from_slice(&s[..n]);
    u64::from_le_bytes(b)
}

const HASH_K: u64 = 0xf135_7aea_2e62_a9c5;

/// The hash of a text shorter than eight bytes, from its key.
fn short_hash(key: u64) -> u64 {
    key.wrapping_mul(HASH_K)
}

/// A hash of a name's text `b` of key `key` ([`Lexer::place`]): of a text
/// shorter than eight bytes, [`short_hash`]; of a longer one, the text
/// read eight bytes at a time, the last eight overlapping the ones before.
fn text_hash(b: &[u8], key: u64) -> u64 {
    let n = b.len();
    if n < 8 {
        return short_hash(key);
    }
    let word = |i: usize| u64::from_le_bytes(b[i..i + 8].try_into().expect("eight bytes"));
    let mix = |h: u64, w: u64| (h.rotate_left(26) ^ w).wrapping_mul(HASH_K);
    let mut h = (n as u64).wrapping_mul(HASH_K);
    let mut i = 0;
    while i + 8 < n {
        h = mix(h, word(i));
        i += 8;
    }
    mix(h, word(n - 8))
}

/// Whether two texts of one length, eight bytes or more, are the same,
/// read eight bytes at a time as [`text_hash`] reads them: `==` on texts
/// calls `memcmp`.
fn same_long_text(a: &[u8], b: &[u8]) -> bool {
    let n = a.len();
    debug_assert!(n >= 8 && b.len() == n);
    let word =
        |b: &[u8], i: usize| u64::from_le_bytes(b[i..i + 8].try_into().expect("eight bytes"));
    let mut i = 0;
    while i + 8 < n {
        if word(a, i) != word(b, i) {
            return false;
        }
        i += 8;
    }
    word(a, n - 8) == word(b, n - 8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_long_text_is_equality() {
        let texts = [
            "abcdefgh",
            "abcdefgi",
            "xbcdefgh",
            "abcdefghi",
            "abcdefghj",
            "abcdefghijklmnopq",
            "abcdefghijklmnopr",
            "abcdefgxijklmnopq",
        ];
        for a in texts {
            for b in texts.iter().filter(|b| b.len() == a.len()) {
                assert_eq!(
                    same_long_text(a.as_bytes(), b.as_bytes()),
                    a == *b,
                    "{a:?} {b:?}"
                );
            }
        }
    }

    #[test]
    fn a_name_keeps_its_place() {
        // Names of every length, read as written and from quotes, and
        // names that end in NUL bytes, read twice over a table that grows:
        // each distinct text at one place, and met again at it.
        let mut texts: Vec<String> = (0..3000).map(|k| format!("n{k}")).collect();
        texts.extend((1..40).map(|n| "abcdefghijklmnopqrstuvwxyz0123456789abcd"[..n].to_owned()));
        texts.extend(
            [
                "x_1",
                "x__",
                "\u{e9}t\u{e9}",
                "a\0",
                "ab\0\0",
                "abcdefg\0",
                "abcdefgh\0",
            ]
            .map(String::from),
        );
        let mut source = String::new();
        for _ in 0..2 {
            for t in &texts {
                source += &format!("'{t}' ");
                if !t.contains('\0') {
                    source += &format!("{t} ");
                }
            }
        }
        let mut lexer = Lexer::new(&source);
        let mut place = std::collections::HashMap::new();
        let mut text = std::collections::HashMap::new();
        loop {
            match lexer.token().tok {
                Tok::Name(id) => {
                    let t = lexer.name(id).to_string();
                    assert_eq!(*place.entry(t.clone()).or_insert(id), id, "{t:?}");
                    assert_eq!(*text.entry(id).or_insert(t.clone()), t, "{id}");
                }
                Tok::Eof => break,
                other => panic!("{other:?}"),
            }
        }
        assert!(lexer.error().is_none());
        assert_eq!(place.len(), texts.len());
    }

    #[test]
    fn exact_decimals_read_as_parse_reads_them() {
        // Whether a number reads in one rounding ([`decimal`]); either way
        // the lexer reads the value `str::parse` does.
        let same = |text: &str| {
            let number = number(text.as_bytes(), 0);
            assert_eq!(number.end, text.len(), "{text}");
            match Lexer::new(text).token().tok {
                Tok::Float(x) => assert_eq!(
                    x.to_bits(),
                    text.parse::<f64>().unwrap().to_bits(),
                    "{text}"
                ),
                Tok::Int(v) => assert_eq!(v, text.parse::<i64>().unwrap(), "{text}"),
                other => panic!("{text} read as {other:?}"),
            }
            decimal(&number).is_some()
        };
        for text in [
            "0.1111111111111111",
            "0.49980008",
            "9.996e-05",
            "0.5",
            "1e22",
            "1.5E+3",
            "0.0",
            "000.000100",
            "9007199254740992",
            "1e00001",
        ] {
            assert!(same(text), "{text}");
        }
        for text in [
            "0.33333333333333337",
            "1e23",
            "9007199254740993.0",
            "1e-23",
            "1e99999",
            "123456789012345678901.5",
            "0.00000000000000000000001",
        ] {
            assert!(!same(text), "{text}");
        }
        // Digits and exponents of every length the path takes, and past it.
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = |m: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % m
        };
        for _ in 0..200_000 {
            let int: String = (0..next(4))
                .map(|_| char::from(b'0' + next(10) as u8))
                .collect();
            let frac: String = (0..next(19))
                .map(|_| char::from(b'0' + next(10) as u8))
                .collect();
            let mut text = if int.is_empty() { "0".to_owned() } else { int };
            if !frac.is_empty() {
                text = format!("{text}.{frac}");
            }
            if next(3) == 0 {
                text = format!("{text}e{}{}", ["", "+", "-"][next(3) as usize], next(30));
            }
            same(&text);
        }
    }

    #[test]
    fn every_kind_of_token_blank_and_comment_on_its_line() {
        // The kinds the scan reads itself and those it leaves to
        // `Lexer::word` and `Lexer::uncommon`, after one another in either
        // order, with the lines counted in both.
        let src = "p(X,'q r'):-\t0.5::a;\r\n% c\n/* d\n e */b, \\+ c>=1 .\n\u{e9}t\u{e9} \u{c9}t\u{e9}\u{a0}\"s\"x";
        let mut lexer = Lexer::new(src);
        let mut toks = Vec::new();
        loop {
            let Spanned { tok, line } = lexer.token();
            toks.push(format!("{line} {}", lexer.show(tok)));
            if tok == Tok::Eof {
                break;
            }
        }
        let want = [
            "1 name `p`",
            "1 `(`",
            "1 variable `X`",
            "1 `,`",
            "1 name `q r`",
            "1 `)`",
            "1 `:-`",
            "1 number `0.5`",
            "1 `::`",
            "1 name `a`",
            "1 `;`",
            "4 name `b`",
            "4 `,`",
            "4 `\\+`",
            "4 name `c`",
            "4 operator `>=`",
            "4 integer `1`",
            "4 `.`",
            "5 name `\u{e9}t\u{e9}`",
            "5 variable `\u{c9}t\u{e9}`",
            "5 name `s`",
            "5 name `x`",
            "5 end of input",
        ];
        assert_eq!(toks, want);
        assert!(lexer.error().is_none());
    }

    #[test]
    fn integers_and_their_overflow() {
        fn tok(src: &str) -> Tok<'_> {
            Lexer::new(src).token().tok
        }
        assert_eq!(tok("9223372036854775807"), Tok::Int(i64::MAX));
        assert_eq!(tok("007"), Tok::Int(7));
        let mut lexer = Lexer::new("9223372036854775808");
        assert_eq!(lexer.token().tok, Tok::Eof);
        assert_eq!(
            lexer.error().unwrap().msg,
            "bad integer `9223372036854775808`"
        );
    }
}
