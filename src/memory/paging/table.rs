use memory::paging::entry::*;
use memory::paging::ENTRY_COUNT;

use memory::FrameAllocator ; 

use core::ops::{Index, IndexMut};
use core::marker::PhantomData;

pub trait TableLevel {}

pub enum Level4 {}
pub enum Level3 {}
pub enum Level2 {}
pub enum Level1 {}

impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}

pub trait HierarchicalLevel: TableLevel {
    type NextLevel: TableLevel;
}

impl HierarchicalLevel for Level4 {
    type NextLevel = Level3;
}

impl HierarchicalLevel for Level3 {
    type NextLevel = Level2;
}

impl HierarchicalLevel for Level2 {
    type NextLevel = Level1;
}

pub const P4: *mut Table<Level4> = 0xffffffff_fffff000 as *mut _;


pub struct Table<L:TableLevel> {
    entries: [Entry; ENTRY_COUNT],
    level : PhantomData<L> ,
}






impl<L> Table<L>  where L: TableLevel{
	pub fn zero(&mut self) {
    	for entry in self.entries.iter_mut() {
        	entry.set_unused();
    	}
	}


}
impl<L> Table<L> where L: HierarchicalLevel{

	pub fn next_table<'a>(&'a self, index: usize) -> Option<&'a Table<L>> where L: HierarchicalLevel {
	    self.next_table_address(index)
	        .map(|address| unsafe { &*(address as *const _) })
	}

	pub fn next_table_mut<'a>(&'a mut self, index: usize) -> Option<&'a mut Table<L>>  where L: HierarchicalLevel {
	    self.next_table_address(index)
	        .map(|address| unsafe { &mut *(address as *mut _) })
	}
	fn next_table_address(&self, index: usize) -> Option<usize> {
	    let entry_flags = self[index].flags();
	    if entry_flags.contains(PRESENT) && !entry_flags.contains(HUGE_PAGE) {
	        let table_address = self as *const _ as usize;
	        Some((table_address << 9) | (index << 12))
	    } else {
	        None
	    }
	}

	pub fn next_table_create<A>(&mut self,
                            index: usize,
                            allocator: &mut A)
                            -> &mut Table<L>   // TODO : Corrections may be required. 
    where A: FrameAllocator
	{
	    if self.next_table(index).is_none() {
	        assert!(!self.entries[index].flags().contains(HUGE_PAGE),
	                "mapping code does not support huge pages");
	        let frame = allocator.allocate_frame().expect("no frames available");
	        self.entries[index].set(frame, PRESENT | WRITABLE);
	        self.next_table_mut(index).unwrap().zero();
	    }
	    self.next_table_mut(index).unwrap()
	}
		
}

impl<L> Index<usize> for Table<L> where L: TableLevel {
    type Output = Entry;

    fn index(&self, index: usize) -> &Entry {
        &self.entries[index]
    }
}

impl<L> IndexMut<usize> for Table<L> where L: TableLevel {
    fn index_mut(&mut self, index: usize) -> &mut Entry {
        &mut self.entries[index]
    }
}


