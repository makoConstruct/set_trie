
#![feature(nll)]

use std::slice::Iter as SliceIter;

pub struct Node<C, V> {
	children: Vec<(C, Node<C, V>)>,
	terminals: Vec<V>,
}

pub struct SetTrie<C, V> {
	pub root: Node<C, V>,
}

struct SupersetIterStage<'a, C, V> where C:'a, V:'a {
	cur: &'a Node<C, V>,
	query_eye: usize, //how far through the query we are
	child_eye: usize, //how far through the children we are
}
pub struct SupersetIter<'a, C, V> where C:'a, V:'a {
	stack: Vec<SupersetIterStage<'a, C, V>>,
	val_eye: usize, //how far through the current terminals of a matched node we are
	query: &'a [C], //the set we are looking for supersets of
}

// struct SubsetIterStage<'a, C, V> where C:'a, V:'a {
// 	cur: &'a Node<C, V>,
// 	query_eye: usize, //how far through the query we are
// 	child_eye: usize, //how far through the children we are
// }
// pub struct SubsetIter<'a, C, V> where C:'a, V:'a {
// 	stack: Vec<SubsetIterStage<'a, C, V>>,
// 	val_eye: usize, //how far through the current terminals of a matched node we are
// 	query: &'a [C], //the set we are looking for supersets of
// }

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


impl<'a, C, V> Iterator for SupersetIter<'a, C, V> where
	C: PartialOrd + PartialEq,
{
	type Item = &'a V;
	
	fn next(&mut self)-> Option<Self::Item> {
		let SupersetIter{
			ref query,
			ref mut val_eye,
			ref mut stack
		} = *self;
		loop { //looping over values and going rootwards
			if let Some(cur_stage) = stack.last_mut() {
				if cur_stage.query_eye == query.len() {
					//then we're in the match zone
					if *val_eye < cur_stage.cur.terminals.len() {
						let cur_stage = stack.last_mut().unwrap();
						let ret = Some(&cur_stage.cur.terminals[*val_eye]);
						*val_eye += 1;
						return ret;
					}else{
						let cur_child_eye = cur_stage.child_eye;
						cur_stage.child_eye += 1;
						*val_eye = 0;
						if cur_child_eye == cur_stage.cur.children.len() {
							//then this level is done
							stack.pop();
						}else{
							let nc = &cur_stage.cur.children.get(cur_child_eye).unwrap().1 as *const _;
							let cqi = cur_stage.query_eye;
							stack.push(SupersetIterStage{
								cur: unsafe{ &*nc }, //we have to make it a ptr to sever the hold on stack. This is safe because the change to stack will not affect the structure of the tree and invalidate the node reference. I feel like there might be some way to express this by having more than one lifetime in the iter, but, wew, maybe later
								query_eye: cqi,
								child_eye: 0,
							});
						}
					}
				}else{
					let cur_child_eye = cur_stage.child_eye;
					cur_stage.child_eye += 1;
					if cur_child_eye == cur_stage.cur.children.len() {
						stack.pop();
					}else{
						let &mut SupersetIterStage {
							ref cur,
							ref query_eye,
							..
						} = cur_stage;
						let cp = &cur.children[cur_child_eye];
						let child_node = &cp.1 as *const _;
						let child_k = &cp.0;
						let qq = &query[*query_eye];
						if child_k > qq {
							stack.pop();
						}else{
							let nqi = if child_k == qq {
								query_eye + 1
							} else {
								*query_eye
							};
							stack.push(SupersetIterStage{
								cur: unsafe{ &*child_node },
								query_eye: nqi,
								child_eye: 0,
							});
						}
					}
				}
			}else{
				return None;
			}
		}
	}
}


impl<C, V> Node<C, V> where
	C: Ord + Clone,
	V: PartialEq,
{
	
	//TODO_PERF: consider binary search instead of linear
	
	fn insert_rec(&mut self, mut ki:SliceIter<C>, v:V) {
		match ki.next() {
			Some(k)=> {
				let finish_insertion = |this:&mut Self, chi:usize, k:&C, ki:SliceIter<C>, v:V|{
					// insert, create a node chain
					let mut rki = ki.rev();
					let mut node_chain:Node<C,V> = Node{ children:vec!(), terminals:vec!(v) };
					while let Some(sk) = rki.next() {
						node_chain = Node{ children:vec!((sk.clone(), node_chain)), terminals:vec!() };
					}
					this.children.insert(chi, (k.clone(), node_chain))
				};
				let mut chi = 0;
				while let Some(&mut (ref ck, ref mut cn)) = self.children.get_mut(chi) {
					if ck == k {
						return cn.insert_rec(ki, v);
					}else if ck > k {
						return finish_insertion(self, chi, k, ki, v);
					}
					chi += 1;
				}
				finish_insertion(self, chi, k, ki, v)
			}
			None=> {
				self.terminals.push(v)
			}
		}
	}
	
	fn remove_rec(&mut self, mut ki:SliceIter<C>, v:&V)-> Option<V> {
		if let Some(k) = ki.next() {
			let mut i = 0;
			while i < self.children.len() {
				let np = self.children.get_mut(i).unwrap();
				if np.0 == *k {
					let ret = np.1.remove_rec(ki, v);
					if np.1.children.is_empty() && np.1.terminals.is_empty() {
						//then delete it
						self.children.remove(i);
					}
					return ret;
				}else if np.0 > *k {
					return None;
				}
				i += 1;
			}
			None
		}else{
			if let Some((i, _)) = self.terminals.iter().enumerate().find(|p| p.1 == v) {
				Some(self.terminals.remove(i))
			}else{
				None
			}
		}
	}
	
	fn contains_rec(&self, mut ki:SliceIter<C>, v:&V)-> bool {
		if let Some(k) = ki.next() {
			let mut i = 0;
			while i < self.children.len() {
				let np = unsafe{self.children.get_unchecked(i)};
				if np.0 == *k {
					return np.1.contains_rec(ki, v);
				}else if np.0 > *k {
					return false;
				}
				i += 1;
			}
			false
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
	
	pub fn superset<'a>(&'a self, k:DefinitelySorted<'a, C>)-> SupersetIter<'a, C, V> {
		SupersetIter {
			stack: vec!(SupersetIterStage{
				cur: &self.root,
				query_eye: 0,
				child_eye: 0,
			}),
			val_eye: 0,
			query: k.0,
		}
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
	use super::*;
	use self::rand::{XorShiftRng, SeedableRng, Rng};
	
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
		let qr = v.superset(assert_sorted(&[1,2]));
		assert!(qr.collect::<Vec<&char>>().as_slice() == &[&'a', &'b']);
	}
	
	fn from_seed(see: usize)-> XorShiftRng {
		let s:[u8; 16] = array_init::array_init(|i| ((i + see).wrapping_mul(77) as u8).wrapping_mul(77) );
		XorShiftRng::from_seed(s)
	}
	
	#[test]
	fn superset_big() {
		for i in 0..57 {
			let mut katy = from_seed(i);
			let mut v = SetTrie::<isize, bool>::new();
			let mut minimal_set = Vec::new();
			let mut acc = 0;
			for _ in 0..10 {
				acc += katy.gen_range(1, 30);
				minimal_set.push(acc);
			}
			
			let mut additions = Vec::new();
			for _ in 0..90 {
				additions.push(katy.gen_range(-30, acc + 100));
			}
			
			//insert non-matching
			for _ in 0..800 {
				let mut keyset = Vec::new();
				for _ in 0..(katy.gen_range(5, 30)) {
					keyset.push(additions[katy.gen_range(0, additions.len())]);
				}
				keyset.sort_unstable();
				v.insert(unsafe{DefinitelySorted::hasty_new(keyset.as_slice())}, false)
			}
			
			//insert matching
			for _ in 0..30 {
				let mut keyset = minimal_set.clone();
				for _ in 0..(katy.gen_range(3, 8)) {
					keyset.push(additions[katy.gen_range(0, additions.len())]);
				}
				keyset.sort_unstable();
				v.insert(unsafe{DefinitelySorted::hasty_new(keyset.as_slice())}, true)
			}
			
			let r:Vec<bool> = v.superset(unsafe{DefinitelySorted::hasty_new(minimal_set.as_mut_slice())}).map(|br| *br).collect();
			assert!(r.len() >= 30);
			assert!(r.iter().filter(|b| **b).count() == 30);
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
}