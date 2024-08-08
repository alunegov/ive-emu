use std::{env, future, sync::{Arc, Mutex}, thread, time::Duration};

use tokio_modbus::{prelude::*, server::rtu::Server};

struct Service {
    n: Arc<Mutex<u16>>,
}

impl tokio_modbus::server::Service for Service {
    type Request = SlaveRequest<'static>;
    type Future = future::Ready<Result<Response, Exception>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        match req.request {
            Request::ReadHoldingRegisters(addr, qty) => {
                println!("ReadHoldingRegisters, {addr} {qty}");
                let mut regs = vec![0; qty.into()];
                let mut i = 0;
                while (i + 2) <= qty {
                    regs[(i + 0) as usize] = 1;
                    regs[(i + 1) as usize] = 0;
                    i += 2;
                }
                println!("<== {:?}", regs);
                future::ready(Ok(Response::ReadHoldingRegisters(regs)))
            }
            Request::ReadInputRegisters(addr, qty) => {
                println!("ReadInputRegisters, {addr} {qty}");
                let nn = {
                    let n = self.n.lock().unwrap();
                    *n
                };
                let resp = from_float(nn as f32);
                let mut regs = vec![0; qty.into()];
                let mut i = 0;
                while (i + 2) <= qty {
                    regs[(i + 0) as usize] = resp[0];
                    regs[(i + 1) as usize] = resp[1];
                    i += 2;
                }
                println!("<== {:?}", regs);
                future::ready(Ok(Response::ReadInputRegisters(regs)))
            }
            Request::WriteMultipleRegisters(addr, data) => {
                println!("WriteMultipleRegisters, {addr} {}", data.len());
                println!("==> {:?}", data);
                future::ready(Ok(Response::WriteMultipleRegisters(addr, data.len() as u16)))
            }
            _ => future::ready(Err(Exception::IllegalFunction)),
        }
    }
}

fn from_float(val: f32) -> [u16; 2] {
    let b = val.to_be_bytes();
    return [u16::from_be_bytes([b[2], b[3]]), u16::from_be_bytes([b[0], b[1]])];
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    let port_path = args.nth(1).unwrap_or_else(|| "COM7".into());

    let builder = tokio_serial::new(port_path.clone(), 115200);
    let server_serial = tokio_serial::SerialStream::open(&builder).unwrap();

    let n = Arc::new(Mutex::new(0));

    println!("Starting up server on {port_path}...");
    let n1 = n.clone();
    let server_thread = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let server = Server::new(server_serial);
        let service = Service { n: n1 };
        rt.block_on(async {
            if let Err(err) = server.serve_forever(service).await {
                eprintln!("{err}");
            }
        })
    });

    let n2 = n.clone();
    let inc_n_thread = thread::spawn(move || {
        loop {
            {
                let n = &mut n2.lock().unwrap();
                **n = (*n).wrapping_add(1);
            }
            thread::sleep(Duration::from_millis(55));
        }
    });

    server_thread.join().unwrap();
    inc_n_thread.join().unwrap();

    Ok(())
}
