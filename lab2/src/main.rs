use core::option::Option;
use std::collections::VecDeque;
use std::env;
use std::fs::OpenOptions;
use std::io::{stdin, stdout, BufRead, Read, Write};
use std::net::TcpStream;
use std::process;
use std::process::{exit, Command, Stdio};
use std::string::String;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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

    let mut seted = false;
    loop {
        if ruptc.load(Ordering::SeqCst) {
            //情况二:终结运行中程序
            ruptc.store(false, Ordering::SeqCst);
        } else {
            print!("# ");
            stdout().flush().expect("Print # failed");
        }

        //输入
        let mut cmd = String::new();
        stdin()
            .lock()
            .read_line(&mut cmd)
            .expect("Read a line from stdin failed");

        if cmd.is_empty() {
            //Ctrl+D 退出shell
            println!("");
            exit(0);
        }

        if ruptc.load(Ordering::SeqCst) {
            //情况一:丢弃未输完命令
            ruptc.store(false, Ordering::SeqCst);
        }
        let mut result = String::new();

        let ex_result = excute(None, &cmd, true, seted);
        let last_child_thread = ex_result.0;
        seted = ex_result.1;

        if last_child_thread.is_none() == true {
            continue;
        }

        last_child_thread
            .expect("excute failed")
            .read_to_string(&mut result)
            .expect("print final result failed");

        if ex_result.2 {
            let mut stream = TcpStream::connect(ex_result.3).expect("connect failed");
            stream.write(result.as_bytes()).expect("write tcp failed");
        } else {
            print!("{}", result);
            stdout().flush().expect("Print result failed");
        }
    }
}

fn find_first(string: &str) -> (&str, i32, &str) {
    //用于拆分"<",">>",">",并得到第一处的拆分
    let mut retype = 0;
    let mut re_find = true;
    let mut fore = string;
    let mut behind = "";
    while re_find {
        re_find = true;
        if fore.find("<").is_some() {
            fore = fore.split_once("<").unwrap().0;
            behind = string.split_once("<").unwrap().1;
            retype = 1;
        } else if fore.find(">>").is_some() {
            fore = fore.split_once(">>").unwrap().0;
            behind = string.split_once(">>").unwrap().1;
            retype = 2;
        } else if fore.find(">").is_some() {
            fore = fore.split_once(">").unwrap().0;
            behind = string.split_once(">").unwrap().1;
            retype = 3;
        } else {
            re_find = false;
        }
    }
    (fore, retype, behind)
}

//递归调用实现多层pipe,在各个线程内使用管道,与主线程形成环
fn excute(
    pipein: Option<process::ChildStdout>,
    cmd: &str,
    isfirst: bool,
    seted: bool,
) -> (Option<process::ChildStdout>, bool, bool, String) {

    let yy = cmd.split_once('|');

    let mut output: Option<process::ChildStdout> = None;
    let mut cmdcurrent = if yy == None { &cmd } else { yy.unwrap().0 };
    let cmdnext = if yy == None { &cmd } else { yy.unwrap().1 };

    let mut cmdpure = cmdcurrent;
    let fore = find_first(cmdcurrent);
    if fore.1 > 0 {
        cmdpure = fore.0;
    }

    let mut stdin = if isfirst {
        Stdio::inherit()
    } else {
        Stdio::from(pipein.expect("pipe with no input"))
    };

    let mut stdout = Stdio::piped();

    let mut istcp;
    let mut redirect = true;
    let mut tcp_write = false;
    let mut tcp_read = false;
    let mut tcp = String::new();
    let mut tcpread = String::new();

    while redirect {
        //处理重定向
        let head = find_first(cmdcurrent);
        //let fore = head.0.trim();
        let filename = find_first(head.2).0.trim();
        if filename.find("/dev/tcp/").is_some() {
            istcp = true;
            tcp = filename.replace("/dev/tcp/", "");
            tcp = tcp.replacen("/", ":", 1);
        } else {
            istcp = false;
        }

        match head.1 {
            1 => {
                cmdcurrent = head.2;
                if istcp {
                    tcp_read = true;
                    let mut stream = TcpStream::connect(&tcp).expect("connect failed");
                    stream
                        .read_to_string(&mut tcpread)
                        .expect("read from tcp failed");
                } else {
                    let fread = OpenOptions::new()
                        .read(true)
                        .open(filename)
                        .expect("there is no such file");
                    stdin = Stdio::from(fread);
                }
            }
            2 => {
                cmdcurrent = head.2;
                if istcp {
                    tcp_write = true;
                } else {
                    let fwrite = OpenOptions::new()
                        .write(true)
                        .append(true)
                        .create(true)
                        .open(filename)
                        .unwrap();
                    stdout = Stdio::from(fwrite);
                }
            }
            3 => {
                cmdcurrent = head.2;
                if istcp {
                    tcp_write = true;
                } else {
                    let fwrite = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .create(true)
                        .open(filename)
                        .unwrap();
                    stdout = Stdio::from(fwrite);
                }
            }
            _ => redirect = false,
        }
    }

    let mut v: VecDeque<&str> = cmdpure.split('$').collect();
    v.pop_front();

    let mut cmdchange = cmdpure.to_owned();
    for ans in v {
        //替换环境变量
        match env::var(ans.trim()) {
            Ok(val) => {
                cmdchange = cmdchange.replacen("$", "", 1);
                cmdchange = cmdchange.replacen(ans, &val, 1);
            }
            _ => {
                if seted {
                    panic!("{}: unbound variable", ans.trim());
                }
            }
        }
    }

    //替换~
    let mut v: VecDeque<&str> = cmdpure.split('$').collect();
    v.pop_front();

    let path = env::var("HOME").expect("No home varible");
    let user = env::var("USER").expect("No user varible");
    let mut temp = String::from("~");
    temp.push_str(&user);
    cmdchange = cmdchange.replace("~root", "/root");
    cmdchange = cmdchange.replace(&temp, &path);

    if cmdchange.find('~').is_some() {
        let offset = cmdchange.find('~').unwrap();

        if cmd.get(offset-1..offset) == Some(&" ") &&
           ( cmd.get(offset+1..offset+2) == Some(&"\n") || 
             cmd.get(offset+1..offset+2) == Some(&" ") ) {
            cmdchange.replace_range(offset..offset+1, &path)
        }
    }

    let mut nextset = false;
    let mut args = cmdchange.split_whitespace();
    let prog = args.next();
    match prog {
        None => panic!("Not program input"),
        Some(prog) => {
            match prog {
                "set" => {
                    if args.next().expect("No enough args of set").trim() == "-u" {
                        nextset = true;
                    }
                }
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
                    let child = if tcp_read {
                        Command::new(prog)
                            .args(args)
                            .arg(tcpread)
                            .stdin(Stdio::null())
                            .stdout(stdout)
                            .spawn()
                            .expect("Failed to execute command")
                    } else {
                        Command::new(prog)
                            .args(args)
                            .stdin(stdin)
                            .stdout(stdout)
                            .spawn()
                            .expect("Failed to execute command")
                    };
                    output = child.stdout;
                }
            }
        }
    }

    if yy == None {
        (output, nextset, tcp_write, tcp)
    } else {
        excute(output, cmdnext, false, nextset)
    }
}
