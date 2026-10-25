use lean4game::desktop::{LocalLeanServer, SecurityError, SecurityPolicy};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

#[test]
fn test_security_url_allowlist() {
    // 1. Allowed official origins
    assert!(SecurityPolicy::is_url_allowed("https://adam.math.hhu.de/"));
    assert!(SecurityPolicy::is_url_allowed("https://adam.math.hhu.de/#/g/leanprover-community/nng4"));
    assert!(SecurityPolicy::is_url_allowed("https://adam.math.hhu.de/#/g/tomyun/lean-game-real-analysis"));
    assert!(SecurityPolicy::is_url_allowed("https://leanprover-community.github.io/"));

    // 2. Allowed local self-contained loopback
    assert!(SecurityPolicy::is_url_allowed("http://127.0.0.1:8080/"));
    assert!(SecurityPolicy::is_url_allowed("http://localhost:8080/"));
    assert!(SecurityPolicy::is_url_allowed("http://127.0.0.1:3000/index.html"));

    // 3. Blocked malicious external domains
    assert!(!SecurityPolicy::is_url_allowed("http://evil-attacker.com"));
    assert!(!SecurityPolicy::is_url_allowed("https://malicious-site.org/phishing"));
    assert!(!SecurityPolicy::is_url_allowed("http://adam.math.hhu.de.evil.com/"));

    // 4. Blocked dangerous URI schemes
    assert!(!SecurityPolicy::is_url_allowed("file:///etc/passwd"));
    assert!(!SecurityPolicy::is_url_allowed("file:///C:/Windows/System32/cmd.exe"));
    assert!(!SecurityPolicy::is_url_allowed("javascript:alert(document.cookie)"));
    assert!(!SecurityPolicy::is_url_allowed("data:text/html,<h1>Hacked</h1>"));
    assert!(!SecurityPolicy::is_url_allowed("shell:startup"));
    assert!(!SecurityPolicy::is_url_allowed("vbscript:MsgBox(1)"));
}

#[test]
fn test_security_path_traversal_protection() {
    let base = PathBuf::from("saves");

    // 1. Safe paths inside base directory
    let safe_relative = Path::new("my_save.json");
    let res = SecurityPolicy::validate_save_path(&base, safe_relative);
    assert_eq!(res, Ok(PathBuf::from("saves/my_save.json")));

    let safe_sub = Path::new("subfolder/save1.json");
    let res_sub = SecurityPolicy::validate_save_path(&base, safe_sub);
    assert_eq!(res_sub, Ok(PathBuf::from("saves/subfolder/save1.json")));

    // 1b. Paths that already include the base directory must NOT duplicate the base prefix (e.g. saves/saves/...)
    let already_prefixed = Path::new("saves/official_lean4game_save.json");
    let res_prefixed = SecurityPolicy::validate_save_path(&base, already_prefixed);
    assert_eq!(
        res_prefixed,
        Ok(PathBuf::from("saves/official_lean4game_save.json")),
        "Must not duplicate base directory into saves/saves/..."
    );

    let curdir_prefixed = Path::new("./saves/official_lean4game_save.json");
    let res_curdir = SecurityPolicy::validate_save_path(&base, curdir_prefixed);
    assert_eq!(
        res_curdir,
        Ok(PathBuf::from("saves/official_lean4game_save.json")),
        "Must strip leading ./ and not duplicate base directory"
    );

    // 2. Blocked path traversal attempts
    let traversal_1 = Path::new("../escaped.json");
    let err_1 = SecurityPolicy::validate_save_path(&base, traversal_1);
    assert!(matches!(err_1, Err(SecurityError::PathTraversalAttempt(_))));

    let traversal_2 = Path::new("../../etc/passwd");
    let err_2 = SecurityPolicy::validate_save_path(&base, traversal_2);
    assert!(matches!(err_2, Err(SecurityError::PathTraversalAttempt(_))));

    let traversal_3 = Path::new("valid/../../dangerous.json");
    let err_3 = SecurityPolicy::validate_save_path(&base, traversal_3);
    assert!(matches!(err_3, Err(SecurityError::PathTraversalAttempt(_))));
}

#[test]
fn test_security_ipc_payload_validation() {
    // 1. Normal safe payload
    let mut safe_payload = HashMap::new();
    safe_payload.insert("nng_level_1".to_string(), "rfl".to_string());
    safe_payload.insert("real_analysis_eps".to_string(), "use 0".to_string());
    assert!(SecurityPolicy::validate_ipc_payload(&safe_payload).is_ok());

    // 2. Malicious key containing null byte
    let mut null_byte_payload = HashMap::new();
    null_byte_payload.insert("malicious\0key".to_string(), "valid_value".to_string());
    assert!(matches!(
        SecurityPolicy::validate_ipc_payload(&null_byte_payload),
        Err(SecurityError::InvalidKey(_))
    ));

    // 3. Oversized key (> 512 bytes)
    let mut oversized_key_payload = HashMap::new();
    let huge_key = "a".repeat(600);
    oversized_key_payload.insert(huge_key, "value".to_string());
    assert!(matches!(
        SecurityPolicy::validate_ipc_payload(&oversized_key_payload),
        Err(SecurityError::InvalidKey(_))
    ));

    // 4. Oversized value (> 2 MB)
    let mut oversized_value_payload = HashMap::new();
    let huge_value = "v".repeat(2 * 1024 * 1024 + 100);
    oversized_value_payload.insert("normal_key".to_string(), huge_value);
    assert!(matches!(
        SecurityPolicy::validate_ipc_payload(&oversized_value_payload),
        Err(SecurityError::InvalidValue(_))
    ));
}

#[test]
fn test_local_lean_server_embedded_endpoints() {
    // Start local server on an OS-allocated random free port (0)
    let port = LocalLeanServer::start_in_background(0).expect("Failed to start server on free port");
    assert!(port > 0);

    // 1. Test /api/health
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("Failed connecting to local server");
    let req = format!("GET /api/health HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n", port);
    stream.write_all(req.as_bytes()).unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    assert!(response.contains("200 OK"));
    assert!(response.contains(r#""status":"ok""#));
    assert!(response.contains(r#""offline":true"#));

    // 2. Test /api/games
    let mut stream2 = TcpStream::connect(("127.0.0.1", port)).expect("Failed connecting to local server");
    let req2 = format!("GET /api/games HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n", port);
    stream2.write_all(req2.as_bytes()).unwrap();

    let mut response2 = String::new();
    stream2.read_to_string(&mut response2).unwrap();
    assert!(response2.contains("200 OK"));
    assert!(response2.contains("tutorial"));
    assert!(response2.contains("addition"));

    // 3. Test /api/verify with a valid proof
    let mut stream3 = TcpStream::connect(("127.0.0.1", port)).expect("Failed connecting to local server");
    let body = r#"{"world_id":"tutorial","level_num":1,"proof_script":"rfl"}"#;
    let req3 = format!(
        "POST /api/verify HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        port,
        body.len(),
        body
    );
    stream3.write_all(req3.as_bytes()).unwrap();

    let mut response3 = String::new();
    stream3.read_to_string(&mut response3).unwrap();
    assert!(response3.contains("200 OK"));
    assert!(response3.contains(r#""completed":true"#));
}
