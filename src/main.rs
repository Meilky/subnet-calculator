use std::io::{stdin, BufWriter, Write};
use std::time::Instant;

type Ip = u32;
type Cidr = u32;

fn create_netmask(cidr: Cidr) -> Result<(Ip, Ip), String> {
    if cidr > 32 {
        return Err("Cidr should be in between 0 and 32!".to_string());
    }

    let right_len = 32 - cidr;
    let all_bits = u32::MAX;
    let mask = (all_bits >> right_len) << right_len;

    Ok((mask, !mask))
}

fn create_network(netmask: &Ip, ip: &Ip) -> Ip {
    ip & netmask
}

fn create_broadcast(network: &Ip, wildmask: &Ip) -> Ip {
    network + wildmask
}

struct SubNet {
    pub network: Ip,
    pub first: Ip,
    pub last: Ip,
    pub broadcast: Ip,
    pub cidr: Cidr,
    pub nb_ip: u32,
}

impl SubNet {
    pub fn from(ip: Ip, cidr: Cidr) -> Result<Self, String> {
        let base: u32 = 2;
        let network: Ip;
        let first: Ip;
        let last: Ip;
        let broadcast: Ip;
        let nb_ip: u32;

        if cidr == 32 {
            network = ip;
            first = ip;
            last = ip;
            broadcast = ip;
            nb_ip = 0;
        } else if cidr == 31 {
            network = create_network(&0xfffffffe, &ip); // 255.255.255.254
            first = network;
            last = network + 1;
            broadcast = last;
            nb_ip = 0;
        } else {
            let result = create_netmask(cidr)?;
            let netmask = result.0;
            let wildmask = result.1;
            network = create_network(&netmask, &ip);
            first = network + 1;
            broadcast = create_broadcast(&network, &wildmask);
            last = broadcast - 1;
            nb_ip = base.pow((32 - cidr).into()) - 2;
        }

        Ok(SubNet {
            network,
            first,
            last,
            broadcast,
            cidr,
            nb_ip,
        })
    }

    pub fn split_in_subs(&self, cidr: Cidr) -> Result<Vec<String>, String> {
        let mut vec: Vec<String> = vec![];

        if cidr < self.cidr {
            return Err("Cidr can't be smaller than the current subnet one!".to_string());
        }

        if cidr == 32 || cidr == 31 {
            return Ok(vec![self.serialise_json()]);
        }

        let base: u32 = 2;

        let mut next_network = self.network;

        for _i in 0..base.pow(cidr - self.cidr) {
            let next_sub = SubNet::from(next_network, cidr)?;
            next_network = next_sub.broadcast + 1;

            vec.push(next_sub.serialise_json());
        }

        Ok(vec)
    }

    pub fn serialise_json(&self) -> String {
        let network: [u8; 4] = self.network.to_be_bytes();
        let first: [u8; 4] = self.first.to_be_bytes();
        let last: [u8; 4] = self.last.to_be_bytes();
        let broadcast: [u8; 4] = self.broadcast.to_be_bytes();

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
            self.nb_ip
        )
    }
}

impl std::fmt::Display for SubNet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.serialise_json())
    }
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

fn find_cidr_from_min_ip(min_ip: u32, cidr: Cidr) -> Result<Cidr, String> {
    let base: u32 = 2;

    if cidr == 32 || cidr == 31 {
        return Ok(cidr);
    }

    for i in (0..=30).rev() {
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

    let sub_cidr = find_cidr_from_min_ip(min_ip, cidr).unwrap();

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
