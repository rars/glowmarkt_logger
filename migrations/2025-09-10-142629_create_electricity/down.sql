-- This file should undo anything in `up.sql`
-- Your SQL goes here
DROP TABLE IF EXISTS electricity_meter_messages;

-- Table to store the power reading, linked to the main message.
DROP TABLE IF EXISTS power_readings;

-- Table for energy import data, linked to the main message.
-- Note: The price information is included here since it is directly tied to the import data.
DROP TABLE energy_import_data;

-- Table for energy export data, linked to the main message.
DROP TABLE energy_export_data;
