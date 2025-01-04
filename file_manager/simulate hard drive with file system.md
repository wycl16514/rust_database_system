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

```
