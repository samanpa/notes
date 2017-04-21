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

    fn grow_by(&mut self, size: usize) {
        self.data.reserve(size);
        let capacity = self.data.capacity();
        unsafe{ self.data.set_len(capacity) };
    }

    pub fn ensure(&mut self, size: usize) {
        if self.limit + size > self.data.len() {
            let increase = self.data.len() - self.limit - size;
            self.grow_by(increase);
        }
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
            let increase = self.limit;
            self.grow_by(increase);
        }
        
        unsafe { self.data.get_unchecked_mut(self.limit..) }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { self.data.get_unchecked(self.readpos..self.limit) }
    }

    pub fn advance_write(&mut self, nbytes: usize) {
        self.limit += nbytes;
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

    pub fn write(&mut self, data: &[u8]) {
        self.ensure(data.len());
        unsafe { self.unchecked_write(data) };
    }
    
    pub unsafe fn unchecked_write(&mut self, data: &[u8]) {
        let dest = self.data.as_mut_ptr().offset(self.limit as isize);
        ptr::copy_nonoverlapping(data.as_ptr(), dest, data.len());
        self.advance_write(data.len());
    }
    
    
}
