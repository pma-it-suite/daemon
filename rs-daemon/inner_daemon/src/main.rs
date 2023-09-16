fn main() {
    println!("Hello, world!");
    os_ops::poll_from_stdin();
}

pub mod os_ops {
    use std::io;
    use std::sync::mpsc;
    use std::sync::mpsc::Receiver;
    use std::sync::mpsc::TryRecvError;
    use std::{thread, time};
    use std::io::Write;

    pub fn read_sys_data() -> String {
        unimplemented!();
    }

    fn post_and_flush(content: &str) {
        println!("{}", content);
        io::stdout().flush().unwrap();
    }

    pub fn poll_from_stdin() {
        let stdin_channel = spawn_stdin_channel();
        loop {
            match stdin_channel.try_recv() {
                Ok(key) => post_and_flush(&format!("Received: {}", key)),
                Err(TryRecvError::Empty) => post_and_flush(&String::from("Channel empty")),
                Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
            }
            sleep(1000);
        }
    }

    fn spawn_stdin_channel() -> Receiver<String> {
        let (tx, rx) = mpsc::channel::<String>();
        thread::spawn(move || loop {
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).unwrap();
            tx.send(buffer).unwrap();
        });
        rx
    }

    fn sleep(millis: u64) {
        let duration = time::Duration::from_millis(millis);
        thread::sleep(duration);
    }
}
