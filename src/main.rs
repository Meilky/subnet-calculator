use std::io::{BufWriter, Write};
use std::time::Instant;

type Ip = u32;
type Cidr = u32;

fn create_netmask(cidr: Cidr) -> Result<(Ip, Ip), String> {
    let base: u32 = 2;
    let base2: u32 = 256;
    let netmask: Ip;
    let wildmask: Ip;

    if (0..8).contains(&cidr) {
        let num: u32 = base.pow(8 - cidr);

        if num >= 255 {
            netmask = 4278190080; // 255.0.0.0
        } else if num <= 1 {
            netmask = 0; // 0.0.0.0
        } else {
            netmask = (u32::MAX - (num * base2.pow(3))) + 1;
        }

        wildmask = !netmask;
    } else if (8..16).contains(&cidr) {
        let num: u32 = base.pow(16 - cidr);

        if num >= 255 {
            netmask = 4294901760; // 255.255.0.0
        } else if num <= 1 {
            netmask = 4278190080; // 255.0.0.0
        } else {
            netmask = 4278190080 + (base2.pow(3) - (num * base2.pow(2)));
        }

        wildmask = !netmask;
    } else if (16..24).contains(&cidr) {
        let num: u32 = base.pow(24 - cidr);

        if num >= 255 {
            netmask = 4294967040; // 255.255.255.0
        } else if num <= 1 {
            netmask = 4294901760; // 255.255.0.0
        } else {
            netmask = 4294901760 + (base2.pow(2) - (num * base2.pow(1)));
        }

        wildmask = !netmask;
    } else if (24..33).contains(&cidr) {
        let num: u32 = base.pow(32 - cidr);

        if num >= 255 {
            netmask = 4294967295; // 255.255.255.255
        } else if num <= 1 {
            netmask = 4294967040; // 255.255.255.0
        } else {
            netmask = 4294967040 + (256 - num);
        }

        wildmask = !netmask;
    } else {
        return Err(String::from("Cidr should be in between 0 and 32!"));
    }

    Ok((netmask, wildmask))
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
            nb_ip = 1;
        } else if cidr == 31 {
            network = create_network(&4294967294, &ip); // 255.255.255.254
            first = network;
            last = network + 1;
            broadcast = last;
            nb_ip = 2;
        } else {
            let result = create_netmask(cidr).unwrap();
            let netmask = result.0;
            let wildmask = result.1;
            network = create_network(&netmask, &ip);
            first = network + 1;
            broadcast = create_broadcast(&network, &wildmask);
            last = broadcast - 1;
            nb_ip = base.pow((32 - cidr).into());
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
            return Err(String::from(
                "Cidr can't be smaller than the current subnet one!",
            ));
        }

        let base: u32 = 2;

        let mut next_network = self.network;

        for _i in 0..base.pow(cidr - self.cidr) {
            let next_sub = SubNet::from(next_network, cidr).unwrap();

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
            "{{\n\t\"network\": \"{}.{}.{}.{}\",\n\t\"first\": \"{}.{}.{}.{}\",\n\t\"last\": \"{}.{}.{}.{}\",\n\t\"broadcast\": \"{}.{}.{}.{}\",\n\t\"nbIp\": {}\n}}",
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

fn main() {
    let start = Instant::now();

    let ip: Ip = 167772160; // 10.0.0.0

    let sub = SubNet::from(ip, 8);

    let subnet = match sub {
        Ok(s) => s,
        Err(e) => panic!("{}", e),
    };

    let subs = subnet.split_in_subs(30);

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
