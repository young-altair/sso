table! {
    /// Representation of the `kv_data` table.
    ///
    /// (Automatically generated by Diesel.)
    kv_data (data_chunk, version_id) {
        /// The `data_chunk` column of the `kv_data` table.
        ///
        /// Its SQL type is `BigInt`.
        ///
        /// (Automatically generated by Diesel.)
        data_chunk -> BigInt,
        /// The `data_value` column of the `kv_data` table.
        ///
        /// Its SQL type is `Binary`.
        ///
        /// (Automatically generated by Diesel.)
        data_value -> Binary,
        /// The `version_id` column of the `kv_data` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        version_id -> Text,
    }
}

table! {
    /// Representation of the `kv_disk` table.
    ///
    /// (Automatically generated by Diesel.)
    kv_disk (disk_id) {
        /// The `created_at` column of the `kv_disk` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        created_at -> Text,
        /// The `updated_at` column of the `kv_disk` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        updated_at -> Text,
        /// The `disk_id` column of the `kv_disk` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        disk_id -> Text,
        /// The `disk_name` column of the `kv_disk` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        disk_name -> Text,
        /// The `disk_chunk_size` column of the `kv_disk` table.
        ///
        /// Its SQL type is `BigInt`.
        ///
        /// (Automatically generated by Diesel.)
        disk_chunk_size -> BigInt,
        /// The `disk_compression` column of the `kv_disk` table.
        ///
        /// Its SQL type is `BigInt`.
        ///
        /// (Automatically generated by Diesel.)
        disk_compression -> BigInt,
        /// The `disk_encryption` column of the `kv_disk` table.
        ///
        /// Its SQL type is `BigInt`.
        ///
        /// (Automatically generated by Diesel.)
        disk_encryption -> BigInt,
        /// The `disk_secret_key` column of the `kv_disk` table.
        ///
        /// Its SQL type is `Binary`.
        ///
        /// (Automatically generated by Diesel.)
        disk_secret_key -> Binary,
        /// The `disk_public_key` column of the `kv_disk` table.
        ///
        /// Its SQL type is `Binary`.
        ///
        /// (Automatically generated by Diesel.)
        disk_public_key -> Binary,
        /// The `disk_version_retention` column of the `kv_disk` table.
        ///
        /// Its SQL type is `BigInt`.
        ///
        /// (Automatically generated by Diesel.)
        disk_version_retention -> BigInt,
        /// The `disk_duration_retention` column of the `kv_disk` table.
        ///
        /// Its SQL type is `BigInt`.
        ///
        /// (Automatically generated by Diesel.)
        disk_duration_retention -> BigInt,
    }
}

table! {
    /// Representation of the `kv_key` table.
    ///
    /// (Automatically generated by Diesel.)
    kv_key (key_id) {
        /// The `created_at` column of the `kv_key` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        created_at -> Text,
        /// The `updated_at` column of the `kv_key` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        updated_at -> Text,
        /// The `key_id` column of the `kv_key` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        key_id -> Text,
        /// The `key_name` column of the `kv_key` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        key_name -> Text,
        /// The `disk_id` column of the `kv_key` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        disk_id -> Text,
        /// The `version_id` column of the `kv_key` table.
        ///
        /// Its SQL type is `Nullable<Text>`.
        ///
        /// (Automatically generated by Diesel.)
        version_id -> Nullable<Text>,
    }
}

table! {
    /// Representation of the `kv_version` table.
    ///
    /// (Automatically generated by Diesel.)
    kv_version (version_id) {
        /// The `created_at` column of the `kv_version` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        created_at -> Text,
        /// The `version_id` column of the `kv_version` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        version_id -> Text,
        /// The `version_hash` column of the `kv_version` table.
        ///
        /// Its SQL type is `Binary`.
        ///
        /// (Automatically generated by Diesel.)
        version_hash -> Binary,
        /// The `version_size` column of the `kv_version` table.
        ///
        /// Its SQL type is `BigInt`.
        ///
        /// (Automatically generated by Diesel.)
        version_size -> BigInt,
        /// The `version_compressed_size` column of the `kv_version` table.
        ///
        /// Its SQL type is `BigInt`.
        ///
        /// (Automatically generated by Diesel.)
        version_compressed_size -> BigInt,
        /// The `key_id` column of the `kv_version` table.
        ///
        /// Its SQL type is `Text`.
        ///
        /// (Automatically generated by Diesel.)
        key_id -> Text,
    }
}

joinable!(kv_data -> kv_version (version_id));
joinable!(kv_key -> kv_disk (disk_id));

allow_tables_to_appear_in_same_query!(kv_data, kv_disk, kv_key, kv_version,);
