#[cfg(never)];

import comm::{ port, chan, recv, send };

use std;

import std::{ treemap, uv };

enum child_message {
    set_log(chan<js::log_message>),
    set_err(chan<js::error_report>),
    log_msg(js::log_message),
    err_msg(js::error_report),
    io_cb(u32, str, u32),
    stdout(str),
    stderr(str),
    spawn(str, str),
    cast(str, str),
    load_url(str),
    exitproc,
    done,
}

enum ioop {
    op_stdout = 0,
    op_stderr = 1,
    op_spawn = 2,
    op_cast = 3,
    op_connect = 4,
    op_recv = 5,
    op_send = 6,
    op_close = 7,
    op_time = 8,
    op_exit = 9
}

fn populate_global_scope(cx : js::context, global : js::object, script : str) {
    js::begin_request(*cx);
    alt io::read_whole_file("xmlhttprequest.js") {
        result::ok(file) {
            let script = js::compile_script(
                cx, global, file, "xmlhttprequest.js", 0u);
            js::execute_script(cx, global, script);
        }
        _ { fail }
    }
    alt io::read_whole_file("dom.js") {
        result::ok(file) {
            let script = js::compile_script(
                cx, global, file, "dom.js", 0u);
            js::execute_script(cx, global, script);
        }
        _ { fail }
    }

    alt io::read_whole_file_str(script) {
        result::ok(file) {
            let script = js::compile_script(
                cx, global, str::bytes(#fmt("try { %s } catch (e) { print('Error: ', e, e.stack) }", file)), script, 0u);
            js::execute_script(cx, global, script);
        }
        _ {
            log(core::error, #fmt("File not found: %s", script));
            js::ext::rust_exit_now(0);
        }
    }
    js::end_request(*cx);
}

fn make_children(msg_chan : chan<child_message>, senduv_chan: chan<chan<uvtmp::iomsg>>) {
    task::spawn {||
        let log_port = port::<js::log_message>();
        send(msg_chan, set_log(chan(log_port)));

        while true {
            let msg = recv(log_port);
            if msg.level == 9u32 {
                send(msg_chan, exitproc);
                break;
            } else {
                send(msg_chan, log_msg(msg));
            }
        }
    };

    task::spawn {||
        let uv_port = port::<uvtmp::iomsg>();
        send(senduv_chan, chan(uv_port));
        while true {
            let msg = recv(uv_port);
            alt msg {
                uvtmp::connected(cd) {
                    send(msg_chan, io_cb(0u32, "onconnect", uvtmp::get_req_id(cd)));
                }
                uvtmp::wrote(cd) {
                    send(msg_chan, io_cb(1u32, "onsend", uvtmp::get_req_id(cd)));
                }
                uvtmp::read(cd, buf, len) {
                    if len == -1 {
                        send(msg_chan, io_cb(3u32, "onclose", uvtmp::get_req_id(cd)));
                    } else {
                        unsafe {
                            let vecbuf = vec::unsafe::from_buf(buf, len as uint);
                            let bufstr = str::from_bytes(vecbuf);
                            send(msg_chan, io_cb(2u32, bufstr, uvtmp::get_req_id(cd)));
                            uvtmp::delete_buf(buf);
                        }
                    }
                }
                uvtmp::timer(req_id) {
                    send(msg_chan, io_cb(4u32, "ontimer", req_id));
                }
                uvtmp::whatever {
                
                }
                uvtmp::exit {
                    send(msg_chan, done);
                    break;
                }
            }
        }
    };
}

fn make_actor(myid : int, myurl : str, thread : uvtmp::thread, maxbytes : u32, out : chan<child_message>, sendchan : chan<(int, chan<child_message>)>) {

    task::spawn {||
        let rt = js::get_thread_runtime(maxbytes);
        let msg_port = port::<child_message>();
        let msg_chan = chan(msg_port);
        send(sendchan, (myid, msg_chan));
        let senduv_port = port::<chan<uvtmp::iomsg>>();
        make_children(chan(msg_port), chan(senduv_port));
        let uv_chan = recv(senduv_port);

        let cx = js::new_context(rt, maxbytes as size_t);
        js::set_options(cx, js::options::varobjfix | js::options::methodjit);
        js::set_version(cx, 185u);

        let clas = js::new_class({ name: "global", flags: 0x47700u32 });
        let global = js::new_compartment_and_global_object(
            cx, clas, js::null_principals());

        js::init_standard_classes(cx, global);
        js::ext::init_rust_library(cx, global);

        let exit = false;
        let setup = 0;
        let childid = 0;

        while !exit {
            let msg = recv(msg_port);
            alt msg {
                set_log(ch) {
                    js::ext::set_log_channel(
                        cx, global, ch);
                    setup += 1;
                }
                load_url(x) {
                    js::begin_request(*cx);
                    js::set_data_property(cx, global, x);
                    let code = "_resume(5, _data, 0); _data = undefined";
                    let script = js::compile_script(cx, global, str::bytes(code), "io", 0u);
                    js::execute_script(cx, global, script);
                    js::end_request(*cx);
                }
                log_msg(m) {                
                    // messages from javascript
                    alt m.level{
                        0u32 { // stdout
                        send(out, stdout(
                            #fmt("[Actor %d] %s",
                            myid, m.message)));
                        }
                        1u32 { // stderr
                            send(out, stderr(
                                #fmt("[ERROR %d] %s",
                                myid, m.message)));
                        }
                        2u32 { // spawn
                            send(out, spawn(
                                #fmt("%d:%d", myid, childid),
                                m.message));
                            childid = childid + 1;
                        }
                        3u32 { // cast
                        }
                        4u32 { // CONNECT
                            uvtmp::connect(
                                thread, m.tag, m.message, uv_chan);
                        }
                        5u32 { // SEND
                            uvtmp::write(
                                thread, m.tag,
                                str::bytes("GET / HTTP/1.0\n\n"),
                                uv_chan);
                        }
                        6u32 { // RECV
                            uvtmp::read_start(thread, m.tag, uv_chan);
                        }
                        7u32 { // CLOSE
                            //log(core::error, "close");
                            uvtmp::close_connection(thread, m.tag);
                        }
                        8u32 { // SETTIMEOUT
                            uvtmp::timer_start(thread, m.timeout, m.tag, uv_chan);
                        }
                        _ {
                            log(core::error, "...");
                        }
                    }
                }
                io_cb(a1, a2, a3) {
                    js::begin_request(*cx);
                    js::set_data_property(cx, global, a2);
                    let code = #fmt("_resume(%u, _data, %u); _data = undefined; XMLHttpRequest.requests_outstanding", a1 as uint, a3 as uint);
                    let script = js::compile_script(cx, global, str::bytes(code), "io", 0u);
                    js::execute_script(cx, global, script);
                    js::end_request(*cx);
                }
                exitproc {
                    send(uv_chan, uvtmp::exit);
                }
                done {
                    exit = true;
                    send(out, done);
                }
                _ { fail "unexpected case" }
            }
            if setup == 1 {
                setup = 2;
                if str::byte_len(myurl) > 4u && str::eq(str::slice(myurl, 0u, 4u), "http") {
                    populate_global_scope(cx, global, "");
                    send(msg_chan, load_url(myurl));
                } else {
                    populate_global_scope(cx, global, myurl);
                }
                let checkwait = js::compile_script(
                    cx, global, str::bytes("if (XMLHttpRequest.requests_outstanding === 0)  jsrust_exit();"), "test.js", 0u);
                js::execute_script(cx, global, checkwait);
            }
        }
    };
}


fn main(args : [str]) {
    let maxbytes = 8u32 * 1024u32 * 1024u32;
    let thread = uvtmp::create_thread();
    uvtmp::start_thread(thread);

    let stdoutport = port::<child_message>();
    let stdoutchan = chan(stdoutport);

    let sendchanport = port::<(int, chan<child_message>)>();
    let sendchanchan = chan(sendchanport);

    let map = treemap::init();

    let argc = vec::len(args);
    let argv = if argc == 1u {
        ["test.js"]
    } else {
        vec::slice(args, 1u, argc)
    };

    let left = 0;

    for argv.each {|x|
        left += 1;
        make_actor(left, x, thread, maxbytes, stdoutchan, sendchanchan);
    }
    let actorid = left;

    iter::repeat(argv.len()) {||
        let (theid, thechan) = recv(sendchanport);
        treemap::insert(map, theid, thechan);
    }

    while true {
        alt recv(stdoutport) {
            stdout(x) { log(core::error, x); }
            stderr(x) { log(core::error, x); }
            spawn(id, src) {
                log(core::error, ("spawn", id, src));
                actorid = actorid + 1;
                left = left + 1;
                task::spawn {||
                    make_actor(actorid, src, thread, maxbytes, stdoutchan, sendchanchan);
                };
            }
            cast(id, msg) {}
            exitproc {
                left = left - 1;
                if left == 0 {
                    let n = @mut 0;
                    fn t(n: @mut int, &&_k: int, &&v: chan<child_message>) {
                        send(v, exitproc);
                        *n += 1;
                    }
                    treemap::traverse(map, bind t(n, _, _));
                    left = *n;
                }
            }
            done {
                left = left - 1;
                if left == 0 {
                    break;
                }
            }
            _ { fail "unexpected case" }
        }
    }
    // temp hack: join never returns right now
    js::ext::rust_exit_now(0);
    uvtmp::join_thread(thread);
    uvtmp::delete_thread(thread);
}
