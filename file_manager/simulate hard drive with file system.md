In this section, let's see how we can design low level file system to support the persistent of data on disk. As we have metioned before, we will use binary data to simulate the hard drive or
more precisely to simulate the track on the hard drive, then each binary file will chop into several blocks for saving data for our database system, first we need to design a way to look at the
block inside the binary file.

First we create a new project by using "cargo new rust_db", then go into the folder, we need to create a sub module for our system. For any big and complex system, the best way to handle it is divide
and conqure, that is chop the big system into composition of several small sub systems, if each sub system still to big to comprehend, then we chop the sub system into more smaller sub systems again.

Now let's create a sub folder name file_mgr and create a mod.rs, in the file we first design an struct for finding block inside the binary file, the code as following:

```rs
pub mod test;

#[derive(Debug)]
struct BlockId {
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
```
BlockId used to indentify where we want to get data, it contains the binary file name which used to simulate the tract, and the block number, using this number we can go into the offset to read
the given binary data.

In order to make sure our code logic is correct, let's add some unit tests for it, create a name test.rs then we add the following tests:

```rs
use super::BlockId;

#[test]
fn test_block_id_funcs() {
    let block_id = BlockId::new("testing.tbl", 2);
    assert_eq!(block_id.file_name(), "testing.tbl");
    assert_eq!(block_id.number(), 2);
    assert_eq!(block_id.to_string(), "file: testing.tbl, block: 2");
}

#[test]
fn test_block_id_equal() {
    let block1 = BlockId::new("testing.tbl", 1);
    let block2 = BlockId::new("testing.tbl", 1);
    let block3 = BlockId::new("testing1.tbl", 1);
    let block4 = BlockId::new("testing.tbl", 2);

    assert_eq!(block1, block2);
    assert_ne!(block1, block3);
    assert_ne!(block1, block4);
}
```

Then go to the root directory of our project and run "cargo test", make sure all two tests can be passed. Now let's see how we can design the Page struct, it is used to help the database engine 
to write or write binary data from given offset for given binary buffer, in mod.rs we add following code:

```py
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

```
The design of Page is used for writing and reading all kinds of data into memory buffer, it provides methods to read and write data type of int, string, and bytes for given offset of the buffer
which wrapped in the paper struct, we need to pay attetion to writing and reading string and bytes, when writing into the buffer for type string or bytes, it will take the first 4 bytes from 
the given offset, then use it to save the length of string or bytes, then write the content for string and bytes at the following place, the read is the same, it first take the 4 bytes from 
the offset of reading, inteprete the 4 bytes as length for the following data, then read the binary data after the 4 bytes, let's check the testing for the Page struct as following, in test.rs:

```rs
#[test]
fn test_page_int() {
   let mut buf = Vec::<u8>::with_capacity(8);
   let mut page = Page::from_buffer(&mut buf);
   let result = page.set_int(2, 1234);
   assert_eq!(result, Ok(()));
   let res = page.get_int(2).unwrap();
   assert_eq!(res, 1234);
}

#[test]
fn test_page_int_buf_overflow() {
    let mut buf = Vec::<u8>::with_capacity(3);
    let mut page = Page::from_buffer(&mut buf);
    let result = page.set_int(0, 0x0fffff);
    assert_eq!(result.is_err(), true);
}

#[test]
fn test_page_bytes() {
    let mut buf = Vec::<u8>::with_capacity(16);
    let mut page = Page::from_buffer(&mut buf);
    let bytes = vec![0x01, 0x02, 0x03, 0x04];
    let res = page.set_bytes(3, bytes.as_slice());
    assert_eq!(res, Ok(()));
    assert_eq!(page.get_bytes(3).unwrap(), bytes);
}

#[test]
fn test_page_bytes_overflow() {
    let mut buf = Vec::<u8>::with_capacity(8);
    let mut page = Page::from_buffer(&mut buf);
    let bytes = vec![0x01, 0x02, 0x03, 0x04];
    let res = page.set_bytes(3, bytes.as_slice());
    assert_eq!(res.is_err(), true);
}

#[test]
fn test_page_string() {
    let mut buf = Vec::<u8>::with_capacity(32);
    let mut page = Page::from_buffer(&mut buf);
    let str = String::from("hello, world!");
    let res = page.set_string(3, &str);
    assert_eq!(res, Ok(()));
    assert_eq!(page.get_string(3).unwrap(), str);
}

#[test]
fn test_page_string_overflow() {
    let mut buf = Vec::<u8>::with_capacity(8);
    let mut page = Page::from_buffer(&mut buf);
    let str = String::from("hello, world!");
    let res = page.set_string(3, &str);
    assert_eq!(res.is_err(), true);
}

```
Run the command of "cargo test" make sure all the tests can be passed. Finally we need to design a file manager to utilize the structs above. The file manager will provide low level data storage
and retrievel support for the database engine, For Postgrel, mySQL, Oracle, sqlServer, sqlite, if you create any tables by using sql, the data related to the table will save on binary file hosts
on your local disk, our file manager will help to save all kinds of data related to table or views on the disk, following is the code :

```rs
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
```

The key for file manager is open_files hashmap, it is wrapped in a RWLock for read write effieciency. For example, there may be two different actions happend on to table, one is wrtie into a 
student table, the other is read into the department table, then this two action can both access to the hashmap at the same time, and the action for writing into the student table may require
to access to a binary file with name "student.tbl", and the reading action may require to read into the binary file "deparment.tbl", since they are not interferencing with each other, thse two
actions can be processed simutaniously.

But if one action is to create a new table name teachers, then this action will require to insert a new binary file name "teacher.tbl" into the hashmap, this will require the write lock for the
hashmap, then it will need to wait until the completes of the two actions above then it can gain the write lock to access to the hashmap. Let's gain more understandings into the file manager with
the following tests:

```rs
#[test]
fn test_file_manage() {
    let mut file_mgr = FileMgr::new("filetest".to_string(), 
    512);
    //read from offset of 512 * 2
    let blk = BlockId::new("testfile", 2);
    let mut buf = vec![0u8; file_mgr.block_size() as usize];
    let mut p1 = Page::from_buffer(&mut buf);
    let pos1 = 88;
    /*
    block begins from offset 512 * 2, we wrtie into offset 88 relativly to
    the block, which means we write to offset from the file 512 * 2 + 88
     */
    let hello = String::from("hello, world!");
    p1.set_string(pos1, &hello);

    // //len of string plus 4 bytes, 4 bytes to indicate the length of the string
    let pos2 = Page::max_length(hello.len() as u64) + pos1 as u64;
    p1.set_int(pos2 as usize, 345);
    /*
    blk indicates wrtie to which file and from what offset
    */
    file_mgr.read_write(&blk, &mut p1, true);

    let mut buf1 = vec![0u8; file_mgr.block_size() as usize];
    let mut p2 = Page::from_buffer(&mut buf1);
    file_mgr.read_write(&blk, &mut p2, false);
    p2.get_int(pos2).unwrap();
    let int_val = p2.get_int(pos2).unwrap();
    assert_eq!(int_val, 345);

    let str_val = p2.get_string(pos1).unwrap();
    assert_eq!("hello, world!", str_val);

}
```
The testing above using file manger to create a local folder to save all low level binary files. Then it creates a BlockId object with file name "testfile", and the BlockId want to access the 
block with index 3, which means it want to access into the binary file with offset of length for two blocks, since we are setting size for a block is 512 bytes, therefore the
BlockId object of "blk" indicates it want to read or write into the binary file of "testfile" from offset of 512 * 2 = 1024, then it go to that offset to write int and string value into the 
given offsets and read them back to check whether the writing is correct or not, run the command of "cargo test" make sure all tests are ok
