use std::time::Duration;

use diesel::{Connection, SqliteConnection};
use paho_mqtt::{self as mqtt, AsyncClient, AsyncReceiver, DisconnectOptionsBuilder, Message};
use serde::{Deserialize, Serialize};
use tokio;
use tokio_stream::StreamExt;
use uuid::Uuid;

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

pub async fn start_mqtt_listener(conn: &mut SqliteConnection, settings: MqttSettings) {
    println!("Starting MQTT listener thread.");

    let uuid_string = Uuid::new_v4().to_string();
    let client_id = format!("mqtt_logger-{}", uuid_string);
    println!("Generated Client ID: {}", client_id);

    let mut client_and_stream: Option<(AsyncClient, AsyncReceiver<Option<Message>>)> = None;

    loop {
        let (client, stream) = match client_and_stream.as_mut() {
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
                            // Now you can work with the structured 'data' object

                            if let Err(err) = insert_electricity_meter_message(conn, &data).await {
                                println!("Unexpected error during DB insert: {}", err);
                            }

                            // if let Err(err) = emit_event("electricityUpdate", data) {
                              //  println!("Unexpected error emitting electricityUpdate event: {}", err);
                           // }
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

async fn disconnect_client(client: &AsyncClient) {
    let opts = DisconnectOptionsBuilder::new()
        .timeout(Duration::from_secs(5))
        .finalize();

    match client.disconnect(Some(opts)).await {
        Ok(_) => println!("Successfully disconnected MQTT client."),
        Err(e) => println!("Error during MQTT client disconnect: {:?}", e),
    }
}

#[tokio::main]
async fn main() {
    let mut connection = db::establish_connection("./glowmarkt.db");

    db::run_migrations(&mut connection);

    // TODO: take these parameters from env vars or args
    let settings = MqttSettings {
        hostname: "".into(),
        topic: "".into(),
        username: "".into(),
        password: "".into(),
    };

    start_mqtt_listener(&mut connection, settings).await;
}
