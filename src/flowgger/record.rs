#[derive(Debug)]
pub struct Pri {
    pub facility: u8,
    pub severity: u8
}

#[derive(Debug)]
pub struct StructuredData {
    pub sd_id: String,
    pub pairs: Vec<(String, String)>
}

impl StructuredData {
    pub fn new(sd_id: &str) -> StructuredData {
        StructuredData {
            sd_id: sd_id.to_string(),
            pairs: Vec::new()
        }
    }
}

#[derive(Debug)]
pub struct Record {
    pub pri: Option<Pri>,
    pub ts: i64,
    pub hostname: String,
    pub appname: Option<String>,
    pub procid: Option<String>,
    pub msgid: Option<String>,
    pub sd: Option<StructuredData>,
    pub msg: Option<String>
}
