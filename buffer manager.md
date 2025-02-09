One most important concern for database system is speed. We can't afford to wait ten minutes for executing a line of sql code like "select * from students". As we have seen before, accessing disk or wrting and reading from file is very expensive
operation, therefore one of most effective way to gurunteen speed is reduce the need of read and write from file and keep data operation in memory as much as possible. This requests us to have a smart way to manage memory and make sure there are
available memory pages for reading and writing data.

The goal of designing buffer manager is to make sure to help the system have its reading and writing operation base on memory pages. It will initially allocate a batch of memory pages as buffer pool, and if other components want to use memory page, 
they can "book" given pages for later usage, this just like you booking a hotel room before hand and you can check into the room when you arrive your destination. The difference is, for a room, it can only be booked by one guest, but for memory
page, it can be booked by several components, and any components can read and write to the page as long as it books the given page.

But how about data consitency if multiple components read and write to the same page? That is not the concern of buffer manager, we will have another component name "concurrency manager". When given component is done with the memory page, it will
ask buffer manager to unpin the given page. If given page is totally unpined, that is not any components need its content anymore, then the buffer manager can "recycle" the page, and when new requests come, the buffer manager will read a block of
data from binary file and save those data in the given page and return the page to requester.

One special case is, when all pages are pinned, that is all pages are contains data that are still in used by some components, and if new component ask the buffer manager to fetch other block of data from binary file, then the buffer manager
will unable to serve the request, and put the client in wating list.

Base on above decription, a buffer would contains several parts, one is the Page we designed in previous section, on is meta data that the given page contains which block for which file, and the buffer is responsible for monitoring its page
and if the page is modified, it is its job to write the modified page back into the binary file of given block. Since we need to minimize any operations related to disk, and the buffer should wait as much modifies as possible and write those
changes to disk at one time, one reasonable strategy is write the content of page to disk until the buffer is unpined. But this may not be the best strategy, if the buffer manager keep the buffer even it is unpined, if there is any requests
for the same block of the given file, and all the data is still keep in the page, then the buffer manager can directly return the page without needing to write the page and read the written content back.

Therefore the buffer manager will use the following strategey to write back given page:

1, if the given buffer is unpined, and there is a request to read new block into memory, then the buffer manager will write the content of page into file and fetch new content for given block.

2. if the given page is requested to written to disk by receovery manager.

All in all, the buffer manager is some how like a kind of cache mechanism, which is oftenly used in application to metigate read write efficiency. If you know about the LRU cache algorithm, which is the Least Recently Use principle, the
cache manager will swap out the content of given cache record if it is unused for the longest time, and the buffer manager will do the same, if there are several pages unpined, the manager will write the content for the unpined page that is
unpined for the longest time. The different between buffer manager and cache manager is, the buffer manager know exactly which page is still in used, and this info can make buffer manager have better performance than cache manager.

Let's see the simple cache algorithm used by the buffer manager, suppose the manager contains three pages initially:

Manager: buffer(P1, P2, P3).

Then the first clint ask the manger to read block 1 from file with name "f1":

Manager: buffer(P2, P3).
c1 -pin:1-> (P1, "f1", block 0),

if the second client also want to read the content of block 0 of file "f1", then the buffer manager will simply return the P1 to 1:

c1, c2 - pin:2 -> (P1, "f1", block 0)

Then, another two clients want to write to file "f2" , block2, and "f3", block 3, then we have:

c3 -> pin: 1 -> (P2, "f2", block 2)

c4 -> pin: 1 -> (P3, "f3", block 3)

If at this time comes another client c5 want to read file "f4", block 4, since there are not available buffer or pages any more, the buffer manager will put client c5 on the wait list for 10 seconds. After 10 seconds, if there is avaiabale
page, then c5 can pin the available page and read the content it needs. Otherwise c5 will wake up from sleep and decide to wait again or give up waiting and doing other jobs.

When c3 complete its job with P2, then P2 will put back into available buffer list, if some other client want to read or write file "f2" block2 as c3 did, then buffer manager will pin p2 again and return p2 directly to the client 
without any changes. Notice that we don't write back content of P2 back to disk file eventhough its changed, but if the client want to read file "f5" block 5, then this time ,the buffer manager will write content of P2 back to file "f2"
and block 2, and fetch content from "f5" block 5, and pin it and return the page to client.

Now let's see how to use code to design the buffer manager, first create a folder name buf_mgr and create a file of name mod.rs, then putting the following code:

```rs
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use crate::file_mgr::*;
use crate::log_mgr::*;
use log::{warn, info};
pub mod test;
pub struct Buffer {
    fm:  Arc<Mutex<FileMgr>>,
    lm:  Arc<Mutex<LogMgr>>,
    page_buf: Vec<u8>,
    blk: BlockId,
    pins: i32,
    tx_num: i32,
    lsn: i32,
}

impl Buffer {
    pub fn new(fm:  Arc<Mutex<FileMgr>>, lm:  Arc<Mutex<LogMgr>>) -> Self {
        let block_size = fm.lock().unwrap().block_size();
        Buffer {
            fm,
            lm,
            page_buf: vec![0u8; block_size as usize],
            blk: BlockId::new("notexist.txt", 0),
            pins: 0,
            tx_num: -1,
            lsn: -1,
        }
    }

    pub fn contents(&mut self) -> Page {
        Page::from_buffer(&mut self.page_buf)
    }

    pub fn block(&self) -> BlockId {
        self.blk.clone()
    }

    pub fn set_modified(&mut self, tx_num: i32, lsn: i32) {
        self.tx_num = tx_num;
        if lsn >= 0 {
            self.lsn = lsn;
        }
    }

    pub fn modifing_tx(&self) -> i32 {
        self.tx_num
    }

    pub fn is_pinned(&self) -> bool {
        self.pins > 0
    }

    pub fn assign_to_block(&mut self, b: BlockId) {
        self.flush();
        self.blk = b.clone();
        let mut p = Page::from_buffer(&mut self.page_buf);
        /*
        we don't have unwrap for read_write since the given file may not
        have the given block then we read nothing from the file
        */
        let result = self.fm.lock().unwrap().read_write(&self.blk, &mut p, false);
        match result {
            Ok(bytes_read) => {
                info!("buffer assign with block: {:?}, with bytes read: {}", self.blk.clone(), bytes_read);
            },
            Err(err) => {
                warn!("buffer assign with block: {:?}, with err: {}", self.blk.clone(), err);
            }
        }
        self.pins = 0;
    }

    pub fn flush(&mut self) {
        if self.tx_num > 0 {
            self.lm.lock().unwrap().flush(self.lsn as u64);
            let mut p = Page::from_buffer(&mut self.page_buf);
            self.fm.lock().unwrap().read_write(&self.blk, &mut p, true).unwrap();
            self.tx_num = -1;
        }
    }

    pub fn pin(&mut self)  {
        self.pins += 1;
        
    }

    pub fn unpin(&mut self) {
        self.pins -= 1;
    }

    pub fn pin_count(&self) -> i32 {
        self.pins
    }

}

pub struct BufferMgr {
    /*
    several threads may access the same buffer at the same time,
    that's why we need to have Arc<Mutex<Buffer>>> as Vec element
    */
    buffer_pool: Arc<Vec<Arc<RwLock<Buffer>>>>,
    num_available: Arc<Mutex<u32>>,
    //telling all threads waiting for buffers to wake up
    wake_up: Arc<AtomicBool>,
}

impl BufferMgr {
    pub fn new(fm:  Arc<Mutex<FileMgr>>, lm:  Arc<Mutex<LogMgr>>, num_buffers: u32) -> Self {
        let mut buf_vec = Vec::with_capacity(num_buffers as usize);
        for _ in 0..num_buffers {
            let buf = Buffer::new(fm.clone(), lm.clone());
            buf_vec.push(Arc::new(RwLock::new(buf)));
        }
        BufferMgr {
            buffer_pool: Arc::new(buf_vec),
            num_available: Arc::new(Mutex::new(num_buffers)),
            wake_up: Arc::new(AtomicBool::new(false)),
        }
    }

    /*
    explain Ordering::Relaxex:
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;

    let data = 0;
    let ready = AtomicBool::new(false);

    // Thread A
    thread::spawn(|| {
        data = 42; // Non-atomic write
        ready.store(true, Ordering::Relaxed); // No guarantee data is visible to Thread B
    });

    // Thread B
    thread::spawn(|| {
        while !ready.load(Ordering::Relaxed) {} // Might loop forever even if `ready` is true
        println!("{}", data); // Could see `0` or `42` (undefined behavior!)
    });
    */

    fn wait(&self) {
        /*
        wait at most 10 seconds to check whether there are buffer 
        avaialbe
        */
        let total_wait = Duration::from_secs(10);
        let check_interval = Duration::from_millis(100);
        let mut waited = Duration::from_secs(0);
        while waited < total_wait && !self.wake_up.load(Ordering::Relaxed) {
            /*
            the calling thread need to wait totally 10 secs, but it will 
            wake up for every 100 milli-second to check whether buffer is available 
            */
            std::thread::sleep(check_interval);
            waited += check_interval;
        }
    }

    fn notify_all(&mut self) {
        self.wake_up.store(true, Ordering::Relaxed);
    }

    pub fn available(&self) -> u32 {
        *self.num_available.lock().unwrap()
    }

    pub fn flush_all(&mut self, tx_num: i32)  {
       for buf_lock in  self.buffer_pool.iter() {
          let mut buf = buf_lock.write().unwrap();
          if buf.modifing_tx() == tx_num {
              buf.flush();
          }
       }
    }

   fn increase_availabe_buff(&mut self, buffer_lock : Arc<RwLock<Buffer>>) {
        let mut num_available = self.num_available.lock().unwrap();
        let mut buf = buffer_lock.write().unwrap();
        //change here
        buf.unpin();
        if !buf.is_pinned() {
            *num_available += 1;
        } 
   }

    pub fn unpin(&mut self ,  buffer_lock: Arc<RwLock<Buffer>>) {
        self.increase_availabe_buff(buffer_lock);
        self.notify_all();
    }

    fn wait_too_long(&self, start_time: Instant) -> bool {
        start_time.elapsed() >= Duration::from_secs(10)
    }

    fn find_existing_buffer(&mut self,blk : BlockId) -> Option<usize>{
        for (i,buf) in  self.buffer_pool.iter().enumerate() {
            let b = buf.read().unwrap().block();
            if b == blk {
                return Some(i);
            }
        }
       None
    }

    fn choose_unpin_buffer(&self) -> Option<usize> {
        for (i,buf) in self.buffer_pool.iter().enumerate() {
            let buf_guard = buf.read().unwrap();
            if !buf_guard.is_pinned() {
                return Some(i);
            }

        }

        None
    }

    fn try_to_pin(&mut self ,blk : BlockId) ->Option<usize> {
        let mut i = self.find_existing_buffer(blk.clone());
        
        if i.is_none() {
            i = self.choose_unpin_buffer();
            if i.is_none() {
                return None;
            }
            let idx = i.unwrap();
            let mut buf_guard = self.buffer_pool[idx].write().unwrap();
            buf_guard.assign_to_block(blk);
        } 

        let mut buf_guard = self.buffer_pool[i.unwrap()].write().unwrap();
        if !buf_guard.is_pinned() {
            let mut num_available = self.num_available.lock().unwrap();
            *num_available -= 1;
            if *num_available == 0 {
                /*
                buffer pool is empty, then thread request for buffer
                need to wait
                */
                self.wake_up.store(false, Ordering::Relaxed);
            }
        }
        
        buf_guard.pin();
        i
    }

    pub fn pin(&mut self, blk: BlockId) -> Option<Arc<RwLock<Buffer>>> {
        let time_stamp = Instant::now();
        let mut i = self.try_to_pin(blk.clone());
        while i.is_none() && !self.wait_too_long(time_stamp) {
            /*
            no buffer available and waiting time is not longer than 10 secs
            then continue to wait,if there is a buffer available, then all
            waiting threads are wake up, and current thread may loss the race
            to gain the buffer, then it will go to wait again
            */
            self.wait();
            i = self.try_to_pin(blk.clone());
        }

        if i.is_none() {
            return None;
        }

        let idx = i.unwrap();
        let buffer_arc = Arc::clone(&self.buffer_pool[idx]);
        Some(buffer_arc)
    }
}
```
Then we create a new file name test.rs, and put the following unit test case:
```rs
use crate::buf_mgr::*;
use crate::file_mgr::*;
use crate::log_mgr::*;

#[test]
fn test_buffer_manager() {
    let  file_mgr = FileMgr::new("buffermgrtest".to_string(), 400);
    let file_mgr_lock = Arc::new(Mutex::new(file_mgr));
    let log_mgr = LogMgr::new(file_mgr_lock.clone(), "buffermgrtest".to_string());
    let log_mgr_lock = Arc::new(Mutex::new(log_mgr));
    //create buffer manager with only 3 buffers
    let mut buf_mgr = BufferMgr::new(file_mgr_lock, log_mgr_lock, 3);

   
    let  buf_block0 = buf_mgr.pin(BlockId::new("testfile.txt", 0)).unwrap();
    assert_eq!(buf_block0.read().unwrap().pin_count(), 1);
   
    let  buf_block1 = buf_mgr.pin(BlockId::new("testfile.txt", 1)).unwrap();
    assert_eq!(buf_block1.read().unwrap().pin_count(), 1);

    //here run out all availabe buffer
    let  buf_block2 = buf_mgr.pin(BlockId::new("testfile.txt", 2)).unwrap();
    assert_eq!(buf_block2.read().unwrap().pin_count(), 1);

    //have one available buffer now, 
    buf_mgr.unpin(buf_block1.clone());
    assert_eq!(buf_block1.read().unwrap().pin_count(), 0);
    

    //should increase the pin of buffer0
    let _ = buf_mgr.pin(BlockId::new("testfile.txt", 0)).unwrap();
    assert_eq!(buf_block0.read().unwrap().pin_count(), 2);


    let  buf_block1 = buf_mgr.pin(BlockId::new("testfile.txt", 1)).unwrap();
     //we pin block1 before and then unpin it, therefore this time 
    //pin number for block1 still 1
    assert_eq!(buf_block1.read().unwrap().pin_count(), 1);

    //we run out all buffers, if we pin buffer now, we will
    //wait for 10 secs
    let start = Instant::now();
    let pin_result = buf_mgr.pin(BlockId::new("testfile.txt", 3));
    let wait_long_enough = start.elapsed() >= Duration::from_secs(10);
    assert_eq!(wait_long_enough, true);
    assert_eq!(pin_result.is_none(), true);

    //unpin one buffer then can pin for block 3
    buf_mgr.unpin(buf_block2.clone());
    assert_eq!(buf_block2.read().unwrap().pin_count(), 0);

    let _ = buf_mgr.pin(BlockId::new("testfile.txt", 3)).unwrap();
    assert_eq!(buf_block2.read().unwrap().pin_count(), 1);
}
```
And run the command : cargo test, and make sure all test case can be passed.



