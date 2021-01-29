use std::boxed::Box;
use std::default::Default;
use std::sync::Once;

use fnv::FnvHashMap;

use m4ri_rust::friendly::BinMatrix;
use m4ri_rust::friendly::BinVector;

use crate::codes::BinaryCode;

/// ``[10, 5]`` Guava code
///
/// Best code found from the GUAVA database version 3.15
///
/// Decodes using Syndrome decoding
#[derive(Clone, Serialize)]
pub struct GuavaCode10_5;

static INIT: Once = Once::new();
static mut GENERATOR_MATRIX: *const BinMatrix = 0 as *const BinMatrix;
static mut PARITY_MATRIX: *const BinMatrix = 0 as *const BinMatrix;
static mut PARITY_MATRIX_T: *const BinMatrix = 0 as *const BinMatrix;
static mut SYNDROME_MAP: *const FnvHashMap<u64, &'static [usize; 1]> = 0 as *const FnvHashMap<u64, &'static [usize; 1]>;

fn init() {
    INIT.call_once(|| {
        unsafe {
            let matrix = Box::new(BinMatrix::from_slices(&[
                &[ 801 ],
                &[ 450 ],
                &[ 708 ],
                &[ 840 ],
                &[ 912 ],
                
            ], 10));
            GENERATOR_MATRIX = Box::into_raw(matrix);

            let matrix = Box::new(BinMatrix::from_slices(&[
                &[ 33 ],
                &[ 482 ],
                &[ 740 ],
                &[ 840 ],
                &[ 912 ],
                
            ], 10));
            let matrix_t = Box::new(matrix.transposed());
            PARITY_MATRIX = Box::into_raw(matrix);
            PARITY_MATRIX_T = Box::into_raw(matrix_t);

            let mut map = Box::new(FnvHashMap::with_capacity_and_hasher(32, Default::default()));
            map.insert(0, &[0]);     // 0 => [0]
            map.insert(1, &[1]);     // 1 => [1]
            map.insert(2, &[2]);     // 2 => [2]
            map.insert(4, &[4]);     // 4 => [4]
            map.insert(8, &[8]);     // 8 => [8]
            map.insert(16, &[16]);     // 16 => [16]
            map.insert(7, &[32]);     // 7 => [32]
            map.insert(14, &[64]);     // 14 => [64]
            map.insert(22, &[128]);     // 22 => [128]
            map.insert(26, &[256]);     // 26 => [256]
            map.insert(28, &[512]);     // 28 => [512]
            map.insert(3, &[3]);     // 3 => [3]
            map.insert(5, &[5]);     // 5 => [5]
            map.insert(9, &[9]);     // 9 => [9]
            map.insert(17, &[17]);     // 17 => [17]
            map.insert(6, &[33]);     // 6 => [33]
            map.insert(15, &[65]);     // 15 => [65]
            map.insert(23, &[129]);     // 23 => [129]
            map.insert(27, &[257]);     // 27 => [257]
            map.insert(29, &[513]);     // 29 => [513]
            map.insert(10, &[10]);     // 10 => [10]
            map.insert(18, &[18]);     // 18 => [18]
            map.insert(12, &[66]);     // 12 => [66]
            map.insert(20, &[130]);     // 20 => [130]
            map.insert(24, &[258]);     // 24 => [258]
            map.insert(30, &[514]);     // 30 => [514]
            map.insert(11, &[11]);     // 11 => [11]
            map.insert(19, &[19]);     // 19 => [19]
            map.insert(13, &[67]);     // 13 => [67]
            map.insert(21, &[131]);     // 21 => [131]
            map.insert(25, &[259]);     // 25 => [259]
            map.insert(31, &[515]);     // 31 => [515]
            
            SYNDROME_MAP = Box::into_raw(map);
        }
    });
}

impl GuavaCode10_5 {
    fn parity_check_matrix_transposed(&self) -> &BinMatrix {
        init();
        unsafe {
            PARITY_MATRIX_T.as_ref().unwrap()
        }
    }
}

impl BinaryCode for GuavaCode10_5 {
    fn name(&self) -> String {
        "[10, 5] Guava code".to_owned()
    }

    fn length(&self) -> usize {
        10
    }

    fn dimension(&self) -> usize {
        5
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
        let mut error = BinVector::with_capacity(10);
        let stor = unsafe { error.get_storage_mut() };
        let errbytes = map[&he.as_u64()];
        debug_assert_eq!(errbytes.len(), 10 / 64 + if 10 % 64 != 0 { 1 } else { 0 });
        stor.clear();
        stor.extend_from_slice(&errbytes[..]);
        unsafe { error.set_len(10) };
        debug_assert_eq!(error.len(), self.length(), "internal: the error vector is of the wrong length");
        let result = c + &error;
        debug_assert_eq!(result.len(), self.length(), "internal: the result vector is of the wrong length");
        debug_assert_eq!((&result * self.parity_check_matrix_transposed()).count_ones(), 0);
        Ok(result)
    }

    fn decode_to_message(&self, c: &BinVector) -> Result<BinVector, &str> {
        
        let mut codeword = self.decode_to_code(c)?;
        codeword.truncate(5);
        Ok(codeword)
        
    }

    fn decode_slice(&self, c: &mut [u64]) {
        init();
        
        debug_assert_eq!(c[10 / 64] & !((1 << 10) - 1), 0, "this message has excess bits");

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
        let code = GuavaCode10_5.generator_matrix();
        assert_eq!(code.ncols(), 10);
        assert_eq!(code.nrows(), 5);
    }

    #[test]
    fn test_decode_sample() {
        let code = GuavaCode10_5;
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
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[false, true, false, false, false, false, false, true, false, true]);
            let codeword = BinVector::from_bools(&[false, true, false, true, false, false, false, true, false, true]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[true, false, false, true, false, true, true, true, true, true]);
            let codeword = BinVector::from_bools(&[true, false, false, true, true, true, true, true, true, true]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[false, true, false, true, false, true, false, false, true, false]);
            let codeword = BinVector::from_bools(&[true, true, false, true, false, true, false, true, true, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[true, false, false, false, true, false, true, true, true, false]);
            let codeword = BinVector::from_bools(&[false, true, false, false, false, false, true, true, true, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[false, true, false, false, true, true, false, false, false, false]);
            let codeword = BinVector::from_bools(&[true, false, false, false, true, true, false, true, false, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[false, false, true, false, false, false, true, true, true, false]);
            let codeword = BinVector::from_bools(&[true, false, true, false, false, true, true, true, true, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[false, false, false, false, true, false, true, false, false, false]);
            let codeword = BinVector::from_bools(&[false, true, false, false, true, false, true, false, false, true]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[false, true, true, true, true, false, false, false, true, true]);
            let codeword = BinVector::from_bools(&[false, false, true, true, true, false, false, false, false, true]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[false, false, true, true, true, false, true, false, false, false]);
            let codeword = BinVector::from_bools(&[false, true, true, true, false, false, true, false, false, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[false, false, true, false, true, true, true, true, true, true]);
            let codeword = BinVector::from_bools(&[true, true, true, false, true, true, false, true, true, true]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[false, false, false, false, true, false, true, false, false, false]);
            let codeword = BinVector::from_bools(&[false, true, false, false, true, false, true, false, false, true]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[false, false, true, false, true, false, false, true, true, false]);
            let codeword = BinVector::from_bools(&[false, true, true, false, true, false, false, true, false, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[true, false, false, true, false, true, false, false, false, true]);
            let codeword = BinVector::from_bools(&[true, true, false, true, true, true, false, false, false, true]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[false, false, false, true, true, true, false, true, true, false]);
            let codeword = BinVector::from_bools(&[true, true, false, true, false, true, false, true, true, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[true, false, false, true, true, false, false, false, false, true]);
            let codeword = BinVector::from_bools(&[false, false, true, true, true, false, false, false, false, true]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[true, false, false, true, true, false, false, false, true, false]);
            let codeword = BinVector::from_bools(&[false, true, false, true, true, false, false, false, true, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[false, false, true, false, false, true, false, false, true, true]);
            let codeword = BinVector::from_bools(&[true, false, false, false, false, true, false, false, true, true]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[true, false, false, false, true, false, false, true, false, true]);
            let codeword = BinVector::from_bools(&[false, false, false, false, true, false, false, true, true, true]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[false, false, true, false, true, true, true, false, true, false]);
            let codeword = BinVector::from_bools(&[false, false, true, false, true, false, true, false, true, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
        {
            let code = GuavaCode10_5;
            let randvec = BinVector::from_bools(&[false, false, true, true, false, false, true, true, true, false]);
            let codeword = BinVector::from_bools(&[false, false, true, true, false, false, false, true, true, false]);
            assert_eq!(code.decode_to_code(&randvec), Ok(codeword));
        }
        
    }

    #[test]
    fn test_generator_representation() {
        init();
        let generator_matrix = unsafe { GENERATOR_MATRIX.as_ref().unwrap() };
        let first_row = generator_matrix.get_window(0, 0, 1, generator_matrix.ncols());
        let vector = BinVector::from_bools(&[ true, false, false, false, false, true, false, false, true, true ]);
        assert_eq!(vector, first_row.as_vector());
    }

}