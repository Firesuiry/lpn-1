//! Defines the algorithms from the classic Blum, Kalai and Wasserman paper
use crate::oracle::query_bits_range;
use crate::oracle::*;
use fnv::FnvHashMap;
use m4ri_rust::friendly::BinVector;
use std::ops;
use std::{default::Default, num::NonZeroUsize};

use rayon::prelude::*;
use unchecked_unwrap::UncheckedUnwrap;

/// The full BKW solving algorithm.
///
/// Does `a-1` applications of [`partition-reduce`] with `$b$` and solves via majority.
///
/// $k' = k - (a-1) * b$
/// $n' = n - (a-1)*2^b
/// $d' = delta^{2*(a-1)}$
pub fn bkw(mut oracle: LpnOracle, a: u32, b: u32) -> BinVector {
    bkw_reduce(&mut oracle, a, b);
    majority(oracle)
}

/// Reduces the LPN problem size using the reduction from Blum, Kalai and Wasserman.
pub fn partition_reduce(oracle: &mut LpnOracle, b: u32) {
    bkw_reduce(oracle, 2, b);
}

fn bkw_reduce_inplace(oracle: &mut LpnOracle, i: usize, b: usize) {
    let num_samples = oracle.samples.len();
    let k = oracle.get_k() as usize;

    let maxj = 2usize.pow(b as u32);
    // max j:
    println!(
        "BKW iteration, {} samples left, expecting to remove {} through indexing method",
        num_samples, maxj
    );

    let mut firsts_idxs: Vec<Option<NonZeroUsize>> = vec![None; maxj];

    let bitrange: ops::Range<usize> = (k - (b * i))..(k - (b * (i - 1)));
    // first collect "firsts" so we can do the later part in parallel
    for (j, q) in oracle.samples[1..].iter_mut().enumerate() {
        let idx = query_bits_range(&q, bitrange.clone()) as usize;
        if firsts_idxs[idx].is_some() {
            if firsts_idxs.iter().any(|item| item.is_none()) {
                break;
            }
        } else {
            // this can never be zero.
            firsts_idxs[idx] = Some(unsafe { NonZeroUsize::new_unchecked(j + 1) });
        }
    }
    let mut firsts = vec![None; maxj];
    firsts_idxs.sort_unstable();
    firsts_idxs
        .into_iter()
        .rev()
        .filter(|x| x.is_some())
        .for_each(|idx| {
            // safe as we've excluded the None values
            let idx = unsafe { idx.unchecked_unwrap() }.get();
            let item = oracle.samples.swap_remove(idx);
            firsts[idx] = Some(item);
        });
    // not consuming the iterator to do as much as possible in-place.
    oracle.samples.par_iter_mut().for_each(|q| {
        let idx = query_bits_range(&q, bitrange.clone()) as usize;
        if let Some(item) = &firsts[idx] {
            q.xor_into(item);
        }
    });
}

fn bkw_reduce_sorted(oracle: &mut LpnOracle, i: usize, b: usize) {
    let k = oracle.get_k();
    let bitrange: ops::Range<usize> = (k - i * b)..k;

    let maxj = 2usize.pow(b as u32);
    // max j:
    println!(
        "BKW iteration, {} samples left, expecting to remove {} through sorting method",
        oracle.samples.len(), maxj
    );

    oracle
        .samples
        .par_sort_unstable_by_key(|q| query_bits_range(q, bitrange.clone()));

    // split into partitions
    println!("Creating partition slices");
    let mut partitions = Vec::with_capacity(2usize.pow(b as u32));
    let oracle_start = oracle.samples.as_ptr() as usize;
    let mut samples = &mut oracle.samples[..];
    while samples.len() > 0 {
        let current_key = query_bits_range(&samples[0], bitrange.clone());
        let take_until =
            samples.partition_point(|q| current_key == query_bits_range(q, bitrange.clone()));
        let (these_samples, new_samples) = samples.split_at_mut(take_until);
        partitions.push(these_samples);
        samples = new_samples;
    }

    println!("Processing partitions");
    partitions.par_iter_mut().for_each(|partition| {
        let first = partition[0];
        partition[1..].iter_mut().for_each(|q| q.xor_into(&first));
    });

    // compute indexes of firsts
    println!("Removing pivots");
    let firsts = partitions.into_par_iter().rev().map(|partition| {
        (partition.as_ptr() as *const _ as usize - oracle_start) / std::mem::size_of::<Sample>()
    }).collect::<Vec<_>>();

    // this is descending because par_iter_map preserves order.
    for index in firsts.into_iter() {
        oracle.samples.swap_remove(index);
    }
}

/// Performs the BKW reduction algorithm, see [`partition_reduce`] for public usage
fn bkw_reduce(oracle: &mut LpnOracle, a: u32, b: u32) {
    let k = oracle.get_k();
    let a = a as usize;
    let b = b as usize;
    assert!(a * b <= k, "a*b <= k");

    for i in 1..a {
        // somewhat empirically decided through benchmark
        // probably related to size of LUT fitting in cache
        if b < 22 {
            bkw_reduce_inplace(oracle, i, b);
        } else {
            bkw_reduce_sorted(oracle, i, b)
        }
    }

    // Set the new k
    oracle.truncate(k - (a - 1) * b);
    println!(
        "BKW iterations done, {} samples left, k' = {}",
        oracle.samples.len(),
        oracle.get_k()
    );
}

/// Recover the secret using the majority strategy from BKW
pub fn majority(oracle: LpnOracle) -> BinVector {
    println!("BKW Solver: majority");
    let b = oracle.get_k();
    debug_assert!(b <= 20, "Don't run BKW on too-large b!");
    println!(
        "Selecting all samples with hw=1 from {} samples",
        oracle.samples.len()
    );
    let samples = oracle
        .samples
        .into_iter()
        .filter_map(|q| if q.count_ones() == 1 { Some(q) } else { None })
        .collect::<Vec<Sample>>();

    // allocate smaller vec
    let mut count_sum: FnvHashMap<StorageBlock, (u64, u64)> =
        FnvHashMap::with_capacity_and_hasher(b, Default::default());

    println!(
        "Sorting out and counting {} samples for majority selection",
        samples.len()
    );
    for query in samples.into_iter() {
        debug_assert_eq!(query.count_ones(), 1);
        let count_sum = count_sum.entry(query.get_block(0)).or_insert((0, 0));
        count_sum.0 += 1;
        if query.get_product() {
            count_sum.1 += 1;
        }
    }

    let mut result = BinVector::with_capacity(b as usize);
    let mut i = 1;
    while i < 1 << b {
        let (count, sum) = count_sum.get(&i).expect("this bucket can't be empty!");
        result.push(*count < 2 * sum);
        i <<= 1;
    }
    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bkw() {
        let a = 4;
        let b = 8;

        let mut oracle: LpnOracle = LpnOracle::new(32, 1.0 / 32.0);
        oracle.get_samples(200_000);

        // get secret for checking
        let secret = &oracle.secret;
        println!("{:x?}", secret);
        let mut secret = secret.as_binvector(oracle.get_k());

        // run bkw
        let solution = bkw(oracle, a, b);
        secret.truncate(solution.len());
        assert_eq!(solution, secret);
    }
}
