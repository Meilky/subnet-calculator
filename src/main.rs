use std::io::{stdin, BufWriter, Write};
use std::time::Instant;

mod parsing;
mod types;
mod subnet;

use crate::types::*;
use crate::parsing::*;
use crate::subnet::SubNet;

fn find_sub_cidr(min_ip: u32, cidr: Cidr) -> Result<Cidr, String> {
    let base: u32 = 2;

    if cidr == 32 || cidr == 31 {
        return Ok(cidr);
    }

    for i in (0..=31).rev() {
        if base.pow(
            (32 - i)
                .try_into()
                .expect("Problem while parsing some numbers"),
        ) - 2
            >= min_ip
        {
            return Ok(i);
        }
    }

    Err("No cidr found for your network".to_string())
}

fn main() {
    let mut raw_ip = String::new();
    println!("Ip address (<ip>/<cidr>):");
    stdin().read_line(&mut raw_ip).expect("failed to readline");

    let mut raw_nb = String::new();
    println!("Minimum number of usable ip per subnets:");
    stdin().read_line(&mut raw_nb).expect("failed to readline");

    if std::path::Path::new("file.json").exists() {
        std::fs::remove_file("file.json").expect("remove failed");
    }

    let start = Instant::now();

    let (ip, cidr, min_ip) = parse_input(raw_ip, raw_nb).unwrap();

    let sub_cidr = find_sub_cidr(min_ip, cidr).unwrap();

    let mut file: BufWriter<std::fs::File> =
        BufWriter::new(std::fs::File::create("file.json").unwrap());

    let _ = file.write("[\n".as_bytes());

    let nb_subnet: u32 = (2 as u32).pow(sub_cidr - cidr);
    let nb_usable_ip: u32 = (2 as u32).pow((32 - sub_cidr).into()) - 2;

    let mut next_network = SubNet::new(ip, sub_cidr, nb_usable_ip);

    if nb_subnet == 1 {
        file.write(&next_network.to_string().as_bytes()).unwrap();

        file.write("\n]".as_bytes()).unwrap();

        let end = Instant::now();

        println!("After write : {:?}", end.duration_since(start));

        panic!();
    }

    for _i in 0..nb_subnet-1 {
        file.write(next_network.to_string().as_bytes()).unwrap();
        file.write(", ".as_bytes()).unwrap();

        next_network = SubNet::new(
            next_network.broadcast + 1,
            next_network.cidr,
            next_network.nb_usable_ip,
        );
    }

    file.write(next_network.to_string().as_bytes()).unwrap();

    file.write("\n]".as_bytes()).unwrap();

    let end = Instant::now();

    println!("After write : {:?}", end.duration_since(start));
}
