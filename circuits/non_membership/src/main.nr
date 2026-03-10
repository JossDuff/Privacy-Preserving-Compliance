use merkle::compute_merkle_root;
use poseidon::poseidon2::Poseidon2;

/// Sorted Merkle tree non-membership proof.
///
/// Proves that `address` is NOT a leaf in the sorted Merkle tree with the given `root`.
/// The prover provides two adjacent leaves (lower_leaf, upper_leaf) that sandwich
/// the address: lower_leaf < address < upper_leaf.
/// Both leaves are verified against the Merkle root, and their adjacency is enforced
/// (upper_index == lower_index + 1).
///
/// Public inputs:  root, address
/// Private inputs: lower_leaf, upper_leaf, lower_index, upper_index,
///                 lower_hash_path, upper_hash_path
fn main(
    // merkle root
    root: pub Field,
    // user address to prove non-membership of
    address: pub Field,
    // the leaf immediately below address in sorted order
    lower_leaf: Field,
    // the leaf immediately above address in sorted order
    upper_leaf: Field,
    // index of lower_leaf in the tree
    lower_index: Field,
    // index of upper_leaf in the tree
    upper_index: Field,
    // merkle path for lower_leaf
    lower_hash_path: [Field; 32],
    // merkle path for upper_leaf
    upper_hash_path: [Field; 32],
) {
    // 1. Verify sandwich: lower_leaf < address < upper_leaf
    assert(lower_leaf.lt(address), "lower_leaf must be less than address");
    assert(address.lt(upper_leaf), "address must be less than upper_leaf");

    // 2. Verify adjacency: the two leaves must be neighbors in the sorted tree
    assert(upper_index == lower_index + 1, "leaves must be adjacent");

    // 3. Verify lower_leaf is in the tree
    let lower_root = compute_merkle_root(lower_leaf, lower_index, lower_hash_path);
    assert(lower_root == root, "lower_leaf merkle proof invalid");

    // 4. Verify upper_leaf is in the tree
    let upper_root = compute_merkle_root(upper_leaf, upper_index, upper_hash_path);
    assert(upper_root == root, "upper_leaf merkle proof invalid");
}

// For tests: Compute the hash of an all-zero subtree of the given depth.
// depth 0 = leaf = 0, depth k = H(E(k-1), E(k-1))
fn empty_subtree_hash(depth: u32) -> Field {
    let mut h: Field = 0;
    for _ in 0..depth {
        h = Poseidon2::hash([h, h], 2);
    }
    h
}

#[test]
fn test_non_membership() {
    // Sorted tree with 2 leaves at indices 0 and 1, rest empty.
    // leaf[0] = 100, leaf[1] = 300
    // Prove that 200 is NOT in the tree.
    let lower_leaf: Field = 100;
    let upper_leaf: Field = 300;
    let address: Field = 200;

    // Build hash path for lower_leaf (index 0):
    //   level 0 sibling = upper_leaf (300)
    //   level k sibling (k>=1) = empty subtree hash of depth k
    let mut lower_hash_path: [Field; 32] = [0; 32];
    lower_hash_path[0] = upper_leaf;
    for i in 1..32 {
        lower_hash_path[i] = empty_subtree_hash(i);
    }

    // Build hash path for upper_leaf (index 1):
    //   level 0 sibling = lower_leaf (100)
    //   level k sibling (k>=1) = same empty subtree hashes
    let mut upper_hash_path: [Field; 32] = [0; 32];
    upper_hash_path[0] = lower_leaf;
    for i in 1..32 {
        upper_hash_path[i] = empty_subtree_hash(i);
    }

    // Compute root from lower_leaf's perspective
    let root = compute_merkle_root(lower_leaf, 0, lower_hash_path);

    main(
        root,
        address,
        lower_leaf,
        upper_leaf,
        0,
        1,
        lower_hash_path,
        upper_hash_path,
    );
}

#[test(should_fail_with = "lower_leaf must be less than address")]
fn test_fails_when_address_below_lower() {
    let lower_leaf: Field = 100;
    let upper_leaf: Field = 300;
    let address: Field = 50; // below lower_leaf

    let mut lower_hash_path: [Field; 32] = [0; 32];
    lower_hash_path[0] = upper_leaf;
    for i in 1..32 {
        lower_hash_path[i] = empty_subtree_hash(i);
    }

    let mut upper_hash_path: [Field; 32] = [0; 32];
    upper_hash_path[0] = lower_leaf;
    for i in 1..32 {
        upper_hash_path[i] = empty_subtree_hash(i);
    }

    let root = compute_merkle_root(lower_leaf, 0, lower_hash_path);
    main(
        root,
        address,
        lower_leaf,
        upper_leaf,
        0,
        1,
        lower_hash_path,
        upper_hash_path,
    );
}

#[test(should_fail_with = "address must be less than upper_leaf")]
fn test_fails_when_address_above_upper() {
    let lower_leaf: Field = 100;
    let upper_leaf: Field = 300;
    let address: Field = 500; // above upper_leaf

    let mut lower_hash_path: [Field; 32] = [0; 32];
    lower_hash_path[0] = upper_leaf;
    for i in 1..32 {
        lower_hash_path[i] = empty_subtree_hash(i);
    }

    let mut upper_hash_path: [Field; 32] = [0; 32];
    upper_hash_path[0] = lower_leaf;
    for i in 1..32 {
        upper_hash_path[i] = empty_subtree_hash(i);
    }

    let root = compute_merkle_root(lower_leaf, 0, lower_hash_path);
    main(
        root,
        address,
        lower_leaf,
        upper_leaf,
        0,
        1,
        lower_hash_path,
        upper_hash_path,
    );
}

#[test(should_fail_with = "leaves must be adjacent")]
fn test_fails_when_leaves_not_adjacent() {
    let lower_leaf: Field = 100;
    let upper_leaf: Field = 300;
    let address: Field = 200;

    let mut lower_hash_path: [Field; 32] = [0; 32];
    lower_hash_path[0] = upper_leaf;
    for i in 1..32 {
        lower_hash_path[i] = empty_subtree_hash(i);
    }

    let mut upper_hash_path: [Field; 32] = [0; 32];
    upper_hash_path[0] = lower_leaf;
    for i in 1..32 {
        upper_hash_path[i] = empty_subtree_hash(i);
    }

    let root = compute_merkle_root(lower_leaf, 0, lower_hash_path);
    // Pass non-adjacent indices (0 and 2 instead of 0 and 1)
    main(
        root,
        address,
        lower_leaf,
        upper_leaf,
        0,
        2,
        lower_hash_path,
        upper_hash_path,
    );
}

#[test(should_fail_with = "upper_leaf merkle proof invalid")]
fn test_fails_with_wrong_root() {
    let lower_leaf: Field = 100;
    let upper_leaf: Field = 300;
    let address: Field = 200;

    let mut lower_hash_path: [Field; 32] = [0; 32];
    lower_hash_path[0] = upper_leaf;
    for i in 1..32 {
        lower_hash_path[i] = empty_subtree_hash(i);
    }

    // Tamper with upper hash path so it doesn't match the root
    let mut upper_hash_path: [Field; 32] = [0; 32];
    upper_hash_path[0] = 999;
    for i in 1..32 {
        upper_hash_path[i] = empty_subtree_hash(i);
    }

    let root = compute_merkle_root(lower_leaf, 0, lower_hash_path);
    main(
        root,
        address,
        lower_leaf,
        upper_leaf,
        0,
        1,
        lower_hash_path,
        upper_hash_path,
    );
}
