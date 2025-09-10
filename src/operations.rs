use chrono::NaiveDateTime;
use diesel::sql_types::Integer;
use diesel::{Connection, SqliteConnection};

use diesel::prelude::*;

use crate::models::{self, ElectricityMeterMessage};

// type RepositoryResult<T> = Result<T, RepositoryError>;

pub async fn insert_electricity_meter_message(
    conn: &mut SqliteConnection,
    message: &crate::ElectricityMeterMessage,
) -> Result<(), diesel::result::Error> {
    use crate::schema::electricity_meter_messages::dsl::*;
    use crate::schema::energy_export_data::dsl::*;
    use crate::schema::energy_import_data::dsl::*;
    use crate::schema::power_readings::dsl::*;

    let message_contents = &message.electricitymeter;

    let format = "%Y-%m-%dT%H:%M:%SZ";

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        let emm = models::NewElectricityMeterMessage {
            timestamp: NaiveDateTime::parse_from_str(&message_contents.timestamp, format)
                .expect("parsed"),
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

        Ok(())
    })?;

    Ok(())
}
