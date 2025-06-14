use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

struct Mailbox {
    queue: VecDeque<i32>,
    finished: bool,
}

impl Mailbox {
    fn new() -> Self {
        Mailbox {
            queue: VecDeque::new(),
            finished: false,
        }
    }

    fn send(&mut self, value: i32) {
        self.queue.push_back(value);
    }

    fn receive(&mut self) -> Option<i32> {
        if self.queue.is_empty() && self.finished {
            None
        } else {
            self.queue.pop_front()
        }
    }

    fn finish(&mut self) {
        self.finished = true;
    }
}

fn producer(mailbox: Arc<(Mutex<Mailbox>, Condvar)>) {
    let (mailbox, condvar) = &*mailbox;
    for i in 0..10 {
        println!("Producing {}", i);
        let mut mb = mailbox.lock().unwrap();
        mb.send(i);
        condvar.notify_one(); // Notify the consumer that a new item is available
        thread::sleep(Duration::from_millis(100));
    }
    let mut mb = mailbox.lock().unwrap();
    mb.finish();
    println!("Producer done");
}

fn consumer(mailbox: Arc<(Mutex<Mailbox>, Condvar)>) {
    let (mailbox, condvar) = &*mailbox;
    loop {
        let mut mb = mailbox.lock().unwrap();
        while mb.queue.is_empty() && !mb.finished {
            println!("Mailbox empty, waiting");
            mb = condvar.wait(mb).unwrap();
        }

        // The mailbox is locked and either has items or is finished.
        if let Some(value) = mb.receive() {
            println!("Received {}", value);
        } else if mb.finished {
            break;
        }
    }
}

fn main() {
    let mailbox = Arc::new((Mutex::new(Mailbox::new()), Condvar::new()));

    // Producer thread
    let producer_mailbox = Arc::clone(&mailbox);
    let producer_thread = thread::spawn(move || {
        producer(producer_mailbox);
    });

    // Consumer thread
    let consumer_mailbox = Arc::clone(&mailbox);
    let consumer_thread = thread::spawn(move || {
        consumer(consumer_mailbox);
    });

    producer_thread.join().unwrap();
    consumer_thread.join().unwrap();
    println!("All done");
}
