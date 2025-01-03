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


