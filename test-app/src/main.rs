#![allow(unused_variables)]

use std::path::PathBuf;
use std::alloc::System;
use clap::{Arg, command, value_parser};
use plugin_core::*;

#[global_allocator]
static ALLOCATOR: System = System;

fn main() -> Result<(), String> {
    let args = command!()
        .arg(Arg::new("plugin-library")
            .short('p')
            .long("plugin-library")
            .required(true)
            .value_parser(value_parser!(PathBuf)))
    .get_matches();

    let mut plugins = ExternalPlugins::default();
    unsafe {
        plugins.load(args.get_one::<PathBuf>("plugin-library").unwrap())?;
    }
    let loader = plugins.loaders.pop().unwrap();

    let content = vec![0u8; 32];
    let factory = Box::leak(loader.load_factory_from_bytes(&content));

    let mut worker = factory.new_worker();

    let mut data = vec![
        DataBuffer{data: 0f64},
        DataBuffer{data: 1f64},
        DataBuffer{data: 2f64},
        DataBuffer{data: 3f64},
        DataBuffer{data: 4f64},
        DataBuffer{data: 5f64},
        DataBuffer{data: 6f64},
        DataBuffer{data: 7f64},
        DataBuffer{data: 8f64},
        DataBuffer{data: 9f64},
    ];

    let n = data.len();
    for i in 0..n {
        let d = data.pop().unwrap();
        worker.consume_inputs(d)?;
    }

    println!("{}", worker);

    Ok(())
}
