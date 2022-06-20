use dtb::{Reader, StructItem};
use multiboot::information::{Multiboot};
use alloc::vec::Vec;
use alloc::string::String;
use crate::arch::x86_64::MEM;
use crate::devicetree::{DeviceTree, DeviceTreeProperty, DeviceTreeBlob};

extern "C" {
    static mb_info: usize;
}

pub fn parse() -> Result<Reader<'static>, &'static str> {
    let blob: &[u8] = include_bytes_aligned!(64, "dtb/basic.dtb");
    let reader = Reader::read(blob).unwrap();

    Ok(reader)
}

pub fn read(reader: &Reader<'_>) {
    loaderlog!("Device Tree:");

    for entry in reader.reserved_mem_entries(){
        loaderlog!("reserved: {:#X?}, {:#X?}", entry.address, entry.size);
    }

    let mut indent = 0;
    for entry in reader.struct_items() {
        match entry {
            StructItem::BeginNode { name } => {
                loaderlog!("{:indent$}{} {{", "", name, indent = indent);
                indent += 2;
            }
            StructItem::EndNode => {
                indent -= 2;
                loaderlog!("{:indent$}}}", "", indent = indent);
            }
            StructItem::Property { name, value } => {
                loaderlog!("{:indent$}{}: {:?}", "", name, value, indent = indent)
            }
        }
    }
}

pub fn read_from_address(addr: usize) {
    let reader = unsafe { Reader::read_from_address(addr).unwrap() };

    for entry in reader.reserved_mem_entries(){
        loaderlog!("reserved: {:#X?}, {:#X?}", entry.address, entry.size);
    }

    let mut indent = 0;
    for entry in reader.struct_items() {
        match entry {
            StructItem::BeginNode { name } => {
                loaderlog!("{:indent$}{} {{", "", name, indent = indent);
                indent += 2;
            }
            StructItem::EndNode => {
                indent -= 2;
                loaderlog!("{:indent$}}}", "", indent = indent);
            }
            StructItem::Property { name, value } => {
                loaderlog!("{:indent$}{}: {:?}", "", name, value, indent = indent)
            }
        }
    }
}

pub unsafe fn from_mb() -> Result<Vec<u8>, &'static str> {
    assert!(mb_info > 0, "Could not find Multiboot information");
	loaderlog!("Found Multiboot information at {:#x}", mb_info);

    // Load the Multiboot information
    let multiboot = Multiboot::from_ptr(mb_info as u64, &mut MEM).unwrap();
    let memory_regions = multiboot
        .memory_regions()
        .expect("Could not find a memory map in the Multiboot information");

    let mut reg: Vec<(u32, u32)> = Vec::new();
    
    for m in memory_regions {
        reg.push((m.base_address() as u32, m.length() as u32)); 
    }

    let mut dt = DeviceTree::new();

    dt.edit_property(&String::from("memory"), &String::from("reg"), DeviceTreeProperty::MultipleUnsignedInt32_32(reg));

    let blob_as_vec = dt.to_blob().unwrap();
    let blob = DeviceTreeBlob::from_slice(blob_as_vec.as_slice()).unwrap();

    if blob.compatibility_check().is_ok() {
        return Ok(dt.to_blob().unwrap());
    }
    else {
        return Err("not compatible");
    }
}
