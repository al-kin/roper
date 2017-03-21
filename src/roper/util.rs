#[allow(dead_code)]

use roper::phylostructs::*;
use rand::*;
use std::cmp::*;
use capstone::*;


fn cs_insn_to_string (insn: &Insn) -> String {
  format!("{} {}", insn.mnemonic()
                       .unwrap_or("?"), 
                   insn.op_str()
                       .unwrap_or("?")) 
}

pub fn disas (insts: &Vec<u8>, mode: MachineMode) -> String {
  let cs : Capstone = 
    Capstone::new(CsArch::ARCH_ARM, mode.cs()).unwrap();
  let dissed : Vec<String> = 
    match cs.disasm(insts, 0, 0) {
      Some(s)  => s.iter().map(|x| cs_insn_to_string(&x)).collect(),
      _        => vec!["[?]".to_string()]
    };
  dissed.join("; ")      
}

pub fn disas32 (inst: u32, mode: MachineMode) -> String {
  let v8 = pack_word32le(inst);
  disas(&v8, mode)
}

pub fn get_word32le (a: &Vec<u8>, offset: usize) -> u32 {
  let mut s : u32 = 0;
 // print!("get_word32le called with offset = {:08x}", offset);
  for i in 0..4 {
    s |= (a[i+offset] as u32) << (i*8);
  }
  //println!(" -> {:08x}", s);
  s
}

pub fn pack_word32le (n: u32) -> Vec<u8> {
  let mut v = Vec::new();
  for i in 0..4 {
    v.push(((n & (0xFF << (i*8))) >> (i*8)) as u8);
  }
  //println!("## {:08x} --> {:02x} {:02x} {:02x} {:02x}",
  //         n, v[0], v[1], v[2], v[3]);
  v
}   

pub fn pack_word32le_vec (v: &Vec<u32>) -> Vec<u8> {
  let mut p : Vec<u8> = Vec::new();
  for ref w in v {
   // println!("## p.len() == {}",p.len());
    p.extend(pack_word32le(**w).iter())
  }
  p
}


pub fn get_word16le (a: &Vec<u8>, offset: usize) -> u16 {
  let mut s : u16 = 0;
  for i in offset..(offset+2) {
    s |= (a[i] as u16) << (i*8);
  }
  s
}

// pretty-print the contents of a vector in hex
pub fn hexvec (v: &Vec<i32>) -> String{
  let vs : Vec<String> = v.iter()
                          .map(|x| format!("{:08x}",x))
                          .collect();
  vs.join(" ")
}

// can be used as part of a crude fitness function
pub fn distance2 (x: &Vec<i32>, y: &Vec<i32>) -> i32 {
  assert_eq!(x.len(), y.len());
  let n = x.len();
  ((0..n).map(|i| {
    let xx: i64;
    let yy: i64;
    if x[i] < y[i] { 
      xx = x[i] as i64;
      yy = y[i] as i64;
    } else { 
      xx = y[i] as i64;
      yy = x[i] as i64;
    };
    min(0xFFFF, (xx - yy).abs())
  }).sum::<i64>() & 0xEFFFFFFF) as i32
}

pub fn hamming_distance (x: &Vec<i32>, y: &Vec<i32>) -> f32 {
  assert_eq!(x.len(), y.len());
  let n = x.len();
  (0..n).map(|i| ((x[i] ^ y[i]).count_ones() as f32 / 16.0).tanh())
         .sum::<f32>() / n as f32
}

pub fn arith_distance (x: &Vec<i32>, y: &Vec<i32>) -> f32 {
  assert_eq!(x.len(), y.len());
  let n = x.len();
  (0..n).map(|i| ((x[i].abs() - y[i].abs()) as f32 
                  / 4096 as f32).abs().tanh())
         .sum::<f32>() / n as f32
}

pub trait Indexable <T: PartialEq> {
  fn index_of (&self, t: T) -> usize;
  fn index_opt (&self, t: T) -> Option<usize>;
}

impl <T: PartialEq> Indexable <T> for Vec<T> {
  fn index_of (&self, t: T) -> usize {
    self.index_opt(t).unwrap() 
  }
  fn index_opt (&self, t: T) -> Option<usize> {
    self.iter().position(|x| x == &t)
  }
}

pub fn u8s_to_u16s (bytes: &Vec<u8>, endian: Endian) -> Vec<u16> {
  let be = if endian == Endian::BIG {8} else {0};
  let le = if endian == Endian::LITTLE {8} else {0};
  let l = bytes.len();
  let mut i = 0;
  let mut out = Vec::new();
  while i < l {
    out.push(((bytes[i] as u16) << be) | ((bytes[i+1] as u16) << le));
    i += 2;
  }
  out
}

pub fn u8s_to_u32s (bytes: &Vec<u8>, endian: Endian) -> Vec<u32> {
  let getter = get_word32le; // check endian and use be version too
  let l = bytes.len();
  let mut i = 0;
  let step = 4;
  let mut out = Vec::new();
  while i < l {
    out.push(getter(bytes, i));
    i += step;
  }
  out
}

pub fn deref_mang (x: u32, 
                   data: &Vec<u32>, 
                   offset: u32) -> u32 {
  match data.index_opt(x) {
    Some(p) => (p as u32 * 4 as u32) + offset,
    None    => x,
  }
}

pub fn mang (ux: u32, rng: &mut ThreadRng) -> u32 {
  let x = ux as i32;
  let die : u8 = rng.gen::<u8>() % 40;
  let r = match die {
    0  => x << 1,
    1  => x << 2,
    3  => x << 4,
    4  => x.rotate_right(8),
    5  => x & 0xFF,
    6  => x & 0xFFFF0000,
    7  => x & 0x0000FFFF,
    8  => !x,
    9  => x + 1,
    10 => x + 2,
    11 => x + 4,
    12 => x + 8,
    13 => x - 1,
    14 => x - 2,
    15 => x - 4,
    16 => x - 8,
    17 => x >> 1,
    18 => x >> 2,
    19 => x >> 4,
    20 => rng.gen::<i32>(),
    21 => x ^ (1 << (rng.gen::<usize>() % 32)), // random bit flip
    22 => x ^ (1 << (rng.gen::<usize>() % 32)), // random bit flip
    23 => x ^ (1 << (rng.gen::<usize>() % 32)), // random bit flip
    24 => x ^ (1 << (rng.gen::<usize>() % 32)), // random bit flip
    25 => x ^ (1 << (rng.gen::<usize>() % 32)), // random bit flip
    26 => x ^ (1 << (rng.gen::<usize>() % 32)), // random bit flip
    27 => x ^ (1 << (rng.gen::<usize>() % 32)), // random bit flip
    28 => x ^ (1 << (rng.gen::<usize>() % 32)), // random bit flip
    29 => x ^ (1 << (rng.gen::<usize>() % 32)), // random bit flip
    30 => x ^ (1 << (rng.gen::<usize>() % 32)), // random bit flip
    _  => x,
  };
  r as u32
}

pub struct Mangler {
  pub words: Vec<u32>,
  pub rng:   ThreadRng,
  cursor:    usize,
}

impl Mangler {
  pub fn new (ws: &Vec<u32>) -> Mangler {
    Mangler {
      words  : ws.clone(),
      rng    : thread_rng(),
      cursor : 0,
    }
  }
}

impl Iterator for Mangler {
  type Item = u32;
  fn next(&mut self) -> Option<u32> {
    /*Some(mang(self.words[self.rng.gen::<usize>() % 
              self.words.len()], &mut self.rng))
              */
    Some(self.words[self.rng.gen::<usize>() % self.words.len()]) 
  }
}

/*
pub fn ranked_ballot (bins: &Vec<i32>, correct: usize) -> f32 {
  let s : f32 = 
    bins.iter()
        .enumerate()
        .collect::<(usize,&i32)>()
        .sort_by_key(|p| p.1) // reverse it tho
        .iter()
        .position(|&x| x.0 == correct) as f32 
      / bins.len() as f32;
  if s == bins.len() as f32 - 1.0 / bins.len() as f32 {
    1.0
  } else {
    s
  }
}
*/

pub fn max_bin (bins: &Vec<i32>) -> usize {
  let mut mb : usize = 0;
  let mut mx : i32 = bins[0];
  for i in 0..bins.len() {
    if bins[i] > mx { 
      mx = bins[i];
      mb = i;
    }
  }
//  println!(">> in max_bin(), mb = {}", mb);
  mb
}

