const MAGIC_HEADER: u16 = 0xBEEF;
pub const MAGIC_HEADER_SIZE: usize = MAGIC_HEADER.to_le_bytes().len();

#[derive(Debug)]
pub enum StorageError {
    DriverError,
    InvalidMagicHeader,
    InvalidData,
}

pub trait Encodable<const SIZE: usize>
where
    Self: Sized,
{
    fn encode(self) -> [u8; SIZE];
    fn decode(data: &[u8]) -> Option<Self>;
}

#[allow(async_fn_in_trait)]
pub trait StorageDriver<const STORAGE_SIZE: usize> {
    async fn read(&self, buffer: &mut [u8]) -> Result<(), StorageError>;
    async fn write(&self, buffer: &[u8]) -> Result<(), StorageError>;
}

/// Persistent storage implementation using a storage driver.
pub struct PersistentStorage<DRIVER: StorageDriver<STORAGE_SIZE>, const STORAGE_SIZE: usize> {
    driver: DRIVER,
}

impl<DRIVER: StorageDriver<STORAGE_SIZE>, const STORAGE_SIZE: usize>
    PersistentStorage<DRIVER, STORAGE_SIZE>
{
    pub fn new(driver: DRIVER) -> Self {
        Self { driver }
    }

    /// Load persistent data from flash
    pub async fn load<const SIZE: usize, T: Encodable<SIZE>>(&self) -> Result<T, StorageError> {
        let mut buffer = [0u8; STORAGE_SIZE];

        match self.driver.read(&mut buffer).await {
            Ok(()) => {
                let magic = u16::from_le_bytes([buffer[0], buffer[1]]);
                if magic == MAGIC_HEADER {
                    return T::decode(&buffer[MAGIC_HEADER_SIZE..STORAGE_SIZE])
                        .ok_or(StorageError::InvalidData);
                }
            }
            Err(_) => {
                return Err(StorageError::DriverError);
            }
        }
        Err(StorageError::InvalidMagicHeader)
    }

    /// Save persistent data to flash
    pub async fn save<const SIZE: usize, T: Encodable<SIZE> + Clone>(
        &self,
        state: &T,
    ) -> Result<(), StorageError> {
        let mut data = [0u8; STORAGE_SIZE];

        data[0..MAGIC_HEADER_SIZE].copy_from_slice(&MAGIC_HEADER.to_le_bytes());
        let encoded = state.clone().encode();
        data[MAGIC_HEADER_SIZE..STORAGE_SIZE].copy_from_slice(&encoded);

        self.driver
            .write(&data)
            .await
            .map_err(|_| StorageError::DriverError)
    }
}
