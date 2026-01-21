use std::io::{BufWriter, Write, stdin};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Instant;

mod parsing;

use crate::parsing::*;

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

fn create_netmask(cidr: u8) -> (u32, u32) {
    let right_len = 32 - cidr;
    let netmask = (u32::MAX >> right_len) << right_len;

    (netmask, !netmask)
}

fn ip_to_string(ip: u32) -> String {
    let ip_bytes: [u8; 4] = ip.to_be_bytes();

    format!(
        "{}.{}.{}.{}",
        ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3],
    )
}

fn make_subnet_bytes(
    network: u32,
    first: u32,
    last: u32,
    broadcast: u32,
    nb_usable_ip: u32,
) -> Vec<u8> {
    let network_bytes: [u8; 4] = network.to_be_bytes();
    let first_bytes: [u8; 4] = first.to_be_bytes();
    let last_bytes: [u8; 4] = last.to_be_bytes();
    let broadcast_bytes: [u8; 4] = broadcast.to_be_bytes();

    format!(
                    //"{{\n\t\"network\": \"{}.{}.{}.{}\",\n\t\"first\": \"{}.{}.{}.{}\",\n\t\"last\": \"{}.{}.{}.{}\",\n\t\"broadcast\": \"{}.{}.{}.{}\",\n\t\"nbUsableIp\": {}\n}}",
                    "\t{{\n\t\t\"Network\": \"{}.{}.{}.{}\",\n\t\t\"First\": \"{}.{}.{}.{}\",\n\t\t\"Last\": \"{}.{}.{}.{}\",\n\t\t\"Broadcast\": \"{}.{}.{}.{}\",\n\t\t\"Size\": \"{}\"\n\t}}",
                    network_bytes[0],
                    network_bytes[1],
                    network_bytes[2],
                    network_bytes[3],
                    first_bytes[0],
                    first_bytes[1],
                    first_bytes[2],
                    first_bytes[3],
                    last_bytes[0],
                    last_bytes[1],
                    last_bytes[2],
                    last_bytes[3],
                    broadcast_bytes[0],
                    broadcast_bytes[1],
                    broadcast_bytes[2],
                    broadcast_bytes[3],
                    nb_usable_ip
                ).as_bytes().to_vec()
}

fn main() {
    let mut raw_ip = String::new();
    println!("Ip address (<ip>/<cidr>):");
    stdin().read_line(&mut raw_ip).expect("failed to readline");

    let mut raw_nb = String::new();
    println!("Minimum number of usable ip per subnets:");
    stdin().read_line(&mut raw_nb).expect("failed to readline");

    let mut raw_thread = String::new();
    println!("Number of thread used for compute:");
    stdin()
        .read_line(&mut raw_thread)
        .expect("failed to readline");

    if std::path::Path::new("file.json").exists() {
        std::fs::remove_file("file.json").expect("remove failed");
    }

    let start = Instant::now();

    let mut file: BufWriter<std::fs::File> =
        BufWriter::new(std::fs::File::create("file.json").unwrap());

    let (ip, cidr, min_ip, nb_thread) = parse_input(raw_ip, raw_nb, raw_thread).unwrap();

    let sub_cidr = find_sub_cidr(min_ip, cidr).unwrap();

    let (sub_netmask, sub_wildmask) = create_netmask(sub_cidr);
    let nb_subnet: u32 = (2 as u32).pow((sub_cidr - cidr) as u32);
    let nb_usable_ip_per_subnet: u32 = (2 as u32).pow((32 - sub_cidr) as u32) - 2;

    let nb_subnet_per_thread = nb_subnet / nb_thread as u32;

    let _ = file.write("[\n".as_bytes());

    let first_network = ip & sub_netmask;

    if nb_subnet < nb_thread as u32 {
        let mut network = first_network;
        let mut first: u32;
        let mut last: u32;
        let mut broadcast: u32;

        if cidr == 32 {
            first = network;
            last = network;
            broadcast = network;
        } else if cidr == 31 {
            first = network;
            last = network + 1;
            broadcast = last;
        } else {
            first = network + 1;
            broadcast = network + sub_wildmask;
            last = broadcast - 1;
        }

        for _ in 0..nb_subnet {
            file.write(&make_subnet_bytes(
                network,
                first,
                last,
                broadcast,
                nb_usable_ip_per_subnet,
            ))
            .unwrap();

            file.write(",\n".as_bytes()).unwrap();

            network = broadcast + 1;
            first = network + 1;
            broadcast = network + nb_usable_ip_per_subnet + 1;
            last = broadcast - 1;
        }

        file.write("]".as_bytes()).unwrap();

        file.flush().unwrap();

        let end = Instant::now();

        println!("Total time : {:?}", end.duration_since(start));

        return;
    }

    let mut handles: Vec<JoinHandle<_>> = vec![];
    let subnets: Arc<Mutex<Vec<(u8, Vec<u8>)>>> = Arc::new(Mutex::new(vec![]));

    for i in 0..nb_thread {
        let subnets = Arc::clone(&subnets);

        let handle = thread::spawn(move || {
            let mut subnets_bytes: Vec<u8> = vec![];

            let mut network =
                first_network + (nb_subnet_per_thread * (i as u32) * (nb_usable_ip_per_subnet + 2));
            let mut first: u32;
            let mut last: u32;
            let mut broadcast: u32;

            if cidr == 32 {
                first = network;
                last = network;
                broadcast = network;
            } else if cidr == 31 {
                first = network;
                last = network + 1;
                broadcast = last;
            } else {
                first = network + 1;
                broadcast = network + sub_wildmask;
                last = broadcast - 1;
            }

            let mut end = nb_subnet_per_thread;

            if i == nb_thread - 1 {
                end = nb_subnet - (nb_subnet_per_thread * (nb_thread - 1) as u32)
            }

            for _ in 0..end {
                subnets_bytes.append(&mut make_subnet_bytes(
                    network,
                    first,
                    last,
                    broadcast,
                    nb_usable_ip_per_subnet,
                ));
                subnets_bytes.append(&mut ",\n".as_bytes().to_vec());

                network = broadcast + 1;
                first = network + 1;
                broadcast = network + nb_usable_ip_per_subnet + 1;
                last = broadcast - 1;
            }

            let mut subs = subnets.lock().unwrap();

            subs.push((i, subnets_bytes));
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let compute = Instant::now();

    println!("Time to compute: {:?}", compute.duration_since(start));

    let subnets = subnets.lock().unwrap();

    for i in 0..nb_thread {
        let (_, subnet) = subnets.iter().find(|(id, _)| *id == i).unwrap();

        file.write_all(&subnet).unwrap();
    }

    let _ = file.write("]".as_bytes());

    file.flush().unwrap();

    let end = Instant::now();

    println!("Time to write: {:?}", end.duration_since(compute));

    println!("Total time: {:?}", end.duration_since(start));
}
