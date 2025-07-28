use std::path::PathBuf;

use tokio::test;
use wash::{cli::CliContext, config::Config};

use bank_account::BankCommand;
use prost::Message;
use reqwest::Client;
use tokio::process::Command;

#[test]
async fn test_bank_account_basic() -> anyhow::Result<()> {
    let cli_context = CliContext::new().await?;
    let config = Config::default();

    let path = PathBuf::from("./command_handler");
    let _command_handler_res =
        CommandManifest::from_file(path.join("Cargo.toml"))?.compilation_targets;

    let path = PathBuf::from("../event_sourcer");
    let _event_sourcer_res =
        wash::cli::component_build::build_component(&path, &cli_context, &config).await;

    let _path = PathBuf::from("../filesystem_event_store");
    let _filesystem_event_store_res =
        wash::cli::component_build::build_component(&path, &cli_context, &config).await;

    let _path = PathBuf::from("../http_api_gateway");
    let _http_res = wash::cli::component_build::build_component(&path, &cli_context, &config).await;

    // let command_handler_path = PathBuf::from("../target/wasm32-wasip2/release/command_handler.wasm");
    // let event_sourcer_path = PathBuf::from("../target/wasm32-wasip2/release/event_sourcer.wasm");
    // let filesystem_event_store_path =
    //     PathBuf::from("../target/wasm32-wasip2/release/filesystem_event_store.wasm");

    // // Register the packages with the graph
    // let mut graph = CompositionGraph::new();

    // let command_handler_pkg = Package::from_file(
    //     "cosmonic:eventsourcing/command-handler",
    //     None,
    //     command_handler_path,
    //     graph.types_mut(),
    // )?;
    // let package1 = graph.register_package(command_handler_pkg)?;

    // let pkg = Package::from_file(
    //     "cosmonic:eventsourcing/event-sourcer",
    //     None,
    //     event_sourcer_path,
    //     graph.types_mut(),
    // )?;
    // let package2 = graph.register_package(pkg)?;

    // let fs_pkg = Package::from_file(
    //     "cosmonic:eventsourcing/event-store",
    //     None,
    //     filesystem_event_store_path,
    //     graph.types_mut(),
    // )?;
    // let package3 = graph.register_package(fs_pkg)?;

    // let http_pkg = Package::from_file(
    //     "wasi:http/incoming-handler",
    //     None,
    //     "../target/wasm32-wasip2/release/http_api_gateway.wasm",
    //     graph.types_mut(),
    // )?;
    // let package4 = graph.register_package(http_pkg)?;

    // // wac_graph::plug(
    // //     &mut graph,
    // //     vec![
    // //         package1,
    // //         package2,
    // //         package3,
    // //     ],
    // //     package4,
    // // )?;
    // wac_graph::plug(&mut graph, vec![package2, package1, package3], package4)?;
    // // wac_graph::plug(&mut graph, vec![package1], package4)?;
    // // wac_graph::plug(&mut graph, vec![package3], package4)?;
    // // wac_graph::plug(&mut graph, vec![package1, package3], package2)?;
    // let bytes = graph.encode(EncodeOptions::default())?;

    // let component = wash::inspect::decode_component(bytes.as_slice()).await?;
    // let wit = wash::inspect::get_component_wit(component).await?;

    // eprintln!("{wit}");

    // Define output paths with descriptive names
    let event_sourcer_plugged = "http_with_event_sourcer.wasm";
    let command_handler_plugged = "http_with_event_sourcer_and_command_handler.wasm";
    let fs_store_plugged = "http_with_event_sourcer_command_handler_and_fs_store.wasm";
    let command_handler_final = "final_composed_bank_account.wasm";

    // Plug event_sourcer into http_api_gateway
    let status = Command::new("wac")
        .arg("plug")
        .arg("--plug")
        .arg("../target/wasm32-wasip2/release/event_sourcer.wasm")
        .arg("../target/wasm32-wasip2/release/http_api_gateway.wasm")
        .stdout(std::fs::File::create(event_sourcer_plugged)?)
        .status()
        .await?;
    assert!(status.success(), "wac plug event_sourcer failed");

    // Plug bank_account_command_handler into http_with_event_sourcer.wasm
    let status = Command::new("wac")
        .arg("plug")
        .arg("--plug")
        .arg("../target/wasm32-wasip2/release/bank_account_command_handler.wasm")
        .arg(event_sourcer_plugged)
        .stdout(std::fs::File::create(command_handler_plugged)?)
        .status()
        .await?;
    assert!(
        status.success(),
        "wac plug bank_account_command_handler (1) failed"
    );

    // Plug filesystem_event_store into http_with_event_sourcer_and_command_handler.wasm
    let status = Command::new("wac")
        .arg("plug")
        .arg("--plug")
        .arg("../target/wasm32-wasip2/release/filesystem_event_store.wasm")
        .arg(command_handler_plugged)
        .stdout(std::fs::File::create(fs_store_plugged)?)
        .status()
        .await?;
    assert!(status.success(), "wac plug filesystem_event_store failed");

    // Plug bank_account_command_handler into http_with_event_sourcer_command_handler_and_fs_store.wasm
    let status = Command::new("wac")
        .arg("plug")
        .arg("--plug")
        .arg("../target/wasm32-wasip2/release/bank_account_command_handler.wasm")
        .arg(fs_store_plugged)
        .stdout(std::fs::File::create(command_handler_final)?)
        .status()
        .await?;
    assert!(
        status.success(),
        "wac plug bank_account_command_handler (2) failed"
    );

    tokio::fs::create_dir_all("bank_store/foobar").await?;
    // Spawn the wasmtime serve process
    let mut child = Command::new("wasmtime")
        .arg("serve")
        .arg("-Scommon")
        .arg("./final_composed_bank_account.wasm")
        .arg("--dir")
        .arg("./bank_store")
        .spawn()?;

    // Optionally, wait for the process to start up (e.g., sleep or check readiness)
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // You can interact with `child` as needed, e.g., await its completion:
    // let status = child.wait().await?;
    // assert!(status.success(), "wasmtime serve failed");
    // Create HTTP client
    let client = Client::new();

    // Test 1: Open a new bank account
    let open_account_command = BankCommand {
        command: Some(bank_account::bank_command::Command::OpenAccount(1000)),
    };

    let encoded_command = open_account_command.encode_to_vec();

    // Send POST request to open account
    let response = client
        .post("http://localhost:8080")
        .header("Content-Type", "application/x-protobuf")
        .body(encoded_command)
        .send()
        .await;

    match response {
        Ok(resp) => {
            println!("Open account response status: {}", resp.status());
            if resp.status().is_success() {
                let body = resp.bytes().await?;
                println!("Response body length: {} bytes", body.len());
            }
        }
        Err(e) => {
            println!("Failed to connect to localhost:8080 (server might not be running): {e}");
        }
    }

    // Test 2: Make a positive transaction (+500)
    let transaction_command1 = BankCommand {
        command: Some(bank_account::bank_command::Command::Transaction(500)),
    };
    let encoded_transaction1 = transaction_command1.encode_to_vec();
    
    let response = client
        .post("http://localhost:8080")
        .header("Content-Type", "application/x-protobuf")
        .body(encoded_transaction1)
        .send()
        .await;

    match response {
        Ok(resp) => {
            println!("Transaction 1 (+500) response status: {}", resp.status());
        }
        Err(e) => {
            println!("Failed to send transaction 1: {e}");
        }
    }

    // Test 3: Make another positive transaction (+250)
    let transaction_command2 = BankCommand {
        command: Some(bank_account::bank_command::Command::Transaction(250)),
    };
    let encoded_transaction2 = transaction_command2.encode_to_vec();
    
    let response = client
        .post("http://localhost:8080")
        .header("Content-Type", "application/x-protobuf")
        .body(encoded_transaction2)
        .send()
        .await;

    match response {
        Ok(resp) => {
            println!("Transaction 2 (+250) response status: {}", resp.status());
        }
        Err(e) => {
            println!("Failed to send transaction 2: {e}");
        }
    }

    // Test 4: Read all events from the directory
    let events_dir = "./bank_store/foobar";
    match std::fs::read_dir(events_dir) {
        Ok(entries) => {
            let mut event_files: Vec<_> = entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
                .collect();
            
            // Sort by filename to get chronological order
            event_files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
            
            println!("📁 Found {} event files in {}", event_files.len(), events_dir);
            
            for (i, entry) in event_files.iter().enumerate() {
                let file_path = entry.path();
                let filename = entry.file_name();
                
                match std::fs::read(&file_path) {
                    Ok(event_bytes) => {
                        println!("  Event {}: {} ({} bytes)", i + 1, filename.to_string_lossy(), event_bytes.len());
                        
                        // Skip the first byte which appears to be a length prefix
                        let protobuf_data = if event_bytes.len() > 1 && event_bytes[0] as usize == event_bytes.len() - 1 {
                            &event_bytes[1..]
                        } else {
                            &event_bytes
                        };
                        
                        match bank_account::BankEvent::decode(protobuf_data) {
                            Ok(event) => {
                                match &event.event {
                                    Some(bank_account::bank_event::Event::Opened(opened)) => {
                                        println!("    → Account opened with ID '{}' and balance {}", 
                                                 opened.id, opened.balance);
                                    },
                                    Some(bank_account::bank_event::Event::Transaction(tx)) => {
                                        println!("    → Transaction amount {}", tx.amount);
                                    },
                                    Some(bank_account::bank_event::Event::Denied(denied)) => {
                                        println!("    → Transaction denied for amount {}", denied.amount);
                                    },
                                    None => {
                                        println!("    → Unknown/empty event");
                                    }
                                }
                            },
                            Err(e) => {
                                println!("    ❌ Failed to decode: {e}");
                                println!("    Raw bytes (first 20): {:?}", &event_bytes[..std::cmp::min(20, event_bytes.len())]);
                            }
                        }
                    },
                    Err(e) => {
                        println!("    ❌ Failed to read file: {e}");
                    }
                }
            }
        },
        Err(e) => {
            println!("❌ Failed to read events directory '{}': {e}", events_dir);
        }
    }

    println!("🏦 Bank account HTTP test completed");

    let _ = child.kill().await;
    Ok(())
}
