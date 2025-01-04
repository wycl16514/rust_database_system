pub mod test;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Seek, SeekFrom, Write};

#[derive(Debug)]
pub struct BlockId {
    //block taken from given binary file
    file_name: String, 
    //block number into the binary file
    blk_num: u32,
}

impl BlockId {
    pub fn new(file_name: &str, blk_num : u32) -> Self {
        BlockId {
            file_name: String::from(file_name),
            blk_num,
        }
    }

    pub fn file_name(&self) -> String {
         self.file_name.clone()
    }

    pub fn number(&self) -> u32 {
        self.blk_num
    }

    pub fn to_string(&self) -> String {
        format!("file: {}, block: {}", self.file_name, self.blk_num)
    }
}

impl PartialEq for BlockId {
    fn eq(&self, other: &Self) -> bool {
        self.file_name == other.file_name && self.blk_num == other.blk_num
    }
}

#[derive(Debug)]
pub struct Page<'t> {
    bb: &'t mut Vec<u8>,
}

impl<'t> Page <'t>{
    pub fn from_buffer(buf: &'t mut Vec<u8>) ->Self {
        Page { bb: buf }
    }

    pub fn get_int(&mut self, offset: u64) -> Result<i32, String> {
        /*
        offset + 4 is type of u64, self.bb.capacity() is type of uside,
        using try_into() convert usize to u64
        try_into is a trait, it will find the given object implement
        the trait for converting to the given type
        */
        if offset + 4 >= self.bb.capacity().try_into().unwrap() {
            let err_msg = format!("get_int overflow, offset+4:{}, buffer cap:{}", offset+4, self.bb.capacity());
            return Err(err_msg);
        }
        /*
        read 4 bytes as int value from given offset
        there is only one mutable reference allowed, since we have a 
        mutable reference by self.bb, then if we allow Cursor to have
        a mutable reference to the same buffer, we need to release
        the mutable reference by bb that is why using *self.bb
        */
        let mut cursor = Cursor::new(&mut *self.bb);
        /*
        if the offset is outside the limit of buffer, throw an Error
        */
        cursor.seek(SeekFrom::Start(offset)).unwrap();
        let val = cursor.read_i32::<BigEndian>().unwrap();
        Ok(val)
    }

    pub fn set_int(&mut self, offset: usize, n: i32) ->Result<(), String>{
        //need to check buffer overflow
        if offset + 4 >= self.bb.capacity() {
            let err_msg =format!("set_int buffer overflow offset+4:{}, buffer cap:{}", offset+4, self.bb.capacity()); 
            return Err(err_msg);
        }

        let mut cursor = Cursor::new(&mut *self.bb); 
        /*
        if the offset is outside the limit of buffer, throw an Error
        */
        cursor.seek(SeekFrom::Start(offset as u64)).unwrap();
        cursor.write_i32::<BigEndian>(n).unwrap();

        Ok(())
    }

    pub fn get_bytes(&mut self, offset: usize) -> Result<Vec<u8>, String>{
        /*
        if offset is wrong, the result is unpredictable
         */
        if offset >= self.bb.capacity() {
            let err_msg = format!("get bytes overflow: offset:{}, buffer cap:{}", offset, self.bb.capacity());
            return Err(err_msg);
        }

        //the first 4 bytes from offset is the length for following bytes
        let mut cursor = Cursor::new(&mut *self.bb); 
        cursor.seek(SeekFrom::Start(offset as u64)).unwrap();
        let bytes_len = cursor.read_u32::<BigEndian>().unwrap();
        let begin = offset+4;

        if begin >= self.bb.len() {
            let err_msg = format!("get bytes with bytes buffer overflow, start pos:{}, buffer len:{}", begin, self.bb.len());
            return Err(err_msg);
        }

        let end = begin + (bytes_len as usize);
        Ok(self.bb[begin..end].to_vec())
    }

    pub fn set_bytes(&mut self, offset: usize, bytes: &[u8]) ->Result<(), String> {
        if offset + 4 + bytes.len() >= self.bb.capacity() {
            let err_msg = format!("set bytes overflow: offset+4+bytes.leng():{}, buffer cap:{}", offset + 4 + bytes.len(), self.bb.capacity());
            return Err(err_msg);
        }
        let mut cursor = Cursor::new(&mut *self.bb); 
        cursor.seek(SeekFrom::Start(offset as u64)).unwrap();
        //use 4 bytes to indicate the following bytes length
        cursor.write_u32::<BigEndian>(bytes.len().try_into().unwrap()).unwrap();
        cursor.write_all(bytes).unwrap();

        Ok(())
    }

    pub fn get_string(&mut self, offset: usize) -> Result<String,String> {
        let str = String::from_utf8(self.get_bytes(offset).unwrap());
        match str {
            Ok(s) => return Ok(s),
            Err(e) => return Err(e.to_string())
        }
    }

    pub fn set_string(&mut self, offset: usize, s: &String) -> Result<(),String> {
        let bytes = s.as_bytes();
        self.set_bytes(offset, bytes)
    }

    pub fn max_length(str_len: u64) -> u64 {
        /*
        buffer length for given string with str_len is 
        4 bytes plus the bytes for the string
        */
        return 4 + str_len;
    }

    pub fn contents(&self) -> &Vec<u8> {
        self.bb
    }
}