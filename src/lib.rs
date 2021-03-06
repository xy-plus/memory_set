#![no_std]

pub mod area;
pub mod handler;
pub mod attr;
pub mod paging;

use self::{area::MemoryArea, handler::MemoryHandler, attr::MemoryAttr};
use paging::InactivePageTable;
use alloc::{boxed::Box, vec::Vec,};

pub struct MemorySet{
    areas : Vec<MemoryArea>,
    page_table : InactivePageTable,
}

impl MemorySet {
    pub fn new() -> Self {
        MemorySet{
            areas : Vec::new(),
            page_table : InactivePageTable::new(),
        }
    }

    pub fn new_kern() -> Self {
        MemorySet{
            areas : Vec::new(),
            page_table : {
                let mut table = InactivePageTable::new();
                table.map_kernel();
                table
            },
        }
    }

    pub fn push(&mut self, start : usize, end : usize, attr : MemoryAttr, handler : impl MemoryHandler) {
        assert!(start <= end, "invalid memory area"); // 首地址应该小于末地址
        assert!(
            self.test_free_area(start, end),  // 查看当前要‘注册’的内存是否已经被‘注册’过了。
            "memory area overlap"
        );
        let area = MemoryArea::new(
            start,
            end,
            Box::new(handler),
            attr
        );
        self.page_table.edit(|pt| area.map(pt));    // 需要按需分配的map
        self.areas.push(area);
    }

    fn test_free_area(&self, start_addr : usize, end_addr : usize) -> bool {
        self.areas
            .iter()
            .find(|area| area.is_overlap_with(start_addr, end_addr))
            .is_none()
    }

    pub unsafe fn activate(&self) {
        self.page_table.activate();
    }

    pub unsafe fn with(&self, f: impl FnOnce()) {
        self.page_table.with(f);
    }

    pub fn token(&self) -> usize {
        self.page_table.token()
    }
}

pub fn remap_kernel(dtb : usize) {
    let offset = - ( KERNEL_OFFSET as isize - MEMORY_OFFSET as isize);

    let mut memset = MemorySet::new();
    memset.push(
        stext as usize,
        etext as usize,
        MemoryAttr::new().set_execute().set_readonly(),
        Linear::new(offset),
    );
    memset.push(
        srodata as usize,
        erodata as usize,
        MemoryAttr::new().set_readonly(),
        Linear::new(offset),
    );
    memset.push(
        sdata as usize,
        edata as usize,
        MemoryAttr::new(),
        Linear::new(offset),
    );
    memset.push(
        bootstack as usize,
        bootstacktop as usize,
        MemoryAttr::new(),
        Linear::new(offset),
    );
    memset.push(
        sbss as usize,
        ebss as usize,
        MemoryAttr::new(),
        Linear::new(offset),
    );
    memset.push(
        dtb as usize,
        dtb as usize + MAX_DTB_SIZE,
        MemoryAttr::new(),
        Linear::new(offset),
    );
    unsafe{
        memset.activate();
    }
}