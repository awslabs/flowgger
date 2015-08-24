#[derive(Debug)]
pub struct Pri {
    pub facility: u8,
    pub severity: u8
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum SDValue {
    String(String),
    Bool(bool),
    F64(f64),
    I64(i64),
    U64(u64)
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
    pub pri: Option<Pri>,
    pub ts: i64,
    pub hostname: String,
    pub appname: Option<String>,
    pub procid: Option<String>,
    pub msgid: Option<String>,
    pub sd: Option<StructuredData>,
    pub msg: Option<String>
}
