
#![feature(nll)]

use std::slice::Iter as SliceIter;

pub struct Node<C, V> {
	children: Vec<(C, Node<C, V>)>,
	terminals: Vec<V>,
}

pub struct SetTrie<C, V> {
	pub root: Node<C, V>,
}

struct SupersetIterStage {
	cur: &Node<C, V>,
	query_eye: usize, //how far through the query we are
	child_eye: usize, //how far through the children we are
}
pub struct SupersetIter<C, V> {
	stack: Vec<SupersetIterStage>,
	val_eye: usize, //how far through the current terminals of a matched node we are
	query: &[C], //the set we are looking for supersets of
}


impl<C, V> SupersetIter<C, V> { //this code probably would have been easier to write with a generator, but I'm not sure whether it'd have been more efficient or not. Regardless, generators are too far from being stable as of now
	fn next(&mut self)-> Option<&V> {
		let SupersetIter{
			ref query,
			ref mut val_eye,
			ref mut stack
		} = *self;
		loop { //looping over values and going rootwards
			if stack.is_empty() {
				return None;
			}else{
				if *val_eye == 0 {
					//then we aren't On this one
					//seek the next matching child
					let cur_stage = stack.back_mut().unwrap();
					if cur_stage.query_eye == query.len() {
						//then everything we come across matches
						if cur_stage.child_eye == cur_stage.cur.children.len() {
							//then this level is done
							stack.pop();
							break;
						}else{
							let nc = cur_stage.cur.children.get(cur_stage.child_eye).unwrap().1;
							cur_stage.child_eye += 1;
							stack.push(SupersetIterStage{
								cur: nc,
								query_eye: cur_stage.query_eye,
								child_eye: 0,
							});
							if nc.terminals.len() != 0 {
								//start going over the values
								let ret = Some(nc.terminals[0]);
								val_eye = 1;
								return ret;
							}// else continue looping over children, going leafwards
						}
					}else{
						if cur_stage.child_eye == cur_stage.cur.children.len() {
							stack.pop();
							break;
						}else{
							let mr = cur_stage.cur.children[cur_stage.child_eye];
							let qq = query[cur_stage.query_eye];
							if mr.0 == *qq {
								stack.push(SupersetIterStage{
									cur: mr.1,
									query_eye: cur_stage.query_eye + 1,
									child_eye: 0,
								});
							}else if mr.0 > *qq {
								//end of possible matches
								stack.pop();
								break;
							}else{
								//then it's smaller and we should leafgo without incrementing the query eye
								stack.push(SupersetIterStage{
									cur: mr.1,
									query_eye: cur_stage.query_eye,
									child_eye: 0,
								});
							}
						}
					}
				}else{
					//then we're browsing this one
					let cur_stage = stack.back_mut().unwrap();
					let ret = Some(cur_stage.cur.terminals[val_eye]); //val_eye is always checked for overrun after incrementation, and is only non-zero when the node is known to have matching terminals
					val_eye += 1;
					if val_eye >= cur_stage.cur.terminals.len() {
						*val_eye = 0;
					}
					return ret;
				}
			}
		}
	}
}


impl<C, V> Node<C, V> where
	C: PartialOrd + Clone,
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
}


impl<C, V> SetTrie<C, V> where
	C: PartialOrd + Clone,
	V: PartialEq,
{
	pub fn new()-> Self { SetTrie{root:Node{children:Vec::new(), terminals:Vec::new()}} }
	
	//TODO: ensure that the input query slices are sorted
	pub fn insert(&mut self, k:&[C], v:V) {
		assert!(k.len() < 1024, "recursion limit exceeded, use shorter keys");
		self.root.insert_rec(k.iter(), v)
	}
	
	pub fn remove(&mut self, k:&[C], v:&V)-> Option<V> {
		self.root.remove_rec(k.iter(), v)
	}
	
	pub fn contains(&self, k:&[C], v:&V)-> bool {
		self.root.contains_rec(k.iter(), v)
	}
	
	pub fn superset(&self, k:&[C])-> Iter<V> {
		
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn insert() {
		let mut v = SetTrie::<usize, char>::new();
		v.insert(&[1,2,3], 'a');
		assert!(v.contains(&[1,2,3], &'a'));
	}
	
	#[test]
	fn remove_small() {
		let mut v = SetTrie::<usize, char>::new();
		v.insert(&[1,2,3], 'a');
		assert!(v.remove(&[1,2,3], &'a').is_some());
		assert!(!v.contains(&[1,2,3], &'a'));
	}
}