use std::{env, future, thread};

use tokio_modbus::{prelude::*, server::rtu::Server};

struct Service;

impl tokio_modbus::server::Service for Service {
    type Request = SlaveRequest<'static>;
    type Future = future::Ready<Result<Response, Exception>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        match req.request {
            Request::ReadHoldingRegisters(addr, qty) => {
                println!("ReadHoldingRegisters, {addr} {qty}");
                let mut regs = vec![0; qty.into()];
                for i in 0..qty {
                    regs[i as usize] = 2 * i;
                }
                future::ready(Ok(Response::ReadHoldingRegisters(regs)))
            }
            Request::ReadInputRegisters(addr, qty) => {
                println!("ReadInputRegisters, {addr} {qty}");
                let mut regs = vec![0; qty.into()];
                for i in 0..qty {
                    regs[i as usize] = i;
                }
                future::ready(Ok(Response::ReadInputRegisters(regs)))
            }
            Request::WriteMultipleRegisters(addr, data) => {
                println!("WriteMultipleRegisters, {addr} {{data.len()}}");
                future::ready(Ok(Response::WriteMultipleRegisters(addr, data.len() as u16)))
            }
            _ => future::ready(Err(Exception::IllegalFunction)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    let port_path = args.nth(1).unwrap_or_else(|| "COM7".into());

    let builder = tokio_serial::new(port_path.clone(), 115200);
    let server_serial = tokio_serial::SerialStream::open(&builder).unwrap();

    println!("Starting up server on {port_path}...");
    let server = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let server = Server::new(server_serial);
        let service = Service;
        rt.block_on(async {
            if let Err(err) = server.serve_forever(service).await {
                eprintln!("{err}");
            }
        })
    });

    let _ = server.join();

    Ok(())
}
