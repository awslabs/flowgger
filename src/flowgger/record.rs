#[derive(Debug, Clone)]
pub enum SDValue {
    String(String),
    Bool(bool),
    F64(f64),
    I64(i64),
    U64(u64),
    Null
}

#[derive(Debug, Clone)]
pub enum SDValueType {
    String, Bool, F64, I64, U64
}

#[derive(Debug)]
pub struct StructuredData {
    pub sd_id: Option<String>,
    pub pairs: Vec<(String, SDValue)>
}

impl StructuredData {
    pub fn new(sd_id: Option<&str>) -> StructuredData {
        StructuredData {
            sd_id: match sd_id {
                Some(sd_id) => Some(sd_id.to_owned()),
                None => None
            },
            pairs: Vec::new()
        }
    }
}

#[derive(Debug)]
pub struct Record {
    pub ts: i64,
    pub hostname: String,
    pub facility: Option<u8>,
    pub severity: Option<u8>,
    pub appname: Option<String>,
    pub procid: Option<String>,
    pub msgid: Option<String>,
    pub msg: Option<String>,
    pub full_msg: Option<String>,
    pub sd: Option<StructuredData>
}
