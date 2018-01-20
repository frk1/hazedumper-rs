use memlib::{Module, Process};
use nom::*;
use std::str;

#[derive(Debug, Clone, PartialEq)]
pub struct ClientClass {
    pub name: String,                   // ptr at 0x08
    pub table: Option<RecvTable>,       // ptr at 0x0C
    pub next: Option<Box<ClientClass>>, // ptr at 0x10
    pub id: i32,                        // ptr at 0x14
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecvTable {
    pub name: String,         // ptr at 0x0C
    pub props: Vec<RecvProp>, // ptr at 0x00, num at 0x04
} // Size: 0x10

#[derive(Debug, Clone, PartialEq)]
pub struct RecvProp {
    pub name: String,             // pName at 0x00
    pub table: Option<RecvTable>, // pDataTable at 0x28
    pub offset: i32,
} // Size: 0x3C

named!(
    parse_cstring<&str>,
    map_res!(take_until!("\0"), str::from_utf8)
);

impl ClientClass {
    // name-ptr, table-ptr, next-ptr, id
    named!(pub parse <(u32, u32, u32, i32)>,
        do_parse!(
            take!(0x08)    >>
            name  : le_u32 >>
            table : le_u32 >>
            next  : le_u32 >>
            id    : le_i32 >>
            ( (name, table, next, id) )
        )
    );

    pub fn new(base: usize, module: &Module) -> Option<Box<ClientClass>> {
        if base == 0 {
            return None;
        }

        debug!("Starting to parse ClientClass at 0x{:X}", base);
        let (pname, ptable, pnext, id) = ClientClass::parse(module.get_slice(base, 0x18, false)?)
            .ok()?
            .1;
        debug!(
            "ClientClass at 0x{:X} => name 0x{:X}, table 0x{:X}, next 0x{:X}, id 0x{:X}",
            base, pname, ptable, pnext, id
        );
        let name = parse_cstring(module.get_slice(pname as usize, 0x80, false)?)
            .ok()?
            .1;
        debug!("ClientClass at 0x{:X} => {}", base, name);

        let table = RecvTable::new(ptable as usize, module);
        let next = match pnext {
            0 => None,
            _ => ClientClass::new(pnext as usize, module),
        };

        Some(Box::new(ClientClass {
            name: name.to_string(),
            table: table,
            next: next,
            id: id,
        }))
    }
}

impl RecvTable {
    // pProps, numProps, pName
    named!(pub parse <(u32, u32, u32)>,
        do_parse!(
            props    : le_u32 >>
            numprops : le_u32 >>
            take!(0x4)        >>
            name     : le_u32 >>
            ( (props, numprops, name) )
        )
    );

    pub fn new(base: usize, module: &Module) -> Option<Self> {
        if base == 0 {
            return None;
        }

        debug!("Starting to parse RecvTable at 0x{:X}", base);
        let (props, numprops, pname) = RecvTable::parse(module.get_slice(base, 0x10, false)?)
            .ok()?
            .1;
        debug!(
            "RecvTable at 0x{:X} => name 0x{:X}, props 0x{:X}, numprops 0x{:X}",
            base, pname, props, numprops
        );
        let name = parse_cstring(module.get_slice(pname as usize, 0x80, false)?)
            .ok()?
            .1;
        debug!("RecvTable at 0x{:X} => {}", base, name);

        let mut vec_props = vec![];
        for i in 0..numprops {
            let prop_base = (props + i * 0x3C) as usize;
            if let Some(prop) = RecvProp::new(prop_base, module) {
                vec_props.push(prop)
            }
        }

        Some(RecvTable {
            name: name.to_string(),
            props: vec_props,
        })
    }
}

impl RecvProp {
    // pname, ptable, offset
    named!(pub parse <(u32, u32, i32)>,
        do_parse!(
            name        : le_u32 >>
            take!(0x24)          >>
            table       : le_u32 >>
            offset      : le_i32 >>
            take!(0xC)           >>
            ( (name, table, offset) )
        )
    );

    pub fn new(base: usize, module: &Module) -> Option<Self> {
        if base == 0 {
            return None;
        }

        debug!("Starting to parse RecvProp at 0x{:X}", base);
        let (pname, ptable, offset) = RecvProp::parse(module.get_slice(base, 0x3C, false)?)
            .ok()?
            .1;
        debug!(
            "RecvProp at 0x{:X} => name 0x{:X}, table 0x{:X}, offset 0x{:X}",
            base, pname, ptable, offset
        );
        let name = parse_cstring(module.get_slice(pname as usize, 0x80, false)?)
            .ok()?
            .1;
        debug!("RecvProp at 0x{:X} => {}", base, name);

        let table = RecvTable::new(ptable as usize, module);

        Some(RecvProp {
            name: name.to_string(),
            table: table,
            offset: offset,
        })
    }
}

pub fn test(first: usize, process: &Process) -> Option<Box<ClientClass>> {
    let module = process.get_module("client.dll")?;
    ClientClass::new(first + module.base, &module)
}
