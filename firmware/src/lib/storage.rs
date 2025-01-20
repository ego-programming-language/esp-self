use esp_idf_svc::{
    nvs::{EspDefaultNvs, EspDefaultNvsPartition, EspNvs, EspNvsPartition, NvsDefault},
    sys::EspError,
};

pub struct KeyValStore {
    namespace: String,
    nvs: EspNvs<NvsDefault>,
}

impl KeyValStore {
    // TODO: HANDLE ERRORS GETTING THE NVS
    pub fn new(namespace: &str) -> KeyValStore {
        let nvs_default_partition: EspNvsPartition<NvsDefault> =
            EspDefaultNvsPartition::take().expect("Cannot get defualt NVS partition");
        let nvs = match EspNvs::new(nvs_default_partition, namespace, true) {
            Ok(nvs) => {
                log::info!("Got namespace {:?} from default partition", namespace);
                nvs
            }
            Err(e) => panic!("Could't get namespace {:?}", e),
        };

        KeyValStore {
            namespace: namespace.to_string(),
            nvs,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<Vec<u8>> {
        let buff: &mut [u8] = &mut [0; 1024];

        match self.nvs.get_raw(key, buff) {
            Ok(v) => match v {
                Some(vv) => Some(vv.to_vec()),
                None => None,
            },
            Err(e) => {
                log::error!("Couldn't get key {} because {:?}", key, e);
                None
            }
        }
    }

    pub fn set(&mut self, key: &str, value: &[u8]) -> Result<(), anyhow::Error> {
        // You can find the meaning of the error codes in:
        // https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/error-codes.html
        self.nvs.set_raw(key, value)?;
        Ok(())
    }
}
