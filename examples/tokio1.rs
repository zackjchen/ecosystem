use std::{thread, time::Duration};

use tokio::{fs, runtime::Builder, time::sleep};

#[tokio::main]
async fn main() {
    let handle = thread::spawn(|| {
        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        rt.spawn(async {
            println!("Future 1!");
            let content = fs::read("Cargo.toml").await.unwrap();
            println!("{:?}", content.len());
        });

        rt.spawn(async {
            println!("Future 2!");
            sleep(Duration::from_millis(900)).await;
            println!("Future 2 done!");
        });

        rt.block_on(async {
            println!("Blocking future!");
            sleep(Duration::from_millis(1000)).await;
            println!("Blocking future done!");
        })
    });

    handle.join().unwrap();
}
