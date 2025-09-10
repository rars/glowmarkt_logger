use chrono::NaiveDateTime;
// use chrono::prelude::*;
use diesel::prelude::*;

// Ensure your `schema.rs` is in the same directory or module.
use crate::schema::*;

// Represents a new ElectricityMeterMessage to be inserted.
#[derive(Insertable)]
#[diesel(table_name = electricity_meter_messages)]
pub struct NewElectricityMeterMessage {
    pub timestamp: NaiveDateTime,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = electricity_meter_messages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ElectricityMeterMessage {
    pub electricity_meter_message_id: i32,
    pub timestamp: NaiveDateTime,
}

// Represents new EnergyExportData to be inserted.
#[derive(Insertable)]
#[diesel(table_name = energy_export_data)]
pub struct NewEnergyExportData {
    pub electricity_meter_message_id: i32,
    pub cumulative: f32,
    pub units: String,
}

// Represents new EnergyExportData to be inserted.
#[derive(Queryable)]
#[diesel(table_name = energy_export_data)]
pub struct EnergyExportData {
    pub energy_export_data_id: i32,
    pub electricity_meter_message_id: i32,
    pub cumulative: f32,
    pub units: String,
}

// Represents new EnergyImportData to be inserted.
#[derive(Insertable)]
#[diesel(table_name = energy_import_data)]
pub struct NewEnergyImportData {
    pub electricity_meter_message_id: i32,
    pub cumulative: f32,
    pub day: f32,
    pub week: f32,
    pub month: f32,
    pub units: String,
    pub mpan: String,
    pub supplier: String,
    pub unitrate: f32,
    pub standingcharge: f32,
}

// Represents a new PowerReading to be inserted.
#[derive(Insertable)]
#[diesel(table_name = power_readings)]
pub struct NewPowerReading {
    pub electricity_meter_message_id: i32,
    pub value: f32,
    pub units: String,
}
