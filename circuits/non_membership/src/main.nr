use merkle::compute_merkle_root;
use poseidon::poseidon2::Poseidon2;

/// Sorted Merkle tree non-membership proof.
///
/// Proves that `address` is NOT a leaf in the sorted Merkle tree with the given `root`.
///
/// Three proof modes (selected by `proof_type`):
///   0 = Sandwich:    lower_leaf < address < upper_leaf, both adjacent and in tree
///   1 = Below min:   address < first leaf (index 0)
///   2 = Above max:   address > last leaf, next position is empty
///
/// Public inputs:  address, root
/// Private inputs: lower_leaf, upper_leaf, lower_index, upper_index,
///                 lower_hash_path, upper_hash_path, proof_type
fn main(
    // user address to prove non-membership of
    address: pub Field,
    // merkle root
    root: pub Field,
    // the leaf immediately below address in sorted order (unused for proof_type=1)
    lower_leaf: Field,
    // the leaf immediately above address in sorted order (unused for proof_type=2)
    upper_leaf: Field,
    // index of lower_leaf in the tree (unused for proof_type=1)
    lower_index: Field,
    // index of upper_leaf in the tree (unused for proof_type=2)
    upper_index: Field,
    // merkle path for lower_leaf (unused for proof_type=1)
    lower_hash_path: [Field; 32],
    // merkle path for upper_leaf; for proof_type=2 this is the path for the empty position
    upper_hash_path: [Field; 32],
    // 0 = sandwich, 1 = below minimum, 2 = above maximum
    proof_type: u8,
) {
    if proof_type == 0 {
        // Normal sandwich: two adjacent leaves bound the address
        assert(lower_leaf.lt(address), "lower_leaf must be less than address");
        assert(address.lt(upper_leaf), "address must be less than upper_leaf");
        assert(upper_index == lower_index + 1, "leaves must be adjacent");

        let lower_root = compute_merkle_root(lower_leaf, lower_index, lower_hash_path);
        assert(lower_root == root, "lower_leaf merkle proof invalid");

        let upper_root = compute_merkle_root(upper_leaf, upper_index, upper_hash_path);
        assert(upper_root == root, "upper_leaf merkle proof invalid");
    } else if proof_type == 1 {
        // Below minimum: address is less than the first leaf in the sorted tree
        assert(address.lt(upper_leaf), "address must be less than first leaf");
        assert(upper_index == 0, "first leaf must be at index 0");

        let upper_root = compute_merkle_root(upper_leaf, upper_index, upper_hash_path);
        assert(upper_root == root, "first leaf merkle proof invalid");
    } else {
        // Above maximum: address is greater than the last populated leaf.
        // We verify the last leaf is in the tree and the next position is empty.
        assert(lower_leaf.lt(address), "last leaf must be less than address");

        let lower_root = compute_merkle_root(lower_leaf, lower_index, lower_hash_path);
        assert(lower_root == root, "last leaf merkle proof invalid");

        // Prove the next position holds an empty leaf (value = 0)
        let empty_root = compute_merkle_root(0, lower_index + 1, upper_hash_path);
        assert(empty_root == root, "next position must be empty");
    }
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

// -- Sandwich (proof_type = 0) ---------------------------------------

#[test]
fn test_sandwich() {
    // Tree: [100, 300], prove 200 is not in the tree.
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
    main(
        address,
        root,
        lower_leaf,
        upper_leaf,
        0,
        1,
        lower_hash_path,
        upper_hash_path,
        0,
    );
}

#[test(should_fail_with = "leaves must be adjacent")]
fn test_sandwich_fails_not_adjacent() {
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
    main(
        address,
        root,
        lower_leaf,
        upper_leaf,
        0,
        2,
        lower_hash_path,
        upper_hash_path,
        0,
    );
}

#[test(should_fail_with = "upper_leaf merkle proof invalid")]
fn test_sandwich_fails_wrong_proof() {
    let lower_leaf: Field = 100;
    let upper_leaf: Field = 300;
    let address: Field = 200;

    let mut lower_hash_path: [Field; 32] = [0; 32];
    lower_hash_path[0] = upper_leaf;
    for i in 1..32 {
        lower_hash_path[i] = empty_subtree_hash(i);
    }

    let mut upper_hash_path: [Field; 32] = [0; 32];
    upper_hash_path[0] = 999;
    for i in 1..32 {
        upper_hash_path[i] = empty_subtree_hash(i);
    }

    let root = compute_merkle_root(lower_leaf, 0, lower_hash_path);
    main(
        address,
        root,
        lower_leaf,
        upper_leaf,
        0,
        1,
        lower_hash_path,
        upper_hash_path,
        0,
    );
}

// -- Below minimum (proof_type = 1) ----------------------------------

#[test]
fn test_below_min() {
    // Tree: [300], prove 100 is not in the tree (below all leaves).
    let upper_leaf: Field = 300;
    let address: Field = 100;

    let mut upper_hash_path: [Field; 32] = [0; 32];
    // Sibling at index 1 is empty (0)
    for i in 1..32 {
        upper_hash_path[i] = empty_subtree_hash(i);
    }

    let root = compute_merkle_root(upper_leaf, 0, upper_hash_path);
    // lower_leaf, lower_index, lower_hash_path are unused -- pass zeros
    main(
        address,
        root,
        0,
        upper_leaf,
        0,
        0,
        [0; 32],
        upper_hash_path,
        1,
    );
}

#[test(should_fail_with = "address must be less than first leaf")]
fn test_below_min_fails_when_address_not_below() {
    // address = 500 is NOT below upper_leaf = 300
    let upper_leaf: Field = 300;
    let address: Field = 500;

    let mut upper_hash_path: [Field; 32] = [0; 32];
    for i in 1..32 {
        upper_hash_path[i] = empty_subtree_hash(i);
    }

    let root = compute_merkle_root(upper_leaf, 0, upper_hash_path);
    main(
        address,
        root,
        0,
        upper_leaf,
        0,
        0,
        [0; 32],
        upper_hash_path,
        1,
    );
}

// -- Above maximum (proof_type = 2) ----------------------------------

#[test]
fn test_above_max() {
    // Tree: [100], prove 200 is not in the tree (above all leaves).
    let lower_leaf: Field = 100;
    let address: Field = 200;

    // Proof for lower_leaf at index 0: sibling at index 1 is empty
    let mut lower_hash_path: [Field; 32] = [0; 32];
    for i in 1..32 {
        lower_hash_path[i] = empty_subtree_hash(i);
    }

    // Proof for empty position at index 1: sibling at index 0 is lower_leaf
    let mut empty_hash_path: [Field; 32] = [0; 32];
    empty_hash_path[0] = lower_leaf;
    for i in 1..32 {
        empty_hash_path[i] = empty_subtree_hash(i);
    }

    let root = compute_merkle_root(lower_leaf, 0, lower_hash_path);
    // upper_leaf and upper_index are unused -- pass zeros
    main(
        address,
        root,
        lower_leaf,
        0,
        0,
        0,
        lower_hash_path,
        empty_hash_path,
        2,
    );
}

#[test(should_fail_with = "last leaf must be less than address")]
fn test_above_max_fails_when_address_not_above() {
    // address = 50 is NOT above lower_leaf = 100
    let lower_leaf: Field = 100;
    let address: Field = 50;

    let mut lower_hash_path: [Field; 32] = [0; 32];
    for i in 1..32 {
        lower_hash_path[i] = empty_subtree_hash(i);
    }

    let mut empty_hash_path: [Field; 32] = [0; 32];
    empty_hash_path[0] = lower_leaf;
    for i in 1..32 {
        empty_hash_path[i] = empty_subtree_hash(i);
    }

    let root = compute_merkle_root(lower_leaf, 0, lower_hash_path);
    main(
        address,
        root,
        lower_leaf,
        0,
        0,
        0,
        lower_hash_path,
        empty_hash_path,
        2,
    );
}

#[test(should_fail_with = "next position must be empty")]
fn test_above_max_fails_when_next_not_empty() {
    // Tree: [100, 300]. Claim 200 is above max using proof_type=2.
    // This should fail because index 1 is NOT empty (it's 300).
    let lower_leaf: Field = 100;
    let address: Field = 200;

    let mut lower_hash_path: [Field; 32] = [0; 32];
    lower_hash_path[0] = 300; // sibling at index 1 is 300
    for i in 1..32 {
        lower_hash_path[i] = empty_subtree_hash(i);
    }

    // Provide the actual hash path for index 1 -- but the leaf there is 300, not 0.
    // The circuit will compute_merkle_root(0, 1, ...) which won't match
    // compute_merkle_root(300, 1, ...), so it should fail.
    let mut empty_hash_path: [Field; 32] = [0; 32];
    empty_hash_path[0] = lower_leaf;
    for i in 1..32 {
        empty_hash_path[i] = empty_subtree_hash(i);
    }

    let root = compute_merkle_root(lower_leaf, 0, lower_hash_path);
    main(
        address,
        root,
        lower_leaf,
        0,
        0,
        0,
        lower_hash_path,
        empty_hash_path,
        2,
    );
}
