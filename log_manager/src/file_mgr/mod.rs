pub mod test;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Seek, SeekFrom, Write, Read};
use std::io;
use std::fs;
use std::collections::HashMap;
use std::fs::File;
use walkdir::WalkDir;
use std::sync::{Arc, RwLock};
use std::path::Path;
use std::fs::OpenOptions;


#[derive(Debug)]
pub struct BlockId {
    //block taken from given binary file
    file_name: String, 
    //block number into the binary file
    blk_num: u64,
}

impl BlockId {
    pub fn new(file_name: &str, blk_num : u64) -> Self {
        BlockId {
            file_name: String::from(file_name),
            blk_num,
        }
    }

    pub fn file_name(&self) -> String {
         self.file_name.clone()
    }

    pub fn number(&self) -> u64 {
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

    pub fn contents(&mut self) -> &mut Vec<u8> {
        self.bb
    }
}

pub struct FileMgr {
    //prepare for concurrent accessing low level binary files
    open_files: Arc<RwLock<HashMap<String, RwLock<File> >>>,
    //dir to save binary file
    directory: String,
    //whether the given directory is exist or not
    //if not then we create the directory and set is_new to true
    //otherwise set to false
    is_new:  bool,

    block_size: u64,
}

fn delete_temp_files(directory: &str) -> io::Result<()> {
    for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // Check if the entry is a file and its name ends with "temp"
        if path.is_file() {
            if let Some(file_name) = path.file_name() {
                if let Some(file_name_str) = file_name.to_str() {
                    if file_name_str.ends_with("temp") {
                        // Delete the file
                        fs::remove_file(&path)?;
                        println!("Deleted: {:?}", path);
                    }
                }
            }
        }
    }

    Ok(())
}

impl FileMgr {
    pub fn new(db_directory: String, block_size: u64) -> Self{
        //check given directory exist or not 
        let is_new = !(Path::new(db_directory.as_str()).exists() && Path::new(db_directory.as_str()).is_dir());
        
        //dierctory not exist then create it
        if is_new {
            fs::create_dir_all(db_directory.clone()).unwrap();
        } else {
            //delete temp files
            let _ = delete_temp_files(&db_directory);
        }

        FileMgr{
            open_files: Arc::new(RwLock::new(HashMap::new())),
            directory: db_directory,
            is_new,
            block_size,
        }
    }

    pub fn read_write(&mut self ,blk: &BlockId, p: &mut Page, is_write: bool) -> Result<usize, String> {
        /*
        read given binary file from given offset, file name and offset
        can get from BlockId, take cocurrent read into concerns
        */
        //check file exist or not, if not should call add_file to create file
        self.add_file(blk.file_name()).unwrap();
        if is_write {
              /*
                if the write position is beyond the length of the file, then we 
                extend the file to the given block
                */
            self.extend(blk);
        }

        let map_guard = self.open_files.read().unwrap();
        let file_name = blk.file_name();
        if let Some(file_lock) = map_guard.get(&file_name) {
            let mut file_guard = file_lock.write().unwrap();
            let meta_data = file_guard.metadata().unwrap();
            let offset = blk.number() * self.block_size;
           
            if offset >= meta_data.len() {
                return Err(format!("offset out bound of given file:{}", file_name));
            }
            
            file_guard.seek(SeekFrom::Start(offset)).unwrap();

            if !is_write {
                let bytes_read = file_guard.read( p.contents()).unwrap();
                Ok(bytes_read)
            } else {
                let bytes_write = file_guard.write(p.contents()).unwrap();
                Ok(bytes_write)
            }
            
        } else {
            Err(format!("file with name:{} not found", file_name))
        }
    }

    fn extend(&mut self, blk :&BlockId) {
           while self.length(blk.file_name()).unwrap() <= blk.number() {
                let _ = self.append(blk.file_name());
           }
       } 

   pub fn append(&mut self, file_name: String) ->Result<BlockId, String> {
      let map_guard = self.open_files.read().unwrap();
      if let Some(file_lock) = map_guard.get(&file_name) {
          let new_blk_num = self.length(file_name.clone()).unwrap();
          //enlarge the file with block size at the end
          let file_guard = file_lock.write().unwrap();
          let meta_data = file_guard.metadata().unwrap();
          let new_size = meta_data.len() + self.block_size;
          file_guard.set_len(new_size).unwrap();
          Ok(BlockId::new(file_name.as_str(), new_blk_num))
      } else {
          return Err(format!("file with name: {} not found", file_name));
      }
   }

   fn length(&self, file_name: String) -> Result<u64, String> {
        let map_guard = self.open_files.read().unwrap();
        if let Some(file_lock) = map_guard.get(&file_name) {
            //compute how many blocks in the file
            let file_guard = file_lock.read().unwrap();
            let meta_data = file_guard.metadata().unwrap();
            Ok(meta_data.len() / self.block_size)
        } else {
            Err(format!("file : {} not found", file_name))
        }
   }

    pub fn is_new(&self) -> bool {
        self.is_new
    }

    fn add_file(&mut self,file_name: String) -> io::Result<()> {
        /*
        check file opened or not, using scope here to make sure the guard
        */
        let map_guard = self.open_files.read().unwrap();
        if map_guard.contains_key(file_name.as_str()) {
            return Ok(())
        }
        //release the read lock for getting write lock at following
        drop(map_guard);

        let file_path = format!("{}/{}", self.directory, file_name);
        //open file for read and write
        let  file = OpenOptions::new()
        .read(true)  // Allow reading
        .write(true) // Allow writing
        .create(true) // Create the file if it doesn't exist
        .open(file_path)?;

        let file_lock = RwLock::new(file);
        let mut map_guard = self.open_files.write().unwrap();
        map_guard.entry(file_name.to_string()).or_insert(file_lock);

        Ok(())
    }

    pub fn block_size(&self) ->u64 {
        self.block_size
    }
}