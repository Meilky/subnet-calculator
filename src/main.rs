use std::io::{stdin, BufWriter, Write};
use std::time::Instant;

mod subnet;
mod utils;

use subnet::SubNet;
use utils::{Cidr, Ip};

fn parse_ip(raw_ip: String) -> Result<Ip, String> {
    let split_ip: Vec<&str> = raw_ip.split(".").collect();

    if split_ip.len() != 4 {
        return Err("The inserted value isn't an proprely formated ip!".to_string());
    }

    let mut ip: Ip = 0;
    let base: u32 = 2;

    for i in 0..4 {
        let s = match split_ip.get(i) {
            Some(v) => v,
            _ => return Err(format!("Invalid ip at position (no value): {}", i)),
        };

        let value = match s.parse::<u32>() {
            Ok(v) => v,
            _ => return Err(format!("Invalid ip at position (no a int): {}", i)),
        };

        ip += value
            * base.pow(
                ((3 - i) * 8)
                    .try_into()
                    .expect("Error will transforming u8 to u32"),
            );
    }

    Ok(ip)
}

fn parse_cidr(raw_cidr: String) -> Result<Cidr, String> {
    let cidr = raw_cidr.parse::<u32>().expect("Cidr isn't a number");

    if cidr > 32 {
        return Err("The cidr should be under 32!".to_string());
    }

    Ok(cidr)
}

fn parse_min_nb(raw_nb: String, cidr: Cidr) -> Result<u32, String> {
    let base: u32 = 2;
    let nb: u32 = raw_nb
        .trim()
        .parse::<u32>()
        .expect("The min nb of ip is not a number");

    if cidr == 32 || cidr == 31 {
        return Ok(0);
    }

    if base.pow(32 - cidr) - 2 < nb {
        return Err(
            "The min nb cannot be bigger then the max number of usable ip of the network"
                .to_string(),
        );
    }

    Ok(nb)
}

fn parse_input(raw_ip: String, raw_nb: String) -> Result<(Ip, Cidr, u32), String> {
    let split_ip: Vec<&str> = raw_ip.trim().split("/").collect();

    if split_ip.len() != 2 {
        return Err("The ip should be formated this way : <ip>/<cidr>".to_string());
    }

    let ip = parse_ip(split_ip.get(0).expect("No ip provided").to_string())?;
    let cidr = parse_cidr(split_ip.get(1).expect("No cidr provided").to_string())?;
    let min_nb: u32 = parse_min_nb(raw_nb, cidr)?;

    Ok((ip, cidr, min_nb))
}

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

    let start = Instant::now();

    let (ip, cidr, min_ip) = parse_input(raw_ip, raw_nb).unwrap();

    let sub_cidr = find_sub_cidr(min_ip, cidr).unwrap();

    let subnet = match SubNet::from(ip, cidr) {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };

    let subs = subnet.split_in_subs(sub_cidr);

    let b_write = Instant::now();
    println!(
        "before write (with serialisation) : {:?}",
        b_write.duration_since(start)
    );

    if std::path::Path::new("file.json").exists() {
        std::fs::remove_file("file.json").expect("remove failed");
    }

    let mut file: BufWriter<std::fs::File> =
        BufWriter::new(std::fs::File::create("file.json").unwrap());

    let _ = write!(file, "[\n");
    for sub in subs.unwrap() {
        let _ = write!(file, "{},", sub);
    }
    let _ = write!(file, "\n]");

    let end = Instant::now();

    println!("After write : {:?}", end.duration_since(start));
}
