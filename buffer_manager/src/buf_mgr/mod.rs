use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use crate::file_mgr::*;
use crate::log_mgr::*;

pub struct Buffer<'a> {
    fm: &'a Mutex<FileMgr>,
    lm: &'a Mutex<LogMgr<'a>>,
    page_buf: Vec<u8>,
    blk: BlockId,
    pins: i32,
    tx_num: i32,
    lsn: i32,
}

impl<'a> Buffer<'a> {
    pub fn new(fm: &'a Mutex<FileMgr>, lm: &'a Mutex<LogMgr<'a>>) -> Self {
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
        self.fm.lock().unwrap().read_write(&self.blk, &mut p, false).unwrap();
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

    pub fn pin(&mut self) {
        self.pins += 1;
    }

    pub fn unpin(&mut self) {
        self.pins -= 1;
    }
}

pub struct BufferMgr<'a> {
    /*
    several threads may access the same buffer at the same time,
    that's why we need to have Arc<Mutex<Buffer>>> as Vec element
    */
    buffer_pool: Arc<Vec<Arc<RwLock<Buffer<'a>>>>>,
    num_available: Arc<Mutex<u32>>,
    //telling all threads waiting for buffers to wake up
    wake_up: Arc<AtomicBool>,
}

impl<'a> BufferMgr<'a> {
    pub fn new(fm: &'a Mutex<FileMgr>, lm: &'a Mutex<LogMgr<'a>>, num_buffers: u32) -> Self {
        let mut buf_vec = Vec::with_capacity(num_buffers as usize);
        for _ in 0..num_buffers {
            let buf = Buffer::new(fm, lm);
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

    pub fn flash_all(&mut self, tx_num: i32)  {
       for buf_lock in  self.buffer_pool.iter() {
          let mut buf = buf_lock.write().unwrap();
          if buf.modifing_tx() == tx_num {
              buf.flush();
          }
       }
    }

   fn increase_availabe_buff(&mut self, buffer_lock : Arc<RwLock<Buffer<'a>>>) {
        let mut num_available = self.num_available.lock().unwrap();
        if !buffer_lock.read().unwrap().is_pinned() {
            *num_available += 1;
        } 
   }

    pub fn unpin(&mut self ,  buffer_lock: Arc<RwLock<Buffer<'a>>>) {
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

    pub fn pin(&mut self, blk: BlockId) -> Option<Arc<RwLock<Buffer<'a>>>> {
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