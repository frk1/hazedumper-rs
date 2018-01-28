#![cfg_attr(feature = "cargo-clippy", allow(double_parens))]

extern crate zero;

use std::collections::BTreeMap;
use std::fmt;
use std::str;

use memlib::{Module, Process};

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct _ClientClass {
    pad1: [u8; 8],
    pub name: u32,  // 0x08
    pub table: u32, // 0x0C
    pub next: u32,  // 0x10
    pub id: i32,    // 0x14
} // Size 0x18

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct _RecvTable {
    pub props: u32,    // 0x00
    pub numprops: u32, // 0x04
    pad1: [u8; 4],     // 0x08
    pub name: u32,     // 0x0C
} // Size 0x10

#[derive(Copy, Clone)]
#[repr(C)]
pub struct _RecvProp {
    pub name: u32,    // 0x00
    pad1: [u8; 0x24], // 0x04
    pub table: u32,   // 0x28
    pub offset: i32,  // 0x2C
} // Size 0x30

unsafe impl zero::Pod for _ClientClass {}
unsafe impl zero::Pod for _RecvTable {}
unsafe impl zero::Pod for _RecvProp {}

impl fmt::Display for _ClientClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ClientClass => name {:#X}, table {:#X}, next {:#X}, id {:#X}",
            self.name, self.table, self.next, self.id
        )
    }
}

impl fmt::Display for _RecvTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "RecvTable => props {:#X}, numprops {:#X}, name {:#X}",
            self.props, self.numprops, self.name
        )
    }
}

impl fmt::Display for _RecvProp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "RecvProp => name {:#X}, table {:#X}, offset {:#X}",
            self.name, self.table, self.offset
        )
    }
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct NetvarManager {
    tables: BTreeMap<String, RecvTable>,
}

impl ClientClass {
    pub fn new(base: usize, module: &Module) -> Option<Box<ClientClass>> {
        if base == 0 {
            return None;
        }

        trace!("Starting to parse ClientClass at {:#X}", base);
        let cc = zero::read::<_ClientClass>(module.get_slice(base, 0x18, false)?);
        trace!("{}", cc);
        let name = zero::read_str(module.get(cc.name as usize, false)?);
        trace!("ClientClass at {:#X} => {}", base, name);

        let table = RecvTable::new(cc.table as usize, module);
        let next = match cc.next {
            0 => None,
            _ => ClientClass::new(cc.next as usize, module),
        };

        Some(Box::new(ClientClass {
            name: name.to_string(),
            table: table,
            next: next,
            id: cc.id,
        }))
    }
}

impl RecvTable {
    pub fn new(base: usize, module: &Module) -> Option<Self> {
        if base == 0 {
            return None;
        }

        trace!("Starting to parse RecvTable at {:#X}", base);
        let table = zero::read::<_RecvTable>(module.get_slice(base, 0x10, false)?);
        trace!("{}", table);
        let name = zero::read_str(module.get(table.name as usize, false)?);
        trace!("RecvTable at {:#X} => {}", base, name);

        let mut vec_props = vec![];
        for i in 0..table.numprops {
            let prop_base = (table.props + i * 0x3C) as usize;
            if let Some(prop) = RecvProp::new(prop_base, module) {
                vec_props.push(prop)
            }
        }

        Some(RecvTable {
            name: name.to_string(),
            props: vec_props,
        })
    }

    pub fn get_offset(&self, netvar_name: &str) -> Option<i32> {
        for prop in &self.props {
            if let Some(o) = prop.get_offset(netvar_name) {
                return Some(o);
            }
        }
        None
    }
}

impl RecvProp {
    pub fn new(base: usize, module: &Module) -> Option<Self> {
        if base == 0 {
            return None;
        }

        trace!("Starting to parse RecvProp at {:#X}", base);
        let prop = zero::read::<_RecvProp>(module.get_slice(base, 0x30, false)?);
        trace!("{}", prop);
        let name = zero::read_str(module.get(prop.name as usize, false)?);
        trace!("RecvProp at {:#X} => {}", base, name);

        let table = RecvTable::new(prop.table as usize, module);

        Some(RecvProp {
            name: name.to_string(),
            table: table,
            offset: prop.offset,
        })
    }

    pub fn get_offset(&self, netvar_name: &str) -> Option<i32> {
        if self.name == netvar_name {
            return Some(self.offset);
        }

        match self.table {
            Some(ref table) => match table.get_offset(netvar_name) {
                Some(o) => Some(o + self.offset),
                _ => None,
            },
            _ => None,
        }
    }
}

impl NetvarManager {
    pub fn new(first: usize, process: &Process) -> Option<Self> {
        let module = process.get_module("client.dll")?;
        debug!("First ClientClass at {:#X}", first);

        let mut cc_opt = ClientClass::new(first + module.base, &module);
        let mut tables = BTreeMap::new();

        while let Some(cc) = cc_opt {
            if let Some(ref table) = cc.table {
                tables.insert(table.name.clone(), table.clone());
            }
            cc_opt = cc.next.clone();
        }

        debug!("NetvarManager adding {} tables!", tables.len());
        Some(NetvarManager { tables })
    }

    pub fn get_offset(&self, table_name: &str, netvar_name: &str) -> Option<i32> {
        self.tables.get(table_name)?.get_offset(netvar_name)
    }
}
