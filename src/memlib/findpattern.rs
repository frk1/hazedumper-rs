// MIT License
//
// Copyright (c) 2018 frk <hazefrk+dev@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

extern crate regex;
use self::regex::bytes::Regex;

/// Enables the user to generate a byte regex out of the normal signature
/// format.
pub fn generate_regex(raw: &str) -> Option<Regex> {
    let mut res = raw
        .to_string()
        .split_whitespace()
        .map(|x| match &x {
            &"?" => ".".to_string(),
            x => format!("\\x{}", x),
        })
        .collect::<Vec<_>>()
        .join("");
    res.insert_str(0, "(?s-u)");
    Regex::new(&res).ok()
}

/// Find pattern.
pub fn find_pattern(data: &[u8], pattern: &str) -> Option<usize> {
    generate_regex(pattern)
        .and_then(|r| r.find(data))
        .and_then(|m| Some(m.start()))
}
