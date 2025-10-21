use std::time::Duration;

use clap::Parser;
use paho_mqtt::{self as mqtt, AsyncClient, AsyncReceiver, Message};
use serde::{Deserialize, Serialize};
use tokio;
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::db::DbPool;
use crate::operations::insert_electricity_meter_message;

mod db;
mod models;
mod operations;
mod schema;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MqttSettings {
    pub hostname: String,
    pub topic: String,
    pub username: String,
    pub password: String,
}

impl MqttSettings {
    pub fn is_complete(&self) -> bool {
        self.hostname.len() > 0
            && self.topic.len() > 0
            && self.username.len() > 0
            && self.password.len() > 0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ElectricityMeter {
    pub timestamp: String,
    pub energy: Energy,
    pub power: Power,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Energy {
    pub export: EnergyExport,
    pub import: EnergyImport,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EnergyExport {
    pub cumulative: f64,
    pub units: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EnergyImport {
    pub cumulative: f64,
    pub day: f64,
    pub week: f64,
    pub month: f64,
    pub units: String,
    pub mpan: String,
    pub supplier: String,
    pub price: ImportPrice,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ImportPrice {
    pub unitrate: f64,
    pub standingcharge: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Power {
    pub value: f64,
    pub units: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ElectricityMeterMessage {
    pub electricitymeter: ElectricityMeter,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ElectricityUpdate {
    is_active: bool,
    message: ElectricityMeterMessage,
}

async fn create_mqtt_client(
    client_id: String,
    settings: &MqttSettings,
) -> Result<AsyncClient, paho_mqtt::Error> {
    let qos = 1;

    let client_options = mqtt::CreateOptionsBuilder::new()
        .server_uri(settings.hostname.clone())
        .client_id(client_id)
        .finalize();

    let client = mqtt::AsyncClient::new(client_options)?;

    let connection_options = mqtt::ConnectOptionsBuilder::new()
        .clean_session(true)
        .user_name(settings.username.clone())
        .password(settings.password.clone())
        .connect_timeout(Duration::from_secs(5))
        .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(30))
        .finalize();

    client.connect(connection_options).await?;

    client.subscribe(settings.topic.clone(), qos).await?;

    Ok(client)
}

pub async fn start_mqtt_listener(pool: DbPool, settings: MqttSettings) {
    println!("Starting MQTT listener thread.");

    let uuid_string = Uuid::new_v4().to_string();
    let client_id = format!("glowmarkt_logger-{}", uuid_string);
    println!("Generated Client ID: {}", client_id);

    let mut client_and_stream: Option<(AsyncClient, AsyncReceiver<Option<Message>>)> = None;

    loop {
        let (_client, stream) = match client_and_stream.as_mut() {
            Some((c, s)) => (c, s),
            None => {
                if settings.is_complete() {
                    println!(
                        "MQTT settings are complete but client is not yet created. Creating MQTT client..."
                    );
                    match create_mqtt_client(client_id.clone(), &settings).await {
                        Ok(mut client) => {
                            let stream = client.get_stream(None);
                            println!("MQTT client and stream created");
                            client_and_stream = Some((client, stream));
                            continue;
                        }
                        Err(e) => {
                            println!("Failed to create client: {}", e);
                        }
                    };
                } else {
                    println!("MQTT settings are not complete");
                }

                // Sleep to avoid tight loop
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        };

        tokio::select! {
            message = stream.next() => {
                if let Some(Some(msg)) = message {
                    match serde_json::from_str::<ElectricityMeterMessage>(&msg.payload_str()) {
                        Ok(data) => {
                            println!("Deserialized data: {:?}", data);

                            let mut conn = match pool.get() {
                                Ok(conn) => conn,
                                Err(err) => {
                                    println!("Failed to get DB connection from pool: {}", err);
                                    continue;
                                }
                            };

                            match insert_electricity_meter_message(&mut conn, &data).await {
                                Ok(inserted) => {
                                    if inserted {
                                        println!("Inserted data into DB");
                                    }
                                }
                                Err(err) => {
                                    println!("Unexpected error during DB insert: {}", err);
                                }
                            }
                        }
                        Err(e) => {
                            println!("Failed to deserialize payload: {}", e);
                        }
                    }
                }
            },
        }
    }
}

/// A program for capturing and storing in a SQLite database MQTT messages published from a Glowmarkt CAD device
#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    /// The path to the SQLite database to store data to
    #[clap(short, long)]
    database: String,

    /// The address of the MQTT message broker to connect to
    #[clap(short, long)]
    broker: String,

    /// The electricity topic that the messages are published to
    #[clap(short, long)]
    topic: String,

    /// The username to connect to the MQTT broker with
    #[clap(short, long)]
    username: String,

    /// The password to connect to the MQTT broker with
    #[clap(short, long)]
    password: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let pool = db::create_pool(&args.database).expect("Failed to create DB pool");

    db::run_migrations(&pool).expect("Failed to run DB migrations");

    let settings = MqttSettings {
        hostname: args.broker,
        topic: args.topic,
        username: args.username,
        password: args.password,
    };

    let checkpoint_pool = pool.clone();
    tokio::spawn(async move {
        start_checkpoint_listener(checkpoint_pool).await;
    });

    start_mqtt_listener(pool, settings).await;
}

pub async fn start_checkpoint_listener(pool: DbPool) {
    println!("Starting checkpoint listener thread.");
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        println!("Triggering WAL checkpoint.");
        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(err) => {
                println!("Failed to get DB connection from pool for checkpoint: {}", err);
                continue;
            }
        };
        use diesel::connection::SimpleConnection;
        if let Err(err) = conn.batch_execute("PRAGMA wal_checkpoint(TRUNCATE);") {
            println!("Failed to trigger WAL checkpoint: {}", err);
        }
    }
}
