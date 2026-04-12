use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

struct MyStruct {
    handle: Option<thread::JoinHandle<()>>,
    stop: Arc<AtomicBool>,
}

impl MyStruct {
    fn new(names: Vec<String>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();
        let handle = thread::spawn(move || {
            for name in names {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                println!("Hello, {}!", name);
                thread::sleep(Duration::from_secs(2));
            }
        });
        Self { handle: Some(handle), stop: stop_clone}
    }
    fn abort(&mut self) {
        println!("Aborting thread...");
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            println!("Joining thread in abort...");
            handle.join().unwrap();
        }
    }
}

impl Drop for MyStruct {
    fn drop(&mut self) {
        println!("Dropping MyStruct...");
        if let Some(handle) = self.handle.take() {
            println!("Joining thread in Drop...");
            handle.join().unwrap();
        }
    }
}

fn main() {
    let mut my_struct = MyStruct::new(vec!["Alice".into(), "Bob".into(), "John".into()]);
    println!("Hello, world!");
    thread::sleep(Duration::from_secs(2));
    my_struct.abort();
}
