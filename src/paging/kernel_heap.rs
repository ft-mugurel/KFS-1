use core::mem::size_of;
use core::ptr;

use super::vmem;
use crate::{pr_debug, pr_warn};

const HEAP_ALIGNMENT: usize = 16;
const HEAP_MAGIC: u32 = 0x4B_48_45_50;
const BLOCK_MAGIC: u32 = 0x4B_42_4C_4B;

#[repr(C, align(16))]
struct HeapChunk {
    magic: u32,
    total_size: usize,
    prev: *mut HeapChunk,
    next: *mut HeapChunk,
}

#[repr(C, align(16))]
struct BlockHeader {
    magic: u32,
    total_size: usize,
    usable_size: usize,
    requested_size: usize,
    chunk: *mut HeapChunk,
    prev_free: *mut BlockHeader,
    next_free: *mut BlockHeader,
    free: u32,
}

#[repr(C, align(16))]
struct BlockFooter {
    magic: u32,
    total_size: usize,
}

static mut HEAP_READY: bool = false;
static mut HEAP_CHUNKS: *mut HeapChunk = ptr::null_mut();
static mut HEAP_FREE_LIST: *mut BlockHeader = ptr::null_mut();

fn align_up(value: usize, align: usize) -> Option<usize> {
    if align == 0 || !align.is_power_of_two() {
        return None;
    }

    let mask = align - 1;
    value.checked_add(mask).map(|rounded| rounded & !mask)
}

fn block_header_size() -> usize {
    size_of::<BlockHeader>()
}

fn block_footer_size() -> usize {
    size_of::<BlockFooter>()
}

fn chunk_header_size() -> usize {
    size_of::<HeapChunk>()
}

fn block_alignment_overhead() -> usize {
    block_header_size() + block_footer_size()
}

fn minimum_free_block_size() -> usize {
    block_alignment_overhead() + HEAP_ALIGNMENT
}

unsafe fn block_footer_ptr(block: *mut BlockHeader) -> *mut BlockFooter {
    (block as *mut u8).add((*block).total_size - block_footer_size()) as *mut BlockFooter
}

unsafe fn next_block_ptr(block: *mut BlockHeader) -> Option<*mut BlockHeader> {
    let next_addr = (block as usize).checked_add((*block).total_size)?;
    let chunk = (*block).chunk;
    let chunk_start = chunk as usize;
    let chunk_end = chunk_start.checked_add((*chunk).total_size)?;

    if next_addr >= chunk_end {
        None
    } else {
        Some(next_addr as *mut BlockHeader)
    }
}

unsafe fn prev_block_ptr(block: *mut BlockHeader) -> Option<*mut BlockHeader> {
    let chunk = (*block).chunk;
    let chunk_start = chunk as usize;
    let block_start = block as usize;
    let first_block_start = chunk_start.checked_add(chunk_header_size())?;

    if block_start <= first_block_start {
        return None;
    }

    let footer_addr = block_start.checked_sub(block_footer_size())?;
    if footer_addr < first_block_start {
        return None;
    }

    let footer = footer_addr as *const BlockFooter;
    if (*footer).magic != BLOCK_MAGIC {
        return None;
    }

    let prev_total = (*footer).total_size;
    if prev_total < minimum_free_block_size() {
        return None;
    }

    let prev_start = block_start.checked_sub(prev_total)?;
    if prev_start < first_block_start {
        return None;
    }

    let prev = prev_start as *mut BlockHeader;
    if (*prev).magic != BLOCK_MAGIC || (*prev).total_size != prev_total {
        return None;
    }

    Some(prev)
}

unsafe fn write_footer(block: *mut BlockHeader) {
    let footer = block_footer_ptr(block);
    (*footer).magic = BLOCK_MAGIC;
    (*footer).total_size = (*block).total_size;
}

unsafe fn free_list_remove(block: *mut BlockHeader) {
    let prev = (*block).prev_free;
    let next = (*block).next_free;

    if !prev.is_null() {
        (*prev).next_free = next;
    } else {
        HEAP_FREE_LIST = next;
    }

    if !next.is_null() {
        (*next).prev_free = prev;
    }

    (*block).prev_free = ptr::null_mut();
    (*block).next_free = ptr::null_mut();
}

unsafe fn free_list_push(block: *mut BlockHeader) {
    (*block).prev_free = ptr::null_mut();
    (*block).next_free = HEAP_FREE_LIST;

    if !HEAP_FREE_LIST.is_null() {
        (*HEAP_FREE_LIST).prev_free = block;
    }

    HEAP_FREE_LIST = block;
}

unsafe fn chunk_list_remove(chunk: *mut HeapChunk) {
    let prev = (*chunk).prev;
    let next = (*chunk).next;

    if !prev.is_null() {
        (*prev).next = next;
    } else {
        HEAP_CHUNKS = next;
    }

    if !next.is_null() {
        (*next).prev = prev;
    }

    (*chunk).prev = ptr::null_mut();
    (*chunk).next = ptr::null_mut();
}

unsafe fn heap_chunk_from_block(block: *mut BlockHeader) -> Option<*mut HeapChunk> {
    let chunk = (*block).chunk;
    if chunk.is_null() {
        return None;
    }

    if (*chunk).magic != HEAP_MAGIC {
        return None;
    }

    Some(chunk)
}

unsafe fn find_free_block(total_size: usize) -> Option<*mut BlockHeader> {
    let mut current = HEAP_FREE_LIST;

    while !current.is_null() {
        if (*current).free != 0 && (*current).total_size >= total_size {
            return Some(current);
        }

        current = (*current).next_free;
    }

    None
}

unsafe fn split_block(block: *mut BlockHeader, total_size: usize) {
    let remaining = (*block).total_size - total_size;
    if remaining < minimum_free_block_size() {
        return;
    }

    let remainder = (block as *mut u8).add(total_size) as *mut BlockHeader;
    (*remainder).magic = BLOCK_MAGIC;
    (*remainder).total_size = remaining;
    (*remainder).usable_size = remaining - block_alignment_overhead();
    (*remainder).requested_size = 0;
    (*remainder).chunk = (*block).chunk;
    (*remainder).prev_free = ptr::null_mut();
    (*remainder).next_free = ptr::null_mut();
    (*remainder).free = 1;
    write_footer(remainder);
    free_list_push(remainder);

    (*block).total_size = total_size;
    (*block).usable_size = total_size - block_alignment_overhead();
    write_footer(block);
}

unsafe fn release_chunk_if_empty(block: *mut BlockHeader) -> bool {
    let chunk = match heap_chunk_from_block(block) {
        Some(chunk) => chunk,
        None => return false,
    };

    let first_block = (chunk as *mut u8).add(chunk_header_size()) as *mut BlockHeader;
    let full_free_span = (*chunk).total_size - chunk_header_size();

    if block as usize == first_block as usize && (*block).total_size == full_free_span {
        chunk_list_remove(chunk);
        if !vmem::vfree(chunk as *mut u8) {
            pr_warn!(
                "kernel heap failed to return chunk to vmem base={:#x}\n",
                chunk as usize
            );
        }
        return true;
    }

    false
}

pub fn init_kernel_heap() {
    unsafe {
        HEAP_READY = true;
        HEAP_CHUNKS = ptr::null_mut();
        HEAP_FREE_LIST = ptr::null_mut();
    }

    pr_debug!("kernel heap initialized\n");
}

fn ensure_heap_ready() {
    unsafe {
        if !HEAP_READY {
            init_kernel_heap();
        }
    }
}

pub fn kmalloc(size: usize) -> Option<*mut u8> {
    if size == 0 {
        pr_warn!("kmalloc rejected zero-sized request\n");
        return None;
    }

    ensure_heap_ready();

    let payload_size = align_up(size, HEAP_ALIGNMENT)?;
    let total_size = align_up(block_alignment_overhead() + payload_size, HEAP_ALIGNMENT)?;

    unsafe {
        let block = find_free_block(total_size);
        if block.is_none() {
            return None;
        }

        let block = block?;
        free_list_remove(block);
        split_block(block, total_size);

        (*block).magic = BLOCK_MAGIC;
        (*block).requested_size = size;
        (*block).usable_size = payload_size;
        (*block).free = 0;
        (*block).prev_free = ptr::null_mut();
        (*block).next_free = ptr::null_mut();
        write_footer(block);

        let user_ptr = (block as *mut u8).add(block_header_size());
        pr_debug!(
            "kmalloc size={} usable={} total={} ptr={:#x}\n",
            size,
            payload_size,
            (*block).total_size,
            user_ptr as usize
        );
        Some(user_ptr)
    }
}

pub fn kfree(ptr: *mut u8) -> bool {
    if ptr.is_null() {
        pr_warn!("kfree rejected null pointer\n");
        return false;
    }

    ensure_heap_ready();

    unsafe {
        let block = ptr.sub(block_header_size()) as *mut BlockHeader;
        if (*block).magic != BLOCK_MAGIC {
            pr_warn!("kfree rejected unknown pointer={:#x}\n", ptr as usize);
            return false;
        }

        if (*block).free != 0 {
            pr_warn!("kfree rejected double free ptr={:#x}\n", ptr as usize);
            return false;
        }

        let chunk = match heap_chunk_from_block(block) {
            Some(chunk) => chunk,
            None => {
                pr_warn!("kfree rejected corrupted chunk ptr={:#x}\n", ptr as usize);
                return false;
            }
        };

        (*block).free = 1;
        (*block).requested_size = 0;
        (*block).usable_size = (*block).total_size - block_alignment_overhead();

        let mut merged = block;

        if let Some(next) = next_block_ptr(merged) {
            if (*next).magic == BLOCK_MAGIC && (*next).free != 0 {
                free_list_remove(next);
                (*merged).total_size += (*next).total_size;
                (*merged).usable_size = (*merged).total_size - block_alignment_overhead();
                write_footer(merged);
            }
        }

        if let Some(prev) = prev_block_ptr(merged) {
            if (*prev).magic == BLOCK_MAGIC && (*prev).free != 0 {
                free_list_remove(prev);
                (*prev).total_size += (*merged).total_size;
                (*prev).usable_size = (*prev).total_size - block_alignment_overhead();
                (*prev).free = 1;
                (*prev).requested_size = 0;
                write_footer(prev);
                merged = prev;
            }
        }

        if release_chunk_if_empty(merged) {
            return true;
        }

        (*merged).chunk = chunk;
        free_list_push(merged);
        write_footer(merged);
        pr_debug!(
            "kfree ptr={:#x} total={} merged_block={:#x}\n",
            ptr as usize,
            (*merged).total_size,
            merged as usize
        );
        true
    }
}

pub fn ksize(ptr: *const u8) -> Option<usize> {
    if ptr.is_null() {
        return None;
    }

    ensure_heap_ready();

    unsafe {
        let block = ptr.sub(block_header_size()) as *const BlockHeader;
        if (*block).magic != BLOCK_MAGIC || (*block).free != 0 {
            return None;
        }

        let chunk = (*block).chunk;
        if chunk.is_null() || (*chunk).magic != HEAP_MAGIC {
            return None;
        }

        Some((*block).requested_size)
    }
}
