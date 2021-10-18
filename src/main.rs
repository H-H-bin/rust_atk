
use serde::{Deserialize, Serialize};
use serialport::{available_ports, SerialPortType};
use wmi::{COMLibrary, WMIConnection, WMIError};

#[cfg(windows)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug)]
struct Win32_POTSModem {
    Name: String,
    STATUS: String,
    AttachedTo: String,
}

#[cfg(any(windows, unix))]
fn list_com_ports() {
    match available_ports() {
        Ok(ports) => {
            match ports.len() {
                0 => println!("No ports found."),
                1 => println!("Found 1 port:"),
                n => println!("Found {} ports:", n),
            };
            for p in ports {
                println!("  {}", p.port_name);
                match p.port_type {
                    SerialPortType::UsbPort(info) => {
                        println!("    Type: USB");
                        println!("    VID:{:04x} PID:{:04x}", info.vid, info.pid);
                        println!(
                            "     Serial Number: {}",
                            info.serial_number.as_ref().map_or("", String::as_str)
                        );
                        println!(
                            "      Manufacturer: {}",
                            info.manufacturer.as_ref().map_or("", String::as_str)
                        );
                        println!(
                            "           Product: {}",
                            info.product.as_ref().map_or("", String::as_str)
                        );
                    }
                    SerialPortType::BluetoothPort => {
                        println!("    Type: Bluetooth");
                    }
                    SerialPortType::PciPort => {
                        println!("    Type: PCI");
                    }
                    SerialPortType::Unknown => {
                        println!("    Type: Unknown");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("{:?}", e);
            eprintln!("Error listing serial ports");
        }
    }
}

#[cfg(windows)]
fn get_modem_ports_and_return_vec_struct() -> Result<Vec<Win32_POTSModem>, WMIError> {
    // Creating new COM Port
    let com_con = COMLibrary::new()?;
    // Create new WMI Connection using COM Port
    let wmi_con = WMIConnection::new(com_con.into())?;

    // let modem_ports: Vec<Win32_POTSModem> = wmi_con.query()?;

    let modem_ports: Vec<Win32_POTSModem> = match wmi_con.query() {
        Ok(modem_ports) => modem_ports,
        Err(e) => {
            return Err(e);
        }
    };
    Ok(modem_ports)
}

fn main() {
    list_com_ports();

    match get_modem_ports_and_return_vec_struct() {
        Ok(modem_ports) => {
            for port in &modem_ports {
                println!("{:#?}", port);
            }
            // println!("{}", modem_ports[0].Name);
            // println!("{}", modem_ports[0].STATUS);
            // println!("{}", modem_ports[0].AttachedTo);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    };
}
