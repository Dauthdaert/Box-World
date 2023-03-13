use std::{
    io::Cursor,
    sync::{Arc, Mutex},
};

use bevy::prelude::info_span;
use rusqlite::{params, Connection};
use zstd::stream::copy_encode;

use super::{data::RawChunk, ChunkPos};

pub fn save_raw_chunks(
    connection_lock: &Arc<Mutex<Connection>>,
    chunks: Vec<(ChunkPos, RawChunk)>,
) {
    let _span = info_span!("Saving chunks to disk").entered();

    let connection = connection_lock.lock().unwrap();
    connection.execute("BEGIN;", []).unwrap();
    for (chunk_pos, chunk_data) in chunks.iter() {
        save_raw_chunk(&connection, chunk_pos, chunk_data);
    }
    connection.execute("COMMIT;", []).unwrap();
}

fn save_raw_chunk(connection: &Connection, chunk_pos: &ChunkPos, chunk_data: &RawChunk) {
    if let Ok(raw_chunk_bin) = bincode::serialize(chunk_data) {
        let mut final_chunk = Cursor::new(raw_chunk_bin);
        let mut output = Cursor::new(Vec::new());
        copy_encode(&mut final_chunk, &mut output, 0).unwrap();
        connection
            .execute(
                "REPLACE INTO blocks (posx, posy, posz, data) values (?1, ?2, ?3, ?4)",
                params![
                    chunk_pos.x,
                    chunk_pos.y,
                    chunk_pos.z,
                    &output.get_ref().clone(),
                ],
            )
            .unwrap();
    }
}
