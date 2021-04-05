use lazy_static::lazy_static;

type T = f32;
const PROB: [T; 5] = [0.0625, 0.25, 0.375, 0.25, 0.0625];
const WIN: T = 1.0;
const LOSE: T = -1.0;
const DRAW: T = 0.0;
const NUM_STATES: usize = 137913936;

lazy_static! {
	static ref DPS: [[[usize;15];8];8] = {
		let mut ans = <[[[usize;15];8];8]>::default();
		ans[0][0][0]=1;
		for i in 0..8 {
			for j in 0..8 {
				for p in 1..15 {
					ans[i][j][p]=ans[i][j][p-1];
					if i!=0 {
						ans[i][j][p]+=ans[i-1][j][p-1];
					}
					if j!=0 {
						ans[i][j][p]+=ans[i][j-1][p-1];
					}
					if p<=4 || p>=13 { // separate part
						if i!=0 && j!=0 {
							ans[i][j][p]+=ans[i-1][j-1][p-1];
						}
					}
				}
			}
		}
		ans
	};
	static ref PC: [[usize;8];8] = {
		let mut ans=<[[usize;8];8]>::default();
		for i0 in 0..8{
			for j0 in 0..8{
				for i in 0..i0 {
					for j in 0..8 {
						ans[i0][j0] += DPS[i][j][14] * (8 - i) * (8 - j);
					}
				}
				for j in 0..j0 {
					ans[i0][j0] += DPS[i0][j][14] * (8 - i0) * (8 - j);
				}
			}
		}
		ans
	};
}

#[derive(Debug)]
struct BitSet {
	arr: [u64; 2154906],
}
impl BitSet {
	fn get(&self, i: usize) -> bool {
		(self.arr[i / 64] >> (i % 64)) & 1 != 0
	}
	fn set(&mut self, i: usize) {
		self.arr[i / 64] |= 1u64 << (i % 64);
	}
}

#[derive(Default, Debug)]
struct State {
	a: u16,
	b: u16,
	sa: u8,
	sb: u8,
}
impl State {
	fn new(a: u16, b: u16, sa: u8, sb: u8) -> Self {
		Self {
			a: a & 0x7ffe,
			b: b & 0x7ffe,
			sa: sa,
			sb: sb,
		}
	}
	fn as_index(&self) -> usize {
		let i0 = self.a.count_ones() as usize;
		let j0 = self.b.count_ones() as usize;
		let mut res = PC[i0][j0];
		res += self.sa as usize * (8 - j0) * DPS[i0][j0][14];
		res += self.sb as usize * DPS[i0][j0][14];
		let mut i = i0;
		let mut j = j0;
		for p in (1..15).rev() {
			if (self.a >> p) & 1 != 0 && (self.b >> p) & 1 != 0 {
				res += DPS[i][j][p - 1];
				res += DPS[i - 1][j][p - 1];
				res += DPS[i][j - 1][p - 1];
				i -= 1;
				j -= 1;
			} else if (self.a >> p) & 1 != 0 {
				res += DPS[i][j][p - 1];
				if j != 0 {
					res += DPS[i][j - 1][p - 1];
				}
				i -= 1;
			} else if (self.b >> p) & 1 != 0 {
				res += DPS[i][j][p - 1];
				j -= 1;
			}
		}
		return res;
	}
}

#[derive(Debug)]
struct Game {
	a: [[u8; 7]; 2],
	turn: bool,
	memo: Vec<T>,
	memod: BitSet,
	cnt: usize,
}
impl Game {
	fn new(turn: bool) -> Self {
		Game {
			a: <[[u8; 7]; 2]>::default(),
			turn: turn,
			memo: vec![DRAW; NUM_STATES],
			memod: BitSet {
				arr: [0u64; 2154906],
			},
			cnt: 0,
		}
	}
	fn score(&self, player: usize) -> usize {
		self.a[player].iter().filter(|x| **x == 15).count()
	}
	fn result(&self) -> u8 {
		if self.score(0) == 7 {
			0
		} else if self.score(1) == 7 {
			1
		} else {
			2
		}
	}
	fn mov(&mut self, p: usize, d: u8) {
		if d == 0 {
			self.turn = !self.turn;
			return;
		}
		self.a[self.turn as usize][p] += d;
		let np = self.a[self.turn as usize][p];
		if np >= 5 && np <= 12 {
			for i in 0..7 {
				if self.a[(!self.turn) as usize][i] == np {
					self.a[(!self.turn) as usize][i] = 0;
				}
			}
		}
		if np != 4 && np != 8 && np != 14 {
			self.turn = !self.turn;
		}
	}
	fn dp(&mut self, s: State) -> T {
		if s.sa == 7 {
			return WIN;
		}
		if s.sb == 7 {
			return LOSE;
		}
		let index = s.as_index();
		if self.memod.get(index) {
			return self.memo[index];
		}
		self.memod.set(index);

		if self.cnt % 1048576 == 0 {
			eprintln!("{}%", self.cnt * 100 / NUM_STATES);
		}
		self.cnt += 1;

		let mut res = DRAW;
		for d in 0..5 {
			let mut best = LOSE;
			let mut found = false;
			if d != 0 {
				let mut ta: u16 = s.a;
				if ta.count_ones() + (s.sa as u32) < 7 {
					ta ^= 1;
				}
				for p in d..16 {
					if (ta >> (p - d)) & 1 != 0
						&& (ta >> p) & 1 == 0
						&& p > 0 && (p != 8 || (s.b >> p) & 1 == 0)
					{
						found = true;
						let t: T = if p == 15 {
							// score
							-self.dp(State::new(s.b, s.a ^ (1 << (p - d)), s.sb, s.sa + 1))
						} else if p >= 5 && p <= 12 && (s.b >> p) & 1 != 0 {
							// eat
							-self.dp(State::new(
								s.b ^ (1 << p),
								s.a ^ (1 << (p - d)) ^ (1 << p),
								s.sb,
								s.sa,
							))
						} else if p == 4 || p == 8 || p == 14 {
							// double turn
							self.dp(State::new(s.a ^ (1 << (p - d)) ^ (1 << p), s.b, s.sa, s.sb))
						} else {
							// normal
							-self.dp(State::new(s.b, s.a ^ (1 << (p - d)) ^ (1 << p), s.sb, s.sa))
						};
						if t > best {
							best = t;
						}
					}
				}
			}
			if !found {
				best = -self.dp(State::new(s.b, s.a, s.sb, s.sa));
			}
			res += PROB[d] * best;
		}
		self.memo[index] = res;
		return res;
	}
	fn dp_mov(&mut self, s: State, d: usize) -> usize {
		if s.sa == 7 {
			panic!();
		}
		if s.sb == 7 {
			panic!();
		}
		let mut ta: u16 = s.a;
		if ta.count_ones() + (s.sa as u32) < 7 {
			ta ^= 1;
		}
		let mut best = LOSE;
		let mut ans: usize = 15;
		for p in d..16 {
			if (ta >> (p - d)) & 1 != 0 && (ta >> p) & 1 == 0 && p > 0 && (p != 8 || (s.b >> p) & 1 == 0)
			{
				let t = if p == 15 {
					-self.dp(State::new(s.b, s.a ^ (1 << (p - d)), s.sb, s.sa + 1))
				} else if p >= 5 && p <= 12 && (s.b >> p) & 1 != 0 {
					-self.dp(State::new(s.b ^ (1 << p), s.a ^ (1 << (p - d)), s.sb, s.sa))
				} else if p == 4 || p == 8 || p == 14 {
					self.dp(State::new(s.a ^ (1 << (p - d)), s.b, s.sa, s.sb))
				} else {
					-self.dp(State::new(s.b, s.a ^ (1 << (p - d)), s.sb, s.sa))
				};
				if t > best {
					best = t;
					ans = p - d;
				}
				eprintln!("confidence for {} = {}", p - d, t);
			}
		}
		return ans;
	}
	fn get_state(&self) -> State {
		let mut ans = State::default();
		for i in 0..7 {
			if self.a[self.turn as usize][i] == 15 {
				ans.sa += 1;
			} else if self.a[self.turn as usize][i] != 0 {
				ans.a |= 1 << self.a[self.turn as usize][i];
			}
			if self.a[(!self.turn) as usize][i] == 15 {
				ans.sb += 1;
			} else if self.a[(!self.turn) as usize][i] != 0 {
				ans.b |= 1 << self.a[(!self.turn) as usize][i];
			}
		}
		return ans;
	}
	fn get_mov(&mut self, d: usize) -> usize {
		let m = self.dp_mov(self.get_state(), d);
		if m == 15 {
			return 7;
		}
		for i in 0..7 {
			if self.a[self.turn as usize][i] as usize == m {
				return i;
			}
		}
		return 8;
	}
}

#[allow(unused_imports)]
use std::io::{stdin, stdout, BufWriter, Write};

#[derive(Default)]
struct Scanner {
	buffer: Vec<String>,
}
impl Scanner {
	fn next<T: std::str::FromStr>(&mut self) -> T {
		loop {
			if let Some(token) = self.buffer.pop() {
				return token.parse().ok().expect("Failed parse");
			}
			let mut input = String::new();
			stdin().read_line(&mut input).expect("Failed read");
			self.buffer = input.split_whitespace().rev().map(String::from).collect();
		}
	}
}

fn main() {
	let mut scan = Scanner::default();
	let out = &mut BufWriter::new(stdout());
	let mut names: [String; 2] = [scan.next::<String>(), scan.next::<String>()];
	let turn = scan.next::<u8>() == 1;
	if turn {
		names.swap(0, 1);
	}
	let mut g = Game::new(turn);
	while g.result() == 2 {
		let d =
			scan.next::<usize>() + scan.next::<usize>() + scan.next::<usize>() + scan.next::<usize>();
		eprintln!("moves for {}:", names[g.turn as usize]);
		let mut m = g.get_mov(d);
		if m != 7 {
			if g.turn {
				m = scan.next();
			} else {
				writeln!(out, "{}", m).unwrap();
				out.flush().unwrap();
			}
			g.mov(m, d as u8);
		} else {
			g.mov(0, 0);
		}
	}
}
