use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn producer(tx: mpsc::Sender<i32>) {
    for i in 0..10 {
        println!("Producing {}", i);
        tx.send(i).unwrap();
        thread::sleep(Duration::from_millis(100));
    }
    println!("Producer done");
}

fn consumer(rx: mpsc::Receiver<i32>) {
    for received in rx {
        println!("Received {}", received);
    }
}

fn main() {
    let (tx, rx) = mpsc::channel();

    // Producer thread
    let producer_thread = thread::spawn(move || {
        producer(tx);
    });

    // Consumer thread
    let consumer_thread = thread::spawn(move || {
        consumer(rx);
    });

    producer_thread.join().unwrap();
    consumer_thread.join().unwrap();
}
