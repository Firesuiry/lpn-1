use fnv::FnvHashMap;
use m4ri_rust::friendly::BinVector;
use oracle::query_bits_range;
use oracle::*;
use std::default::Default;
use std::ops;

/// The reduction from BKW
///
/// also known as partition-reduce in the Bogos/Vaudenay extra material
///
/// $k' = k - (a-1) * b$
/// $n' = n - (a-1)*2^b
/// $d' = delta^{2*(a-1)}$
pub fn bkw_reduction(oracle: &mut LpnOracle, a: u32, b: u32) {
    let k = oracle.k;
    assert!(a * b <= k, "a*b <= k");

    for i in 1..a {
        let num_queries = oracle.queries.len();
        println!("BKW iteration {}, {} queries left", i, num_queries);
        // Partition into V_j
        // max j:
        let maxj = 2usize.pow(b);
        let query_capacity = num_queries / maxj;

        let mut vector_partitions: Vec<Vec<&Query>> = Vec::with_capacity(maxj);
        // using [v; n] clones and won't set capacity correctly.
        for _ in 0..maxj {
            vector_partitions.push(Vec::with_capacity(query_capacity));
        }
        let mut firsts: Vec<Option<&Query>> = vec![None; maxj];

        let bitrange: ops::Range<usize> = ((k - (b * i)) as usize)..((k - (b * (i - 1))) as usize);
        let mut indexes = Vec::with_capacity(maxj);
        for (j, q) in oracle.queries.iter_mut().enumerate() {
            let idx = query_bits_range(&(q.a), &bitrange) as usize;
            q.a.truncate((k - (b * i)) as usize);
            if let Some(first) = firsts[idx] {
                q.a += &first.a;
                q.s ^= first.s;
            } else {
                firsts[idx] = Some(q);
                indexes.push(j);
            }
        }
        indexes.into_iter().for_each(|idx| {
            oracle.queries.swap_remove(idx);
        });
    }

    // Set the new k
    oracle.k = k - (a - 1) * b;
    oracle.secret.truncate(oracle.k as usize);
    println!(
        "BKW iterations done, {} queries left, k' = {}",
        oracle.queries.len(),
        oracle.k
    );

}

pub fn bkw_solve(oracle: LpnOracle) -> BinVector {
    println!("BKW Solver");
    let b = oracle.queries[0].a.len();
    debug_assert!(b <= 20, "Don't run BKW on too large b!");
    println!(
        "Selecting all queries with hw=1 from {} queries",
        oracle.queries.len()
    );
    let range = 0..(b);
    let queries = oracle
        .queries
        .into_iter()
        .filter(|q| q.count_ones() == 1)
        .map(|q| (query_bits_range(&q.a, &range), q.s))
        .collect::<Vec<(u64, bool)>>();

    // allocate smaller vec
    let mut counts: FnvHashMap<u64, u64> =
        FnvHashMap::with_capacity_and_hasher(b, Default::default());
    let mut sums: FnvHashMap<u64, u64> =
        FnvHashMap::with_capacity_and_hasher(b, Default::default());

    println!(
        "Sorting out and counting {} queries for majority selection",
        queries.len()
    );
    for query in queries.into_iter() {
        debug_assert_eq!(query.0.count_ones(), 1);
        let count = counts.entry(query.0).or_insert(0);
        *count += 1;
        if query.1 {
            let sum = sums.entry(query.0).or_insert(0);
            *sum += 1;
        }
    }

    let mut result = BinVector::with_capacity(b as usize);
    let mut i = 1 << (b - 1);
    while i > 0 {
        let count = counts.get(&i).expect("this bucket can't be empty!");
        if let Some(sum) = sums.get(&i) {
            result.push(*count < 2 * sum);
        } else {
            result.push(false);
        }
        i >>= 1;
    }
    result
}
