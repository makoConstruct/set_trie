
#![feature(nll, test)]

use std::slice::Iter as SliceIter;


pub struct Node<C, V> {
	children: Vec<(C, Node<C, V>)>,
	terminals: Vec<V>,
}
pub struct SetTrie<C, V> {
	pub root: Node<C, V>,
}


#[derive(Copy, Clone)]
pub struct DefinitelySorted<'a, C>(&'a [C]) where C:'a;

impl<'a, C> DefinitelySorted<'a, C> where C: Ord {
	pub fn new(v:&'a mut [C])-> DefinitelySorted<'a, C> {
		v.sort_unstable();
		DefinitelySorted(v)
	}
	pub unsafe fn hasty_new(v:&'a [C])-> DefinitelySorted<'a, C> { //v *must* be sorted
		DefinitelySorted(v)
	}
}
pub fn make_sorted<'a, C>(v:&'a mut [C])-> DefinitelySorted<'a, C> where C:Ord {
	DefinitelySorted::new(v)
}
pub fn assert_sorted<'a, C>(v:&'a [C])-> DefinitelySorted<'a, C> where C:Ord {
	if v.windows(2).all(|w| w[0] < w[1]) {
		unsafe{ DefinitelySorted::hasty_new(v) }
	}else{
		panic!("assertion failed: this thing isn't sorted")
	}
}



impl<C, V> Node<C, V> where
	C: Ord + Clone,
	V: PartialEq,
{
	fn insert_rec(&mut self, mut ki:SliceIter<C>, v:V) {
		if let Some(ref k) = ki.next() {
			match self.children.binary_search_by_key(k, |ok| &ok.0 ) {
				Ok(ia)=> {
					self.children[ia].1.insert_rec(ki, v);
				}
				Err(ia)=> {
					// insert, create a node chain
					let mut rki = ki.rev();
					let mut node_chain:Node<C,V> = Node{ children:vec!(), terminals:vec!(v) };
					while let Some(sk) = rki.next() {
						node_chain = Node{ children:vec!((sk.clone(), node_chain)), terminals:vec!() };
					}
					self.children.insert(ia, ((*k).clone(), node_chain));
				}
			}
		}else{
			self.terminals.push(v);
		}
	}
	
	fn remove_rec(&mut self, mut ki:SliceIter<C>, v:&V)-> Option<V> {
		if let Some(ref k) = ki.next() {
			match self.children.binary_search_by_key(k, |cp| &cp.0) {
				Ok(i)=> {
					let np = &mut self.children[i];
					let ret = np.1.remove_rec(ki, v);
					if np.1.children.is_empty() && np.1.terminals.is_empty() {
						self.children.remove(i);
					}
					ret
				},
				Err(_)=> {
					None
				},
			}
		}else{
			//found
			if let Some((i, _)) = self.terminals.iter().enumerate().find(|p| p.1 == v) {
				Some(self.terminals.remove(i))
			}else{
				None
			}
		}
	}
	
	fn contains_rec(&self, mut ki:SliceIter<C>, v:&V)-> bool {
		if let Some(k) = ki.next() {
			match self.children.binary_search_by_key(&k, |cp| &cp.0) {
				Ok(i)=> {
					let np = &self.children[i];
					np.1.contains_rec(ki, v)
				}
				Err(_)=> {
					false
				}
			}
		}else{
			return self.terminals.iter().find(|p| *p == v).is_some();
		}
	}
	
	fn report_subsets<'a>(&'a self, mut ki:SliceIter<'a, C>, out:&mut Vec<&'a V>){
		self.terminals.iter().for_each(|v| out.push(v));
		while let Some(c) = ki.next() {
			if let Ok(spot) = self.children.binary_search_by_key(&c, |&(ref k, _)| k) {
				self.children[spot].1.report_subsets(ki.clone(), out);
			}
		}
	}
	
	fn report_supersets<'a>(&'a self, mut ki:SliceIter<'a, C>, out: &mut Vec<&'a V>){
		let kic = ki.clone();
		if let Some(c) = ki.next() {
			for cp in self.children.iter() {
				if &cp.0 < c {
					cp.1.report_supersets(kic.clone(), out);
				}else if &cp.0 == c {
					cp.1.report_supersets(ki.clone(), out);
				}else{
					break;
				}
			}
		}else{
			self.terminals.iter().for_each(|v| out.push(v));
			for cp in self.children.iter() {
				cp.1.report_supersets(ki.clone(), out);
			}
		}
	}
}


impl<C, V> SetTrie<C, V> where
	C: Ord + Clone,
	V: PartialEq,
{
	pub fn new()-> Self { SetTrie{root:Node{children:Vec::new(), terminals:Vec::new()}} }
	
	//TODO: ensure that the input query slices are sorted
	pub fn insert(&mut self, k:DefinitelySorted<C>, v:V) {
		assert!(k.0.len() < 1024, "recursion limit exceeded, use shorter keys");
		self.root.insert_rec(k.0.iter(), v)
	}
	
	pub fn remove(&mut self, k:DefinitelySorted<C>, v:&V)-> Option<V> {
		self.root.remove_rec(k.0.iter(), v)
	}
	
	pub fn contains(&self, k:DefinitelySorted<C>, v:&V)-> bool {
		self.root.contains_rec(k.0.iter(), v)
	}
	
	pub fn supersets<'a>(&'a self, k:DefinitelySorted<'a, C>)-> Vec<&'a V> {
		let mut ret = Vec::new();
		self.root.report_supersets(k.0.iter(), &mut ret);
		ret
	}
	
	pub fn subsets<'a>(&'a self, k:DefinitelySorted<'a, C>)-> Vec<&'a V> {
		let mut ret = Vec::new();
		self.root.report_subsets(k.0.iter(), &mut ret);
		ret
	}
}


#[cfg(test)]
mod tests {
	extern crate rand;
	extern crate array_init;
	extern crate test;
	use super::*;
	use self::rand::{XorShiftRng, SeedableRng, Rng};
	use self::test::Bencher;
	
	#[test]
	fn insert() {
		let mut v = SetTrie::<usize, char>::new();
		v.insert(unsafe{DefinitelySorted::hasty_new(&[1,2,3])}, 'a');
		assert!(v.contains(unsafe{DefinitelySorted::hasty_new(&[1,2,3])}, &'a'));
	}
	
	#[test]
	fn remove_small() {
		let mut v = SetTrie::<usize, char>::new();
		v.insert(unsafe{DefinitelySorted::hasty_new(&[1,2,3])}, 'a');
		assert!(v.remove(unsafe{DefinitelySorted::hasty_new(&[1,2,3])}, &'a').is_some());
		assert!(!v.contains(unsafe{DefinitelySorted::hasty_new(&[1,2,3])}, &'a'));
	}
	
	#[test]
	fn superset_small() {
		let mut v = SetTrie::<usize, char>::new();
		v.insert(assert_sorted(&[1,2,3]), 'a');
		v.insert(assert_sorted(&[1,2,4]), 'b');
		v.insert(assert_sorted(&[0,2,4]), 'c');
		
		let qr = v.supersets(assert_sorted(&[1,2]));
		
		assert!(qr.len() == 2);
		assert!(qr.contains(&&'a'));
		assert!(qr.contains(&&'b'));
	}
	
	#[test]
	fn readme_example() {
		let mut v = SetTrie::<usize, char>::new();
		v.insert(assert_sorted(&[1,2,3]), 'a');
		v.insert(assert_sorted(&[1,2,4]), 'b');
		v.insert(assert_sorted(&[1,2]), 'c');
		v.insert(assert_sorted(&[7]), 'd');
		v.insert(assert_sorted(&[]), 'e');

		let supersets = v.supersets(assert_sorted(&[1,2]));
		assert!(supersets.len() == 3);
		assert!(supersets.contains(&&'a'));
		assert!(supersets.contains(&&'b'));
		assert!(supersets.contains(&&'c'));

		let subsets = v.subsets(assert_sorted(&[1,2,3]));
		assert!(subsets.len() == 3);
		assert!(subsets.contains(&&'a'));
		assert!(subsets.contains(&&'c'));
		assert!(subsets.contains(&&'e'));
	}
	
	fn from_seed(see: usize)-> XorShiftRng {
		let s:[u8; 16] = array_init::array_init(|i| ((i + see).wrapping_mul(77) as u8).wrapping_mul(77) );
		XorShiftRng::from_seed(s)
	}
	
	fn generate_big_superset_example(seed:usize)-> (SetTrie<isize, bool>, Vec<isize>) {
		let mut katy = from_seed(seed);
		let mut v = SetTrie::<isize, bool>::new();
		let n_minimal = 10;
		let n_other = 90;
		let n_all = n_minimal + n_other;
		
		let mut bag = Vec::with_capacity(n_all);
		let mut acc = 0;
		for _ in 0..n_all {
			acc += katy.gen_range(1, 30);
			bag.push(acc);
		}
		
		let take_from = |v:&mut Vec<isize>, katy: &mut XorShiftRng|-> isize {
			let i = katy.gen_range(0, v.len());
			if v.len() == 1 {
				v[0]
			}else{
				let endi = v.len()-1;
				v.swap(i, endi);
				v.pop().unwrap()
			}
		};
		
		let mut in_set = Vec::with_capacity(n_other);
		for _ in 0..n_minimal {
			in_set.push(take_from(&mut bag, &mut katy));
		}
		
		let mut out_set = Vec::with_capacity(n_other);
		for _ in 0..n_other {
			out_set.push(take_from(&mut bag, &mut katy));
		}
		
		//insert matching
		for _ in 0..30 {
			let mut keyset = in_set.clone();
			for _ in 0..(katy.gen_range(3, 8)) {
				keyset.push(out_set[katy.gen_range(0, out_set.len())]);
			}
			v.insert(make_sorted(keyset.as_mut_slice()), true)
		}
		
		//insert non-matching
		for _ in 0..800 {
			let mut keyset = Vec::new();
			for _ in 0..(katy.gen_range(5, 30)) {
				keyset.push(out_set[katy.gen_range(0, out_set.len())]);
			}
			v.insert(make_sorted(keyset.as_mut_slice()), false)
		}
		
		(v, in_set)
	}
	
	
	fn big_insertion_set(seed:usize)-> Vec<Vec<isize>> {
		let mut katy = from_seed(seed);
		let n_cats_total = 300;
		let number_to_insert = 1024;
		
		let mut bag = Vec::with_capacity(n_cats_total);
		let mut acc = 0;
		for _ in 0..n_cats_total {
			acc += katy.gen_range(1, 30);
			bag.push(acc);
		}
		
		let cop_unique = |n:usize, katy: &mut XorShiftRng, take:&mut Vec<isize>|-> Vec<isize> { //takes n unique random samples from take. Has to rearrange take take such a way as to prevent repeats
			assert!(n < take.len());
			(0..n).map(|i| {
				let at = katy.gen_range(i, take.len());
				take.swap(i, at);
				take[i]
			}).collect()
		};
		
		(0..number_to_insert).map(|_|{
			let mut dis = cop_unique(katy.gen_range(4, 30), &mut katy, &mut bag);
			dis.sort_unstable();
			dis
		}).collect()
	}
	
	#[test]
	fn superset_big() {
		for i in 0..57 {
			let (v, mut search_set) = generate_big_superset_example(i);
			let r:Vec<&bool> = v.supersets(make_sorted(search_set.as_mut_slice()));
			assert!(r.len() >= 30);
			assert!(r.iter().filter(|b| ***b).count() == 30);
		}
	}
	
	#[test]
	fn subset_small(){
		let mut v = SetTrie::<usize, char>::new();
		v.insert(assert_sorted(&[1,2,3]), 'a');
		v.insert(assert_sorted(&[1,2]), 'b');
		v.insert(assert_sorted(&[0,2,4]), 'c');
		v.insert(assert_sorted(&[0]), 'd');
		v.insert(assert_sorted(&[0,3]), 'e');
		v.insert(assert_sorted(&[]), 'f');
		
		let results = v.subsets(assert_sorted(&[1,2,3]));
		assert!(results.contains(&&'a'));
		assert!(results.contains(&&'b'));
		assert!(results.contains(&&'f'));
	}
	
	#[bench]
	fn supersets_bench(b: &mut Bencher) {
		let (v, mut qs) = generate_big_superset_example(43);
		let query_set = make_sorted(qs.as_mut_slice());
		b.iter(||{
			v.supersets(query_set)
		});
	}
	
	#[bench]
	fn insert_rec_bench(b: &mut Bencher) {
		let set = big_insertion_set(89);
		b.iter(||{
			let mut v = SetTrie::new();
			for (i, vr) in set.iter().enumerate() {
				v.insert(unsafe{DefinitelySorted::hasty_new(vr.as_slice())}, i);
			}
		});
	}
}