## Set Trie

[The Set Trie: Efficient subset and superset queries](http://osebje.famnit.upr.si/~savnik/drafts/settrie0.pdf)

A Set Trie maintains a mapping from sets of `C` to values of `V`. For a set of `C`s, it can very efficiently find all of the `V`s mapped from supersets of that set, or from subsets of that set.

```rust
let mut v = SetTrie::<usize, char>::new();
v.insert(assert_sorted(&[1,2,3]), 'a');
v.insert(assert_sorted(&[1,2,4]), 'b');
v.insert(assert_sorted(&[0,2,4]), 'c');

let qr = v.superset(assert_sorted(&[1,2]));
assert!(qr.collect::<Vec<&char>>().as_slice() == &[&'a', &'b']);
```