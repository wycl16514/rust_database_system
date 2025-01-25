One most important concern for database system is speed. We can't afford to wait ten minutes for executing a line of sql code like "select * from students". As we have seen before, accessing disk or wrting and reading from file is very expensive
operation, therefore one of most effective way to gurunteen speed is reduce the need of read and write from file and keep data operation in memory as much as possible. This requests us to have a smart way to manage memory and make sure there are
available memory pages for reading and writing data.

The goal of designing buffer manager is to make sure to help the system have its reading and writing operation base on memory pages. It will initially allocate a batch of memory pages as buffer pool, and if other components want to use memory page, 
they can "book" given pages for later usage, this just like you booking a hotel room before hand and you can check into the room when you arrive your destination. The difference is, for a room, it can only be booked by one guest, but for memory
page, it can be booked by several components, and any components can read and write to the page as long as it books the given page.

But how about data consitency if multiple components read and write to the same page? That is not the concern of buffer manager, we will have another component name "concurrency manager".


