use lean4game::desktop::local_server::{LocalLeanServer, VerificationRequest, VerificationResponse};
use lean4game::game::package::GamePackage;
use std::io::{Read, Write};
use std::net::TcpStream;

#[test]
fn test_embedded_html_client_contains_pedagogical_dialogue_hints_and_infoview() {
    let html = LocalLeanServer::embedded_html_client();

    // Verify Pedagogical Story and Dialogue components
    assert!(html.contains("id=\"dialogue-body\""), "Embedded client must have dialogue body container");
    assert!(html.contains("id=\"tab-btn-story\""), "Embedded client must have story tab");
    assert!(html.contains("id=\"theorem-signature\""), "Embedded client must have theorem signature display");

    // Verify Step-by-Step Hints components
    assert!(html.contains("id=\"tab-btn-hints\""), "Embedded client must have hints tab");
    assert!(html.contains("id=\"hints-list\""), "Embedded client must have hints accordion list");
    assert!(html.contains("id=\"hints-counter\""), "Embedded client must have hints counter badge");
    assert!(html.contains("hint-accordion-card"), "Embedded client must have hint accordion card styling");

    // Verify Tactics and Theorems Inventory components
    assert!(html.contains("id=\"tab-btn-tactics\""), "Embedded client must have tactics reference tab");
    assert!(html.contains("id=\"tab-btn-theorems\""), "Embedded client must have theorems inventory tab");
    assert!(html.contains("id=\"level-tactics-chips\""), "Embedded client must have level tactics chips");
    assert!(html.contains("id=\"tactics-reference-list\""), "Embedded client must have tactics guide list");
    assert!(html.contains("id=\"theorems-inventory-list\""), "Embedded client must have unlocked theorems list");

    // Verify Game Selector and Worlds Map
    assert!(html.contains("id=\"game-select\""), "Embedded client must have game selector dropdown");
    assert!(html.contains("id=\"tab-btn-worlds\""), "Embedded client must have worlds map tab");
    assert!(html.contains("id=\"worlds-map-list\""), "Embedded client must have worlds list");

    // Verify Editor with Line Numbers and Math Symbol Toolbar
    assert!(html.contains("id=\"line-numbers\""), "Embedded client must have line numbers gutter");
    assert!(html.contains("id=\"code-editor\""), "Embedded client must have code editor textarea");
    assert!(html.contains("symbol-toolbar"), "Embedded client must have math symbols toolbar");
    assert!(html.contains("insertSym('⊢')"), "Embedded client must support turnstile insertion");
    assert!(html.contains("insertSym('←')"), "Embedded client must support reverse rewrite arrow insertion");

    // Verify Lean 4 Infoview
    assert!(html.contains("Lean 4 Infoview"), "Embedded client must have Lean 4 Infoview");
    assert!(html.contains("id=\"infoview-body\""), "Embedded client must have Infoview body container");
    assert!(html.contains("celebration-card"), "Embedded client must have completion celebration card");
    assert!(html.contains("turnstile"), "Embedded client must have turnstile styling for goals");
}

#[test]
fn test_local_server_api_games_returns_complete_curriculum_with_hints_and_dialogue() {
    let port = LocalLeanServer::start_in_background(0).expect("Failed to start local server on free port");
    assert!(port > 0);

    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("Failed connecting to local server");
    let req = format!("GET /api/games HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n", port);
    stream.write_all(req.as_bytes()).unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    assert!(response.contains("200 OK"));
    let body_pos = response.find("\r\n\r\n").expect("HTTP headers end");
    let body = &response[body_pos + 4..];

    let games: Vec<GamePackage> = serde_json::from_str(body).expect("Valid GamePackage JSON list");
    assert!(games.len() >= 3, "Curriculum must have at least 3 games (NNG, Real Analysis, Set Theory)");

    // Verify Natural Numbers Game
    let nng = games.iter().find(|g| g.id == "natural_numbers").expect("NNG game package");
    assert_eq!(nng.title, "Natural Number Game (NNG4)");
    assert!(!nng.worlds.is_empty());

    let tutorial = nng.worlds.iter().find(|w| w.id == "tutorial").expect("Tutorial world");
    let tut1 = tutorial.levels.iter().find(|l| l.number == 1).expect("Tutorial level 1");

    // Verify doc (story/dialogue), hints, and available_tactics are present and populated
    assert!(!tut1.doc.is_empty(), "Level doc must not be empty");
    assert!(tut1.doc.contains("Welcome to the Lean 4 Game!"), "Doc must contain pedagogical narrative");
    assert!(!tut1.hints.is_empty(), "Level hints must not be empty");
    assert!(!tut1.available_tactics.is_empty(), "Level available tactics must not be empty");
    assert!(tut1.available_tactics.contains(&"rfl".to_string()));

    // Verify Real Analysis Game
    let ra = games.iter().find(|g| g.id == "real_analysis").expect("Real analysis package");
    assert_eq!(ra.title, "Real Analysis, The Game");
    let rf = ra.worlds.iter().find(|w| w.id == "real_foundations").expect("Real foundations world");
    let ra1 = rf.levels.iter().find(|l| l.number == 1).expect("Real analysis level 1");
    assert!(!ra1.doc.is_empty());
    assert!(!ra1.hints.is_empty());
}

#[test]
fn test_local_server_verify_multi_game_and_goal_infos() {
    let port = LocalLeanServer::start_in_background(0).expect("Failed to start local server on free port");
    assert!(port > 0);

    // 1. Verify NNG tutorial level 1 with "rfl"
    {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("Failed connecting to local server");
        let v_req = VerificationRequest {
            game_id: Some("natural_numbers".to_string()),
            world_id: "tutorial".to_string(),
            level_num: 1,
            proof_script: "rfl".to_string(),
        };
        let body = serde_json::to_string(&v_req).unwrap();
        let req = format!(
            "POST /api/verify HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            port,
            body.len(),
            body
        );
        stream.write_all(req.as_bytes()).unwrap();

        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        assert!(response.contains("200 OK"));
        let body_pos = response.find("\r\n\r\n").expect("HTTP headers end");
        let resp_obj: VerificationResponse = serde_json::from_str(&response[body_pos + 4..]).unwrap();
        assert!(resp_obj.completed);
        assert!(resp_obj.success);
        assert!(resp_obj.goals.is_empty());
    }

    // 2. Verify Real Analysis level 1 with "rw [sub_self_real, abs_zero]"
    {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("Failed connecting to local server");
        let v_req = VerificationRequest {
            game_id: Some("real_analysis".to_string()),
            world_id: "real_foundations".to_string(),
            level_num: 1,
            proof_script: "rw [sub_self_real, abs_zero]".to_string(),
        };
        let body = serde_json::to_string(&v_req).unwrap();
        let req = format!(
            "POST /api/verify HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            port,
            body.len(),
            body
        );
        stream.write_all(req.as_bytes()).unwrap();

        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        assert!(response.contains("200 OK"));
        let body_pos = response.find("\r\n\r\n").expect("HTTP headers end");
        let resp_obj: VerificationResponse = serde_json::from_str(&response[body_pos + 4..]).unwrap();
        assert!(resp_obj.completed);
        assert!(resp_obj.success);
    }

    // 3. Verify empty proof returns initial goals and hypotheses in goal_infos
    {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("Failed connecting to local server");
        let v_req = VerificationRequest {
            game_id: Some("natural_numbers".to_string()),
            world_id: "tutorial".to_string(),
            level_num: 1,
            proof_script: "-- Unsolved".to_string(),
        };
        let body = serde_json::to_string(&v_req).unwrap();
        let req = format!(
            "POST /api/verify HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            port,
            body.len(),
            body
        );
        stream.write_all(req.as_bytes()).unwrap();

        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        assert!(response.contains("200 OK"));
        let body_pos = response.find("\r\n\r\n").expect("HTTP headers end");
        let resp_obj: VerificationResponse = serde_json::from_str(&response[body_pos + 4..]).unwrap();
        assert!(!resp_obj.completed);
        assert_eq!(resp_obj.goal_infos.len(), 1);
        assert_eq!(resp_obj.goal_infos[0].target, "x + 0 = x + 0");
        assert!(resp_obj.goal_infos[0].hypotheses.iter().any(|h| h.contains("x : ℕ")));
    }
}

#[test]
fn test_local_server_api_worlds_endpoint() {
    let port = LocalLeanServer::start_in_background(0).expect("Failed to start local server on free port");
    assert!(port > 0);

    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("Failed connecting to local server");
    let req = format!("GET /api/worlds HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n", port);
    stream.write_all(req.as_bytes()).unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    assert!(response.contains("200 OK"));
    assert!(response.contains("tutorial"));
    assert!(response.contains("addition"));
}
