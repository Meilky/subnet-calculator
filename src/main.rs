use std::io::{stdin, BufWriter, Write};
use std::time::Instant;

type Ip = u32;
type Cidr = u32;

fn create_netmask(cidr: Cidr) -> (Ip, Ip) {
    let right_len = 32 - cidr;
    let netmask = (u32::MAX >> right_len) << right_len;

    (netmask, !netmask)
}

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

struct SubNet {
    pub network: Ip,
    pub first: Ip,
    pub last: Ip,
    pub broadcast: Ip,
    pub cidr: Cidr,
    pub nb_usable_ip: u32,
}

fn make_network(ip: Ip, cidr: Cidr, nb_usable_ip: u32) -> SubNet {
    let network: Ip;
    let first: Ip;
    let last: Ip;
    let broadcast: Ip;

    if cidr == 32 {
        network = ip;
        first = ip;
        last = ip;
        broadcast = ip;
    } else if cidr == 31 {
        network = ip & 0xfffffffe;
        first = network;
        last = network + 1;
        broadcast = last;
    } else {
        let (netmask, wildmask) = create_netmask(cidr);
        network = ip & netmask;
        first = network + 1;
        broadcast = network + wildmask;
        last = broadcast - 1;
    }

    SubNet {
        network,
        first,
        last,
        broadcast,
        cidr,
        nb_usable_ip,
    }
}

fn stringify_network(subnet: &SubNet) -> String {
    let network: [u8; 4] = subnet.network.to_be_bytes();
    let first: [u8; 4] = subnet.first.to_be_bytes();
    let last: [u8; 4] = subnet.last.to_be_bytes();
    let broadcast: [u8; 4] = subnet.broadcast.to_be_bytes();

    format!(
            "{{\n\t\"network\": \"{}.{}.{}.{}\",\n\t\"first\": \"{}.{}.{}.{}\",\n\t\"last\": \"{}.{}.{}.{}\",\n\t\"broadcast\": \"{}.{}.{}.{}\",\n\t\"nbUsableIp\": {}\n}}",
            network[0],
            network[1],
            network[2],
            network[3],
            first[0],
            first[1],
            first[2],
            first[3],
            last[0],
            last[1],
            last[2],
            last[3],
            broadcast[0],
            broadcast[1],
            broadcast[2],
            broadcast[3],
            subnet.nb_usable_ip
        )
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

    let mut next_network = make_network(ip, sub_cidr, nb_usable_ip);

    if nb_subnet == 1 {
        file.write(stringify_network(&next_network).to_owned().as_bytes()).unwrap();

        file.write("\n]".as_bytes()).unwrap();

        let end = Instant::now();

        println!("After write : {:?}", end.duration_since(start));

        panic!();
    }

    for _i in 0..nb_subnet {
        file.write(stringify_network(&next_network).to_owned().as_bytes()).unwrap();

        next_network = make_network(
            next_network.broadcast + 1,
            next_network.cidr,
            next_network.nb_usable_ip,
        );
    }

    file.write("\n]".as_bytes()).unwrap();

    let end = Instant::now();

    println!("After write : {:?}", end.duration_since(start));
}
