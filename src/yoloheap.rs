

// This is a greedy heap, meaning it will search for the first free block and split it if possible.
// If it is not possible we just return that block. This is a simpler implementation of a heap, its good for my project.
use crate::byte_utils::BytesConverter;
use crate::yoloheap::constants::*;

#[allow(dead_code)]
pub mod constants {
    pub const HEADER_SIZE: usize = 1;                                // Size of header in bytes.
    pub const FOOTER_SIZE: usize = HEADER_SIZE;                      // Size of footer in bytes.
    pub const HEADER_FOOTER_SIZE: usize = HEADER_SIZE + FOOTER_SIZE; // Size of header and footer.
    pub const MINIMUM_BLOCK_SIZE: usize = 4;                         // The minimum size a block can be.
    pub const MINIMUM_ALLOCATED_SIZE: usize = 2;                     // The minimum size that a user can ask the be allocated.
    pub const MAX_BLOCK_SIZE: u8 = u8::MAX - PB_B_ALLOCED;           // The maximum size a block can be.
    pub const B_ALLOCED: u8 = 1;                                     // Value in header/footer if block is allocated.
    pub const PB_ALLOCED: u8 = 2;                                    // Value in header/footer if previous block is allocated.
    pub const PB_B_ALLOCED: u8 = 3;                                  // Value in header/footer if block and previous block is allocated.
    pub const BOTTOM_OF_HEAP: usize = 0;                             // Index flag used to check if it is the bottom of the heap.
    pub const BLOCK_SIZE_MASK: u8 = 0xFC;                            // Mask for bitwise anding a header/footer to get the block size.
    pub const BLOCK_ALLOC_VALS_MASK: u8 = 0x3;                       // Mask for bitwise anding a header/footer to get the allocation values:
    pub const DEFAULT_ALLOC: u8 = 0;                                 // Used when a header/footer is being created with 0 in alloc and prev alloc.
}


#[derive(Copy, Clone, Debug, PartialEq)]
struct _Header {
    pub block_size:   usize,
    pub block_alloc:  u8,
    pub pblock_alloc: u8,
}

type _Footer = _Header;

impl _Header {

    // Creates a _new header/footer, makes sure to check that invariants hold:
    //      1: Block size is at least 4, and less that 255 (this is always true due to type usize)
    //      2: Block and Pblock alloc is either 0 or 1.
    fn _new(block_size: usize, block_alloc: u8, pblock_alloc: u8) -> Self {

        // Important invariant for the header.
        assert!(block_size >= 4); 
        assert!(block_size % 4 == 0);
        assert!(block_alloc == 0 || block_alloc == 1);
        assert!(pblock_alloc == 0 || pblock_alloc == 1);

        Self {
            block_size,
            block_alloc,
            pblock_alloc,
        }
    }

    // Basic minimun size block, only used in edge cases.
    fn _default() -> Self {
        _Header::_new(MINIMUM_BLOCK_SIZE, DEFAULT_ALLOC, DEFAULT_ALLOC)
    }

    // Returns the block size from a header/footer.
    fn _get_bsize_from_byte(byte: &u8) -> usize {
        (byte & BLOCK_SIZE_MASK) as usize
    }

    // Get block allocation value.
    fn _get_alloc_val_from_byte(byte: &u8) -> u8 {
        byte & 1
    }

    // Get the previous block allocation value.
    fn _get_prev_alloc_val_from_byte(byte: &u8) -> u8 {
        (byte & 2) >> 1
    }

    // Returns the combined allocation value.
    fn _get_alloc_vals_from_byte(byte: &u8) -> u8 {
        byte & BLOCK_ALLOC_VALS_MASK
    }

    
    // The header is 1 byte: 
    // -- Least significant bit is block_alloc
    // -- Second bit from LS side is pblock_alloc
    // -- Size is represented as the whole byte, but the two LSBytes are flipped to 0
    //    max block size is therefore uint8_max 255 - 3 => 252.
    pub fn _to_byte(&self) -> u8 {
        self.block_size as u8 + self.block_alloc + (self.pblock_alloc << 1)
    }

    // Construct a header/footer from a byte.
    // Does not check invariants, since it calls _Header::_new().
    pub fn _from_byte(byte: &u8) -> Self {

        let size         = _Header::_get_bsize_from_byte(byte);
        let block_alloc  = _Header::_get_alloc_val_from_byte(byte);
        let pblock_alloc = _Header::_get_prev_alloc_val_from_byte(byte);

        _Header::_new(size, block_alloc, pblock_alloc)
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct Heap {
    pub heap: Vec<u8>,
    pub size: usize,
}

impl Heap {
    fn _print_heap(&self) {
        for i in &self.heap {
            println!("{i}");
        }
    }
    

    // Creates a _new heap of size: size. 
    // Initializes a header and footer so the heap is one large free block.
    // Asserts that size i at least 4.
    
    #[allow(dead_code)] 
    pub fn new_heap(size: usize) -> Self {
        assert!(size >= 4); // Size invariant.
        
        // Cocky move, but if the size given is not a multiple of 4, we round up to closest multiple of 4.
        // This could be anoying to use otherwise. Hacker function from chat.
        let mut _size = size;

        // Hacker function that changes size to closest (roof) multiple of 4.
        if size % 4 == 0 { } else { _size = (size + 3) & !3 }

        let init_h = _Header::_new(_size, 0, 0);
        let init_f = _Footer::_new(_size, 0, 0);
        let mut h = vec![0; _size];
        
        h[BOTTOM_OF_HEAP] = init_h._to_byte();
        h[_size-1]        = init_f._to_byte();

        Self {
            heap: h,
            size: _size,
        }  
    }

    // Writes a header and footer from index.
    // -- h: _Header to be written as header and footer.
    // -- i: Index to write the _new header.
    fn _write_header_footer(&mut self, h: &_Header, i: usize) {
        // If the block goes out of bounds, then something is wrong so we panic.
        assert!(i + h.block_size - FOOTER_SIZE < self.size);

        // We know that either it is the first block or a block above.
        // This panics if the first block is corrupt.
        println!("I: {i}");
        assert!(i == BOTTOM_OF_HEAP || i >= MINIMUM_BLOCK_SIZE);

        // Write the header to i and footer to i + size - footer_size.
        self.heap[i] = h._to_byte();
        self.heap[i + h.block_size - FOOTER_SIZE] = h._to_byte();
    }
    
    // Returns the index the free blocks header, of at least size 'size'.
    // If the current block size minus size + header + footer, is greater then minimum block size
    // Then we create a _new block of size 'size' + header + footer.
    // Returns 'None' if there are no free blocks.
    // This function always makes sure that the multiple of 4 invariant. 
    fn _find_free_block(&mut self, size: usize) -> Option<(usize, usize)> {

        // Panics if size if size is below minimum allowed size or if it is larger then the heap - header footer.
        // In all other cases there could be space.
        assert!(size >= 1);

        let mut i = 0;
        while i < self.size {

            // Gets the current header from bytes.
            let curr_header = _Header::_from_byte(&self.heap[i]);

            // We now have to check if a multiple of 4 is possible.
            // This rounds the smallest possible size to the nearest multiple of 4.
            let minimum_size = (size + HEADER_FOOTER_SIZE + 3) & !3;

            // We always assume that current block is of size multiple of 4.
            // If the size is smaller or equal the current block size and it is not allocated we good.
            if curr_header.block_size >= minimum_size && curr_header.block_alloc == 0 {

                // Need to check if If we can split the block.  
                // We check if the size if not too small.
                // We also check if the rest block is a multiple of 4.
                // If we can then we do it.
                if curr_header.block_size - minimum_size >= MINIMUM_BLOCK_SIZE && (curr_header.block_size - minimum_size) % 4 == 0 {

                    // The _new block size is just size + header + footer.
                    // The _new alloc is just if previous is alloced.
                    let new_size   = minimum_size;
                    let new_header = _Header::_new(new_size, 0, curr_header.pblock_alloc);
 
                    // The rest-block is just size of curr_block - new_size.
                    // Same show here, we already now that the block will not be allocated.
                    // We also know that previous is not allocated since this is ALWAYS above a free block.
                    let rest_size   = curr_header.block_size - new_size;
                    let rest_header = _Header::_new(rest_size, DEFAULT_ALLOC, DEFAULT_ALLOC);

                    // we insert the _new header and footer to create our _new block.
                    // And we then update the footer for the old block, as well as insert af _new header above the _new block.
                    //self._print_heap();
                    self._write_header_footer(&new_header, i);
                    self._write_header_footer(&rest_header, i + new_size);

                    // We just return the index here, as the heap is updated
                    return Some((i, new_size));
                }
                // If block is not allocated but cant be split, we just return it.
                return Some((i, curr_header.block_size));
            }
            // Look at the next block.
            i += curr_header.block_size;
        }
        // If no block is available for that size, return None.
        None  
    }

    // Allocates a block of (hopefully) size 'size'.
    
    #[allow(dead_code)]
    pub fn allocate(&mut self, size: usize) -> Option<usize> {


        // If size is below 2, then the block would not be MINIMUM_BLOCK_SIZE and we panic.
        assert!(size >= MINIMUM_ALLOCATED_SIZE);

        let good_block = self._find_free_block(size);
        
        match good_block {
            Some((i, bsize)) => {

                // Set the current block as allocated
                let mut curr_h = _Header::_from_byte(&self.heap[i]);
                curr_h.block_alloc = 1;

                // Check if there is a previous block, then set current block to pblock_alloc.
                if i > BOTTOM_OF_HEAP {
                    // THOUGHT: Since we use immidiate coalecing, if it is not the bottom block, prev would always be allocated?
                    // Now we must toggle this to prev alloc, if previous is alloced.
                    let prev_f = _Footer::_from_byte(&self.heap[i-FOOTER_SIZE]);  
                    if prev_f.block_alloc == 1 {
                        // Then set curr_h prev to alloc.
                        curr_h.pblock_alloc = 1;
                    }
                }
                // Now we just write the _new header/footer for the allocated block.
                self._write_header_footer(&curr_h, i);
    
                // Now, set pblock_alloc in the next block's header if it exists
                let next_block_start = i + bsize;

                if next_block_start < self.size {
                    let mut next_h = _Header::_from_byte(&self.heap[next_block_start]);
                    
                    // We know that it was not marked to begin with, otherwise it is corrupt.
                    assert_eq!(next_h.pblock_alloc, 0);
                    next_h.pblock_alloc = 1; // Mark previous as allocated
                    self._write_header_footer(&next_h, next_block_start); // Write updated header and footer
                    
                }
                
                // Return the start of allocated data (right after the header)
                Some(i + HEADER_SIZE)
            }
            // Find free block returned None, so we do the same since there is no free block to match our demand.
            None => None,
        }
    }


    // Coalesses the heap, always called after free().
    // -- Ptr: pointer to the header of the block that was freed.
    fn _coalesse(&mut self, ptr: usize) {
        // We know _coalesse is called after every free.
        // This means that there are at most 1 block below and one above the block that was freed.

        // TODO: Invariants.

        // The _new size is at least the size of the current block.
        let curr_h = _Header::_from_byte(&self.heap[ptr-HEADER_SIZE]);
        let mut final_size = curr_h.block_size;
        let mut final_prev_alloc = 0;
        let mut final_header_index = ptr - HEADER_SIZE;

        // Initially we assume neither way is possible.
        let mut up_flag   = 0;
        let mut down_flag = 0;
        
        // If it is not the bottom most block, we know that a below block exists.
        if ptr != BOTTOM_OF_HEAP + HEADER_SIZE { down_flag = 1 }

        // To check if up is possible we need the header that ptr is pointing to.
        if curr_h.block_size + ptr < self.size { up_flag = 1 }

        // Start by checking the below block.
        // If below block is not allocated we update the final size.
        // We set the final previous alloc to the same as the below block.
        // And the header index (for _write_header_footer) is the same as the header index for below block.
        if down_flag == 1 {
            let below_f = _Footer::_from_byte(&self.heap[ptr-HEADER_FOOTER_SIZE]);
            if below_f.block_alloc == 0 {
                final_size += below_f.block_size;
                final_prev_alloc = below_f.pblock_alloc;
                final_header_index -= below_f.block_size;
            }
        }

        // IMPORTANT: We DO NOT update the pblock_alloc value of above block before overwriting.
        // Now we check the above block.
        println!("CALSCS!!: {}", &self.heap[ptr+curr_h.block_size-HEADER_SIZE]);
        self._print_heap();
        if up_flag == 1 {
            let above_h = _Header::_from_byte(&self.heap[ptr+curr_h.block_size-HEADER_SIZE]);
            if above_h.block_alloc == 0 {
                final_size += above_h.block_size;
            }   
        }
        
        self._print_heap();
        // Write to the above allocated block, if it exists, that prev is now free.
        // If it was not the final block we update.
        if final_header_index + final_size < self.size {
            let mut above_above_h = _Header::_from_byte(&self.heap[final_header_index + final_size]);
            above_above_h.pblock_alloc = 0;
            self._write_header_footer(&above_above_h, final_header_index + final_size);
        }

        // Now we have the header index, size and the allocation values.
        // We can then construct a header and write the _new block.
        let final_header = _Header::_new(final_size, DEFAULT_ALLOC, final_prev_alloc);
        self._write_header_footer(&final_header, final_header_index);

        // Does not return anything. PRAY!

    }

    
    // TODO: REMove assets
    #[allow(dead_code)]
    pub fn free(&mut self, ptr: usize) -> Result<(), &'static str> {

        if ptr >= self.size {
            return Err("Error: Pointer is greater then the size of the heap.");
        }

        if ptr < HEADER_SIZE {
            return Err("Error trying to free pointer which is at bottom of heap <0>, this is not allowed");
        }

        // Check that the pointer is valid.
        // Has to be a multiple of 4 + HEADER_FOOTER_SIZE
        if ptr % 4 != HEADER_SIZE {
            return Err("Error: Invalid pointer given.");
        }

        // Gets the header of the block.
        let mut header = _Header::_from_byte(&self.heap[ptr-HEADER_SIZE]);
        let bsize = header.block_size;
        let footer = _Footer::_from_byte(&self.heap[ptr+bsize-HEADER_FOOTER_SIZE]);
        
        // The block must be allocated.
        if header.block_alloc != B_ALLOCED {
            return Err("Error: Trying to free a block that is not allocated.");
        }

        // Check if size is within bounds.
        if ptr-HEADER_SIZE+bsize > self.size {
            return Err("Error: Block goes out of bounds, possible wrong header.");
        }

        // check if footer is identical.
        if header != footer {
            return Err("Error: Header and footer does not match.");
        }

        // Set block to not allocated. 
        // Write it back to the heap.
        header.block_alloc = 0;
        self._write_header_footer(&header, ptr-HEADER_SIZE);


        // Coallesse the heap, hehe hope it works!
        self._coalesse(ptr);
        Ok(())
    }

    // HELPER
    // Takes a ptr to the first element in a block and returns the available space for data (blocksize - HEADER_FOOTER_SIZE),
    // This makes no assumptions about the state of the block.
    fn _check_bsize(&self, ptr: &usize) -> usize {

        let h = _Header::_from_byte(&self.heap[ptr-HEADER_SIZE]);
        h.block_size - HEADER_FOOTER_SIZE

    }

    // Writes 'n' bytes from src to self starting at ptr + offset.
    
    #[allow(dead_code)]
    pub fn write_bytes(&mut self, ptr: &usize, src: &impl BytesConverter, n: usize, offset: usize) -> Result<(), &'static str> {
        // Gets the number of spaces available for writing.
        let bbytes = self._check_bsize(ptr);

        // If we try to write too many bytes or too little, return err.
        if bbytes < n {
            return Err("Error: Trying to write more bytes that the size of the block.");
        }

        if n == 0 {
            return Err("Error: Trying to write 0 bytes.");
        }

        // We know block is allocated, and that we have enough space to write, so we write.
        // We copy the src into out heap and return.
        unsafe {
            std::ptr::copy_nonoverlapping(src.to_bytes().as_ptr(), &mut self.heap[*ptr+offset] as *mut u8, n);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_header_from_bytes() {
        let h_n_n = _Header::_new(12, 0, 0);
        let h_y_n = _Header::_new(12, 1, 0);
        let h_n_y = _Header::_new(12, 0, 1);
        let h_y_y = _Header::_new(12, 1, 1);

        assert_eq!(h_n_n, _Header::_from_byte(&(12)));
        assert_eq!(h_y_n, _Header::_from_byte(&(12 + B_ALLOCED)));
        assert_eq!(h_n_y, _Header::_from_byte(&(12 + PB_ALLOCED)));
        assert_eq!(h_y_y, _Header::_from_byte(&(12 + PB_B_ALLOCED)));
    }

    #[test]
    fn test_header_to_bytes() {
        let h_n_n = _Header::_new(12, 0, 0);
        let h_y_n = _Header::_new(12, 1, 0);
        let h_n_y = _Header::_new(12, 0, 1);
        let h_y_y = _Header::_new(12, 1, 1);

        assert_eq!(h_n_n._to_byte(), 12);
        assert_eq!(h_y_n._to_byte(), 12 + B_ALLOCED);
        assert_eq!(h_n_y._to_byte(), 12 + PB_ALLOCED);
        assert_eq!(h_y_y._to_byte(), 12 + PB_B_ALLOCED);
    }

    #[test]
    fn test_new_heap() {
        let heap = Heap::new_heap(32);
        assert_eq!(heap.size, 32);
    }

    #[test]
    fn test_new_heap_round() {
        let heap = Heap::new_heap(29);
        assert_eq!(heap.size, 32);
    }

    #[test]
    #[should_panic]
    fn test_heap_size_invariant() {
        let _ = Heap::new_heap(3);
    }

    #[test]
    fn test_right_init_header_footer() {
        let heap = Heap::new_heap(32);
        let h = _Header::_from_byte(&heap.heap[BOTTOM_OF_HEAP]);
        let f = _Footer::_from_byte(&heap.heap[heap.size-FOOTER_SIZE]);
        assert_eq!(h, f);
    }   

    #[test]
    fn test_write_header_footer() {
        let mut heap = Heap::new_heap(32);
        let h = _Header::_new(32, 1, 0);
        let f = _Footer::_from_byte(&heap.heap[heap.size-FOOTER_SIZE]);
        assert_ne!(h, f);

        heap._write_header_footer(&h, 0);
        let f1 = _Footer::_from_byte(&heap.heap[heap.size-FOOTER_SIZE]);
        let h1 = _Header::_from_byte(&heap.heap[BOTTOM_OF_HEAP]);
        assert_eq!(h, f1);
        assert_eq!(h1, f1);
    }

    #[test]
    fn test_find_free_block_init_nomodify() {
        let heap_size = 32;
        let mut heap = Heap::new_heap(heap_size);
        let fb_not = heap._find_free_block(heap_size);
        assert_eq!(fb_not, None);

        let fb_yes = heap._find_free_block(28);
        assert_eq!(fb_yes, Some((0, heap_size)));

    }

    #[test]
    fn test_find_free_block_squish() {
        let heap_size = 32;
        let mut heap = Heap::new_heap(heap_size);
        let fb_not = heap._find_free_block(10); // Same as call with 10.
        assert_eq!(fb_not, Some((0, 10+HEADER_FOOTER_SIZE))); // +2 Because 8+header footer size gets rounded to 12.
        assert_eq!(heap.heap[10+HEADER_FOOTER_SIZE] as usize, heap_size-(10+HEADER_FOOTER_SIZE));
        assert_eq!(heap.heap[heap_size - FOOTER_SIZE] as usize, heap_size-(10+HEADER_FOOTER_SIZE));
    }   

    #[test]
    fn test_find_free_block_squish_largest() {
        let heap_size = 32;
        let mut heap = Heap::new_heap(heap_size);
        let fb_not = heap._find_free_block(24);
        assert_eq!(fb_not, Some((0, 24+2+HEADER_FOOTER_SIZE))); // Rounding
        assert_eq!(heap.heap[24+2+HEADER_FOOTER_SIZE] as usize, heap_size-(24+2+HEADER_FOOTER_SIZE));
        assert_eq!(heap.heap[heap_size - FOOTER_SIZE] as usize, heap_size-(24+2+HEADER_FOOTER_SIZE));
    }   

    #[test]
    fn test_allocate_simple() {
        let heap_size = 12;
        let mut heap = Heap::new_heap(heap_size);
        let mut old_heap = heap.clone();

        let _ = old_heap._find_free_block(4);
        let ptr = heap.allocate(4);

        // All elements are initialized as 0, should point to first element.
        assert_eq!(heap.heap[ptr.unwrap()], 0);

        let alloc_h = _Header::_from_byte(&heap.heap[BOTTOM_OF_HEAP]);
        let noalloc_h = _Header::_from_byte(&old_heap.heap[BOTTOM_OF_HEAP]);

        assert_eq!(alloc_h.block_size, noalloc_h.block_size);
        assert_eq!(alloc_h.pblock_alloc, noalloc_h.pblock_alloc);
        assert_ne!(alloc_h.block_alloc, noalloc_h.block_alloc);
    }

    #[test]
    fn test_allocate_no_space() {
        let heap_size = 12;
        let mut heap = Heap::new_heap(heap_size);
        let old_heap = heap.clone();
        let ptr = heap.allocate(12);

        assert_eq!(ptr, None);
        assert_eq!(heap.heap, old_heap.heap);
    }

    #[test]
    fn test_multi_allocate() {
        let heap_size = 20; 
        let mut heap = Heap::new_heap(heap_size);
        let ptr1 = heap.allocate(4).unwrap();
        let ptr2 = heap.allocate(6).unwrap(); // Becomes 8 with rounding

        assert_eq!(heap.heap[ptr1-HEADER_SIZE], 8 + 1); // Size 8 with rounding + 1 for allocation
        assert_eq!(heap.heap[ptr2-HEADER_SIZE], 8 + 1 + 2); // We ask for 6, round to 8 (+2), alloc +1 and prev alloc +2.

        let p_alloc_h = _Header::_from_byte(&heap.heap[ptr2-HEADER_SIZE]);
        assert_eq!(p_alloc_h.block_alloc, 1);
        assert_eq!(p_alloc_h.pblock_alloc, 1);
    }

    #[test]
    fn test_multi_allocate_nospace() {
        let heap_size = 20;
        let mut heap = Heap::new_heap(heap_size);
        let ptr1 = heap.allocate(4).unwrap();
        assert_eq!(heap.heap[ptr1-HEADER_SIZE], 8 + 1); // 8 + 1 for size rounded and allocated +1.

        let ptr2 = heap.allocate(12);
        assert!(ptr2.is_none());
    }

    #[test]
    fn test_free_empty() {
        // We always allow a free except if the pointer is out of bounds.
    }

    #[test]
    fn test_write_exact() {
        let heap_size = 20; 
        let data = vec![1, 2, 3, 4];
        let data_size = data.len();
        
        let mut heap = Heap::new_heap(heap_size);
        let ptr = heap.allocate(4).unwrap();
        let wres = heap.write_bytes(&ptr, &data, data_size, 0);

        assert!(wres.is_ok());
        assert_eq!(heap.heap[ptr], data[0]);
        assert_eq!(heap.heap[ptr+1], data[1]);
        assert_eq!(heap.heap[ptr+2], data[2]);
        assert_eq!(heap.heap[ptr+3], data[3]);

    }

    #[test]
    fn test_write_less_then_block() {
        let heap_size = 20;
        let data = vec![1, 2, 3, 4];
        let data_size = data.len();
        
        let mut heap = Heap::new_heap(heap_size);
        let ptr = heap.allocate(6).unwrap();
        let wres = heap.write_bytes(&ptr, &data, data_size, 0);
        heap._print_heap();
        assert!(wres.is_ok());
        assert_eq!(heap.heap[ptr], data[0]);
        assert_eq!(heap.heap[ptr+1], data[1]);
        assert_eq!(heap.heap[ptr+2], data[2]);
        assert_eq!(heap.heap[ptr+3], data[3]);
    }

    #[test]
    fn test_write_more_then_block() {
        let heap_size = 20;
        let data = vec![1, 2, 3, 4, 5];
        let data_size = data.len();
        
        let mut heap = Heap::new_heap(heap_size);
        
        let ptr = heap.allocate(2).unwrap();
        
        let wres = heap.write_bytes(&ptr, &data, data_size, 0);

        assert!(wres.is_err());
    }

    #[test]
    fn test_write_multiple() {
        let heap_size = 20;
        let data = vec![1, 2, 3];
        
        let data_size = data.len();
        
        let mut heap = Heap::new_heap(heap_size);
        let ptr = heap.allocate(6).unwrap();

        let wres = heap.write_bytes(&ptr, &data, data_size, 0);
        let wres_ = heap.write_bytes(&ptr, &data, data_size, data_size);

        assert!(wres.is_ok());
        assert!(wres_.is_ok());

        assert_eq!(heap.heap[ptr],   data[0]);
        assert_eq!(heap.heap[ptr+1], data[1]);
        assert_eq!(heap.heap[ptr+2], data[2]);

        assert_eq!(heap.heap[ptr+3], data[0]);
        assert_eq!(heap.heap[ptr+4], data[1]);
        assert_eq!(heap.heap[ptr+5], data[2]);
    }

    #[test] 
    fn write_data_zsize() {
        let heap_size = 20;
        let data = vec![];
        let data_size = data.len();
        
        let mut heap = Heap::new_heap(heap_size);
        let ptr = heap.allocate(4).unwrap();
        let wres = heap.write_bytes(&ptr, &data, data_size, 0);

        assert!(wres.is_err());
    }

    #[test]
    fn test_free_single_block() {
        let mut heap = Heap::new_heap(16);
        let ptr = heap.allocate(4).unwrap(); // block size is 8
        let ptrr = ptr;

        heap.free(ptr).expect("Failed to free block");
        heap._print_heap();
        // After freeing, check if the block is correctly marked as free
        let header = _Header::_from_byte(&heap.heap[ptrr - HEADER_SIZE]);
        assert_eq!(header.block_alloc, 0, "Block should be marked as free");
    }

    #[test]
    fn test_free_with_coalescing_adjacent_blocks() {
        let mut heap = Heap::new_heap(32);

        let ptr1 = heap.allocate(4).expect("Failed to allocate first block");
        let ptr2 = heap.allocate(4).expect("Failed to allocate second block");

        heap.free(ptr1).expect("Failed to free first block");
        heap.free(ptr2).expect("Failed to free second block");

        // Verify coalescing: the whole space should be free now
        let header = _Header::_from_byte(&heap.heap[BOTTOM_OF_HEAP]);
        assert_eq!(header.block_size, 32, "Blocks should be coalesced into a single large free block");
        assert_eq!(header.block_alloc, 0, "Coalesced block should be marked as free");
        assert_eq!(header.pblock_alloc, 0, "Should also be zero, might as well check for debugging.");
    }

    #[test]
    fn test_free_non_adjacent_blocks_no_coalescing() {
        let mut heap = Heap::new_heap(32);

        // Allocate three blocks
        let ptr1 = heap.allocate(4).expect("Failed to allocate first block");
        let _ptr2 = heap.allocate(4).expect("Failed to allocate second block");
        let ptr3 = heap.allocate(4).expect("Failed to allocate third block");
        
        // Free the first and third blocks, leaving the second allocated
        heap.free(ptr1).expect("Failed to free first block");
        heap.free(ptr3).expect("Failed to free third block");

        // Verify that no coalescing happened across the allocated middle block
        let header1 = _Header::_from_byte(&heap.heap[BOTTOM_OF_HEAP]);
        let header3 = _Header::_from_byte(&heap.heap[ptr3 - HEADER_SIZE]);
        
        assert_eq!(header1.block_alloc, 0, "First block should be marked as free");
        assert_eq!(header3.block_alloc, 0, "Third block should be marked as free");

        let mid_header = _Header::_from_byte(&heap.heap[_ptr2 - HEADER_SIZE]);
        assert_eq!(mid_header.block_alloc, 1, "Middle block should remain allocated");
    }

    #[test]
    fn test_free_coalescing_multiple_adjacent_blocks() {
        let mut heap = Heap::new_heap(64);

        // Allocate four blocks
        let ptr1 = heap.allocate(4).expect("Failed to allocate first block");
        let ptr2 = heap.allocate(4).expect("Failed to allocate second block");
        let ptr3 = heap.allocate(4).expect("Failed to allocate third block");
        let ptr4 = heap.allocate(4).expect("Failed to allocate fourth block");

        // Free all blocks to test coalescing of multiple adjacent blocks
        heap.free(ptr1).expect("Failed to free first block");
        heap.free(ptr2).expect("Failed to free second block");
        heap.free(ptr3).expect("Failed to free third block");
        heap.free(ptr4).expect("Failed to free fourth block");

        // After freeing, the entire heap should be a single large free block
        let header = _Header::_from_byte(&heap.heap[BOTTOM_OF_HEAP]);
        assert_eq!(header.block_size, 64, "All blocks should be coalesced into a single large free block");
        assert_eq!(header.block_alloc, 0, "Coalesced block should be marked as free");
    }
}