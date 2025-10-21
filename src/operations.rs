use chrono::{NaiveDateTime, ParseError};
use diesel::prelude::*;
use diesel::sql_types::Integer;
use std::fmt;

use crate::db::DbConnection;
use crate::models::{self, ElectricityMeterMessage};

#[derive(Debug)]
pub enum InsertError {
    DbError(diesel::result::Error),
    TimeParseError(ParseError),
}

impl fmt::Display for InsertError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InsertError::DbError(ref err) => write!(f, "Database Error: {}", err),
            InsertError::TimeParseError(ref err) => write!(f, "Timestamp Parse Error: {}", err),
        }
    }
}

impl std::error::Error for InsertError {}

impl From<diesel::result::Error> for InsertError {
    fn from(err: diesel::result::Error) -> InsertError {
        InsertError::DbError(err)
    }
}

impl From<ParseError> for InsertError {
    fn from(err: ParseError) -> InsertError {
        InsertError::TimeParseError(err)
    }
}

pub async fn insert_electricity_meter_message(
    conn: &mut DbConnection,
    message: &crate::ElectricityMeterMessage,
) -> Result<bool, InsertError> {
    use crate::schema::electricity_meter_messages::dsl::*;
    use crate::schema::energy_export_data::dsl::*;
    use crate::schema::energy_import_data::dsl::*;
    use crate::schema::power_readings::dsl::*;

    let message_contents = &message.electricitymeter;

    let format = "%Y-%m-%dT%H:%M:%SZ";

    conn.transaction::<_, InsertError, _>(|conn| {
        let parsed_timestamp = NaiveDateTime::parse_from_str(&message_contents.timestamp, format)?;

        let existing_message = electricity_meter_messages
            .filter(timestamp.eq(parsed_timestamp))
            .first::<models::ElectricityMeterMessage>(conn)
            .optional()?;

        if existing_message.is_some() {
            println!(
                "Skipping duplicate message with timestamp: {}",
                &message_contents.timestamp
            );
            return Ok(false);
        }

        let emm = models::NewElectricityMeterMessage {
            timestamp: parsed_timestamp,
        };

        diesel::insert_into(electricity_meter_messages)
            .values(&emm)
            .execute(conn)?;

        let last_id =
            diesel::dsl::sql::<Integer>("SELECT last_insert_rowid()").get_result::<i32>(conn)?;

        let created_user: ElectricityMeterMessage = electricity_meter_messages
            .filter(
                crate::schema::electricity_meter_messages::dsl::electricity_meter_message_id
                    .eq(last_id),
            )
            .first(conn)?;

        let message_id = created_user.electricity_meter_message_id;

        let export = models::NewEnergyExportData {
            electricity_meter_message_id: message_id,
            cumulative: message_contents.energy.export.cumulative as f32,
            units: message_contents.energy.export.units.clone(),
        };

        diesel::insert_into(energy_export_data)
            .values(&export)
            .execute(conn)?;

        let energy_import = &message_contents.energy.import;

        let import = models::NewEnergyImportData {
            electricity_meter_message_id: message_id,
            cumulative: energy_import.cumulative as f32,
            day: energy_import.day as f32,
            week: energy_import.week as f32,
            month: energy_import.month as f32,
            units: energy_import.units.clone(),
            mpan: energy_import.mpan.clone(),
            supplier: energy_import.supplier.clone(),
            unitrate: energy_import.price.unitrate as f32,
            standingcharge: energy_import.price.standingcharge as f32,
        };

        diesel::insert_into(energy_import_data)
            .values(&import)
            .execute(conn)?;

        let power_reading = &message_contents.power;

        let power = models::NewPowerReading {
            electricity_meter_message_id: message_id,
            value: power_reading.value as f32,
            units: power_reading.units.clone(),
        };

        diesel::insert_into(power_readings)
            .values(&power)
            .execute(conn)?;

        Ok(true)
    })
}
