use std::time::{Instant, Duration};
use std::env::{args, var};
use std::process::Command;
use std::thread;
use std::sync::mpsc;

enum Status {
    Ok,
    Failed,
}

fn main() {
    let tmux_win_id = match var("TMX_WINID") {
        Ok(id) => id,
        Err(_) => {
            eprintln!("TMX_WINID is not set");
            return
        }
    };

    let mut all_args = args().into_iter().skip(1).collect::<Vec<_>>();
    let execute = match all_args.first() {
        Some(a) => a.to_string(),
        None => return,
    };

    all_args.remove(0);

    let (tx, rx) = mpsc::channel();

    let now = Instant::now();
    {
        let tmux_win_id = tmux_win_id.clone();
        thread::spawn(move || {
            run_command(execute, all_args, tmux_win_id, tx);
        });
    }

    loop {
        match rx.try_recv() {
            Ok(Status::Ok) => {
                return
            }
            Ok(Status::Failed) => {
                return
            }
            Err(err) => {
                if let mpsc::TryRecvError::Disconnected = err {
                    // There is nothing we can do here
                    return;
                }
                let seconds = now.elapsed().as_secs_f32();
                rename_tmux_window(&format!("Building ({:.2})", seconds), &tmux_win_id);
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

fn rename_tmux_window(title: &str, tmux_win_id: &str) {
    let _ = Command::new("tmux")
        .arg("renamew")
        .arg("-t")
        .arg(tmux_win_id)
        .arg(title)
        .output();
}

fn run_command(command: String, args: Vec<String>, tmux_win_id: String, tx: mpsc::Sender<Status>) {
    let now = Instant::now();
    let cmd = Command::new(command)
        .args(args)
        .output();

    match cmd {
        Ok(_) => {
            let elapsed = now.elapsed();
            let seconds = elapsed.as_secs();
            let _ = tx.send(Status::Ok);
            rename_tmux_window(&format!("Ok ({})", seconds), &tmux_win_id);
        }
        Err(_) => { 
            let _ = tx.send(Status::Failed); 
            rename_tmux_window("Err", &tmux_win_id);
        }
    }
}
