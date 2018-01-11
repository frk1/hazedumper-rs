extern crate memlib;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ClientClass {
    base: u32,
    ptr_next_class: u32,
    ptr_recv_table: u32,
    class_id: i32,
    class_name: String,
}

#[derive(Debug)]
pub struct RecvTable {
    base: u32,
    ptr_recv_prop: u32,
    count_props: u32,
    table_name: String,
}

#[derive(Debug)]
pub struct RecvProp {
    base: u32,
    ptr_recv_table: u32,
    offset: u32,
    var_name: String,
}

#[derive(Debug)]
pub struct NetvarManager {
    pub recv_tables: HashMap<String, RecvTable>,
}

fn read_c_string(process: &memlib::process::Process, offset: usize, count: usize) -> String {
    let mut buf = vec![0u8; count];
    process.read_ptr(buf.as_mut_ptr(), offset, count);

    let cstr = String::from_utf8_lossy(&buf).to_string();
    let first_0 = cstr.find('\0').unwrap_or_else(|| cstr.len());
    cstr[..first_0].to_string()
}

impl ClientClass {
    pub fn new() -> Self {
        ClientClass {
            base: 0,
            ptr_next_class: 0,
            ptr_recv_table: 0,
            class_id: 0,
            class_name: "".to_string(),
        }
    }

    pub fn update(&mut self, process: &memlib::process::Process, ptr: u32) {
        let ptr_class_name: u32 = process.read(ptr as usize + 0x8).unwrap();

        self.base = ptr;
        self.ptr_next_class = process.read(ptr as usize + 0x10).unwrap();
        self.ptr_recv_table = process.read(ptr as usize + 0xC).unwrap();
        self.class_id = process.read(ptr as usize + 0x14).unwrap();
        self.class_name = read_c_string(process, ptr_class_name as usize, 0x20);
    }

    pub fn get(process: &memlib::process::Process, ptr: u32) -> Option<Self> {
        if ptr == 0 {
            return None;
        }

        let mut buf = Self::new();
        buf.update(process, ptr);
        Some(buf)
    }
}

impl RecvTable {
    pub fn new() -> Self {
        RecvTable {
            base: 0,
            ptr_recv_prop: 0,
            count_props: 0,
            table_name: "".to_string(),
        }
    }

    pub fn update(&mut self, process: &memlib::process::Process, ptr: u32) {
        let ptr_table_name: u32 = process.read(ptr as usize + 0xC).unwrap();

        self.base = ptr;
        self.ptr_recv_prop = process.read(ptr as usize).unwrap();
        self.count_props = process.read(ptr as usize + 0x4).unwrap();
        self.table_name = read_c_string(process, ptr_table_name as usize, 0x20);
    }

    pub fn get(process: &memlib::process::Process, ptr: u32) -> Option<Self> {
        if ptr == 0 {
            return None;
        }

        let mut buf = Self::new();
        buf.update(process, ptr);
        Some(buf)
    }

    fn get_prop(&self, i: u32) -> u32 {
        self.ptr_recv_prop + 0x3C * i
    }
}

impl RecvProp {
    pub fn new() -> Self {
        RecvProp {
            base: 0,
            ptr_recv_table: 0,
            offset: 0,
            var_name: "".to_string(),
        }
    }

    pub fn update(&mut self, process: &memlib::process::Process, ptr: u32) {
        let ptr_var_name: u32 = process.read(ptr as usize).unwrap();

        self.base = ptr;
        self.ptr_recv_table = process.read(ptr as usize + 0x28).unwrap();
        self.offset = process.read(ptr as usize + 0x2C).unwrap();
        self.var_name = read_c_string(process, ptr_var_name as usize, 0x40);
    }

    pub fn get(process: &memlib::process::Process, ptr: u32) -> Option<Self> {
        if ptr == 0 {
            return None;
        }

        let mut buf = Self::new();
        buf.update(process, ptr);
        Some(buf)
    }
}

impl NetvarManager {
    fn new() -> Self {
        NetvarManager {
            recv_tables: HashMap::new(),
        }
    }

    pub fn load_netdata(process: &memlib::process::Process, mut ptr_class: usize) -> Option<Self> {
        let process: &memlib::process::Process = process;
        let mut manager = Self::new();

        // We have to dereference twice
        ptr_class = process.read(ptr_class).unwrap();

        while let Some(cclass) = ClientClass::get(process, ptr_class as u32) {
            if let Some(table) = RecvTable::get(process, cclass.ptr_recv_table) {
                manager.recv_tables.insert(table.table_name.clone(), table);
            }
            ptr_class = cclass.ptr_next_class as usize;
        }

        Some(manager)
    }

    fn get_recv_prop_ref(
        &self,
        process: &memlib::process::Process,
        table: &RecvTable,
        prop_name: &str,
    ) -> Option<usize> {
        if table.count_props == 0 {
            return None;
        }

        let mut offset: usize = 0;
        for i in 0..table.count_props {
            if let Some(prop) = RecvProp::get(process, table.get_prop(i)) {
                RecvTable::get(process, prop.ptr_recv_table).map_or((), |child| {
                    offset += self.get_recv_prop_ref(process, &child, prop_name)
                        .map_or(0, |tmp| prop.offset as usize + tmp);
                });
                if prop_name == prop.var_name {
                    return Some(offset + prop.offset as usize);
                }
            }
        }

        if offset > 0 {
            Some(offset)
        } else {
            None
        }
    }

    pub fn get_recv_prop(
        &self,
        process: &memlib::process::Process,
        table_name: &str,
        prop_name: &str,
    ) -> Option<usize> {
        self.recv_tables
            .get(table_name)
            .and_then(|table| self.get_recv_prop_ref(process, table, prop_name))
    }
}
