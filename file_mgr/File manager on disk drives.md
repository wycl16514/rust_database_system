For database system to make data persistent, that is data can't be disappeared when we turn off the computer, it needs to save data on hard drive. But accessing hard drive is a very slow operation, 
an effiecient database system depends on how we organize data on the disk and make them easy to retrievel. 

A  hard drive is like following:

![image](https://github.com/user-attachments/assets/af613c79-a369-43dc-addb-5351d03daf39)



It is made up by several "disks", and each disk has two side, up and down, each side is name platter, and it contains maganetic materials. There is a drive motor the spin all those disks at the same
time. Data are save on platters, and platter can divide into several tracks, that is those "circles" on the platter. The track will divide into several units, which units is name sector, sector is
the smallest unit to store data and normally its volumn is 512 bytes. 

For file system base on given operating system, it won't access one sector at one time, it will read content of several continuos sectors together, such continuous sectors are form in a group called
block. Normally a block size is 4K, which is made up by 8 sectors. When the file system want to get data from hard drive, it need to locate the data on which disk, then on which platter(up side or 
down side) then locate which track, then on which block, then the operating system will send signals to hard drive ask it to move the read/write head to the given track, then rotate those disk 
together and move the beginning of sector that containing the given data directly under the read write header, then the file system will read data by blocks, that is it will read data 8 sectors
at one time. If the data you want is not more than 8 sectors which is 4kb, then we can get the data by one time operation , if the data is more than 1 block, then we need to move the head and
spin those disk for several times.

From the view point of os, it dose not look at one sector at a time, it look at several continues sectos as group at a time, such contninuos group is block we mentioned aboved. And os assign each
block with a number, for example for a track has 124 sectors, if we take 8 sectors as a block, then there would be 124/8 = 32 blocks, then each block will have number from 0 to 31, if the os want
to read data in to number2, then it will goto the 17th sector and read 8 sectors beginning from the 17th sector.

When os want to read or write into a block, it will read data from the block into memory unit called page(normally 4k), then write or read from data in the memory page, if it is write, then os write
into the memory page, then send the changed data back to block on given track by the read/write header. When designing our own database system, it is impossible for us to directly interact with 
hard drive then decide which block or sector to read or write, we make a detour, that is we use raw binary file to simulate the hard drive, then we chop the binary file into several blocks, for
example we create a binary file with length of 1024k = 1M, if each block has 4k, then a binary file will contain 1024/4 = 256 block, if we going to read data of 7k beginning from the 2th block, 
then we will goto the offset of 8k into the file and read 8k data from there, remember we can only read or write data in unit of block.

