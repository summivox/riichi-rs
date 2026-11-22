use std::fs::File;
use std::io;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use itertools::Itertools;

use riichi_decomp_table::*;

pub fn table_to_string(table: &CTable) -> String {
    const GROUP_STR: [&str; 16] = [
        ",10", ",00", ",11", ",01", ",12", ",02", ",13", ",03",
        ",14", ",04", ",15", ",05", ",16", ",06", ",17", ",18",
    ];
    let mut lines: Vec<String> = vec![];
    for (&key, &value) in table.iter() {
        for grouping in c_entry_iter_alts(key, value) {
            let mid_str = grouping.groups
                .map(|ks| GROUP_STR[ks as usize])
                .sorted()
                .join("");
            if let Some(pair) = grouping.pair() {
                lines.push(format!("{:09o}{},2{}", key, mid_str, pair));
            } else {
                lines.push(format!("{:09o}{}", key, mid_str));
            }
        }
    }
    lines.sort();
    lines.join("\n")
}

fn main() -> io::Result<()> {
    let t1 = std::time::Instant::now();
    let c_table = make_c_table();
    let t2 = std::time::Instant::now();
    let dt_c = t2.duration_since(t1);
    let w_table = make_w_table(&c_table);
    let t3 = std::time::Instant::now();
    let dt_w = t3.duration_since(t2);

    println!("C table: {} us", dt_c.as_micros());
    println!("W table: {} us", dt_w.as_micros());
    println!("W table len: {}", w_table.len());

    let c_key = 0o333320000u32;
    let &c_value = c_table.get(&c_key).unwrap();
    for x in c_entry_iter_alts(c_key, c_value) {
        println!("{:?}", x);
    }

    println!();
    let w_key = 0o311111113u32;
    let &w_value = w_table.get(&w_key).unwrap();
    for x in w_entry_iter_alts(w_key, w_value) {
        println!("{:?}", x);
    }

    BufWriter::new(File::create(PathBuf::from("data/c.txt"))?)
        .write_all(table_to_string(&c_table).as_bytes())?;

    let mut c_map_gen = phf_codegen::Map::<u32>::new();
    for (k, v) in c_table.iter() {
        c_map_gen.entry(*k, format!("{}u64", v.packed()));
    }
    let mut map_file = BufWriter::new(File::create(PathBuf::from("data/c.rs"))?);
    write!(map_file,
           "static C_TABLE: phf::Map<u32, u64> = \n{};\n",
           c_map_gen.build())?;

    /*
    let (key, v) =
        w_table.iter().max_by_key(|(k, v)| v.len()).unwrap();
    println!("max len: W[{:09o}] = {}", key, v.len());
    for entry in v.iter() {
        println!("{:?}", *entry);
    }
    */

    Ok(())
}
