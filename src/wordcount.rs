use std::cmp::Reverse;
use std::fs::File;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::io::{BufWriter, Write};
use std::marker::PhantomData;


pub struct WordRef<'a> {
    start: *const u8,
    lenhash: u64,
    _data: PhantomData<&'a [u8]>,
}

fn fnv32(data: &[u8]) -> u32 {
    let mut hash = 0x811c9dc5u32;
    for &b in data {
        unsafe {
            core::intrinsics::assume(b'a' <= b && b <= b'z');
        }

        hash = hash.wrapping_mul(0x01000193) ^ (b as u32)
    }
    hash
}

impl<'a> WordRef<'a> {
    // precondition: word contains only lowercase english characters
    unsafe fn from_ascii_word(word: &[u8]) -> Self {
        let hash = (fnv32(word) as u64) << 32;
        WordRef {
            start: word.as_ptr(),
            lenhash: word.len() as u64 | hash,
            _data: PhantomData,
        }
    }

    fn as_slice(&self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts(self.start, (self.lenhash & 0xffffffff) as usize) }
    }
}

unsafe fn fast_eq(mut left: *const u8, mut right: *const u8, mut len: u32) -> bool {
    if len < 6 {
        // there are no fnv32 hash collisions for [a-z] strings with equal and len < 6
        return true;
    }

    let nwords = len / 4;
    for i in 0..nwords {
        let l = core::ptr::read_unaligned((left as *const u32).add(i as usize));
        let r = core::ptr::read_unaligned((right as *const u32).add(i as usize));

        // if hash matches it is unlikely that strings are not the same
        if core::intrinsics::unlikely(l != r) {
            return false;
        }
    }

    let n = nwords * 4;
    len -= n;
    left = left.add(n as usize);
    right = right.add(n as usize);
    core::intrinsics::assume(len < 4);

    for i in 0..len {
        // if hash matches it is unlikely that strings are not the same
        if core::intrinsics::unlikely(*left.add(i as usize) != *right.add(i as usize)) {
            return false;
        }
    }

    return true;
}

impl PartialEq for WordRef<'_> {
    fn eq(&self, other: &WordRef) -> bool {
        self.lenhash == other.lenhash
            && unsafe { fast_eq(self.start, other.start, (self.lenhash & 0xffffffff) as u32) }
    }
}

impl Eq for WordRef<'_> {}

impl Hash for WordRef<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let hashval = self.lenhash & 0xffffffff00000000;
        state.write_u64(hashval | hashval >> 32);
    }
}

#[derive(Default)]
struct IdHasher {
    value: u64,
}

impl Hasher for IdHasher {
    fn write(&mut self, _bytes: &[u8]) {
        // never used
        todo!()
    }

    fn write_u64(&mut self, value: u64) {
        self.value = value;
    }

    fn finish(&self) -> u64 {
        self.value
    }
}

fn is_alpha(byte: u8) -> bool {
    (b'a' <= byte && byte <= b'z') || (b'A' <= byte && byte <= b'Z')
}

// NOTE: predondition is_alhpa(byte)
fn to_lower(byte: u8) -> u8 {
    byte | (1 << 5)
}

fn wordcount(text: &mut [u8]) -> Vec<(&[u8], u32)> {
    let mut words =
        std::collections::HashMap::with_hasher(BuildHasherDefault::<IdHasher>::default());

    unsafe {
        let mut start = text.as_mut_ptr();
        let eof = text.as_mut_ptr().add(text.len());

        while start != eof && !is_alpha(*start) {
            *start = to_lower(*start);
            start = start.add(1);
        }

        let mut end = start;
        loop {
            if start == eof {
                break;
            }

            while end != eof && is_alpha(*end) {
                *end = to_lower(*end);
                end = end.add(1);
            }

            let word = std::slice::from_raw_parts(start, end as usize - start as usize);
            let word = WordRef::from_ascii_word(word);
            *words.entry(word).or_insert(0u32) += 1;

            while end != eof && !is_alpha(*end) {
                end = end.add(1);
            }

            start = end;
        }
    }

    let mut counts: Vec<_> = words
        .into_iter()
        .map(|(word, freq)| (word.as_slice(), freq))
        .collect();
    counts.sort_unstable_by_key(|&(word, freq)| (Reverse(freq), word));
    counts
}

pub fn run(input: &str, output: &str) -> std::io::Result<()> {
    let input = File::open(&input)?;
    let output = File::create(&output)?;

    let mut map = unsafe { memmap::MmapOptions::new().map_copy(&input)? };
    let counts = wordcount(&mut map[..]);

    let mut out = BufWriter::new(output);
    for (word, freq) in counts {
        let w = unsafe { std::str::from_utf8_unchecked(word) };
        writeln!(&mut out, "{} {}", freq, w)?;
    }

    Ok(())
}