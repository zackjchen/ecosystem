//! tokio async task seed message to worker for expensive blocking task
//! 一个异步运行时传递消息给同步的线程
use std::thread;

use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel(32);
    // 同步的线程
    let worker_handle = worker(rx);

    // 异步的任务
    tokio::spawn(async move {
        loop {
            tx.send("Future 1".to_string()).await?;
        }
        // 由于上面用了? 操作符，但是是个死循环，导致它不能推断出返回值是什么，所以要声明返回值
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });
    worker_handle.join().unwrap();
    Ok(())
}

fn worker(mut rx: mpsc::Receiver<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while let Some(msg) = rx.blocking_recv() {
            let ret = expensive_blocking(msg);
            println!("{}", ret);
        }
    })
}
fn expensive_blocking(s: String) -> String {
    thread::sleep(std::time::Duration::from_secs(2));
    blake3::hash(s.as_bytes()).to_string()
}
