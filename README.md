## Set Trie

[The Set Trie: Efficient subset and superset queries](http://osebje.famnit.upr.si/~savnik/drafts/settrie0.pdf)

A Set Trie maintains a mapping from sets of `C` to values of `V`. For a set of `C`s, it can very efficiently find all of the `V`s mapped from supersets of that set, or from subsets of that set.

```rust
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
```