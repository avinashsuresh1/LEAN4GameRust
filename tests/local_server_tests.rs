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
    assert!(games.len() >= 7, "Curriculum must have all 7 canonical games from adam.math.hhu.de");

    // 1. Verify Natural Numbers Game
    let nng = games.iter().find(|g| g.id == "natural_numbers").expect("NNG game package");
    assert_eq!(nng.title, "Natural Number Game (NNG4)");
    assert!(!nng.worlds.is_empty());
    let tutorial = nng.worlds.iter().find(|w| w.id == "tutorial").expect("Tutorial world");
    let tut1 = tutorial.levels.iter().find(|l| l.number == 1).expect("Tutorial level 1");
    assert!(!tut1.doc.is_empty());
    assert!(tut1.doc.contains("Welcome to the Lean 4 Game!"));
    assert!(!tut1.hints.is_empty());

    // 2. Verify Scribble / Robo
    let robo = games.iter().find(|g| g.id == "robo").expect("Robo game package");
    assert_eq!(robo.title, "Scribble / Robo");
    assert_eq!(robo.icon, "🧝");
    let r_logic = robo.worlds.iter().find(|w| w.id == "robo_logic").expect("Robo logic world");
    let r1 = r_logic.levels.iter().find(|l| l.number == 1).expect("Robo level 1");
    assert!(r1.doc.contains("Scribble"));

    // 3. Verify Real Analysis Game
    let ra = games.iter().find(|g| g.id == "real_analysis").expect("Real analysis package");
    assert_eq!(ra.title, "Real Analysis, The Game");

    // 4. Verify Set Theory Game
    let st = games.iter().find(|g| g.id == "set_theory").expect("Set theory package");
    assert_eq!(st.title, "Set Theory & Logic Game");

    // 5. Verify A Lean Intro to Logic
    let logic = games.iter().find(|g| g.id == "intro_to_logic").expect("Logic game package");
    assert_eq!(logic.title, "A Lean Intro to Logic");

    // 6. Verify Reintroduction to Proofs
    let riehl = games.iter().find(|g| g.id == "reintroduction_to_proofs").expect("Emily Riehl game package");
    assert_eq!(riehl.title, "Reintroduction to Proofs");

    // 7. Verify Linear Algebra Game
    let la = games.iter().find(|g| g.id == "linear_algebra").expect("Linear algebra package");
    assert_eq!(la.title, "Linear Algebra Done Right in Lean");
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

    // 3. Verify Scribble/Robo level 1 with "rfl"
    {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("Failed connecting to local server");
        let v_req = VerificationRequest {
            game_id: Some("robo".to_string()),
            world_id: "robo_logic".to_string(),
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
    }

    // 4. Verify A Lean Intro to Logic level 1 with "intro h\nrcases h with ⟨hp, hq⟩\nexact hp"
    {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("Failed connecting to local server");
        let v_req = VerificationRequest {
            game_id: Some("intro_to_logic".to_string()),
            world_id: "logic_connectives".to_string(),
            level_num: 1,
            proof_script: "intro h\nrcases h with ⟨hp, hq⟩\nexact hp".to_string(),
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

    // 5. Verify empty proof returns initial goals and hypotheses in goal_infos
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

#[test]
fn test_embedded_html_contains_ghost_text_and_no_placeholder() {
    let html = LocalLeanServer::embedded_html_client();

    // Verify ghost suggestion overlay element is present
    assert!(html.contains("id=\"ghost-suggestion\""), "Must contain ghost suggestion element");
    assert!(html.contains("class=\"ghost-suggestion\""), "Must have ghost suggestion CSS class");

    // Verify textarea exists without hardcoded placeholder attribute
    assert!(html.contains("id=\"code-editor\""), "Must contain code editor textarea");
    assert!(
        !html.contains("id=\"code-editor\" class=\"code-textarea\" spellcheck=\"false\" placeholder="),
        "Code editor textarea must not use hardcoded placeholder"
    );

    // Verify dynamic ghost text update function and input listener
    assert!(html.contains("function updateGhostText()"), "Must define updateGhostText function");
    assert!(html.contains("updateGhostText();"), "Must invoke updateGhostText on editor modifications");
    assert!(html.contains("activeVerifyController"), "Must utilize AbortController to cancel stale in-flight requests");
}

#[test]
fn test_local_server_handles_chunked_and_split_post_body() {
    let port = LocalLeanServer::start_in_background(0).expect("Failed to start local server on free port");
    assert!(port > 0);

    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("Failed connecting to local server");
    let v_req = VerificationRequest {
        game_id: Some("natural_numbers".to_string()),
        world_id: "tutorial".to_string(),
        level_num: 1,
        proof_script: "rfl".to_string(),
    };
    let body = serde_json::to_string(&v_req).unwrap();
    let headers = format!(
        "POST /api/verify HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        port,
        body.len()
    );

    // 1. Send headers first in a separate TCP segment
    stream.write_all(headers.as_bytes()).unwrap();
    stream.flush().unwrap();

    // 2. Simulate network delay/fragmentation where body arrives later
    std::thread::sleep(std::time::Duration::from_millis(50));

    // 3. Send body in subsequent segment
    stream.write_all(body.as_bytes()).unwrap();
    stream.flush().unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    assert!(response.contains("200 OK"), "Response must be 200 OK");
    assert!(
        !response.contains("Malformed request body: EOF while parsing a value"),
        "Must not fail with EOF while parsing body when split across TCP packets: {}",
        response
    );

    let body_pos = response.find("\r\n\r\n").expect("HTTP headers end");
    let resp_obj: VerificationResponse = serde_json::from_str(&response[body_pos + 4..]).unwrap();
    assert!(resp_obj.completed, "Level 1 with rfl must complete successfully");
    assert!(resp_obj.success);
}

#[test]
fn test_local_server_handles_empty_post_body_gracefully() {
    let port = LocalLeanServer::start_in_background(0).expect("Failed to start local server on free port");
    assert!(port > 0);

    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("Failed connecting to local server");
    let req = format!(
        "POST /api/verify HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        port
    );
    stream.write_all(req.as_bytes()).unwrap();
    stream.flush().unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    assert!(response.contains("200 OK"));
    let body_pos = response.find("\r\n\r\n").expect("HTTP headers end");
    let resp_obj: VerificationResponse = serde_json::from_str(&response[body_pos + 4..]).unwrap();
    assert!(!resp_obj.completed);
    assert!(!resp_obj.success);
    assert!(resp_obj.message.contains("Empty verification request"));
}

#[test]
fn test_game_selection_dropdown_contrast_and_reset_progress() {
    let html = LocalLeanServer::embedded_html_client();

    // Verify high-contrast dropdown styles
    assert!(html.contains("select.game-dropdown {"), "Must have select.game-dropdown CSS");
    assert!(html.contains("color: #ffffff;"), "Dropdown must use pure white high-contrast text");
    assert!(html.contains("select.game-dropdown option {"), "Must style dropdown option elements for contrast");
    assert!(html.contains("border: 1.5px solid #89b4fa;"), "Dropdown must have high-visibility border");

    // Verify reset progress functionality
    assert!(html.contains("resetAllProgress()"), "Must have Reset Progress button in header");
    assert!(html.contains("function resetAllProgress()"), "Must define resetAllProgress function");
    assert!(html.contains("localStorage.clear();"), "resetAllProgress must clear localStorage");
    assert!(html.contains("sessionStorage.removeItem(\"__lean4game_hydrated__\");"), "resetAllProgress must clear session hydration marker");
}

#[test]
fn test_embedded_html_contains_draft_autosave_listeners() {
    let html = LocalLeanServer::embedded_html_client();

    // Verify proof draft autosave function
    assert!(html.contains("function saveCurrentProofDraft()"), "Must define saveCurrentProofDraft function");
    assert!(html.contains("lean4game_proof_"), "saveCurrentProofDraft must use level-specific proof keys");

    // Verify draft is saved before level transition
    assert!(
        html.contains("function loadLevel(gameId, worldId, levelNum) {\n            saveCurrentProofDraft();"),
        "loadLevel must invoke saveCurrentProofDraft before changing active level"
    );

    // Verify debounced input autosave on editor
    assert!(html.contains("draftDebounceTimer = setTimeout(saveCurrentProofDraft, 250);"), "Editor input must auto-save proof draft with debounce");

    // Verify window unload and pagehide lifecycle auto-save
    assert!(html.contains("window.addEventListener(\"beforeunload\", saveCurrentProofDraft);"), "Must register beforeunload autosave handler");
    assert!(html.contains("window.addEventListener(\"pagehide\", saveCurrentProofDraft);"), "Must register pagehide autosave handler");

    // Verify snippet and solution insertion auto-save
    assert!(html.contains("insertSnippet(text) {\n            const editor"), "insertSnippet must be defined");
    assert!(html.contains("saveCurrentProofDraft();\n        }\n\n        function updateGhostText()"), "insertSnippet must auto-save after snippet insertion");

    // Verify resetProofCode removes draft from localStorage
    assert!(html.contains("localStorage.removeItem(saveKey);"), "resetProofCode must clear the draft from localStorage");
}


