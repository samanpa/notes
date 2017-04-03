use std::vec::Vec;
use std::ptr;

pub struct Buff {
    data : Vec<u8>,
    readpos: usize,
    limit: usize,
    //mark: usize,
}

impl Buff {
    pub fn with_capacity(size: usize) -> Self {
        let vec = vec![0; size];
        Buff{data: vec, readpos: 0, limit: 0}
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        if self.readpos == self.limit {
            self.readpos = 0;
            self.limit = 0;
        }
        else if self.readpos != 0 {
            self.compact();
        }
        else if self.limit == self.data.len() {
            self.data.reserve(self.limit);
            let capacity = self.data.capacity();
            unsafe{ self.data.set_len(capacity) };
        }
        
        unsafe { self.data.get_unchecked_mut(self.limit..) }
    }

    pub fn as_slice(&mut self) -> &[u8] {
        unsafe { self.data.get_unchecked(self.readpos..self.limit) }
    }

    pub fn advance_write(&mut self, nbytes: usize) {
        self.limit = self.limit + nbytes;
    }

    pub fn advance_read(&mut self, nbytes: usize) {
        self.readpos = self.readpos + nbytes;
    }

    pub fn remaining(&self) -> usize {
        self.data.capacity() - (self.limit - self.readpos)
    }

    fn compact(&mut self) {
        let size = self.limit - self.readpos;
        unsafe { ptr::copy(self.data.as_ptr().offset(self.readpos as isize)
                           , self.data.as_mut_ptr()
                           , size) };
        self.limit = size;
        self.readpos = 0;
    }
    
}
