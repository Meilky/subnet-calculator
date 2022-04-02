pub type Ip = u32;
pub type Cidr = u32;

pub fn create_netmask(cidr: Cidr) -> Result<(Ip, Ip), String> {
    if cidr > 32 {
        return Err("Cidr should be in between 0 and 32!".to_string());
    }

    let right_len = 32 - cidr;
    let netmask = (u32::MAX >> right_len) << right_len;

    Ok((netmask, !netmask))
}

pub fn create_network(netmask: &Ip, ip: &Ip) -> Ip {
    ip & netmask
}

pub fn create_broadcast(network: &Ip, wildmask: &Ip) -> Ip {
    network + wildmask
}
