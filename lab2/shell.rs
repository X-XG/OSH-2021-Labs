use std::io::{stdin,stdout, BufRead, Write, Read};
use std::env;
use std::process::{exit, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use core::option::Option;
use std::process;

extern crate ctrlc;

fn main() -> ! {
    let ruptc = Arc::new(AtomicBool::new(false));
    let r = ruptc.clone();
    ctrlc::set_handler(move || {
        //处理ctrl-C中断
        r.store(true, Ordering::SeqCst);
        print!("\n# ");
        stdout().flush().expect("Print # failed");
    })
    .expect("Error setting Ctrl-C handler");

    loop {
        if ruptc.load(Ordering::SeqCst) {
            //情况二:终结运行中程序
            ruptc.store(false, Ordering::SeqCst);
        } else {
            print!("# ");
            stdout().flush().expect("Print # failed");
        }

        let mut cmd = String::new();
        for line_res in stdin().lock().lines() {
            let line = line_res.expect("Read a line from stdin failed");
            cmd = line;
            break;
        }
        if ruptc.load(Ordering::SeqCst) {
            //情况一:丢弃未输完命令
            ruptc.store(false, Ordering::SeqCst);
        }
        
        let mut result = String::new();
        excute(None, &cmd, true)
            .unwrap()
            .read_to_string(&mut result)
            .expect("print final result failed");
        println!("{}", result);
    }
}

fn excute(
    pipein: Option<process::ChildStdout>,
    cmd: &str,
    isfirst: bool,
) -> Option<process::ChildStdout> {
    let yy = cmd.split_once('|');

    let mut output: Option<process::ChildStdout> = None;
    let cmdcurrent = if yy == None { &cmd } else { yy.unwrap().0 };
    let cmdnext = if yy == None { &cmd } else { yy.unwrap().1 };

    let mut args = cmdcurrent.split_whitespace();
    let prog = args.next();
    match prog {
        None => panic!("Not program input"),
        Some(prog) => {
            match prog {
                "cd" => {
                    let dir = args.next().expect("No enough args to set current dir");
                    env::set_current_dir(dir).expect("Changing current dir failed");
                }
                /*"pwd" => {
                    let err = "Getting current dir failed";
                    println!("{}", env::current_dir().expect(err).to_str().expect(err));
                }*/
                "export" => {
                    for arg in args {
                        let mut assign = arg.split("=");
                        let name = assign.next().expect("No variable name");
                        let value = assign.next().expect("No variable value");
                        env::set_var(name, value);
                    }
                }
                "exit" => {
                    exit(0);
                }
                _ => {
                    let child = if isfirst {
                        Command::new(prog)
                            .args(args)
                            .stdout(Stdio::piped())
                            .spawn()
                            .expect("Failed to execute first command")
                    } else {
                        Command::new(prog)
                            .args(args)
                            .stdin(Stdio::from(pipein.expect("pipe with no input")))
                            .stdout(Stdio::piped())
                            .spawn()
                            .expect("Failed to execute post command")
                    };
                    output = child.stdout;
                }
            }
        }
    }
    if yy == None {
        output
    } else {
        excute(output, cmdnext, false)
    }
}
