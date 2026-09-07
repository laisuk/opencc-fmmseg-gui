/*
 * cjk_encoding_detector.rs
 *
 * Pure Rust CJK/Unicode encoding detector.
 *
 * Unicode/BOM/UTF-16 detection: project code.
 * Big5/GB18030 statistical fallback: adapted from uchardet 0.0.5 /
 * Mozilla Universal Charset Detector.
 *
 * This module is intentionally dependency-free and platform-independent.
 * It operates only on &[u8], performs no file I/O, and does not depend on
 * Tauri, serde, encoding libraries, or OS APIs.
 *
 * ***** BEGIN LICENSE BLOCK *****
 * Version: MPL 1.1/GPL 2.0/LGPL 2.1
 *
 * The contents of this file derived from Mozilla charset detector code are
 * subject to the Mozilla Public License Version 1.1 (the "License"); you may
 * not use those portions except in compliance with the License. You may obtain
 * a copy of the License at http://www.mozilla.org/MPL/
 *
 * Software distributed under the License is distributed on an "AS IS" basis,
 * WITHOUT WARRANTY OF ANY KIND, either express or implied. See the License
 * for the specific language governing rights and limitations under the License.
 *
 * The Original Code is Mozilla charset detector code.
 *
 * The Initial Developer of the Original Code is
 * Netscape Communications Corporation.
 * Portions created by the Initial Developer are Copyright (C) 1998-2001
 * the Initial Developer. All Rights Reserved.
 *
 * Alternatively, the contents derived from Mozilla code may be used under the
 * terms of either the GNU General Public License Version 2 or later (the
 * "GPL"), or the GNU Lesser General Public License Version 2.1 or later
 * (the "LGPL"), in which case the provisions of the GPL or LGPL are applicable
 * instead of those above.
 *
 * ***** END LICENSE BLOCK *****
 */

const MAX_LEGACY_SAMPLE_SIZE: usize = 128 * 1024;
const MIN_LEGACY_CONFIDENCE: f32 = 0.20;
const MIN_LEGACY_MARGIN: f32 = 0.05;

const STATE_START: usize = 0;
#[allow(dead_code)]
const STATE_ERROR: usize = 1;
const STATE_ITS_ME: usize = 2;

/// Encodings recognized by the detector.
///
/// GB2312 and GBK are intentionally reported as [`EncodingKind::Gb18030`],
/// because GB18030 is the superset decoder expected by callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EncodingKind {
    Unknown,
    Ascii,
    Utf8,
    Utf8Bom,
    Utf16Le,
    Utf16LeBom,
    Utf16Be,
    Utf16BeBom,
    Big5,
    Gb18030,
}

impl EncodingKind {
    /// Stable human-readable encoding name.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Ascii => "ASCII",
            Self::Utf8 => "UTF-8",
            Self::Utf8Bom => "UTF-8 BOM",
            Self::Utf16Le => "UTF-16 LE",
            Self::Utf16LeBom => "UTF-16 LE BOM",
            Self::Utf16Be => "UTF-16 BE",
            Self::Utf16BeBom => "UTF-16 BE BOM",
            Self::Big5 => "Big5",
            Self::Gb18030 => "GB18030",
        }
    }

    #[allow(dead_code)]
    pub const fn is_unicode(self) -> bool {
        matches!(
            self,
            Self::Ascii
                | Self::Utf8
                | Self::Utf8Bom
                | Self::Utf16Le
                | Self::Utf16LeBom
                | Self::Utf16Be
                | Self::Utf16BeBom
        )
    }
}

/// Encoding detection result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetectionResult {
    pub encoding: EncodingKind,
    pub bom_size: usize,
    pub confidence: f32,
}

impl DetectionResult {
    pub const fn new(encoding: EncodingKind, bom_size: usize, confidence: f32) -> Self {
        Self {
            encoding,
            bom_size,
            confidence,
        }
    }

    #[allow(dead_code)]
    pub const fn detected(self) -> bool {
        !matches!(self.encoding, EncodingKind::Unknown)
    }

    #[allow(dead_code)]
    pub const fn has_bom(self) -> bool {
        self.bom_size != 0
    }

    #[allow(dead_code)]
    pub const fn is_unicode(self) -> bool {
        self.encoding.is_unicode()
    }

    #[allow(dead_code)]
    pub const fn encoding_name(self) -> &'static str {
        self.encoding.name()
    }
}

impl core::fmt::Display for DetectionResult {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.encoding.name())
    }
}

/// Detect an encoding from a byte slice.
///
/// Detection order:
/// BOM -> ASCII -> strict UTF-8 -> BOM-less UTF-16 heuristic ->
/// Big5/GB18030 statistical fallback -> Unknown.
///
/// This function allocates nothing and performs no I/O.
pub fn detect(data: &[u8]) -> DetectionResult {
    if data.is_empty() {
        return unknown();
    }

    // 1. BOM.
    if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return DetectionResult::new(EncodingKind::Utf8Bom, 3, 1.0);
    }
    if data.starts_with(&[0xFF, 0xFE]) {
        return DetectionResult::new(EncodingKind::Utf16LeBom, 2, 1.0);
    }
    if data.starts_with(&[0xFE, 0xFF]) {
        return DetectionResult::new(EncodingKind::Utf16BeBom, 2, 1.0);
    }

    // 2. ASCII.
    if is_ascii(data) {
        return DetectionResult::new(EncodingKind::Ascii, 0, 1.0);
    }

    // 3. Strict UTF-8.
    if is_valid_utf8(data) {
        return DetectionResult::new(EncodingKind::Utf8, 0, 1.0);
    }

    // 4. BOM-less UTF-16.
    if looks_like_utf16_le(data) {
        return DetectionResult::new(EncodingKind::Utf16Le, 0, 0.90);
    }
    if looks_like_utf16_be(data) {
        return DetectionResult::new(EncodingKind::Utf16Be, 0, 0.90);
    }

    // 5. Chinese legacy fallback.
    detect_chinese_legacy(data)
}

/// Detect a subrange without allocating a new buffer.
///
/// Returns `None` when `offset/count` is outside `data`.
#[allow(dead_code)]
pub fn detect_range(data: &[u8], offset: usize, count: usize) -> Option<DetectionResult> {
    let end = offset.checked_add(count)?;
    let slice = data.get(offset..end)?;
    Some(detect(slice))
}

const fn unknown() -> DetectionResult {
    DetectionResult::new(EncodingKind::Unknown, 0, 0.0)
}

fn is_ascii(data: &[u8]) -> bool {
    data.iter().all(|&b| b < 0x80)
}

/// Strict UTF-8 validation kept explicit to mirror the C# and C++ detectors.
fn is_valid_utf8(data: &[u8]) -> bool {
    let mut i = 0usize;

    while i < data.len() {
        let c0 = data[i];

        if c0 <= 0x7F {
            i += 1;
            continue;
        }

        if (0xC2..=0xDF).contains(&c0) {
            if i + 1 >= data.len() {
                return false;
            }
            let c1 = data[i + 1];
            if c1 & 0xC0 != 0x80 {
                return false;
            }
            i += 2;
            continue;
        }

        if (0xE0..=0xEF).contains(&c0) {
            if i + 2 >= data.len() {
                return false;
            }

            let c1 = data[i + 1];
            let c2 = data[i + 2];

            if c2 & 0xC0 != 0x80 {
                return false;
            }

            match c0 {
                0xE0 => {
                    if !(0xA0..=0xBF).contains(&c1) {
                        return false;
                    }
                }
                0xED => {
                    // Reject UTF-16 surrogate range U+D800..U+DFFF.
                    if !(0x80..=0x9F).contains(&c1) {
                        return false;
                    }
                }
                _ => {
                    if c1 & 0xC0 != 0x80 {
                        return false;
                    }
                }
            }

            i += 3;
            continue;
        }

        if (0xF0..=0xF4).contains(&c0) {
            if i + 3 >= data.len() {
                return false;
            }

            let c1 = data[i + 1];
            let c2 = data[i + 2];
            let c3 = data[i + 3];

            if c2 & 0xC0 != 0x80 || c3 & 0xC0 != 0x80 {
                return false;
            }

            match c0 {
                0xF0 => {
                    if !(0x90..=0xBF).contains(&c1) {
                        return false;
                    }
                }
                0xF4 => {
                    if !(0x80..=0x8F).contains(&c1) {
                        return false;
                    }
                }
                _ => {
                    if c1 & 0xC0 != 0x80 {
                        return false;
                    }
                }
            }

            i += 4;
            continue;
        }

        return false;
    }

    true
}

fn looks_like_utf16(data: &[u8], zero_byte_index: usize) -> bool {
    if data.len() < 4 || data.len() & 1 != 0 {
        return false;
    }

    let mut zero_bytes = 0usize;
    let mut pairs = 0usize;

    for pair in data.chunks_exact(2) {
        let zero_byte = pair[zero_byte_index];
        let other_byte = pair[1 - zero_byte_index];

        if zero_byte == 0 && other_byte != 0 {
            zero_bytes += 1;
        }

        pairs += 1;
    }

    pairs != 0 && zero_bytes * 100 / pairs >= 60
}

fn looks_like_utf16_le(data: &[u8]) -> bool {
    looks_like_utf16(data, 1)
}

fn looks_like_utf16_be(data: &[u8]) -> bool {
    looks_like_utf16(data, 0)
}

fn detect_chinese_legacy(data: &[u8]) -> DetectionResult {
    let sample_len = data.len().min(MAX_LEGACY_SAMPLE_SIZE);
    let sample = &data[..sample_len];

    let big5_confidence = probe_big5(sample);
    let gb18030_confidence = probe_gb18030(sample);

    let best = big5_confidence.max(gb18030_confidence);
    let second = big5_confidence.min(gb18030_confidence);

    if best < MIN_LEGACY_CONFIDENCE || best - second < MIN_LEGACY_MARGIN {
        return unknown();
    }

    if big5_confidence > gb18030_confidence {
        DetectionResult::new(EncodingKind::Big5, 0, best)
    } else {
        DetectionResult::new(EncodingKind::Gb18030, 0, best)
    }
}

fn probe_big5(data: &[u8]) -> f32 {
    let mut state_machine = CodingStateMachine::new(
        &BIG5_CLASS_TABLE,
        5,
        &BIG5_STATE_TABLE,
        &BIG5_CHAR_LEN_TABLE,
    );

    let mut distribution =
        DistributionAnalysis::new(&BIG5_FREQUENT_BITS, 5376, 0.75, get_big5_order);

    feed_prober(data, &mut state_machine, &mut distribution);
    distribution.confidence()
}

fn probe_gb18030(data: &[u8]) -> f32 {
    let mut state_machine = CodingStateMachine::new(
        &GB18030_CLASS_TABLE,
        7,
        &GB18030_STATE_TABLE,
        &GB18030_CHAR_LEN_TABLE,
    );

    let mut distribution =
        DistributionAnalysis::new(&GB2312_FREQUENT_BITS, 3760, 0.90, get_gb2312_order);

    feed_prober(data, &mut state_machine, &mut distribution);
    distribution.confidence()
}

fn feed_prober(
    data: &[u8],
    state_machine: &mut CodingStateMachine<'_>,
    distribution: &mut DistributionAnalysis<'_>,
) {
    for (i, &byte) in data.iter().enumerate() {
        let state = state_machine.next_state(byte);

        if state == STATE_ITS_ME {
            break;
        }

        if state != STATE_START {
            continue;
        }

        if state_machine.current_char_len == 2 && i > 0 {
            distribution.handle_one_char(data[i - 1], data[i]);
        }
    }
}

type OrderGetter = fn(u8, u8) -> i32;

struct DistributionAnalysis<'a> {
    frequent_bits: &'a [u32],
    table_size: usize,
    typical_distribution_ratio: f32,
    get_order: OrderGetter,
    total_chars: usize,
    freq_chars: usize,
}

impl<'a> DistributionAnalysis<'a> {
    const MINIMUM_DATA_THRESHOLD: usize = 4;

    fn new(
        frequent_bits: &'a [u32],
        table_size: usize,
        typical_distribution_ratio: f32,
        get_order: OrderGetter,
    ) -> Self {
        Self {
            frequent_bits,
            table_size,
            typical_distribution_ratio,
            get_order,
            total_chars: 0,
            freq_chars: 0,
        }
    }

    fn handle_one_char(&mut self, first: u8, second: u8) {
        let order = (self.get_order)(first, second);
        if order < 0 {
            return;
        }

        self.total_chars += 1;
        let order = order as usize;

        if order < self.table_size && is_frequent(self.frequent_bits, order) {
            self.freq_chars += 1;
        }
    }

    fn confidence(&self) -> f32 {
        if self.total_chars == 0 || self.freq_chars <= Self::MINIMUM_DATA_THRESHOLD {
            return 0.01;
        }

        if self.total_chars == self.freq_chars {
            return 0.99;
        }

        let ratio = self.freq_chars as f32
            / ((self.total_chars - self.freq_chars) as f32 * self.typical_distribution_ratio);

        ratio.min(0.99)
    }
}

struct CodingStateMachine<'a> {
    class_table: &'a [u8],
    class_factor: usize,
    state_table: &'a [u8],
    char_len_table: &'a [u8],
    current_state: usize,
    current_char_len: usize,
}

impl<'a> CodingStateMachine<'a> {
    fn new(
        class_table: &'a [u8],
        class_factor: usize,
        state_table: &'a [u8],
        char_len_table: &'a [u8],
    ) -> Self {
        Self {
            class_table,
            class_factor,
            state_table,
            char_len_table,
            current_state: STATE_START,
            current_char_len: 0,
        }
    }

    fn next_state(&mut self, value: u8) -> usize {
        let byte_class = self.class_table[value as usize] as usize;

        if self.current_state == STATE_START {
            self.current_char_len = self.char_len_table[byte_class] as usize;
        }

        self.current_state =
            self.state_table[self.current_state * self.class_factor + byte_class] as usize;

        self.current_state
    }
}

fn get_gb2312_order(first: u8, second: u8) -> i32 {
    if first >= 0xB0 && second >= 0xA1 {
        94 * (first as i32 - 0xB0) + second as i32 - 0xA1
    } else {
        -1
    }
}

fn get_big5_order(first: u8, second: u8) -> i32 {
    if first < 0xA4 {
        return -1;
    }

    if second >= 0xA1 {
        157 * (first as i32 - 0xA4) + second as i32 - 0xA1 + 63
    } else {
        157 * (first as i32 - 0xA4) + second as i32 - 0x40
    }
}

fn is_frequent(bits: &[u32], order: usize) -> bool {
    (bits[order >> 5] & (1u32 << (order & 31))) != 0
}

const BIG5_CLASS_TABLE: [u8; 256] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1,
    4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
    4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0,
];

const BIG5_STATE_TABLE: [u8; 24] = [
    1, 0, 0, 3, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0,
];

const BIG5_CHAR_LEN_TABLE: [u8; 5] = [0, 1, 1, 2, 0];

const GB18030_CLASS_TABLE: [u8; 256] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 1, 1, 1, 1, 1, 1,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 4,
    5, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
    6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 0,
];

const GB18030_STATE_TABLE: [u8; 48] = [
    1, 0, 0, 0, 0, 0, 3, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 1, 1, 0, 4, 1, 0, 0, 1, 1, 1, 1,
    1, 1, 5, 1, 1, 1, 2, 1, 1, 1, 0, 0, 0, 0, 0, 0,
];

const GB18030_CHAR_LEN_TABLE: [u8; 7] = [0, 1, 1, 1, 1, 1, 2];

const BIG5_FREQUENT_BITS: [u32; 168] = [
    0x20BA8DE9, 0x40E91D50, 0x4B4C4826, 0x3012B810, 0xA08482CD, 0x17200A22, 0x2A264062, 0x14C0A142,
    0x04891300, 0x0F2CC002, 0x198F8400, 0x03022704, 0x00070831, 0x44180E04, 0x05A00831, 0x00011142,
    0x00000001, 0x00044000, 0x00888808, 0x01C00511, 0x80000082, 0xA0A10180, 0x07201001, 0x4074002A,
    0x40005021, 0x20400008, 0x00000100, 0xA2000102, 0x00000002, 0x04308225, 0x02000000, 0x00800010,
    0x21021640, 0x22002000, 0x04488038, 0x06090000, 0x00200200, 0x00022000, 0x00000423, 0x01804024,
    0x02001000, 0x00400082, 0x00008000, 0x8004020A, 0x00000004, 0x20280008, 0x6D104000, 0x00C08000,
    0x40008000, 0x00000000, 0x01000888, 0x00000000, 0x00A20250, 0x11010040, 0x00040000, 0x00100020,
    0x00200100, 0x20000000, 0x00000020, 0x08010800, 0x21010000, 0x00008014, 0x00802A14, 0x00001010,
    0x060000C0, 0x0108050C, 0x20801000, 0x20000080, 0x00008000, 0x00400008, 0x00839000, 0x01010180,
    0x00000204, 0x00200000, 0x00018412, 0x001410D4, 0x20800003, 0x00810002, 0x40000240, 0x00000100,
    0x00020000, 0x0000A008, 0x00000000, 0x06000000, 0x10040000, 0x20010100, 0x00000010, 0x00000048,
    0x12040008, 0x08080000, 0xA1000280, 0x00008000, 0x00000010, 0x03A00000, 0x04000000, 0x00000028,
    0x01001000, 0x00040000, 0x02000000, 0x00200810, 0x08000020, 0x40A86000, 0x20401000, 0x000020B8,
    0x01040000, 0x00040000, 0x00026000, 0x00004200, 0x00000000, 0x20040000, 0x00000000, 0x04100000,
    0x00010080, 0x00002C00, 0x04404000, 0x00012000, 0x00480000, 0x00040800, 0x00008000, 0x0000D800,
    0x10000000, 0x00001042, 0x00001000, 0x00000000, 0x00000400, 0x00000000, 0x08000508, 0x00000000,
    0x00000000, 0x00000000, 0x02800200, 0x00080410, 0x00000080, 0x00000000, 0x00000000, 0x01000000,
    0x12000000, 0x00000080, 0x00000020, 0x20000000, 0x00000000, 0x50020008, 0x00000000, 0x02800200,
    0x00000020, 0x00010000, 0x00000001, 0x00008000, 0x00000000, 0x00000000, 0x00000000, 0x00000210,
    0x00000080, 0x00000000, 0x00000000, 0x00080000, 0x82800020, 0x00000000, 0x00000000, 0x000A8000,
    0x00010000, 0x00010000, 0x00000000, 0x00180000, 0x00000001, 0x40800000, 0x01000010, 0x00220000,
];

const GB2312_FREQUENT_BITS: [u32; 118] = [
    0x008A0000, 0x01010000, 0x08000800, 0x09204021, 0x00200020, 0x20002482, 0x05C00000, 0x04000213,
    0x34200010, 0x00004001, 0x00008025, 0x20008000, 0x00000804, 0x10000424, 0x04028640, 0x23A60141,
    0x1100A000, 0x48001000, 0x08008004, 0x000800C0, 0x020A0800, 0x01414020, 0x01004084, 0x20008000,
    0x810C0601, 0x40204400, 0x814A09E1, 0x00400040, 0x00098320, 0x00004580, 0x47004000, 0x40000080,
    0x038B2000, 0x00000005, 0x4C800402, 0xA0C4004C, 0x0612640A, 0x00000904, 0x00052024, 0x81120001,
    0x286010C0, 0x00415002, 0x12010004, 0x92000005, 0x00200800, 0x08005480, 0x0080A000, 0x00081000,
    0x80005000, 0x86007000, 0x14000288, 0x00083100, 0x00100200, 0x00040000, 0x00400030, 0x00000100,
    0x43103000, 0x80010001, 0x04110400, 0x40400000, 0x000A0140, 0x40000002, 0x00000008, 0x00000000,
    0x00008000, 0x00010400, 0x00802000, 0x00000848, 0x00010002, 0x04420000, 0x04910210, 0x64800640,
    0x04400010, 0x01001800, 0x12000400, 0x4C300040, 0x56121080, 0x48062F97, 0x001000C7, 0x42800101,
    0x000020A4, 0x00000004, 0x80008074, 0x80000000, 0x10081300, 0x19022000, 0x00000C40, 0x04808080,
    0x40042001, 0x00202180, 0x04140102, 0x00410008, 0x82800208, 0x18189103, 0x00001145, 0x0008B41A,
    0x40801080, 0x00080010, 0x00003010, 0x00600140, 0x01412020, 0x20259000, 0x81082013, 0x50000208,
    0x00088284, 0x00400010, 0x021A0912, 0x00030004, 0x00002288, 0x04064000, 0x08900000, 0x248D4080,
    0x2212491E, 0x0001AA0A, 0x0A440400, 0x08402002, 0x84412030, 0x00000180,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_ascii() {
        let result = detect(b"plain ASCII text");
        assert_eq!(result.encoding, EncodingKind::Ascii);
        assert_eq!(result.confidence, 1.0);
    }

    #[test]
    fn detects_utf8_bom() {
        let result = detect(&[0xEF, 0xBB, 0xBF, 0xE4, 0xB8, 0xAD]);
        assert_eq!(result.encoding, EncodingKind::Utf8Bom);
        assert_eq!(result.bom_size, 3);
    }

    #[test]
    fn detects_utf8_without_bom() {
        let result = detect("繁體中文".as_bytes());
        assert_eq!(result.encoding, EncodingKind::Utf8);
        assert_eq!(result.bom_size, 0);
    }

    #[test]
    fn detects_utf16le_without_bom() {
        // Repeated U+00E9 (é). Using plain ASCII letters here would also be
        // valid UTF-8/ASCII bytes because NUL is an ASCII byte, and detection
        // intentionally checks ASCII/UTF-8 before the BOM-less UTF-16 heuristic.
        let data = [0xE9, 0x00, 0xE9, 0x00, 0xE9, 0x00, 0xE9, 0x00];
        assert_eq!(detect(&data).encoding, EncodingKind::Utf16Le);
    }

    #[test]
    fn detects_utf16be_without_bom() {
        // Repeated U+00E9 (é); see the LE test above for why ASCII letters
        // are not a suitable fixture for this detector's ordering.
        let data = [0x00, 0xE9, 0x00, 0xE9, 0x00, 0xE9, 0x00, 0xE9];
        assert_eq!(detect(&data).encoding, EncodingKind::Utf16Be);
    }

    #[test]
    fn detects_big5() {
        let data = [
            0xC1, 0x63, 0xC5, 0xE9, 0xA4, 0xA4, 0xA4, 0xE5, 0xB4, 0xFA, 0xB8, 0xD5, 0xA1, 0x41,
            0xB3, 0x6F, 0xAC, 0x4F, 0xA4, 0x40, 0xAC, 0x71, 0xA5, 0xCE, 0xA8, 0xD3, 0xC0, 0xCB,
            0xAC, 0x64, 0xBD, 0x73, 0xBD, 0x58, 0xB0, 0xBB, 0xB4, 0xFA, 0xAA, 0xBA, 0xA4, 0xE5,
            0xA6, 0x72, 0xA1, 0x43, 0xA4, 0xA4, 0xA4, 0xE5, 0xA6, 0x72, 0xB2, 0xC5, 0xC0, 0xB3,
            0xB8, 0xD3, 0xA8, 0xAC, 0xB0, 0xF7, 0xA6, 0x68, 0xA1, 0x43,
        ];
        let result = detect(&data);
        assert_eq!(result.encoding, EncodingKind::Big5);
        assert!(result.confidence >= MIN_LEGACY_CONFIDENCE);
    }

    #[test]
    fn detects_gb_family_as_gb18030() {
        let data = [
            0xBC, 0xF2, 0xCC, 0xE5, 0xD6, 0xD0, 0xCE, 0xC4, 0xB2, 0xE2, 0xCA, 0xD4, 0xA3, 0xAC,
            0xD5, 0xE2, 0xCA, 0xC7, 0xD2, 0xBB, 0xB6, 0xCE, 0xD3, 0xC3, 0xC0, 0xB4, 0xBC, 0xEC,
            0xB2, 0xE9, 0xB1, 0xE0, 0xC2, 0xEB, 0xBC, 0xEC, 0xB2, 0xE2, 0xB5, 0xC4, 0xCE, 0xC4,
            0xD7, 0xD6, 0xA1, 0xA3, 0xD6, 0xD0, 0xCE, 0xC4, 0xD7, 0xD6, 0xB7, 0xFB, 0xD3, 0xA6,
            0xB8, 0xC3, 0xD7, 0xE3, 0xB9, 0xBB, 0xB6, 0xE0, 0xA1, 0xA3,
        ];
        let result = detect(&data);
        assert_eq!(result.encoding, EncodingKind::Gb18030);
        assert!(result.confidence >= MIN_LEGACY_CONFIDENCE);
    }

    #[test]
    fn shift_jis_is_not_forced_into_chinese_legacy_encoding() {
        let data = [
            0x93, 0xFA, 0x96, 0x7B, 0x8C, 0xEA, 0x82, 0xCC, 0x95, 0xB6, 0x8F, 0xCD, 0x82, 0xC5,
            0x82, 0xB7, 0x81, 0x42, 0x95, 0xB6, 0x8E, 0x9A, 0x83, 0x52, 0x81, 0x5B, 0x83, 0x68,
            0x82, 0xCC, 0x8C, 0x9F, 0x8F, 0x6F, 0x83, 0x65, 0x83, 0x58, 0x83, 0x67, 0x82, 0xF0,
            0x8D, 0x73, 0x82, 0xA2, 0x82, 0xDC, 0x82, 0xB7, 0x81, 0x42, 0x82, 0xB1, 0x82, 0xEA,
            0x82, 0xCD, 0x53, 0x68, 0x69, 0x66, 0x74, 0x2D, 0x4A, 0x49, 0x53, 0x82, 0xC5, 0x82,
            0xB7, 0x81, 0x42,
        ];
        assert_eq!(detect(&data).encoding, EncodingKind::Unknown);
    }

    #[test]
    fn invalid_utf8_is_rejected() {
        // Overlong encoding.
        assert!(!is_valid_utf8(&[0xC0, 0xAF]));
        // UTF-16 surrogate U+D800 encoded as UTF-8.
        assert!(!is_valid_utf8(&[0xED, 0xA0, 0x80]));
        // > U+10FFFF.
        assert!(!is_valid_utf8(&[0xF4, 0x90, 0x80, 0x80]));
    }

    #[test]
    fn range_detection_is_allocation_free_and_checked() {
        let data = b"xx\xEF\xBB\xBFabc";
        let result = detect_range(data, 2, 6).unwrap();
        assert_eq!(result.encoding, EncodingKind::Utf8Bom);
        assert!(detect_range(data, usize::MAX, 1).is_none());
    }
}
