extern crate driverkit;

pub fn main() {

    let driver = driverkit::barrelfish::PciDriver::new(0,0,0);
    println!("{:?}", driver);

}
