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




