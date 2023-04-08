use std::sync::Arc;

use sch::create_system;
use sch::scheduler::{Context, Scheduler};

#[tokio::main]
async fn main() {
    let data = Arc::new("world");
    let mut sched = Scheduler::new(data, [hello(), world(), wait(), race(), condition(), finish()]);

    sched.run().await;
}



create_system! {
    async fn hello(_ctx: Context, _data: Arc<&'static str>) {
        print!("Hello, ");
    }
}

create_system! {
    after = [hello];

    async fn world(_ctx: Context, data: Arc<&'static str>) {
        println!("{}!", data);
    }
}

fn wait() -> sch::system::System<&'static str> {
    sch::system::System {
        name: "wait",
        after: &["world"],
        run: Box::new(|ctx, data| Box::pin({
            async fn wait(_ctx: Context, _data: Arc<&'static str>) {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }

            wait(ctx, data)
        }))
    }
}

create_system! {
    after = [wait];

    async fn race(_ctx: Context, _data: Arc<&'static str>) {
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        print!("race ");
    }
}

create_system! {
    after = [wait];

    async fn condition(_ctx: Context, _data: Arc<&'static str>) {
        tokio::time::sleep(std::time::Duration::from_millis(251)).await;
        print!("condition ");
    }
}

create_system! {
    after = [race, condition];

    async fn finish(_ctx: Context, _data: Arc<&'static str>) {
        println!("!");
    }
}
