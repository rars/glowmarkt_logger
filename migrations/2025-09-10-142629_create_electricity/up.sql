-- Your SQL goes here
CREATE TABLE electricity_meter_messages (
  electricity_meter_message_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  timestamp TIMESTAMP NOT NULL
);

-- Table to store the power reading, linked to the main message.
CREATE TABLE power_readings (
  power_reading_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  electricity_meter_message_id INTEGER NOT NULL,
  value REAL NOT NULL,
  units TEXT NOT NULL,
  FOREIGN KEY(electricity_meter_message_id) REFERENCES electricity_meter_messages(electricity_meter_message_id)
);

-- Table for energy import data, linked to the main message.
-- Note: The price information is included here since it is directly tied to the import data.
CREATE TABLE energy_import_data (
  energy_import_data_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  electricity_meter_message_id INTEGER NOT NULL,
  cumulative REAL NOT NULL,
  day REAL NOT NULL,
  week REAL NOT NULL,
  month REAL NOT NULL,
  units TEXT NOT NULL,
  mpan TEXT NOT NULL,
  supplier TEXT NOT NULL,
  unitrate REAL NOT NULL,
  standingcharge REAL NOT NULL,
  FOREIGN KEY(electricity_meter_message_id) REFERENCES electricity_meter_messages(electricity_meter_message_id)
);

-- Table for energy export data, linked to the main message.
CREATE TABLE energy_export_data (
  energy_export_data_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
  electricity_meter_message_id INTEGER NOT NULL,
  cumulative REAL NOT NULL,
  units TEXT NOT NULL,
  FOREIGN KEY(electricity_meter_message_id) REFERENCES electricity_meter_messages(electricity_meter_message_id)
);