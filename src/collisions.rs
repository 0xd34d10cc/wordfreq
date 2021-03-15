#![feature(core_intrinsics)]

struct U32Bitmask(Vec<u8>);

impl Default for U32Bitmask {
    fn default() -> Self {
        U32Bitmask(vec![0u8; 1 << (32 - 3)])
    }
}

impl U32Bitmask {
    fn set(&mut self, i: u32) {
        let idx = i / 8;
        let offset = i % 8;
        self.0[idx as usize] |= (1 << offset);
    }

    fn get(&self, i: u32) -> bool {
        let idx = i / 8;
        let offset = i % 8;
        (self.0[idx as usize] & (1 << offset)) != 0
    }
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

const MAX_LEN: usize = 8;

type Word = [u8; MAX_LEN];

const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

fn to_word(mut data: u64) -> Word {
    let mut word = [0; MAX_LEN];
    let mut i = 0;
    while data > 0 {
        let c = data % ALPHABET.len() as u64;
        data /= ALPHABET.len() as u64;
        word[i] = ALPHABET[c as usize];
        i += 1;
    }

    word
}

fn check_len(len: usize) -> usize {
    let mut n = 0;

    let mut hashes = U32Bitmask::default();
    let last_word = (ALPHABET.len() as u64).pow(len as u32);
    let first_word = (ALPHABET.len() as u64).pow(len as u32 - 1);
    for i in first_word..last_word {
        let word = to_word(i);
        let hash = fnv32(&word[..len]);

        // if len == 6 && hash == 0x68410b61 {
        //     panic!("Found a collision at {} = 0x{:x}, word = {:?}", len, hash,
        //         std::str::from_utf8(&word[..len]).unwrap());
        // }

        if hashes.get(hash) {
            n += 1;
            // panic!("Found a collision at {} = 0x{:x}, word = {:?}", len, hash,
            // std::str::from_utf8(&word[..len]).unwrap());
        }

        hashes.set(hash);
    }

    n
}

fn main() {
    println!("{:x} {:x}", fnv32(b"uyaaac"), fnv32(b"nthykb"));

    for len in 1..MAX_LEN {
        eprintln!("Checking len {}", len);
        eprintln!("{} collisions found", check_len(len));
    }
}