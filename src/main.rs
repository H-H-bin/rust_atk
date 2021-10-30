use serde::{Deserialize, Serialize};
use serialport::{available_ports, DataBits, SerialPortType, StopBits};
use std::io::{self, Write};
use std::time::Duration;
use wmi::{COMLibrary, WMIConnection, WMIError};

use rustyline::error::ReadlineError;
use rustyline::Editor;

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

    // 1. Get port name
    //TODO: if there only one port, then just open it, if there more then one port,
    //      pop up for user to chose one.

    let modem_port = get_modem_ports_and_return_vec_struct().unwrap();
    let port_name = &modem_port[0].AttachedTo;
    let baud_rate = 115200;

    // 2. Open Port
    let builder = serialport::new(port_name, baud_rate)
        .stop_bits(StopBits::One)
        .data_bits(DataBits::Eight)
        .timeout(Duration::from_millis(10));
    println!("{:?}", &builder);

    //3. Write to port
    let mut port = builder.open().unwrap_or_else(|e| {
        eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
        ::std::process::exit(1);
    });

    //TODO:Add auto complete with AT command history
    //TODO:Colors

    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline("❯❯ ");
        match readline {
            Ok(line) => {
                let line = line + "\r";
                rl.add_history_entry(line.as_str());

                //3. Write to port
                match port.write(&line.as_bytes()) {
                    Ok(_) => {}
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }

                //4. Read data
                let mut serial_buf: Vec<u8> = vec![0; 1000];
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(t) => io::stdout().write_all(&serial_buf[..t]).unwrap(),
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}
