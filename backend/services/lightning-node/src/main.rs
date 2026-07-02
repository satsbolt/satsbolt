use actix_web::{get, post, web, App, HttpServer, HttpResponse, Responder, middleware::Logger};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::str::FromStr;
use log::{info, error, warn};
use ldk_node::Builder;
use ldk_node::bitcoin::Network;
use ldk_node::lightning::ln::msgs::SocketAddress;

#[derive(Clone)]
struct AppState {
    node: Option<Arc<ldk_node::Node>>,
    is_mock: bool,
    mock_invoices: Arc<Mutex<std::collections::HashMap<String, MockInvoice>>>,
    api_server_url: String,
    internal_service_secret: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct MockInvoice {
    payment_hash: String,
    amount_msat: u64,
    description: String,
    user_id: String,
    settled: bool,
}

#[derive(Deserialize)]
struct CreateInvoiceRequest {
    amount_msat: u64,
    description: String,
    expiry_secs: u32,
    user_id: String,
}

#[derive(Serialize)]
struct CreateInvoiceResponse {
    invoice: String,
    payment_hash: String,
}

#[derive(Deserialize)]
struct PayInvoiceRequest {
    invoice: String,
    withdrawal_id: String,
}

#[derive(Serialize)]
struct PayInvoiceResponse {
    status: String,
    payment_hash: String,
    fee_msat: u64,
}

#[derive(Serialize)]
struct StatusResponse {
    node_id: String,
    is_running: bool,
    is_mock: bool,
    num_channels: usize,
    num_peers: usize,
}

#[derive(Deserialize)]
struct MockReceivePaymentRequest {
    payment_hash: String,
    amount_msat: u64,
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// Handler: Create Invoice
#[post("/invoice")]
async fn create_invoice(
    state: web::Data<AppState>,
    req: web::Json<CreateInvoiceRequest>,
) -> impl Responder {
    if state.is_mock {
        // Mock mode invoice generation
        let payment_hash = format!("mock_hash_{}", uuid::Uuid::new_v4().simple());
        let invoice_str = format!("lnbc100u1mockinvoice_{}", uuid::Uuid::new_v4().simple());
        
        let mock_inv = MockInvoice {
            payment_hash: payment_hash.clone(),
            amount_msat: req.amount_msat,
            description: req.description.clone(),
            user_id: req.user_id.clone(),
            settled: false,
        };
        
        state.mock_invoices.lock().unwrap().insert(payment_hash.clone(), mock_inv);
        
        return HttpResponse::Ok().json(CreateInvoiceResponse {
            invoice: invoice_str,
            payment_hash,
        });
    }

    // Real mode invoice generation
    if let Some(ref node) = state.node {
        use ldk_node::lightning_invoice::{Bolt11InvoiceDescription, Description};
        
        let desc = match Description::new(req.description.clone()) {
            Ok(d) => d,
            Err(e) => return HttpResponse::BadRequest().body(format!("Invalid description: {:?}", e)),
        };
        let invoice_desc = Bolt11InvoiceDescription::Direct(desc);

        match node.bolt11_payment().receive(req.amount_msat, &invoice_desc, req.expiry_secs) {
            Ok(invoice) => {
                let payment_hash = hex_encode(invoice.payment_hash().as_ref());
                HttpResponse::Ok().json(CreateInvoiceResponse {
                    invoice: invoice.to_string(),
                    payment_hash,
                })
            }
            Err(e) => {
                error!("LDK receive payment error: {:?}", e);
                HttpResponse::InternalServerError().body(format!("Failed to create invoice: {:?}", e))
            }
        }
    } else {
        HttpResponse::InternalServerError().body("Node not initialized")
    }
}

// Handler: Pay Invoice
#[post("/pay")]
async fn pay_invoice(
    state: web::Data<AppState>,
    req: web::Json<PayInvoiceRequest>,
) -> impl Responder {
    if state.is_mock {
        info!("Mock paying invoice: {}", req.invoice);
        // Simulate a delay/network call
        let payment_hash = format!("mock_pay_hash_{}", uuid::Uuid::new_v4().simple());
        return HttpResponse::Ok().json(PayInvoiceResponse {
            status: "success".to_string(),
            payment_hash,
            fee_msat: 1000, // mock 1 sat fee
        });
    }

    if let Some(ref node) = state.node {
        use ldk_node::lightning_invoice::Bolt11Invoice;
        match Bolt11Invoice::from_str(&req.invoice) {
            Ok(invoice) => {
                match node.bolt11_payment().send(&invoice, None) {
                    Ok(payment_id) => {
                        let hash_str = hex_encode(&payment_id.0);
                        HttpResponse::Ok().json(PayInvoiceResponse {
                            status: "success".to_string(),
                            payment_hash: hash_str,
                            fee_msat: 2000, // default placeholder fee for success
                        })
                    }
                    Err(e) => {
                        error!("LDK send payment error: {:?}", e);
                        HttpResponse::BadRequest().body(format!("Payment failed: {:?}", e))
                    }
                }
            }
            Err(e) => {
                HttpResponse::BadRequest().body(format!("Invalid invoice: {:?}", e))
            }
        }
    } else {
        HttpResponse::InternalServerError().body("Node not initialized")
    }
}

// Handler: Get Status
#[get("/status")]
async fn get_status(state: web::Data<AppState>) -> impl Responder {
    if state.is_mock {
        return HttpResponse::Ok().json(StatusResponse {
            node_id: "03mocknodeid0000000000000000000000000000000000000000000000000000".to_string(),
            is_running: true,
            is_mock: true,
            num_channels: 3,
            num_peers: 2,
        });
    }

    if let Some(ref node) = state.node {
        HttpResponse::Ok().json(StatusResponse {
            node_id: node.node_id().to_string(),
            is_running: true, // If node is started successfully
            is_mock: false,
            num_channels: node.list_channels().len(),
            num_peers: node.list_peers().len(),
        })
    } else {
        HttpResponse::Ok().json(StatusResponse {
            node_id: "".to_string(),
            is_running: false,
            is_mock: false,
            num_channels: 0,
            num_peers: 0,
        })
    }
}

// Handler: Mock Receive Payment
#[post("/mock/receive-payment")]
async fn mock_receive_payment(
    state: web::Data<AppState>,
    req: web::Json<MockReceivePaymentRequest>,
) -> impl Responder {
    info!("Mock receiving payment for hash: {}", req.payment_hash);
    
    // Check if it's a mock invoice we generated
    {
        let mut invoices = state.mock_invoices.lock().unwrap();
        if let Some(inv) = invoices.get_mut(&req.payment_hash) {
            inv.settled = true;
        }
    }
    
    // Make the notification callback to api-server
    let client = reqwest::Client::new();
    let amount_sats = req.amount_msat / 1000;
    let payload = serde_json::json!({
        "payment_hash": req.payment_hash,
        "amount_sats": amount_sats
    });
    
    let url = format!("{}/api/v1/internal/settle-deposit", state.api_server_url);
    match client.post(&url)
        .header("Authorization", format!("Bearer {}", state.internal_service_secret))
        .json(&payload)
        .send()
        .await {
            Ok(res) => {
                if res.status().is_success() {
                    info!("Successfully notified api-server of settled payment");
                    HttpResponse::Ok().body("Payment receipt simulated successfully")
                } else {
                    let status = res.status();
                    error!("api-server internal settlement endpoint failed with status: {:?}", status);
                    HttpResponse::InternalServerError().body(format!("Callback failed with status {:?}", status))
                }
            }
            Err(e) => {
                error!("Failed to request api-server settlement callback: {:?}", e);
                HttpResponse::InternalServerError().body(format!("Callback failed: {:?}", e))
            }
        }
}

// Event Polling loop for real LDK node
fn spawn_event_processor(node: Arc<ldk_node::Node>, api_server_url: String, secret: String) {
    std::thread::spawn(move || {
        info!("LDK background event processor started");
        loop {
            let event = node.wait_next_event();
            info!("Received LDK event: {:?}", event);
            match event {
                ldk_node::Event::PaymentReceived { payment_hash, amount_msat, .. } => {
                    info!("Payment received: hash={:?}, amount={}", payment_hash, amount_msat);
                    
                    let hex_hash = hex_encode(&payment_hash.0);
                    let amount_sats = amount_msat / 1000;
                    let payload = serde_json::json!({
                        "payment_hash": hex_hash,
                        "amount_sats": amount_sats
                    });
                    
                    let client = reqwest::blocking::Client::new();
                    let url = format!("{}/api/v1/internal/settle-deposit", api_server_url);
                    match client.post(&url)
                        .header("Authorization", format!("Bearer {}", secret))
                        .json(&payload)
                        .send() {
                            Ok(res) => {
                                if res.status().is_success() {
                                    info!("Settled deposit on api-server for hash: {}", hex_hash);
                                } else {
                                    error!("api-server returned error for hash {}: {:?}", hex_hash, res.status());
                                }
                            }
                            Err(e) => {
                                error!("Failed to contact api-server to settle deposit for hash {}: {:?}", hex_hash, e);
                            }
                        }
                }
                _ => {}
            }
            let _ = node.event_handled();
        }
    });
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let _ = dotenvy::dotenv();

    let api_server_url = std::env::var("API_SERVER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let internal_service_secret = std::env::var("INTERNAL_SERVICE_SECRET").unwrap_or_else(|_| "super-secret-token".to_string());
    let ldk_mock_mode = std::env::var("LDK_MOCK_MODE").unwrap_or_else(|_| "false".to_string()) == "true";
    let data_dir = std::env::var("LDK_DATA_DIR").unwrap_or_else(|_| "./.ldk_data".to_string());
    let peer_port = std::env::var("LDK_PEER_PORT").unwrap_or_else(|_| "9735".to_string())
        .parse::<u16>().unwrap_or(9735);

    let bitcoin_rpc_host = std::env::var("BITCOIN_RPC_HOST").unwrap_or_else(|_| "localhost".to_string());
    let bitcoin_rpc_port = std::env::var("BITCOIN_RPC_PORT").unwrap_or_else(|_| "18443".to_string())
        .parse::<u16>().unwrap_or(18443);
    let bitcoin_rpc_user = std::env::var("BITCOIN_RPC_USER").unwrap_or_else(|_| "satsbolt".to_string());
    let bitcoin_rpc_pass = std::env::var("BITCOIN_RPC_PASS").unwrap_or_else(|_| "satsboltpass".to_string());

    let mut is_mock = ldk_mock_mode;
    let mut node_opt = None;

    if !is_mock {
        info!("Attempting to initialize LDK Node in real mode...");
        let mut builder = Builder::new();
        builder.set_network(Network::Regtest);
        let _ = builder.set_storage_dir_path(data_dir);
        
        if let Ok(addr) = SocketAddress::from_str(&format!("0.0.0.0:{}", peer_port)) {
            let _ = builder.set_listening_addresses(vec![addr]);
        }

        builder.set_chain_source_bitcoind_rpc(
            bitcoin_rpc_host,
            bitcoin_rpc_port,
            bitcoin_rpc_user,
            bitcoin_rpc_pass,
        );

        match builder.build() {
            Ok(node) => {
                match node.start() {
                    Ok(_) => {
                        info!("LDK Node started successfully. Node ID: {}", node.node_id());
                        let shared_node = Arc::new(node);
                        spawn_event_processor(shared_node.clone(), api_server_url.clone(), internal_service_secret.clone());
                        node_opt = Some(shared_node);
                    }
                    Err(e) => {
                        warn!("Failed to start LDK Node: {:?}. Falling back to MOCK mode.", e);
                        is_mock = true;
                    }
                }
            }
            Err(e) => {
                warn!("Failed to build LDK Node: {:?}. Falling back to MOCK mode.", e);
                is_mock = true;
            }
        }
    }

    if is_mock {
        info!("Lightning Node running in MOCK mode.");
    }

    let state = AppState {
        node: node_opt,
        is_mock,
        mock_invoices: Arc::new(Mutex::new(std::collections::HashMap::new())),
        api_server_url,
        internal_service_secret,
    };

    let server_port = std::env::var("LDK_SERVICE_PORT").unwrap_or_else(|_| "8081".to_string())
        .parse::<u16>().unwrap_or(8081);

    info!("Starting lightning-node HTTP API server on port {}...", server_port);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(state.clone()))
            .service(create_invoice)
            .service(pay_invoice)
            .service(get_status)
            .service(mock_receive_payment)
    })
    .bind(("0.0.0.0", server_port))?
    .run()
    .await
}
