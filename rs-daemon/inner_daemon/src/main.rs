#[tokio::main]
async fn main() {
    println!("Hello, world!");
    os_ops::serve().await;
}

pub mod os_ops {
    use serde::{Deserialize, Serialize};
    use serde_json;
    use std::process::Command;
    use std::{thread, time};
    use sys_info::{cpu_num, cpu_speed, loadavg, mem_info, os_release, os_type};
    use warp::Filter;


    pub async fn serve() -> () {
        let info = warp::path!("info").map(|| {
            let info_str = handle_info_fn();
            dbg!(&info_str);
            warp::reply::html(info_str)
        });

        let sleep = warp::path!("sleep").map(|| {
            handle_sleep();
            warp::reply::html("sleep")
        });
        let echo = warp::path!("echo").map(|| warp::reply::html("echo"));
        let routes = info.or(echo).or(sleep);

        warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    }

    fn handle_sleep() {
                Command::new("osascript")
                    .arg("-e")
                    .arg(r#"tell app "System Events" to sleep"#)
                    .output()
                    .expect("Failed to send sleep command");
    }

    fn handle_info_fn() -> String {
        get_info_str().expect("get info")
    }


    pub fn get_input_file() -> String {
        "/Users/felipearce/Desktop/projects/shellhacks2023/daemon/rs-daemon/inner_daemon/in.txt"
            .to_string()
    }

    pub fn get_output_file() -> String {
        "/Users/felipearce/Desktop/projects/shellhacks2023/daemon/rs-daemon/inner_daemon/out.txt"
            .to_string()
    }




    fn _sleep(millis: u64) {
        let duration = time::Duration::from_millis(millis);
        thread::sleep(duration);
    }

    #[derive(Serialize, Deserialize)]
    struct SystemInfo {
        cpu_count: u32,
        cpu_speed: Option<u64>,
        load_avg: (f64, f64, f64),
        mem_total: u64,
        mem_free: u64,
        os_type: String,
        os_release: String,
    }

    fn get_info_str() -> Result<String, serde_json::Error> {
        let cpu_count = cpu_num().unwrap_or(0);
        let cpu_speed = cpu_speed().ok();
        let load_avg_result = loadavg();
        let load_avg = if let Ok(la) = load_avg_result {
            (la.one, la.five, la.fifteen)
        } else {
            (0.0, 0.0, 0.0)
        };
        let mem_result = mem_info();
        let (mem_total, mem_free) = if let Ok(mem) = mem_result {
            (mem.total, mem.free)
        } else {
            (0, 0)
        };

        let os_type = os_type().unwrap_or_else(|_| "".to_string());
        let os_release = os_release().unwrap_or_else(|_| "".to_string());

        let system_info = SystemInfo {
            cpu_count,
            cpu_speed,
            load_avg,
            mem_total,
            mem_free,
            os_type,
            os_release,
        };

        let json_str = serde_json::to_string(&system_info)?;
        Ok(format!("{}", json_str))
    }
}
