pub(crate) enum AlignedI16Data<'a> {
    AlreadyAligned(&'a [u8]),
    Copied(Vec<i16>),
}

impl<'a> AlignedI16Data<'a> {
    pub fn new(src: &'a [u8]) -> Self {
        let bytes_ptr = src.as_ptr();

        // If `src`'s memory is not 16-bit aligned, create a new 16-bit aligned
        // memory area and copy the contents of `src` to it. Otherwise, simply
        // keep the original slice.
        if bytes_ptr as usize % 2 == 0 {
            AlignedI16Data::AlreadyAligned(src)
        } else {
            let num_bytes = src.len();
            let num_words = num_bytes / 2;

            let mut words = Vec::<i16>::with_capacity(num_words);
            let words_ptr = words.as_mut_ptr() as *mut u8;

            unsafe {
                std::ptr::copy_nonoverlapping(bytes_ptr, words_ptr, num_bytes);
                words.set_len(num_words);
            }

            AlignedI16Data::Copied(words)
        }
    }

    pub fn as_ptr(&self) -> *const i16 {
        match self {
            AlignedI16Data::AlreadyAligned(d) => d.as_ptr() as *const i16,
            AlignedI16Data::Copied(d) => d.as_ptr(),
        }
    }
}
