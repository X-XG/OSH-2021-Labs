use std::io::{stdin,stdout, BufRead, Write, Read};
use std::env;
use std::process::{exit, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use core::option::Option;
use std::process;
use std::fs::OpenOptions;
use std::string::String;

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

        //输入
        let mut cmd = String::new();
        stdin().lock().read_line(&mut cmd).expect("Read a line from stdin failed");
        
        if cmd.is_empty() {     //Ctrl+D 退出shell
            println!("");
            exit(0);
        }    

        if ruptc.load(Ordering::SeqCst) {
            //情况一:丢弃未输完命令
            ruptc.store(false, Ordering::SeqCst);
        }
        
        let mut result = String::new();
        let last_child_thread = excute(None, &cmd, true);
        if last_child_thread.is_none() == true {continue;}
        last_child_thread.expect("excute failed")
            .read_to_string(&mut result)
            .expect("print final result failed");
        print!("{}",result);
        stdout().flush().expect("Print result failed");
    }
}

fn find_first(string: &str) -> (&str,i32,&str) {
    //用于拆分"<",">>",">",并得到第一处的拆分
    let mut retype=0;
    let mut re_find=true;
    let mut temp = string;
    let mut behind = "";
    while re_find { 
        re_find=true;
        if temp.find("<").is_some() {
            temp = temp.split_once("<").unwrap().0;
            behind = string.split_once("<").unwrap().1;
            retype=1;
        } else if temp.find(">>").is_some() {
            temp = temp.split_once(">>").unwrap().0;
            behind = string.split_once(">>").unwrap().1;
            retype=2;
        } else if temp.find(">").is_some() {
            temp = temp.split_once(">").unwrap().0;
            behind = string.split_once(">").unwrap().1;
            retype=3;
        } else {
            re_find = false;
        }
    }
    (temp,retype,behind)
}

//递归调用实现多层pipe,在各个线程内使用管道,与主线程形成环
fn excute(
    pipein: Option<process::ChildStdout>,
    cmd: &str,
    isfirst: bool,
) -> Option<process::ChildStdout> {
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

    let mut stdout=Stdio::piped();

    let mut redirect = true;
    //let mut is_fread = false;
    while redirect {
        //处理重定向
        let head = find_first(cmdcurrent);
        let filename = find_first(head.2).0.trim();
        //println!("0: {}\n{}\n\n\n",filename,cmdcurrent);
        match head.1 {
            1 => {
                cmdcurrent = head.2;
                let fread = OpenOptions::new().read(true).open(filename).expect("there is no such file");
                stdin=Stdio::from(fread);
                //is_fread = true;
            }
            2 => {
                cmdcurrent = head.2;
                let fwrite = OpenOptions::new().write(true).append(true).create(true).open(filename).unwrap();
                stdout=Stdio::from(fwrite); 
                //println!("2: {}\n{}\n",filename,cmdcurrent);
            }
            3 => {
                cmdcurrent = head.2;
                let fwrite = OpenOptions::new().write(true).truncate(true).create(true).open(filename).unwrap();
                stdout=Stdio::from(fwrite);
                //println!("3: {}\n{}\n",filename,cmdcurrent);
            }
            _ => { redirect = false }
        }
    }
    
    let mut args = cmdpure.split_whitespace();
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
                    let child = Command::new(prog)
                        .args(args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn()
                        .expect("Failed to execute command");
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
