use std::io::{stdin, BufWriter, Write};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Instant;

mod parsing;
mod subnet;

use crate::parsing::*;
use crate::subnet::SubNet;

fn find_sub_cidr(min_ip: u32, cidr: u8) -> Result<u8, String> {
    let base: u32 = 2;

    if cidr == 32 || cidr == 31 {
        return Ok(cidr);
    }

    for i in (0..=31).rev() {
        if base.pow(32 - i) - 2 >= min_ip {
            return Ok(i as u8);
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

    let nb_subnet: u32 = (2 as u32).pow((sub_cidr - cidr) as u32);

    let nb_usable_ip: u32 = (2 as u32).pow((32 - sub_cidr) as u32) - 2;

    let next_network = SubNet::new(ip, sub_cidr, nb_usable_ip);

    if nb_subnet == 1 {
        file.write(&next_network.to_string().as_bytes()).unwrap();

        file.write("\n]".as_bytes()).unwrap();

        let end = Instant::now();

        println!("After write : {:?}", end.duration_since(start));

        return;
    }

    let nb_thread: u8 = 8;
    let nb_subnet_per_subnet = nb_subnet / nb_thread as u32;

    let mut handles: Vec<JoinHandle<_>> = vec![];
    let subnets: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(vec![]));

    for i in 0..nb_thread {
        let subnets = Arc::clone(&subnets);

        let handle = thread::spawn(move || {
            let mut subnets_bytes: Vec<u8> = vec![];

            let mut next_sub = SubNet::new(
                next_network.network + (nb_subnet_per_subnet * (i as u32) * (nb_usable_ip + 2)),
                sub_cidr,
                nb_usable_ip,
            );

            for j in (nb_subnet_per_subnet * i as u32)..(nb_subnet_per_subnet * (i + 1) as u32) {
                if j == nb_subnet {
                    break;
                }

                subnets_bytes.append(&mut next_sub.to_string().as_bytes().to_vec());
                subnets_bytes.append(&mut ",".as_bytes().to_vec());

                next_sub = SubNet::new(next_sub.broadcast + 1, next_sub.cidr, nb_usable_ip);
            }

            let mut subs = subnets.lock().unwrap();

            subs.push(subnets_bytes);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let compute = Instant::now();

    println!("Time to compute: {:?}", compute.duration_since(start));

    for subnet in subnets.lock().unwrap().iter() {
        file.write_all(&subnet).unwrap();
    }

    let _ = file.write("\n]".as_bytes());

    file.flush().unwrap();

    let end = Instant::now();

    println!("Time to write: {:?}", end.duration_since(compute));

    println!("Total time: {:?}", end.duration_since(start));
}
