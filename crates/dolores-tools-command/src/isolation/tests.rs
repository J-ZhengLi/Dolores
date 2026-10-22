use super::*;
use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    time::{Duration, Instant},
};

fn node(work: &Path) -> PathBuf {
    crate::resolve(work, "node").unwrap()
}
fn finish(
    mut running: crate::process::Running,
    mut stdout: File,
    mut stderr: File,
) -> (i32, String, String) {
    let end = Instant::now() + Duration::from_secs(8);
    let mut out = vec![];
    let mut err = vec![];
    let mut buffer = [0; 8192];
    loop {
        for (pipe, bytes) in [(&mut stdout, &mut out), (&mut stderr, &mut err)] {
            while let Ok(n) = crate::process::read_available(pipe, &mut buffer) {
                if n == 0 {
                    break;
                }
                bytes.extend_from_slice(&buffer[..n]);
                assert!(bytes.len() <= 32768);
            }
        }
        if let Some(code) = running.try_exit().unwrap() {
            assert!(
                running.stop_checked(),
                "owned tree and isolation grants must close"
            );
            for (pipe, bytes) in [(&mut stdout, &mut out), (&mut stderr, &mut err)] {
                while let Ok(n) = crate::process::read_available(pipe, &mut buffer) {
                    if n == 0 {
                        break;
                    }
                    bytes.extend_from_slice(&buffer[..n]);
                }
            }
            return (
                code,
                String::from_utf8(out).unwrap(),
                String::from_utf8(err).unwrap(),
            );
        }
        assert!(Instant::now() < end, "bounded isolation fixture expired");
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn state_empty(host: &Host) {
    assert_eq!(
        std::fs::read_dir(&host.root).unwrap().count(),
        1,
        "only owner.lock remains"
    );
}

#[test]
fn reviewed_grants_reject_replacement_and_stop_before_child_effects() {
    let temp = tempfile::tempdir().unwrap();
    let work = temp.path().join("reviewed-work");
    let original = temp.path().join("original-work");
    std::fs::create_dir(&work).unwrap();
    let host = Host::open(&temp.path().join("private-state")).unwrap();
    let cancel = CancellationToken::new();
    let review = host.review(&work, &[], &cancel).unwrap();
    state_empty(&host); // Review itself creates no profile or ACL grant.
    std::fs::rename(&work, &original).unwrap();
    std::fs::create_dir(&work).unwrap();
    assert!(host
        .prepare_reviewed(&node(&work), &review, &cancel)
        .err()
        .unwrap()
        .contains("changed after review"));
    state_empty(&host);
    let review = host.review(&work, &[], &cancel).unwrap();
    let lease = host
        .prepare_reviewed(&node(&work), &review, &cancel)
        .unwrap();
    assert_eq!(lease.executable().file_name().unwrap(), "node.exe");
    cancel.cancel();
    let result = spawn_cancellable(
        lease,
        &[
            "-e".into(),
            "require('fs').writeFileSync('MUST_NOT_RUN','BAD')".into(),
        ],
        &crate::process::environment(),
        &cancel,
    );
    assert!(result.err().unwrap().contains("stopped"));
    assert!(!work.join("MUST_NOT_RUN").exists());
    state_empty(&host);
    assert!(host
        .review(&work, &[], &cancel)
        .err()
        .unwrap()
        .contains("stopped"));
    let fresh = CancellationToken::new();
    let review = host.review(&work, &[], &fresh).unwrap();
    let lease = host
        .prepare_reviewed(&node(&work), &review, &fresh)
        .unwrap();
    let (running, input, out, err) = spawn_cancellable(
        lease,
        &["-e".into(), "process.stdout.write('FRESH_REVIEW')".into()],
        &crate::process::environment(),
        &fresh,
    )
    .unwrap();
    drop(input);
    assert_eq!(
        finish(running, out, err),
        (0, "FRESH_REVIEW".into(), "".into())
    );
    state_empty(&host);
}

#[test]
fn lpac_native_transport_enforces_grants_and_network_with_exact_unicode_pipes() {
    let temp = tempfile::tempdir().unwrap();
    let work = temp.path().join("workspace 世界");
    let readonly = temp.path().join("read-only");
    std::fs::create_dir(&work).unwrap();
    std::fs::create_dir(&readonly).unwrap();
    std::fs::write(work.join("input"), "PUBLIC_WORK").unwrap();
    std::fs::write(readonly.join("input"), "PUBLIC_READ").unwrap();
    let outside = temp.path().join("outside");
    std::fs::write(&outside, "KEEP").unwrap();
    let host = Host::open(&temp.path().join("private-state")).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    // Real unrestricted positive control, with an owned listener.
    let connected = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (accepted, _) = listener.accept().unwrap();
    drop((accepted, connected));
    let code = format!(
        r#"const fs=require('fs'),net=require('net'); const result={{stdin:fs.readFileSync(0,'utf8'),read:fs.readFileSync('input','utf8'),extra:fs.readFileSync({read},'utf8')}}; fs.writeFileSync('created','PUBLIC_CREATED'); for(const [key,p] of [['outside',{outside}],['readonly',{read}]]){{try{{fs.writeFileSync(p,'BAD');result[key]='WRITTEN'}}catch(e){{result[key]=e.code}}}} const s=net.createConnection({{host:'127.0.0.1',port:{port}}});s.on('connect',()=>{{result.network='CONNECTED';done()}});s.on('error',e=>{{result.network=e.code;done()}});const timer=setTimeout(()=>{{result.network='TIMEOUT';done()}},1000);function done(){{clearTimeout(timer);s.destroy();process.stdout.write(JSON.stringify(result));process.stderr.write('PUBLIC_STDERR 世界')}}"#,
        read = serde_json::to_string(&readonly.join("input")).unwrap(),
        outside = serde_json::to_string(&outside).unwrap()
    );
    let lease = host
        .prepare(&node(&work), &work, std::slice::from_ref(&readonly))
        .unwrap();
    let (running, mut stdin, stdout, stderr) =
        spawn(lease, &["-e".into(), code], &crate::process::environment()).unwrap();
    stdin.write_all("PUBLIC_STDIN 世界\n".as_bytes()).unwrap();
    drop(stdin);
    let (exit, out, err) = finish(running, stdout, stderr);
    assert_eq!(exit, 0, "{err}");
    let value: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(value["stdin"], "PUBLIC_STDIN 世界\n");
    assert_eq!(value["read"], "PUBLIC_WORK");
    assert_eq!(value["extra"], "PUBLIC_READ");
    assert_eq!(value["network"], "EACCES");
    assert!(["EACCES", "EPERM"].contains(&value["outside"].as_str().unwrap()));
    assert!(["EACCES", "EPERM"].contains(&value["readonly"].as_str().unwrap()));
    assert_eq!(std::fs::read_to_string(&outside).unwrap(), "KEEP");
    assert_eq!(
        std::fs::read_to_string(readonly.join("input")).unwrap(),
        "PUBLIC_READ"
    );
    assert_eq!(
        std::fs::read_to_string(work.join("created")).unwrap(),
        "PUBLIC_CREATED"
    );
    assert_eq!(err, "PUBLIC_STDERR 世界");
    state_empty(&host);
    host.recover().unwrap();
    state_empty(&host);
}

#[test]
fn failed_setup_and_path_aliases_never_launch_and_cleanup_allows_a_fresh_review() {
    let temp = tempfile::tempdir().unwrap();
    let work = temp.path().join("work");
    std::fs::create_dir(&work).unwrap();
    std::fs::write(work.join("input"), "PUBLIC").unwrap();
    let host = Host::open(&temp.path().join("state")).unwrap();
    assert!(host
        .prepare_inner(&node(&work), &work, &[], Some(0))
        .is_err());
    state_empty(&host);
    assert!(!work.join("marker").exists());
    std::fs::hard_link(work.join("input"), work.join("hardlink")).unwrap();
    assert!(host
        .prepare(&node(&work), &work, &[])
        .err()
        .unwrap()
        .contains("hard-linked"));
    state_empty(&host);
    std::fs::remove_file(work.join("hardlink")).unwrap();
    let mut lease = host.prepare(&node(&work), &work, &[]).unwrap();
    assert!(
        Host::open(&temp.path().join("state")).is_err(),
        "exclusive recovery owner"
    );
    lease.close().unwrap();
    state_empty(&host);
    let broken = temp.path().join("outside");
    std::fs::create_dir(&broken).unwrap();
    let junction = work.join("junction");
    let result = std::process::Command::new(crate::resolve(&work, "python").unwrap())
        .args([
            "-c",
            "import _winapi,sys; _winapi.CreateJunction(sys.argv[1],sys.argv[2])",
        ])
        .arg(&broken)
        .arg(&junction)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "public directory-junction fixture must exist: stdout={} stderr={}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(host
        .prepare(&node(&work), &work, &[])
        .err()
        .unwrap()
        .contains("reparse"));
    std::fs::remove_dir(&junction).unwrap();
    state_empty(&host);
}

#[test]
fn concurrent_package_grants_preserve_each_other_and_cleanup_resumes_after_a_partial_delete() {
    let temp = tempfile::tempdir().unwrap();
    let work = temp.path().join("work");
    std::fs::create_dir(&work).unwrap();
    let host = Host::open(&temp.path().join("state")).unwrap();
    let mut first = host.prepare(&node(&work), &work, &[]).unwrap();
    let second = host.prepare(&node(&work), &work, &[]).unwrap();
    first.close().unwrap();
    let (running, input, out, err) = spawn(
        second,
        &[
            "-e".into(),
            "require('fs').writeFileSync('second-package','PUBLIC');process.stdout.write('OK')"
                .into(),
        ],
        &crate::process::environment(),
    )
    .unwrap();
    drop(input);
    let (exit, out, err) = finish(running, out, err);
    assert_eq!((exit, out), (0, "OK".into()), "{err}");
    assert_eq!(
        std::fs::read_to_string(work.join("second-package")).unwrap(),
        "PUBLIC"
    );
    state_empty(&host);
    // Simulate interruption after owned SID/profile revocation and journal
    // deletion but before the final marker/directory deletion.
    let orphan = host.root.join(uuid::Uuid::new_v4().to_string());
    std::fs::create_dir(&orphan).unwrap();
    std::fs::write(orphan.join("revoked"), "").unwrap();
    std::fs::write(orphan.join("unexpected-public-file"), "KEEP").unwrap();
    assert!(host.recover().is_err());
    assert_eq!(
        std::fs::read_to_string(orphan.join("unexpected-public-file")).unwrap(),
        "KEEP"
    );
    std::fs::remove_file(orphan.join("unexpected-public-file")).unwrap();
    host.recover().unwrap();
    state_empty(&host);
}

#[test]
#[ignore = "Only invoked by the public hard-crash fixture below"]
fn crash_worker() {
    let directory = PathBuf::from(
        std::env::var_os("DOLORES_ISOLATION_CRASH_FIXTURE").expect("owned fixture directory"),
    );
    let work = directory.join("work");
    let host = Host::open(&directory.join("state")).unwrap();
    let lease = host.prepare(&node(&work), &work, &[]).unwrap();
    let port: u16 = std::env::var("DOLORES_PUBLIC_CRASH_PORT")
        .unwrap()
        .parse()
        .unwrap();
    let grand = format!(
        r#"const fs=require('fs'),net=require('net');let denied;try{{fs.readFileSync('../outside');denied=false}}catch(e){{denied=['EPERM','EACCES'].includes(e.code)}}const s=net.createConnection({{host:'127.0.0.1',port:{port}}});let timer;function ready(network){{clearTimeout(timer);s.destroy();fs.writeFileSync('grand-ready',JSON.stringify({{outside:denied,network}}));setTimeout(()=>fs.writeFileSync('late-grand','BAD'),4000)}}s.once('connect',()=>ready('CONNECTED'));s.once('error',e=>ready(e.code));timer=setTimeout(()=>ready('TIMEOUT'),1000);"#
    );
    let code=format!("const fs=require('fs');fs.writeFileSync('root-ready','PUBLIC');require('child_process').spawn(process.execPath,['-e',{}],{{stdio:'ignore'}});setTimeout(()=>fs.writeFileSync('late-root','BAD'),4000);",serde_json::to_string(&grand).unwrap());
    let (_running, _input, _out, _err) =
        spawn(lease, &["-e".into(), code], &crate::process::environment()).unwrap();
    std::thread::sleep(Duration::from_secs(20));
    panic!("The parent must hard-kill this owned fixture process");
}

#[test]
fn hard_owner_crash_kills_container_descendants_and_recovery_revokes_without_replay() {
    let temp = tempfile::tempdir().unwrap();
    let work = temp.path().join("work");
    std::fs::create_dir(&work).unwrap();
    std::fs::write(temp.path().join("outside"), "KEEP").unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let connected = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (accepted, _) = listener.accept().unwrap();
    drop((connected, accepted));
    let mut worker = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "isolation::tests::crash_worker",
            "--nocapture",
        ])
        .env("DOLORES_ISOLATION_CRASH_FIXTURE", temp.path())
        .env(
            "DOLORES_PUBLIC_CRASH_PORT",
            listener.local_addr().unwrap().port().to_string(),
        )
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(8);
    while !work.join("grand-ready").exists()
        && Instant::now() < deadline
        && worker.try_wait().unwrap().is_none()
    {
        std::thread::sleep(Duration::from_millis(10));
    }
    let ready = std::fs::read_to_string(work.join("grand-ready")).ok();
    worker.kill().unwrap();
    worker.wait().unwrap();
    let ready: serde_json::Value =
        serde_json::from_str(ready.as_deref().expect("grandchild receipt before crash")).unwrap();
    assert_eq!(
        ready,
        serde_json::json!({"outside":true,"network":"EACCES"}),
        "grandchild retains filesystem and network denial"
    );
    std::thread::sleep(Duration::from_millis(4500));
    assert!(!work.join("late-root").exists());
    assert!(!work.join("late-grand").exists());
    assert_eq!(
        std::fs::read_to_string(temp.path().join("outside")).unwrap(),
        "KEEP"
    );
    let host = Host::open(&temp.path().join("state")).unwrap();
    assert!(
        std::fs::read_dir(&host.root).unwrap().count() > 1,
        "crash retains durable cleanup intent"
    );
    // Both paths are within this owned temporary fixture. A replacement root
    // must never be mistaken for the old grant target after an owner crash.
    let moved = temp.path().join("moved-work");
    assert!(moved.starts_with(temp.path()) && work.starts_with(temp.path()));
    std::fs::rename(&work, &moved).unwrap();
    std::fs::create_dir(&work).unwrap();
    std::fs::write(work.join("replacement"), "KEEP_REPLACEMENT").unwrap();
    assert!(host.recover().is_err());
    assert_eq!(
        std::fs::read_to_string(work.join("replacement")).unwrap(),
        "KEEP_REPLACEMENT"
    );
    std::fs::remove_file(work.join("replacement")).unwrap();
    std::fs::remove_dir(&work).unwrap();
    std::fs::rename(&moved, &work).unwrap();
    host.recover().unwrap();
    host.recover().unwrap();
    state_empty(&host);
    assert_eq!(
        std::fs::read_to_string(work.join("root-ready")).unwrap(),
        "PUBLIC"
    );
    assert!(!work.join("late-root").exists());
    let lease = host.prepare(&node(&work), &work, &[]).unwrap();
    let (running, input, out, err) = spawn(
        lease,
        &[
            "-e".into(),
            "process.stdout.write('EXPLICIT_FRESH_RUN')".into(),
        ],
        &crate::process::environment(),
    )
    .unwrap();
    drop(input);
    let (exit, out, err) = finish(running, out, err);
    assert_eq!((exit, out), (0, "EXPLICIT_FRESH_RUN".into()), "{err}");
    state_empty(&host);
}

#[test]
#[ignore = "Only invoked by the public ACL-rights fixture below"]
fn rights_worker() {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, WRITE_DAC,
    };
    let paths: Vec<PathBuf> =
        serde_json::from_str(&std::env::var("DOLORES_PUBLIC_RIGHTS_FIXTURE").unwrap()).unwrap();
    let results: Vec<i32> = paths
        .iter()
        .map(|p| {
            std::fs::OpenOptions::new()
                .access_mode(WRITE_DAC)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .share_mode(7)
                .open(p)
                .err()
                .and_then(|e| e.raw_os_error())
                .unwrap_or(0)
        })
        .collect();
    std::fs::write("rights.json", serde_json::to_vec(&results).unwrap()).unwrap();
}

#[test]
fn container_cannot_use_user_ownership_to_expand_readonly_or_runtime_acl_rights() {
    let temp = tempfile::tempdir().unwrap();
    let work = temp.path().join("work");
    let readonly = temp.path().join("readonly");
    std::fs::create_dir(&work).unwrap();
    std::fs::create_dir(&readonly).unwrap();
    std::fs::write(readonly.join("input"), "KEEP").unwrap();
    let host = Host::open(&temp.path().join("state")).unwrap();
    let executable = std::env::current_exe().unwrap();
    let mut env = crate::process::environment();
    let paths = vec![
        readonly.join("input"),
        work.clone(),
        host.root.clone(),
        executable.clone(),
    ];
    env.push((
        "DOLORES_PUBLIC_RIGHTS_FIXTURE".into(),
        serde_json::to_string(&paths).unwrap(),
    ));
    let args: Vec<String> = [
        "--ignored",
        "--exact",
        "isolation::tests::rights_worker",
        "--nocapture",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let (running, input, out, err) =
        crate::process::spawn_stdio(&executable, &args, &work, &env).unwrap();
    drop(input);
    let (exit, _, err) = finish(running, out, err);
    assert_eq!(exit, 0, "{err}");
    assert_eq!(
        std::fs::read_to_string(work.join("rights.json")).unwrap(),
        "[0,0,0,0]",
        "unrestricted API control"
    );
    let lease = host
        .prepare(&executable, &work, std::slice::from_ref(&readonly))
        .unwrap();
    let mut isolated_paths = paths;
    isolated_paths[3] = lease.executable();
    env.last_mut().unwrap().1 = serde_json::to_string(&isolated_paths).unwrap();
    let (running, input, out, err) = spawn(lease, &args, &env).unwrap();
    drop(input);
    let (exit, out, err) = finish(running, out, err);
    assert_eq!(exit, 0, "stdout={out} stderr={err}");
    assert_eq!(
        std::fs::read_to_string(work.join("rights.json")).unwrap(),
        "[5,5,5,5]",
        "WRITE_DAC must be denied even though the user owns each object"
    );
    state_empty(&host);
}

#[test]
fn lpac_denies_ipv4_ipv6_udp_and_inbound_bind_after_matching_native_controls() {
    use std::net::UdpSocket;
    let temp = tempfile::tempdir().unwrap();
    let work = temp.path().join("work");
    std::fs::create_dir(&work).unwrap();
    let host = Host::open(&temp.path().join("state")).unwrap();
    let tcp4 = TcpListener::bind("127.0.0.1:0").unwrap();
    let tcp6 = TcpListener::bind("[::1]:0").expect("IPv6 positive control must be available");
    let udp = UdpSocket::bind("127.0.0.1:0").unwrap();
    udp.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    let script = format!(
        r#"const net=require('net'),dgram=require('dgram');let timer;const result={{}};function done(k,v){{if(k in result)return;result[k]=v;if(Object.keys(result).length===4){{clearTimeout(timer);process.stdout.write(JSON.stringify(result))}}}}for(const [k,h,p] of [['tcp4','127.0.0.1',{}],['tcp6','::1',{}]]){{const s=net.createConnection({{host:h,port:p}});s.once('connect',()=>{{s.destroy();done(k,'CONNECTED')}});s.once('error',e=>{{s.destroy();done(k,e.code)}})}}const u=dgram.createSocket('udp4');function udone(v){{try{{u.close()}}catch{{}}done('udp',v)}}u.once('error',e=>udone(e.code));try{{u.send('PUBLIC_UDP',{},'127.0.0.1',e=>udone(e?e.code:'SENT'))}}catch(e){{udone(e.code)}}const server=net.createServer(s=>s.destroy());server.once('error',e=>done('listen',e.code));server.listen(0,'127.0.0.1',()=>{{server.close();done('listen','LISTENING')}});timer=setTimeout(()=>{{for(const k of ['tcp4','tcp6','udp','listen'])if(!(k in result))done(k,'TIMEOUT')}},1500);"#,
        tcp4.local_addr().unwrap().port(),
        tcp6.local_addr().unwrap().port(),
        udp.local_addr().unwrap().port()
    );
    let args = vec!["-e".into(), script];
    let executable = node(&work);
    let (running, input, out, err) =
        crate::process::spawn_stdio(&executable, &args, &work, &crate::process::environment())
            .unwrap();
    drop(input);
    let (exit, out, err) = finish(running, out, err);
    assert_eq!(exit, 0, "{err}");
    let native: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        native,
        serde_json::json!({"tcp4":"CONNECTED","tcp6":"CONNECTED","udp":"SENT","listen":"LISTENING"})
    );
    let mut received = [0; 32];
    let (n, _) = udp.recv_from(&mut received).unwrap();
    assert_eq!(&received[..n], b"PUBLIC_UDP");
    let lease = host.prepare(&executable, &work, &[]).unwrap();
    let (running, input, out, err) = spawn(lease, &args, &crate::process::environment()).unwrap();
    drop(input);
    let (exit, out, err) = finish(running, out, err);
    assert_eq!(exit, 0, "{err}");
    let isolated: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        isolated,
        serde_json::json!({"tcp4":"EACCES","tcp6":"EACCES","udp":"EACCES","listen":"EACCES"})
    );
    state_empty(&host);
}

// Opt-in environment qualification, never an ordinary deterministic CI case.
// Only public DNS names and a TLS handshake endpoint are contacted; no payload,
// provider credentials, private hostnames or profile contents are sent.
#[test]
#[ignore = "Explicit public-network qualification; needs external positive controls"]
fn public_network_and_system_dns_qualification() {
    use std::net::UdpSocket;
    let temp = tempfile::tempdir().unwrap();
    let work = temp.path().join("work");
    std::fs::create_dir(&work).unwrap();
    let host = Host::open(&temp.path().join("state")).unwrap();
    let route = UdpSocket::bind("0.0.0.0:0").unwrap();
    // UDP connect chooses a route without sending a datagram.
    route.connect("1.1.1.1:443").unwrap();
    let local = route.local_addr().unwrap().ip();
    let listener = TcpListener::bind((local, 0)).unwrap();
    let script = format!(
        r#"
const net=require('net'),dns=require('dns');const results={{}};
function done(k,v){{if(k in results)return;results[k]=v;if(Object.keys(results).length===3){{console.log(JSON.stringify(results));process.exit(0)}}}}
for(const [k,h,p] of [['lan',{},{}],['external','1.1.1.1',443]]){{const s=net.createConnection({{host:h,port:p}});s.once('connect',()=>{{s.destroy();done(k,'CONNECTED')}});s.once('error',e=>{{s.destroy();done(k,e.code)}});s.setTimeout(2500,()=>{{s.destroy();done(k,'TIMEOUT')}})}}
dns.lookup('example.com',{{all:true}},(e,a)=>done('systemDns',e?e.code:a.length?'RESOLVED':'EMPTY'));
setTimeout(()=>{{for(const k of ['lan','external','systemDns'])done(k,'TIMEOUT')}},3500);
"#,
        serde_json::to_string(&local.to_string()).unwrap(),
        listener.local_addr().unwrap().port()
    );
    let executable = node(&work);
    let args = vec!["-e".into(), script];
    let (running, input, out, err) =
        crate::process::spawn_stdio(&executable, &args, &work, &crate::process::environment())
            .unwrap();
    drop(input);
    let (exit, out, err) = finish(running, out, err);
    assert_eq!(exit, 0, "{err}");
    let native: serde_json::Value = serde_json::from_str(&out).unwrap();
    let lease = host.prepare(&executable, &work, &[]).unwrap();
    let (running, input, out, err) = spawn(lease, &args, &crate::process::environment()).unwrap();
    drop(input);
    let (exit, out, err) = finish(running, out, err);
    assert_eq!(exit, 0, "{err}");
    let isolated: serde_json::Value = serde_json::from_str(&out).unwrap();
    println!("PUBLIC_NETWORK native={native} isolated={isolated}");
    state_empty(&host);
    assert_eq!(
        native,
        serde_json::json!({"lan":"CONNECTED","external":"CONNECTED","systemDns":"RESOLVED"}),
        "positive controls unavailable; no denial qualification"
    );
    assert_eq!(isolated["lan"], "EACCES");
    assert_eq!(isolated["external"], "EACCES");
    assert_ne!(
        isolated["systemDns"], "RESOLVED",
        "system resolver can bypass direct socket denial"
    );
    assert_ne!(
        isolated["systemDns"], "TIMEOUT",
        "a timeout is not proof of denial"
    );
}

#[test]
fn cleanup_refuses_a_new_junction_before_acl_propagation_and_preserves_its_outside_target() {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Security::Authorization::{GetSecurityInfo, SE_FILE_OBJECT};
    fn dacl(path: &Path) -> Vec<u8> {
        let root = acl::root(path).unwrap();
        let mut descriptor = null_mut();
        let mut acl = null_mut();
        assert_eq!(
            unsafe {
                GetSecurityInfo(
                    root.file.as_raw_handle(),
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION,
                    null_mut(),
                    null_mut(),
                    &mut acl,
                    null_mut(),
                    &mut descriptor,
                )
            },
            0
        );
        let descriptor = Local(descriptor);
        assert!(!acl.is_null());
        let bytes =
            unsafe { std::slice::from_raw_parts(acl.cast::<u8>(), (*acl).AclSize as usize) }
                .to_vec();
        drop(descriptor);
        bytes
    }
    let temp = tempfile::tempdir().unwrap();
    let work = temp.path().join("work");
    let outside = temp.path().join("outside");
    std::fs::create_dir(&work).unwrap();
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(outside.join("keep"), "KEEP").unwrap();
    let before = dacl(&outside);
    let host = Host::open(&temp.path().join("state")).unwrap();
    let lease = host.prepare(&node(&work), &work, &[]).unwrap();
    let (mut process, input, _out, _err) = spawn(
        lease,
        &["-e".into(), "process.stdout.write('PUBLIC_DONE')".into()],
        &crate::process::environment(),
    )
    .unwrap();
    drop(input);
    let end = Instant::now() + Duration::from_secs(4);
    while process.try_exit().unwrap().is_none() {
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(5));
    }
    let junction = work.join("new-junction");
    let result = std::process::Command::new(crate::resolve(&work, "python").unwrap())
        .args([
            "-c",
            "import _winapi,sys; _winapi.CreateJunction(sys.argv[1],sys.argv[2])",
        ])
        .arg(&outside)
        .arg(&junction)
        .output()
        .unwrap();
    assert!(result.status.success());
    assert!(
        !process.stop_checked(),
        "unexpected alias must retain cleanup intent"
    );
    let error = host.recover().err().unwrap();
    assert!(error.contains("reparse"), "{error}");
    assert_eq!(
        dacl(&outside),
        before,
        "cleanup cannot propagate an ACL through the new alias"
    );
    assert_eq!(
        std::fs::read_to_string(outside.join("keep")).unwrap(),
        "KEEP"
    );
    std::fs::remove_dir(&junction).unwrap();
    host.recover().unwrap();
    state_empty(&host);
    assert_eq!(dacl(&outside), before);
}
