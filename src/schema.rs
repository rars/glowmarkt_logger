// @generated automatically by Diesel CLI.

diesel::table! {
    electricity_meter_messages (electricity_meter_message_id) {
        electricity_meter_message_id -> Integer,
        timestamp -> Timestamp,
    }
}

diesel::table! {
    energy_export_data (energy_export_data_id) {
        energy_export_data_id -> Integer,
        electricity_meter_message_id -> Integer,
        cumulative -> Float,
        units -> Text,
    }
}

diesel::table! {
    energy_import_data (energy_import_data_id) {
        energy_import_data_id -> Integer,
        electricity_meter_message_id -> Integer,
        cumulative -> Float,
        day -> Float,
        week -> Float,
        month -> Float,
        units -> Text,
        mpan -> Text,
        supplier -> Text,
        unitrate -> Float,
        standingcharge -> Float,
    }
}

diesel::table! {
    power_readings (power_reading_id) {
        power_reading_id -> Integer,
        electricity_meter_message_id -> Integer,
        value -> Float,
        units -> Text,
    }
}

diesel::joinable!(energy_export_data -> electricity_meter_messages (electricity_meter_message_id));
diesel::joinable!(energy_import_data -> electricity_meter_messages (electricity_meter_message_id));
diesel::joinable!(power_readings -> electricity_meter_messages (electricity_meter_message_id));

diesel::allow_tables_to_appear_in_same_query!(
    electricity_meter_messages,
    energy_export_data,
    energy_import_data,
    power_readings,
);
