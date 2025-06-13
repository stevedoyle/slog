#include <condition_variable>
#include <iostream>
#include <mutex>
#include <queue>
#include <thread>

std::mutex mtx;
std::condition_variable cv;
std::queue<int> data_queue;
bool finished = false;

void producer() {
  for (auto i = 0; i < 10; ++i) {
    {
      std::lock_guard<std::mutex> lock(mtx);
      data_queue.push(i);
      std::cout << "Produced: " << i << std::endl;
    } // Lock is released here.
    cv.notify_one(); // Wake up one consumer.
  }

  {
    std::lock_guard<std::mutex> lock(mtx);
    finished = true;
  }
  cv.notify_all(); // Wake up all consumers to check the finished flag.
}

void consumer() {
  while (true) {
    std::unique_lock<std::mutex> lock(mtx);
    // Wait until the queue is NOT empty OR production is finished.
    cv.wait(lock, [] { return !data_queue.empty() || finished; });

    // If production is done and the queue is empty, we can exit.
    if (data_queue.empty() && finished) {
      break;
    }

    int data = data_queue.front();
    data_queue.pop();
    std::cout << "Consumed: " << data << std::endl;
  } // Lock is released when it goes out of scope and before the next iteration.
}

int main() {
  std::thread prod_thread(producer);
  std::thread cons_thread(consumer);

  prod_thread.join();
  cons_thread.join();

  return 0;
}
