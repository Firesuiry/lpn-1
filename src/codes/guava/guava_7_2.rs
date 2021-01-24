use std::boxed::Box;
use std::default::Default;
use std::sync::Once;

use fnv::FnvHashMap;

use m4ri_rust::friendly::BinMatrix;
use m4ri_rust::friendly::BinVector;

use crate::codes::BinaryCode;

/// ``[7, 2]`` Guava code
///
/// Best code found from the GUAVA database version 3.15
///
/// Decodes using Syndrome decoding
#[derive(Clone, Serialize)]
pub struct GuavaCode7_2;

static INIT: Once = Once::new();
static mut GENERATOR_MATRIX: *const BinMatrix = 0 as *const BinMatrix;
static mut PARITY_MATRIX: *const BinMatrix = 0 as *const BinMatrix;
static mut PARITY_MATRIX_T: *const BinMatrix = 0 as *const BinMatrix;
static mut SYNDROME_MAP: *const FnvHashMap<u64, &'static [usize; 1]> = 0 as *const FnvHashMap<u64, &'static [usize; 1]>;

fn init() {
    INIT.call_once(|| {
        unsafe {
            let matrix = Box::new(BinMatrix::from_slices(&[
                &[ 101 ],
                &[ 122 ],
                
            ], 7));
            GENERATOR_MATRIX = Box::into_raw(matrix);

            let matrix = Box::new(BinMatrix::from_slices(&[
                &[ 81 ],
                &[ 18 ],
                &[ 84 ],
                &[ 24 ],
                &[ 96 ],
                
            ], 7));
            let matrix_t = Box::new(matrix.transposed());
            PARITY_MATRIX = Box::into_raw(matrix);
            PARITY_MATRIX_T = Box::into_raw(matrix_t);

            let mut map = Box::new(FnvHashMap::with_capacity_and_hasher(32, Default::default()));
            map.insert(0, &[0]);     // 0 => [0]
            map.insert(1, &[1]);     // 1 => [1]
            map.insert(2, &[2]);     // 2 => [2]
            map.insert(4, &[4]);     // 4 => [4]
            map.insert(8, &[8]);     // 8 => [8]
            map.insert(15, &[16]);     // 15 => [16]
            map.insert(16, &[32]);     // 16 => [32]
            map.insert(21, &[64]);     // 21 => [64]
            map.insert(3, &[3]);     // 3 => [3]
            map.insert(5, &[5]);     // 5 => [5]
            map.insert(9, &[9]);     // 9 => [9]
            map.insert(14, &[17]);     // 14 => [17]
            map.insert(17, &[33]);     // 17 => [33]
            map.insert(20, &[65]);     // 20 => [65]
            map.insert(6, &[6]);     // 6 => [6]
            map.insert(10, &[10]);     // 10 => [10]
            map.insert(13, &[18]);     // 13 => [18]
            map.insert(18, &[34]);     // 18 => [34]
            map.insert(23, &[66]);     // 23 => [66]
            map.insert(12, &[12]);     // 12 => [12]
            map.insert(11, &[20]);     // 11 => [20]
            map.insert(7, &[24]);     // 7 => [24]
            map.insert(24, &[40]);     // 24 => [40]
            map.insert(29, &[72]);     // 29 => [72]
            map.insert(31, &[48]);     // 31 => [48]
            map.insert(26, &[80]);     // 26 => [80]
            map.insert(19, &[35]);     // 19 => [35]
            map.insert(22, &[67]);     // 22 => [67]
            map.insert(25, &[41]);     // 25 => [41]
            map.insert(28, &[73]);     // 28 => [73]
            map.insert(30, &[49]);     // 30 => [49]
            map.insert(27, &[81]);     // 27 => [81]
            
            SYNDROME_MAP = Box::into_raw(map);
        }
    });
}

impl GuavaCode7_2 {
    fn parity_check_matrix_transposed(&self) -> &BinMatrix {
        init();
        unsafe {
            PARITY_MATRIX_T.as_ref().unwrap()
        }
    }
}

impl BinaryCode for GuavaCode7_2 {
    fn name(&self) -> String {
        "[7, 2] Guava code".to_owned()
    }

    fn length(&self) -> usize {
        7
    }

    fn dimension(&self) -> usize {
        2
    }

    fn generator_matrix(&self) -> &BinMatrix {
        init();
        unsafe {
            GENERATOR_MATRIX.as_ref().unwrap()
        }
    }

    fn parity_check_matrix(&self) -> &BinMatrix {
        init();
        unsafe {
            PARITY_MATRIX.as_ref().unwrap()
        }
    }

    fn decode_to_code(&self, c: &BinVector) -> Result<BinVector, &str> {
        init();
        let map = unsafe {
            SYNDROME_MAP.as_ref().unwrap()
        };
        debug_assert_eq!(c.len(), self.length(), "the length doesn't match the expected length (length of the code)");
        let he = c * self.parity_check_matrix_transposed();
        let mut error = BinVector::with_capacity(7);
        let stor = unsafe { error.get_storage_mut() };
        let errbytes = map[&he.as_u64()];
        debug_assert_eq!(errbytes.len(), 7 / 64 + if 7 % 64 != 0 { 1 } else { 0 });
        stor.clear();
        stor.extend_from_slice(&errbytes[..]);
        unsafe { error.set_len(7) };
        debug_assert_eq!(error.len(), self.length(), "internal: the error vector is of the wrong length");
        let result = c + &error;
        debug_assert_eq!(result.len(), self.length(), "internal: the result vector is of the wrong length");
        debug_assert_eq!((&result * self.parity_check_matrix_transposed()).count_ones(), 0);
        Ok(result)
    }

    fn decode_to_message(&self, c: &BinVector) -> Result<BinVector, &str> {
        
        let mut codeword = self.decode_to_code(c)?;
        codeword.truncate(2);
        Ok(codeword)
        
    }

    fn decode_slice(&self, c: &mut [u64]) {
        init();
        
        debug_assert_eq!(c[7 / 64] & !((1 << 7) - 1), 0, "this message has excess bits");

        let map = unsafe {
            SYNDROME_MAP.as_ref().unwrap()
        };
        let he = &BinMatrix::from_slices(&[&c[..]], self.length()) * self.parity_check_matrix_transposed();
        let error = map[unsafe { &he.get_word_unchecked(0, 0) }];
        c.iter_mut().zip(error.iter().copied()).for_each(|(sample, error)| *sample ^= error as u64);
    }

    
}

#[cfg(test)]
mod tests {
    use super::*;
    use m4ri_rust::friendly::BinVector;
    use crate::oracle::Sample;

    #[test]
    fn size() {
        let code = GuavaCode7_2.generator_matrix();
        assert_eq!(code.ncols(), 7);
        assert_eq!(code.nrows(), 2);
    }

    #[test]
    fn test_decode_sample() {
        let code = GuavaCode7_2;
        for _ in 0..1000 {
            // setup
            let vec = BinVector::random(code.length());
            let mut sample_a = Sample::from_binvector(&vec, false);
            let mut sample_b = Sample::from_binvector(&vec, true);
            
            let decoded_vec = code.decode_to_message(&vec).unwrap();
            println!("decoded_vec: {:?}", decoded_vec);

            // test vectors
            let decoded_vec_sample_a = Sample::from_binvector(&decoded_vec, false);
            let decoded_vec_sample_b = Sample::from_binvector(&decoded_vec, true);

            code.decode_sample(&mut sample_a);
            code.decode_sample(&mut sample_b);
            assert_eq!(sample_a.get_product(), false);
            assert_eq!(sample_b.get_product(), true);
            assert_eq!(sample_a, decoded_vec_sample_a);
            assert_eq!(sample_b, decoded_vec_sample_b);
        }
    }

    #[test]
    fn random_decode_tests() {

        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[false, false, true, false, false, false, false]);
            let codeword = BinVector::from_bools(&[false, false, false, false, false, false, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[true, true, true, true, true, false, true]);
            let codeword = BinVector::from_bools(&[true, true, true, true, true, false, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[true, false, true, false, true, true, true]);
            let codeword = BinVector::from_bools(&[true, false, true, false, false, true, true]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[true, false, true, false, false, true, true]);
            let codeword = BinVector::from_bools(&[true, false, true, false, false, true, true]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[true, true, true, false, true, false, true]);
            let codeword = BinVector::from_bools(&[true, true, true, true, true, false, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[true, false, true, true, false, false, false]);
            let codeword = BinVector::from_bools(&[true, true, true, true, true, false, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[false, false, true, true, true, true, true]);
            let codeword = BinVector::from_bools(&[false, true, false, true, true, true, true]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[false, false, true, false, true, false, false]);
            let codeword = BinVector::from_bools(&[false, false, false, false, false, false, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[false, true, true, true, false, true, true]);
            let codeword = BinVector::from_bools(&[false, true, false, true, true, true, true]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[false, false, false, false, false, false, true]);
            let codeword = BinVector::from_bools(&[false, false, false, false, false, false, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[true, false, false, false, false, false, true]);
            let codeword = BinVector::from_bools(&[false, false, false, false, false, false, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[false, true, false, true, true, true, true]);
            let codeword = BinVector::from_bools(&[false, true, false, true, true, true, true]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[false, false, true, true, true, false, true]);
            let codeword = BinVector::from_bools(&[true, true, true, true, true, false, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[true, true, false, true, false, false, true]);
            let codeword = BinVector::from_bools(&[false, true, false, true, true, true, true]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[true, false, true, false, true, true, true]);
            let codeword = BinVector::from_bools(&[true, false, true, false, false, true, true]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[true, false, true, true, true, false, true]);
            let codeword = BinVector::from_bools(&[true, true, true, true, true, false, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[true, false, false, true, false, true, false]);
            let codeword = BinVector::from_bools(&[false, false, false, false, false, false, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[true, true, false, false, true, true, true]);
            let codeword = BinVector::from_bools(&[false, true, false, true, true, true, true]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[false, false, false, false, true, true, false]);
            let codeword = BinVector::from_bools(&[false, false, false, false, false, false, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode7_2;
            let randvec = BinVector::from_bools(&[true, false, false, true, false, false, false]);
            let codeword = BinVector::from_bools(&[false, false, false, false, false, false, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
    }

    #[test]
    fn test_generator_representation() {
        init();
        let generator_matrix = unsafe { GENERATOR_MATRIX.as_ref().unwrap() };
        let first_row = generator_matrix.get_window(0, 0, 1, generator_matrix.ncols());
        let vector = BinVector::from_bools(&[ true, false, true, false, false, true, true ]);
        assert_eq!(vector, first_row.as_vector());
    }

}
