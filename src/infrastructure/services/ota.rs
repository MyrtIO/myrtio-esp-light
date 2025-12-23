use core::cell::RefCell;

use super::http::connection::AsyncChunkedReader;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::Duration;
use embedded_storage::nor_flash::NorFlash;
use embedded_storage::nor_flash::ReadNorFlash;
use esp_bootloader_esp_idf::ota::Ota;
use esp_bootloader_esp_idf::ota::OtaImageState;
use esp_bootloader_esp_idf::{
    ota_updater::OtaUpdater,
    partitions::{FlashRegion, PARTITION_TABLE_MAX_LEN},
};
use esp_println::println;
use esp_storage::FlashStorage;
use static_cell::StaticCell;

use crate::infrastructure::services::http::HttpError;

// use super::http::HttpConnection;

const ALIGN: usize = 4;
const ERASE_SECTOR: u32 = 4096;

#[derive(Debug)]
pub(crate) enum OtaError {
    Erase,
    InvalidPartitionTable,
    Write,
    Read,
    Activate,
    Flash,
}

#[allow(clippy::cast_possible_truncation)]
pub(crate) async fn update_from_http(
    conn: &mut impl AsyncChunkedReader,
    flash: *mut FlashStorage<'static>,
) -> Result<(), OtaError> {
    let flash_ref = unsafe { &mut *flash };
    let mut part_buffer = [0u8; PARTITION_TABLE_MAX_LEN];
    let mut updater = OtaUpdater::new(flash_ref, &mut part_buffer)
        .map_err(|_| OtaError::InvalidPartitionTable)?;

    let (mut partition, _part_type) = updater
        .next_partition()
        .map_err(|_| OtaError::InvalidPartitionTable)?;

    let part_capacity = partition.capacity() as u32;
    let erase_size =
        conn.content_length().saturating_add(ERASE_SECTOR - 1) / ERASE_SECTOR * ERASE_SECTOR;
    let erase_size = erase_size.min(part_capacity);
    if partition.erase(0, erase_size).is_err() {
        return Err(OtaError::Erase);
    }

    let mut written: u32 = 0;
    let mut received: usize = 0;
    let mut tail = [0xFFu8; ALIGN];
    let mut tail_len: usize = 0;

    let mut is_eof = false;
    while !is_eof {
        conn.read_and_then(|chunk| {
            if chunk.is_empty() {
                is_eof = true;
            } else {
                write_aligned_data(
                    &mut partition,
                    chunk,
                    &mut written,
                    &mut tail,
                    &mut tail_len,
                )
                .unwrap();
                received += chunk.len();
            }
        })
        .await
        .map_err(|_| OtaError::Read)?;
    }

    // Write final tail
    if tail_len > 0 {
        partition
            .write(written, &tail)
            .map_err(|_| OtaError::Write)?;
    }

    updater
        .activate_next_partition()
        .and_then(|()| updater.set_current_ota_state(OtaImageState::New))
        .map_err(|_| OtaError::Activate)?;

    Ok(())
}

#[allow(clippy::cast_possible_truncation)]
fn write_aligned_data<F: embedded_storage::nor_flash::NorFlash>(
    partition: &mut F,
    data: &[u8],
    written: &mut u32,
    tail: &mut [u8; 4],
    tail_len: &mut usize,
) -> Result<(), OtaError> {
    let mut idx = 0;

    // Complete partial word
    if *tail_len > 0 {
        let need = 4 - *tail_len;
        let take = need.min(data.len());
        tail[*tail_len..*tail_len + take].copy_from_slice(&data[..take]);
        *tail_len += take;
        idx += take;

        if *tail_len == 4 {
            partition
                .write(*written, tail)
                .map_err(|_| OtaError::Flash)?;
            *written += 4;
            *tail_len = 0;
            tail.fill(0xFF);
        }
    }

    // Write aligned bulk
    let rem = &data[idx..];
    let aligned_len = rem.len() & !3;
    if aligned_len > 0 {
        partition
            .write(*written, &rem[..aligned_len])
            .map_err(|_| OtaError::Flash)?;
        *written += aligned_len as u32;
    }

    // Keep trailing bytes
    let tail_bytes = &rem[aligned_len..];
    if !tail_bytes.is_empty() {
        tail[..tail_bytes.len()].copy_from_slice(tail_bytes);
        *tail_len = tail_bytes.len();
    }

    Ok(())
}
